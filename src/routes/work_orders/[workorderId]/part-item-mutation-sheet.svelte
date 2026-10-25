<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";

  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import DateTimePicker from "$lib/components/date-time-picker.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";
  import {
    sheetHint,
    sheetInput,
    sheetLabel,
    sheetPresetButton,
    sheetSelectTrigger,
  } from "$lib/components/sheet/field-styles";
  import RequiredMark from "$lib/components/sheet/required-mark.svelte";
  import SheetSection from "$lib/components/sheet/sheet-section.svelte";
  import SheetSwitchRow from "$lib/components/sheet/sheet-switch-row.svelte";

  import {
    commands,
    type DiscountType,
    type PartStatus,
    type PreviewPartItemQuote,
    type VatScheme,
    type WorkOrderType,
  } from "$lib/bindings";
  import { zBigString } from "$lib/schemas/zbig";
  import {
    discountFields,
    discountFromEntity,
    discountToPayload,
    refineDiscount,
  } from "$lib/schemas/zdiscount";
  import { settingsState } from "$lib/states/settings.svelte";
  import { unwrap } from "$lib/tools";
  import { cn } from "$lib/utils";
  import {
    schemeNeedsZeroRate,
    VAT_SCHEMES,
    vatSchemeLabels,
  } from "$lib/vat-scheme";

  import DiscountField from "../discount-field.svelte";
  import {
    createCreatePartItemMutation,
    createPartItemByIdQuery,
    createPartItemVatQuery,
    createUpdatePartItemMutation,
    createVoidPartItemVatMutation,
    liveVatEntry,
  } from "../queries";
  import PartStatusSelector from "./part-status-selector.svelte";

  let {
    partItemId = null,
    workOrderId = null,
    workOrderType = "Regular",
    open = $bindable(false),
  }: {
    partItemId?: number | null;
    workOrderId?: string | null;
    workOrderType?: WorkOrderType;
    open: boolean;
  } = $props();

  const formID = "part-item-mutation-form";
  const isEdit = $derived(partItemId != null);

  async function fetchPartQuote(data: {
    cost: string;
    markup: string;
    discountType: DiscountType;
    discountValue: string;
    isFree: boolean;
  }): Promise<PreviewPartItemQuote | null> {
    if (!workOrderId) return null;
    try {
      return await unwrap(
        commands.previewPartItem({
          workOrderId,
          baseCost: data.cost || "0",
          markup: data.markup || "0",
          discountType: data.discountType,
          discountValue: data.discountValue || "0",
          isFree: data.isFree,
        }),
      );
    } catch {
      return null;
    }
  }

  let liveQuote = $state<PreviewPartItemQuote | null>(null);
  let quoteSeq = 0;

  $effect(() => {
    if (!open || !workOrderId) return;
    const snapshot = {
      cost: $formData.cost,
      markup: $formData.markup,
      discountType: $formData.discountType as DiscountType,
      discountValue: $formData.discountValue,
      isFree: $formData.isFree,
    };
    const seq = ++quoteSeq;
    const timer = setTimeout(() => {
      fetchPartQuote(snapshot).then((q) => {
        if (seq === quoteSeq) liveQuote = q;
      });
    }, 200);
    return () => clearTimeout(timer);
  });

  const liveWarning = $derived(
    liveQuote && liveQuote.belowCost ? liveQuote : null,
  );

  const fmt = (v: string) => Big(v).toFixed(2);

  const businessIsNotVatPayer = $derived(
    settingsState.settings?.companyInfo == null ||
      settingsState.settings.companyInfo.vatPayer === false,
  );

  const zeroRateStandardWarning = $derived.by(() => {
    if (
      businessIsNotVatPayer ||
      !$formData.vatRate.trim() ||
      $formData.vatScheme !== "standard"
    ) {
      return false;
    }
    try {
      return Big($formData.vatRate.replace(",", ".")).eq(0);
    } catch {
      return false;
    }
  });

  let priceConfirmOpen = $state(false);
  let pendingSubmitData = $state<z.infer<typeof partItemSchema> | null>(null);
  let pendingQuote = $state<PreviewPartItemQuote | null>(null);

  const queryClient = useQueryClient();

  const createPartItemMutation = createCreatePartItemMutation(queryClient);
  const updatePartItemMutation = createUpdatePartItemMutation(queryClient);

  const partVatQuery = createPartItemVatQuery(() => partItemId);
  const livePartVat = $derived(liveVatEntry(partVatQuery.data));
  const partVatLocked = $derived(Boolean(livePartVat));
  const businessIsVatPayer = $derived(settingsState.isVatPayer);
  const voidPartVatMutation = createVoidPartItemVatMutation(queryClient);
  let partVoidOpen = $state(false);
  let partCreditNoteNumber = $state("");
  let partCreditNoteDate = $state("");

  const creditPreview = $derived.by(() => {
    try {
      const gross = new Big($formData.cost || "0").times(
        $formData.quantity || "0",
      );
      const rate = new Big(
        ($formData.purchaseVatRate || "0").replace(",", "."),
      );
      if (gross.lte(0) || rate.lte(0)) {
        return { base: new Big(0), vat: new Big(0) };
      }
      const vat = $formData.purchasePriceIncludesVat
        ? gross.times(rate).div(new Big(100).plus(rate))
        : gross.times(rate).div(100);
      const base = $formData.purchasePriceIncludesVat
        ? gross.minus(vat)
        : gross;
      return { base, vat };
    } catch {
      return { base: new Big(0), vat: new Big(0) };
    }
  });

  async function voidPartVat() {
    if (!partItemId) return;
    try {
      await voidPartVatMutation.mutateAsync({
        id: partItemId,
        creditNoteNumber: partCreditNoteNumber.trim() || null,
        creditNoteDate: partCreditNoteDate || null,
      });
      partVoidOpen = false;
      partCreditNoteNumber = "";
      partCreditNoteDate = "";
      toast.success("ПДВ сторновано", {
        description:
          "Додано сторно-запис із датою коригування. Запчастину знову можна редагувати.",
        position: "bottom-left",
      });
    } catch {
      // The mutation already reported the failure.
    }
  }

  const defaultMarkup = $derived(settingsState.settings?.defaultMarkup ?? "0");

  let partItemsQuery = createPartItemByIdQuery(() => partItemId);
  let partItem = $derived(partItemsQuery.data);

  const partItemSchema = z
    .object({
      status: z.string().default("Quoted"),
      code: z.string().nullable(),
      name: z.string().min(1, { error: "Вкажіть назву" }),
      quantity: zBigString({
        requiredError: "Вкажіть кількість",
        min: { value: 0, message: "Кількість не може бути від'ємним числом" },
      }).default("1"),
      unitType: z.string().min(1, { error: "Вкажіть одиницю вимірювання" }),
      isFree: z.boolean().default(false),
      cost: zBigString({
        requiredError: "Вкажіть ціну",
        min: { value: 0, message: "Ціна не може бути від'ємним числом" },
      }).default("0"),
      markup: zBigString({
        requiredError: "Вкажіть націнку у відсотках",
        min: { value: 0, message: "Націнка не може бути від'ємним числом" },
        max: { value: 300, message: "Націнка має бути від 0% до 300%" },
      }).default("0"),
      vatRate: z.string().trim().default(""),
      vatScheme: z
        .enum<
          VatScheme[]
        >(["standard", "zero", "exempt", "not_object", "export"])
        .default("standard"),
      purchaseVatRate: zBigString({
        requiredError: "Вкажіть ставку ПДВ закупівлі",
        min: { value: 0, message: "Ставка ПДВ має бути від 0 до 100" },
        max: { value: 100, message: "Ставка ПДВ має бути від 0 до 100" },
      }).default("20"),
      purchasePriceIncludesVat: z.boolean().default(true),
      purchaseScheme: z
        .enum<
          VatScheme[]
        >(["standard", "zero", "exempt", "not_object", "export"])
        .default("standard"),
      supplierName: z.string().trim().max(200).default(""),
      supplierCode: z.string().trim().max(20).default(""),
      invoiceNumber: z.string().trim().max(60).default(""),
      invoiceDate: z
        .union([z.literal(""), z.string().regex(/^\d{4}-\d{2}-\d{2}$/)])
        .default(""),
      vatCredited: z.boolean().default(false),
      ...discountFields,
    })
    .superRefine(refineDiscount)
    .superRefine((data, ctx) => {
      if (!data.vatCredited) return;
      let rate = 0;
      try {
        rate = Number(new Big(data.purchaseVatRate || "0").toString());
      } catch {
        // handled by zBigString
      }
      if (rate <= 0) {
        ctx.addIssue({
          code: "custom",
          message: "Для податкового кредиту вкажіть ставку ПДВ",
          path: ["purchaseVatRate"],
        });
      }
      if (!data.supplierName.trim()) {
        ctx.addIssue({
          code: "custom",
          message: "Вкажіть постачальника для податкового кредиту",
          path: ["supplierName"],
        });
      }
      if (!data.invoiceNumber.trim()) {
        ctx.addIssue({
          code: "custom",
          message: "Вкажіть номер податкової накладної",
          path: ["invoiceNumber"],
        });
      }
      if (!data.invoiceDate) {
        ctx.addIssue({
          code: "custom",
          message: "Вкажіть дату податкової накладної",
          path: ["invoiceDate"],
        });
      }
    });

  const form = superForm(defaults(zod4(partItemSchema)), {
    SPA: true,
    validators: zod4Client(partItemSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      // Fresh server-side quote at submit
      const quote = await fetchPartQuote(nativeForm.data);

      if (quote && quote.belowCost) {
        pendingSubmitData = nativeForm.data;
        pendingQuote = quote;
        priceConfirmOpen = true;
        return;
      }

      await submitPartItem(nativeForm.data, cancel);
    },
  });

  async function submitPartItem(
    data: z.infer<typeof partItemSchema>,
    cancel?: () => void,
  ) {
    const discountPayload = discountToPayload(
      data.discountType,
      data.discountValue,
    );

    const saleRate = data.vatRate.trim().replace(",", ".");
    const payload = {
      status: data.status as PartStatus,
      code: data.code,
      name: data.name,
      quantity: data.quantity,
      unitType: data.unitType,
      baseCost: data.cost,
      markup: data.markup,
      ...discountPayload,
      isFree: data.isFree,
      vatRate: saleRate || null,
      vatScheme: saleRate ? data.vatScheme : null,
      purchaseVatRate: data.purchaseVatRate.trim().replace(",", "."),
      purchasePriceIncludesVat: data.purchasePriceIncludesVat,
      purchaseScheme: data.purchaseScheme,
      supplierName: data.supplierName.trim() || null,
      supplierCode: data.supplierCode.trim() || null,
      invoiceNumber: data.invoiceNumber.trim() || null,
      invoiceDate: data.invoiceDate || null,
      vatCredited: data.vatCredited,
    };

    if (isEdit && partItemId) {
      await updatePartItemMutation
        .mutateAsync({ id: partItemId, payload })
        .catch(() => {});
    } else {
      await createPartItemMutation
        .mutateAsync({
          workOrderId: workOrderId!,
          status: data.status as PartStatus,
          code: data.code,
          name: data.name,
          quantity: data.quantity,
          unitType: data.unitType,
          baseCost: data.cost,
          markup: data.markup,
          ...discountPayload,
          isFree: data.isFree,
          vatRate: saleRate || null,
          vatScheme: saleRate ? data.vatScheme : null,
          purchaseVatRate: data.purchaseVatRate.trim().replace(",", "."),
          purchasePriceIncludesVat: data.purchasePriceIncludesVat,
          purchaseScheme: data.purchaseScheme,
          supplierName: data.supplierName.trim() || null,
          supplierCode: data.supplierCode.trim() || null,
          invoiceNumber: data.invoiceNumber.trim() || null,
          invoiceDate: data.invoiceDate || null,
          vatCredited: data.vatCredited,
        })
        .catch(() => {});
    }
    // Prevent superforms from re-applying the submitted data to the form
    // after the handler, which would overwrite the reset below and leave
    // stale values in the fields for the next time the sheet opens.
    cancel?.();
    open = false;
    form.reset();
    baseline = null;
    loadedKey = null;
  }

  async function confirmPriceAndSubmit() {
    priceConfirmOpen = false;
    if (pendingSubmitData) {
      await submitPartItem(pendingSubmitData);
      pendingSubmitData = null;
      pendingQuote = null;
    }
  }

  function cancelPriceConfirm() {
    priceConfirmOpen = false;
    pendingSubmitData = null;
    pendingQuote = null;
  }

  const { form: formData, enhance, submitting } = form;

  let baseline = $state<string | null>(null);
  let loadedKey = $state<string | null>(null);
  const dirty = $derived(
    baseline !== null && JSON.stringify($formData) !== baseline,
  );

  function markBaseline() {
    baseline = JSON.stringify($formData);
  }

  $effect(() => {
    if (!open) {
      baseline = null;
      loadedKey = null;
      return;
    }
    const key = isEdit ? String(partItemId) : "new";
    if (loadedKey === key) return;
    if (isEdit && !partItem) return;
    loadedKey = key;

    priceConfirmOpen = false;
    pendingSubmitData = null;
    pendingQuote = null;

    if (partItem) {
      $formData.status = partItem.status;
      $formData.code = partItem.code;
      $formData.name = partItem.name;
      $formData.quantity = partItem.quantity;
      $formData.unitType = partItem.unitType;
      $formData.isFree = partItem.isFree;
      $formData.cost = partItem.baseCost;
      $formData.markup = partItem.markup;
      $formData.discountType = partItem.discountType;
      $formData.discountValue = discountFromEntity(partItem);
      $formData.vatRate = partItem.vatRate ?? "";
      $formData.vatScheme = partItem.vatScheme ?? "standard";
      $formData.purchaseVatRate = partItem.purchaseVatRate;
      $formData.purchasePriceIncludesVat = partItem.purchasePriceIncludesVat;
      $formData.purchaseScheme = partItem.purchaseScheme;
      $formData.supplierName = partItem.supplierName ?? "";
      $formData.supplierCode = partItem.supplierCode ?? "";
      $formData.invoiceNumber = partItem.invoiceNumber ?? "";
      $formData.invoiceDate = partItem.invoiceDate ?? "";
      $formData.vatCredited = partItem.vatCredited;
    } else {
      form.reset();
      $formData.unitType = "штука";
      $formData.markup = defaultMarkup;
      $formData.purchaseVatRate =
        settingsState.settings?.defaultVatRate ?? "20";
      // Parts on warranty orders are free by default; uncheck to bill.
      $formData.isFree = workOrderType === "Warranty";
    }

    void tick().then(() => {
      if (!open || loadedKey !== key) return;
      markBaseline();
      setTimeout(() => {
        if (open && loadedKey === key) markBaseline();
      }, 150);
    });
  });
</script>

<MutationSheetShell
  bind:open
  {formID}
  {dirty}
  bodyClass="gap-0"
  title={isEdit ? "Редагувати запчастину" : "Створити запчастину"}
  description={isEdit
    ? "Змініть дані запчастини"
    : "Внесіть дані нової запчастини"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    baseline = null;
    loadedKey = null;
    open = false;
  }}
>
  <div class="flex flex-col gap-4">
    <Form.Field {form} name="status" class="space-y-1.5">
      <Form.Control>
        {#snippet children({ props })}
          <Label class={sheetLabel}>Статус</Label>
          <div class="block" {...props}>
            <PartStatusSelector bind:status={$formData.status as PartStatus} />
          </div>
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <div class="grid grid-cols-[minmax(0,0.8fr)_minmax(0,1.2fr)] gap-3">
      <Form.Field {form} name="code" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Код</Label>
            <Input
              {...props}
              data-sheet-autofocus
              bind:value={$formData.code}
              placeholder="FLT-001"
              class={sheetInput}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="name" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Назва<RequiredMark /></Label>
            <Input
              {...props}
              bind:value={$formData.name}
              placeholder="Наприклад: Фільтр масляний"
              class={sheetInput}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>

    <div
      class="grid grid-cols-[minmax(0,0.45fr)_minmax(0,1fr)] items-start gap-3"
    >
      <Form.Field {form} name="quantity" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Кількість<RequiredMark /></Label>
            <Input
              {...props}
              bind:value={$formData.quantity}
              type="text"
              inputmode="decimal"
              class={sheetInput}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <div class="space-y-1.5">
        <Form.Field {form} name="unitType" class="space-y-1.5">
          <Form.Control>
            {#snippet children({ props })}
              <Label class={sheetLabel}>
                Одиниці вимірювання<RequiredMark />
              </Label>
              <div class="flex items-center gap-1.5">
                <Input
                  {...props}
                  bind:value={$formData.unitType}
                  placeholder="штука"
                  class={cn(sheetInput, "min-w-0 flex-1")}
                />
                <div class="flex shrink-0 items-center gap-1.5">
                  {#each ["штука", "літр"] as unit (unit)}
                    <Button
                      type="button"
                      variant="outline"
                      size="xs"
                      class={sheetPresetButton}
                      onclick={() => ($formData.unitType = unit)}
                    >
                      {unit}
                    </Button>
                  {/each}
                </div>
              </div>
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>
    </div>

    <div class="grid grid-cols-2 items-start gap-3">
      <Form.Field {form} name="cost" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Ціна<RequiredMark /></Label>
            <div class="relative">
              <CurrencyInput
                {...props}
                bind:value={$formData.cost}
                class={cn(sheetInput, "pr-14")}
                onkeydown={(event: KeyboardEvent) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    const input = event.currentTarget as HTMLInputElement;
                    input.blur();
                    input.closest("form")?.requestSubmit();
                  }
                }}
              />
              <span
                class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground"
              >
                грн.
              </span>
            </div>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <Form.Field {form} name="markup" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Націнка<RequiredMark /></Label>
            <div class="relative">
              <Input
                {...props}
                bind:value={$formData.markup}
                type="text"
                inputmode="decimal"
                class={cn(sheetInput, "pr-8")}
              />
              <span
                class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground"
              >
                %
              </span>
            </div>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>

    <Form.Field {form} name="discountValue" class="space-y-1.5">
      <Form.Control>
        {#snippet children({ props })}
          <Label class={sheetLabel}>Знижка</Label>
          <DiscountField
            bind:type={$formData.discountType}
            bind:value={$formData.discountValue}
            inputProps={props}
            variant="sheet"
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
      {#if liveWarning}
        {#if liveWarning.kept !== liveWarning.discountedPrice}
          <p class="text-xs leading-relaxed text-amber-600 dark:text-amber-500">
            Увага: після сплати ПДВ залишиться {fmt(liveWarning.kept)} грн. (ціна
            {fmt(liveWarning.discountedPrice)} грн.), що нижче за собівартість {fmt(
              liveWarning.cost,
            )} грн.
          </p>
        {:else}
          <p class="text-xs leading-relaxed text-amber-600 dark:text-amber-500">
            Увага: кінцева ціна {fmt(liveWarning.discountedPrice)} грн. нижча за собівартість
            {fmt(liveWarning.cost)} грн.
          </p>
        {/if}
      {/if}
    </Form.Field>

    <Form.Field {form} name="isFree">
      <Form.Control>
        {#snippet children({ props })}
          <SheetSwitchRow
            id="part-is-free"
            label="Безкоштовно — не входить у вартість"
            checked={$formData.isFree}
            onCheckedChange={(checked) => ($formData.isFree = checked)}
            switchProps={props}
          >
            Коли клієнт не платить за позицію зовсім (гарантійна заміна,
            подарунок). Позиція не входить у вартість замовлення, і збиток за
            нею не фіксується. Якщо клієнт платить зі знижкою — використовуйте
            поле «Знижка» вище.
          </SheetSwitchRow>
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  </div>

  {#if businessIsVatPayer || livePartVat}
    <SheetSection title="ПДВ продажу">
      {#if livePartVat}
        <div class="mb-4 rounded-[10px] border border-border bg-card p-3">
          <div class="flex items-start justify-between gap-3">
            <div class="flex items-start gap-2">
              <ShieldCheckIcon
                class="mt-0.5 size-4 shrink-0 text-emerald-600"
              />
              <div class="space-y-0.5">
                <p class="text-sm font-medium">ПДВ закупівлі зафіксовано</p>
                <p class="text-xs text-muted-foreground">
                  {livePartVat.taxPoint} · ПН {livePartVat.documentNumber ??
                    "—"}
                </p>
                <p class="text-xs text-muted-foreground">
                  База {Big(livePartVat.base).toFixed(2)} грн · ПДВ {Big(
                    livePartVat.amount,
                  ).toFixed(2)} грн
                </p>
              </div>
            </div>
            {#if businessIsVatPayer}
              <Button
                variant="secondary"
                size="sm"
                onclick={() => (partVoidOpen = true)}
                disabled={voidPartVatMutation.isPending}
              >
                <RotateCcwIcon class="size-4" />
                Сторнувати
              </Button>
            {/if}
          </div>
          {#if !businessIsVatPayer}
            <p class="mt-2 text-xs text-muted-foreground">
              Історичний запис ПДВ. Бізнес зараз не є платником ПДВ, тому запис
              доступний лише для перегляду.
            </p>
          {/if}
        </div>
      {/if}

      {#if businessIsVatPayer}
        <fieldset disabled={partVatLocked} class="contents">
          <div class="flex flex-col gap-4">
            <div class="grid grid-cols-[minmax(0,0.7fr)_minmax(0,1.3fr)] gap-3">
              <Form.Field {form} name="vatRate" class="space-y-1.5">
                <Form.Control>
                  {#snippet children({ props })}
                    <Label class={sheetLabel}>Ставка ПДВ</Label>
                    <div class="relative">
                      <Input
                        {...props}
                        bind:value={$formData.vatRate}
                        type="text"
                        inputmode="decimal"
                        class={cn(sheetInput, "pr-8")}
                      />
                      <span
                        class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground"
                      >
                        %
                      </span>
                    </div>
                  {/snippet}
                </Form.Control>
                <Form.FieldErrors />
              </Form.Field>

              <Form.Field {form} name="vatScheme" class="space-y-1.5">
                <Form.Control>
                  {#snippet children({ props })}
                    <Label class={sheetLabel}>Режим ПДВ</Label>
                    <Select.Root
                      type="single"
                      value={$formData.vatScheme}
                      onValueChange={(value) => {
                        if (!value) return;
                        $formData.vatScheme = value as VatScheme;
                        if (schemeNeedsZeroRate($formData.vatScheme)) {
                          $formData.vatRate = "0";
                        }
                      }}
                    >
                      <Select.Trigger {...props} class={sheetSelectTrigger}>
                        {vatSchemeLabels[$formData.vatScheme]}
                      </Select.Trigger>
                      <Select.Content>
                        {#each VAT_SCHEMES as scheme (scheme)}
                          <Select.Item value={scheme}>
                            {vatSchemeLabels[scheme]}
                          </Select.Item>
                        {/each}
                      </Select.Content>
                    </Select.Root>
                  {/snippet}
                </Form.Control>
                <Form.FieldErrors />
              </Form.Field>
            </div>

            <p class={sheetHint}>Порожньо — ставка й режим замовлення.</p>
            {#if zeroRateStandardWarning}
              <p
                class="text-xs leading-relaxed text-amber-600 dark:text-amber-500"
              >
                0 % зі стандартним режимом не потрапить у реєстр ПДВ. Для
                нульової або пільгової позиції оберіть відповідний режим.
              </p>
            {/if}
          </div>
        </fieldset>
      {/if}
    </SheetSection>
  {/if}

  {#if businessIsVatPayer}
    <SheetSection title="ПДВ закупівлі">
      <fieldset disabled={partVatLocked} class="contents">
        <div class="flex flex-col gap-4">
          <div class="grid grid-cols-[minmax(0,0.7fr)_minmax(0,1.3fr)] gap-3">
            <Form.Field {form} name="purchaseVatRate" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Ставка ПДВ закупівлі</Label>
                  <div class="relative">
                    <Input
                      {...props}
                      bind:value={$formData.purchaseVatRate}
                      type="text"
                      inputmode="decimal"
                      class={cn(sheetInput, "pr-8")}
                    />
                    <span
                      class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground"
                    >
                      %
                    </span>
                  </div>
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>

            <Form.Field {form} name="purchaseScheme" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Режим ПДВ</Label>
                  <Select.Root
                    type="single"
                    value={$formData.purchaseScheme}
                    onValueChange={(value) => {
                      if (!value) return;
                      $formData.purchaseScheme = value as VatScheme;
                      if (schemeNeedsZeroRate($formData.purchaseScheme)) {
                        $formData.purchaseVatRate = "0";
                        $formData.vatCredited = false;
                      }
                    }}
                  >
                    <Select.Trigger {...props} class={sheetSelectTrigger}>
                      {vatSchemeLabels[$formData.purchaseScheme]}
                    </Select.Trigger>
                    <Select.Content>
                      {#each VAT_SCHEMES as scheme (scheme)}
                        <Select.Item value={scheme}>
                          {vatSchemeLabels[scheme]}
                        </Select.Item>
                      {/each}
                    </Select.Content>
                  </Select.Root>
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </div>

          <Form.Field {form} name="purchasePriceIncludesVat">
            <Form.Control>
              {#snippet children({ props })}
                <SheetSwitchRow
                  id="part-purchase-includes-vat"
                  label="Закупівельна ціна вже включає ПДВ"
                  checked={$formData.purchasePriceIncludesVat}
                  onCheckedChange={(checked) =>
                    ($formData.purchasePriceIncludesVat = checked)}
                  switchProps={props}
                />
              {/snippet}
            </Form.Control>
            <Form.FieldErrors />
          </Form.Field>

          <div class="grid grid-cols-2 gap-3">
            <Form.Field {form} name="supplierName" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Постачальник</Label>
                  <Input
                    {...props}
                    bind:value={$formData.supplierName}
                    class={sheetInput}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>

            <Form.Field {form} name="supplierCode" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Код (ЄДРПОУ / РНОКПП)</Label>
                  <Input
                    {...props}
                    bind:value={$formData.supplierCode}
                    class={sheetInput}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <Form.Field {form} name="invoiceNumber" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Номер податкової накладної</Label>
                  <Input
                    {...props}
                    bind:value={$formData.invoiceNumber}
                    class={sheetInput}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>

            <Form.Field {form} name="invoiceDate" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Дата податкової накладної</Label>
                  <DateTimePicker
                    bind:value={$formData.invoiceDate}
                    class={sheetInput}
                    {...props}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </div>

          <Form.Field {form} name="vatCredited">
            <Form.Control>
              {#snippet children({ props })}
                <SheetSwitchRow
                  id="part-vat-credited"
                  label="Включити ПДВ закупівлі до податкового кредиту"
                  checked={$formData.vatCredited}
                  onCheckedChange={(checked) =>
                    ($formData.vatCredited = checked)}
                  switchProps={props}
                >
                  {#if $formData.vatCredited}
                    До кредиту: {creditPreview.vat.toFixed(2)} грн (база {creditPreview.base.toFixed(
                      2,
                    )} грн)
                  {/if}
                </SheetSwitchRow>
              {/snippet}
            </Form.Control>
            <Form.FieldErrors />
          </Form.Field>

          {#if $formData.vatCredited && !$formData.purchasePriceIncludesVat}
            <p class={sheetHint}>
              Закупівельна ціна без ПДВ: до кредиту додається {creditPreview.vat.toFixed(
                2,
              )} грн, а собівартість запчастини — це вказана ціна.
            </p>
          {/if}
        </div>
      </fieldset>
    </SheetSection>
  {/if}
</MutationSheetShell>

<AlertDialog.Root bind:open={partVoidOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Сторнувати ПДВ закупівлі?</AlertDialog.Title>
      <AlertDialog.Description>
        Буде додано сторно-запис із датою коригування, а запчастину знову можна
        буде редагувати. Початковий запис залишиться в реєстрі.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <div class="grid gap-3 py-2">
      <div class="grid gap-1.5">
        <Label for="part-credit-note">Номер коригування</Label>
        <Input
          id="part-credit-note"
          bind:value={partCreditNoteNumber}
          placeholder="РК-000123"
        />
      </div>
      <div class="grid gap-1.5">
        <Label>Дата коригування</Label>
        <DateTimePicker bind:value={partCreditNoteDate} />
      </div>
    </div>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Скасувати</AlertDialog.Cancel>
      <AlertDialog.Action onclick={voidPartVat}>Сторнувати</AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<AlertDialog.Root bind:open={priceConfirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Ціна нижча за собівартість</AlertDialog.Title>
      <AlertDialog.Description>
        {#if pendingQuote && pendingQuote.belowCost}
          {#if pendingQuote.kept !== pendingQuote.discountedPrice}
            Після сплати ПДВ залишиться {fmt(pendingQuote.kept)} грн., що нижче за
            собівартість {fmt(pendingQuote.cost)} грн. (клієнт заплатить {fmt(
              pendingQuote.discountedPrice,
            )} грн.). Зберегти?
          {:else}
            Кінцева ціна становить{" "}
            {fmt(pendingQuote.discountedPrice)} грн., що нижче за собівартість{" "}
            {fmt(pendingQuote.cost)} грн. Зберегти?
          {/if}
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={cancelPriceConfirm}>
        Скасувати
      </AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmPriceAndSubmit}>
        Зберегти
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
