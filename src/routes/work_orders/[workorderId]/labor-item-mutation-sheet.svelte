<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { tick } from "svelte";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";
  import {
    sheetHint,
    sheetInput,
    sheetLabel,
    sheetSecondaryButton,
    sheetSelectTrigger,
  } from "$lib/components/sheet/field-styles";
  import RequiredMark from "$lib/components/sheet/required-mark.svelte";
  import SheetSection from "$lib/components/sheet/sheet-section.svelte";
  import SheetSwitchRow from "$lib/components/sheet/sheet-switch-row.svelte";

  import {
    commands,
    type DiscountType,
    type LaborPriceType,
    type PreviewLaborItemQuote,
    type VatScheme,
    type WorkOrderType,
  } from "$lib/bindings";
  import {
    discountFields,
    discountFromEntity,
    discountToPayload,
    refineDiscount,
  } from "$lib/schemas/zdiscount";
  import {
    laborPriceFields,
    laborPriceFromEntity,
    laborPriceToPayload,
    refineLaborPrice,
  } from "$lib/schemas/zlaborprice";
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
    createCreateLaborItemMutation,
    createLaborItemByIdQuery,
    createUpdateLaborItemMutation,
  } from "../queries";
  import LaborPriceField from "./labor-price-field.svelte";

  let {
    laborItemId,
    workOrderId,
    workOrderType = "Regular",
    open = $bindable(false),
  }: {
    laborItemId: number | null;
    workOrderId: string | null;
    workOrderType?: WorkOrderType;
    open: boolean;
  } = $props();

  const formID = "labor-item-mutation-form";
  const isEdit = $derived(laborItemId != null);

  async function fetchLaborQuote(data: {
    laborPriceType: LaborPriceType;
    laborPrice: string;
    discountType: DiscountType;
    discountValue: string;
    isFree: boolean;
  }): Promise<PreviewLaborItemQuote | null> {
    if (!workOrderId) return null;
    try {
      const isFixed = data.laborPriceType === "Fixed";
      return await unwrap(
        commands.previewLaborItem({
          workOrderId,
          priceType: data.laborPriceType,
          fixedPrice: isFixed ? data.laborPrice || "0" : "0",
          laborHours: isFixed ? "0" : data.laborPrice || "0",
          discountType: data.discountType,
          discountValue: data.discountValue || "0",
          isFree: data.isFree,
        }),
      );
    } catch {
      return null;
    }
  }

  let liveQuote = $state<PreviewLaborItemQuote | null>(null);
  let quoteSeq = 0;

  $effect(() => {
    if (!open || !workOrderId) return;
    const snapshot = {
      laborPriceType: $formData.laborPriceType as LaborPriceType,
      laborPrice: $formData.laborPrice,
      discountType: $formData.discountType as DiscountType,
      discountValue: $formData.discountValue,
      isFree: $formData.isFree,
    };
    const seq = ++quoteSeq;
    const timer = setTimeout(() => {
      fetchLaborQuote(snapshot).then((q) => {
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
  let pendingSubmitData = $state<z.infer<typeof laborItemSchema> | null>(null);
  let pendingQuote = $state<PreviewLaborItemQuote | null>(null);

  const queryClient = useQueryClient();

  const createLaborItemMutation = createCreateLaborItemMutation(queryClient);
  const updatLaborItemMutation = createUpdateLaborItemMutation(queryClient);

  let laborItemByIdQuery = createLaborItemByIdQuery(() => laborItemId);
  let laborItem = $derived(laborItemByIdQuery.data);

  const laborItemSchema = z
    .object({
      code: z.string().nullable(),
      name: z.string().min(1, { error: "Вкажіть назву" }),
      isFree: z.boolean().default(false),
      vatRate: z.string().trim().default(""),
      vatScheme: z
        .enum<
          VatScheme[]
        >(["standard", "zero", "exempt", "not_object", "export"])
        .default("standard"),
      ...laborPriceFields,
      ...discountFields,
    })
    .superRefine(refineLaborPrice)
    .superRefine(refineDiscount);

  const form = superForm(defaults(zod4(laborItemSchema)), {
    SPA: true,
    validators: zod4Client(laborItemSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      // Fresh server-side quote at submit
      const quote = await fetchLaborQuote(nativeForm.data);

      if (quote && quote.belowCost) {
        pendingSubmitData = nativeForm.data;
        pendingQuote = quote;
        priceConfirmOpen = true;
        return;
      }

      await submitLaborItem(nativeForm.data, cancel);
    },
  });

  async function submitLaborItem(
    data: z.infer<typeof laborItemSchema>,
    cancel?: () => void,
  ) {
    const laborPricePayload = laborPriceToPayload(
      data.laborPriceType,
      data.laborPrice,
    );

    const discountPayload = discountToPayload(
      data.discountType,
      data.discountValue,
    );
    const vatRate = data.vatRate.trim().replace(",", ".");
    const vatPayload = {
      vatRate: vatRate || null,
      vatScheme: vatRate ? data.vatScheme : null,
    };

    submitError = null;
    try {
      if (isEdit && laborItem) {
        await updatLaborItemMutation.mutateAsync({
          id: laborItem.id,
          payload: {
            serviceId: null,
            code: data.code,
            name: data.name,
            ...laborPricePayload,
            ...discountPayload,
            ...vatPayload,
            isFree: data.isFree,
          },
        });
      } else {
        await createLaborItemMutation.mutateAsync({
          workOrderId: workOrderId!,
          serviceId: null,
          code: data.code,
          name: data.name,
          ...laborPricePayload,
          ...discountPayload,
          ...vatPayload,
          isFree: data.isFree,
        });
      }
    } catch (error) {
      submitError =
        error instanceof Error ? error.message : "Не вдалося зберегти";
      return;
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
      await submitLaborItem(pendingSubmitData);
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
  let submitError = $state<string | null>(null);
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
      submitError = null;
      return;
    }
    const key = isEdit ? String(laborItemId) : "new";
    if (loadedKey === key) return;
    if (isEdit && !laborItem) return;
    loadedKey = key;

    priceConfirmOpen = false;
    pendingSubmitData = null;
    pendingQuote = null;

    if (laborItem) {
      $formData.code = laborItem.code;
      $formData.name = laborItem.name;
      $formData.isFree = laborItem.isFree;
      $formData.vatRate = laborItem.vatRate ?? "";
      $formData.vatScheme = laborItem.vatScheme ?? "standard";
      $formData.laborPriceType = laborItem.priceType;
      $formData.laborPrice = laborPriceFromEntity(laborItem);
      $formData.discountType = laborItem.discountType;
      $formData.discountValue = discountFromEntity(laborItem);
    } else {
      form.reset();
      // Warranty work is free by default (most claims cover labor too);
      // flip the switch when the client pays for the labor.
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

  $effect(() => {
    if (!priceConfirmOpen) return;
    const onKeydown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      cancelPriceConfirm();
    };
    window.addEventListener("keydown", onKeydown, true);
    return () => window.removeEventListener("keydown", onKeydown, true);
  });
</script>

{#snippet priceConfirmOverlay()}
  {#if priceConfirmOpen}
    <div
      class="absolute inset-0 z-20 flex items-center justify-center bg-background/70 p-6 backdrop-blur-sm"
      role="presentation"
      data-sheet-escape-owner="true"
      onclick={(event) => {
        if (event.target === event.currentTarget) cancelPriceConfirm();
      }}
    >
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="labor-price-confirm-title"
        aria-describedby="labor-price-confirm-description"
        class="w-full max-w-sm rounded-[12px] border border-border bg-popover p-5 shadow-2xl"
      >
        <h2
          id="labor-price-confirm-title"
          class="text-[15px] font-semibold tracking-[-0.01em]"
        >
          Ціна нижча за собівартість
        </h2>
        <p
          id="labor-price-confirm-description"
          class="mt-1.5 text-[13px] leading-relaxed text-muted-foreground"
        >
          {#if pendingQuote && pendingQuote.belowCost}
            {#if pendingQuote.kept !== pendingQuote.total}
              Після сплати ПДВ залишиться {fmt(pendingQuote.kept)} грн., що нижче
              за собівартість {fmt(pendingQuote.cost)} грн. (клієнт заплатить{" "}
              {fmt(pendingQuote.total)} грн.). Зберегти?
            {:else}
              Кінцева ціна становить {fmt(pendingQuote.total)} грн., що нижче за собівартість
              {fmt(pendingQuote.cost)} грн. Зберегти?
            {/if}
          {/if}
        </p>
        <div class="mt-5 flex items-center justify-end gap-2">
          <Button
            type="button"
            variant="ghost"
            class={cn(sheetSecondaryButton, "text-muted-foreground")}
            onclick={cancelPriceConfirm}
          >
            Скасувати
          </Button>
          <Button
            type="button"
            variant="destructive"
            class={sheetSecondaryButton}
            onclick={confirmPriceAndSubmit}
          >
            Зберегти
          </Button>
        </div>
      </div>
    </div>
  {/if}
{/snippet}

<MutationSheetShell
  bind:open
  {formID}
  {dirty}
  bodyClass="gap-0"
  title={isEdit ? "Редагувати сервіс" : "Створити сервіс"}
  description={isEdit ? "Змініть дані сервісу" : "Внесіть дані нового сервісу"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  error={submitError}
  enhanceAction={enhance}
  overlay={priceConfirmOverlay}
  onCancel={() => {
    form.reset();
    baseline = null;
    loadedKey = null;
    submitError = null;
    open = false;
  }}
>
  <div class="flex flex-col gap-4">
    <div class="grid grid-cols-[minmax(0,0.8fr)_minmax(0,1.2fr)] gap-3">
      <Form.Field {form} name="code" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Код</Label>
            <Input
              {...props}
              data-sheet-autofocus
              bind:value={$formData.code}
              placeholder="LAB-001"
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
              placeholder="Наприклад: Заміна ременя ГРМ"
              class={sheetInput}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </div>

    <div class="flex flex-col gap-4">
      <Form.Field {form} name="laborPrice" class="space-y-1.5">
        <Form.Control>
          {#snippet children({ props })}
            <Label class={sheetLabel}>Вартість<RequiredMark /></Label>
            <LaborPriceField
              bind:type={$formData.laborPriceType}
              bind:value={$formData.laborPrice}
              inputProps={props}
            />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

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
          <div
            class="rounded-[8px] border border-rose-500/35 bg-rose-500/10 px-3 py-2.5"
          >
            {#if liveWarning.kept !== liveWarning.total}
              <p
                class="text-xs leading-relaxed text-rose-700 dark:text-rose-400"
              >
                Після сплати ПДВ залишиться {fmt(liveWarning.kept)} грн. (ціна
                {fmt(liveWarning.total)} грн.), що нижче за собівартість{" "}
                {fmt(liveWarning.cost)} грн.
              </p>
            {:else}
              <p
                class="text-xs leading-relaxed text-rose-700 dark:text-rose-400"
              >
                Кінцева ціна {fmt(liveWarning.total)} грн. нижча за собівартість
                {fmt(liveWarning.cost)} грн.
              </p>
            {/if}
          </div>
        {:else if liveQuote && !liveQuote.costBasisKnown}
          <p class={sheetHint}>
            Собівартість не відстежується для фіксованої ціни при погодинному
            типі витрат на роботу — перевірка на ціну нижче собівартості не
            виконується.
          </p>
        {/if}
      </Form.Field>

      <Form.Field {form} name="isFree">
        <Form.Control>
          {#snippet children({ props })}
            <SheetSwitchRow
              id="labor-is-free"
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
  </div>

  {#if !businessIsNotVatPayer}
    <SheetSection title="ПДВ">
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
          <p class="text-xs leading-relaxed text-amber-600 dark:text-amber-500">
            0 % зі стандартним режимом не потрапить у реєстр ПДВ. Для нульової
            або пільгової позиції оберіть відповідний режим.
          </p>
        {/if}
      </div>
    </SheetSection>
  {/if}
</MutationSheetShell>
