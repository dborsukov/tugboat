<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { goto } from "$app/navigation";

  import CarIcon from "@lucide/svelte/icons/car";
  import ClipboardListIcon from "@lucide/svelte/icons/clipboard-list";
  import CreditCardIcon from "@lucide/svelte/icons/credit-card";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import IdCardIcon from "@lucide/svelte/icons/id-card";
  import LockIcon from "@lucide/svelte/icons/lock";
  import UsersIcon from "@lucide/svelte/icons/users";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Textarea } from "$lib/components/ui/textarea";

  import ComboSelect from "$lib/components/combo-select.svelte";
  import CurrencyInput from "$lib/components/currency-input.svelte";
  import DateTimePicker from "$lib/components/date-time-picker.svelte";
  import MutationSheetSectionTitle from "$lib/components/mutation-sheet-section-title.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";
  import PhoneInput from "$lib/components/phone-input.svelte";

  import {
    commands,
    type DiscountType,
    type LaborCostType,
    type PreviewWorkOrderQuote,
    type VatScheme,
  } from "$lib/bindings";
  import { zBigString } from "$lib/schemas/zbig";
  import {
    discountFields,
    discountFromEntity,
    discountToPayload,
    refineDiscount,
  } from "$lib/schemas/zdiscount";
  import {
    laborCostFields,
    laborCostFromEntity,
    laborCostToPayload,
  } from "$lib/schemas/zlaborcost";
  import { settingsState } from "$lib/states/settings.svelte";
  import { unwrap } from "$lib/tools";
  import type { CreateOrEditProps } from "$lib/types";
  import { cn } from "$lib/utils";
  import {
    schemeNeedsZeroRate,
    VAT_SCHEMES,
    vatSchemeLabels,
  } from "$lib/vat-scheme";

  import { createCarDetailsQuery } from "../cars/queries";
  import ClientBadge from "../clients/client-badge.svelte";
  import { createClientsQuery } from "../clients/queries";
  import { workerFullName } from "../workers/labels";
  import { createWorkersQuery } from "../workers/queries";
  import WorkerOption from "../workers/worker-option.svelte";
  import DiscountField from "./discount-field.svelte";
  import LaborCostField from "./labor-cost-field.svelte";
  import {
    createCreateWorkOrderMutation,
    createUpdateWorkOrderMutation,
    createWorkOrderDetailsQuery,
  } from "./queries";

  type Props = CreateOrEditProps<{ carId: number }, { workOrderId: string }> & {
    open: boolean;
    isLocked?: boolean;
  };

  let { open = $bindable(false), isLocked = false, ...rest }: Props = $props();

  const formID = "work-order-mutation-form";
  const isEdit = $derived(rest.mode === "edit");
  const carId = $derived(rest.mode === "create" ? rest.carId : null);
  const workOrderId = $derived(rest.mode === "edit" ? rest.workOrderId : null);

  const queryClient = useQueryClient();

  const updateWorkOrderMutation = createUpdateWorkOrderMutation(queryClient);
  const createWorkOrderMutation = createCreateWorkOrderMutation(queryClient);

  const clientsQuery = createClientsQuery();
  const workersQuery = createWorkersQuery();
  const carDetailsQuery = createCarDetailsQuery(() => carId);

  const rateVariants = $derived(settingsState.settings?.rateVariants ?? []);
  const defaultRate = $derived(
    rateVariants.find((variant) => variant.isDefault) ?? null,
  );

  let workOrderDetailsQuery = createWorkOrderDetailsQuery(() => workOrderId);
  let workOrderDetails = $derived(workOrderDetailsQuery.data);

  const ownerClient = $derived(
    isEdit
      ? (workOrderDetails?.client ?? null)
      : (carDetailsQuery.data?.client ?? null),
  );

  const driverFieldsEnabled = $derived(
    ownerClient?.clientType === "fop" || ownerClient?.clientType === "company",
  );

  const businessIsNotVatPayer = $derived(
    settingsState.settings?.companyInfo == null ||
      settingsState.settings.companyInfo.vatPayer === false,
  );
  const businessIsVatPayer = $derived(!businessIsNotVatPayer);

  async function fetchWorkOrderQuote(data: {
    rate: string;
    laborCostType: LaborCostType;
    laborCost: string;
    discountType: DiscountType;
    discountValue: string;
    vatRate: string;
    vatScheme: VatScheme;
    pricesIncludeVat: boolean;
  }): Promise<PreviewWorkOrderQuote | null> {
    if (!isEdit || !workOrderId) return null;
    try {
      return await unwrap(
        commands.previewWorkOrder({
          workOrderId,
          rate: data.rate || "0",
          laborCostType: data.laborCostType,
          laborCostValue: data.laborCost || "0",
          discountType: data.discountType,
          discountValue: data.discountValue || "0",
          vatRate: data.vatRate || "0",
          vatScheme: data.vatScheme,
          pricesIncludeVat: data.pricesIncludeVat,
        }),
      );
    } catch {
      return null;
    }
  }

  // Live (debounced) quote for the inline warning while editing.
  let liveQuote = $state<PreviewWorkOrderQuote | null>(null);
  let quoteSeq = 0;

  $effect(() => {
    if (!open || !isEdit || !workOrderId) return;
    const snapshot = {
      rate: $formData.rate,
      laborCostType: $formData.laborCostType as LaborCostType,
      laborCost: $formData.laborCost,
      discountType: $formData.discountType as DiscountType,
      discountValue: $formData.discountValue,
      vatRate: $formData.vatRate,
      vatScheme: $formData.vatScheme as VatScheme,
      pricesIncludeVat: $formData.pricesIncludeVat,
    };
    const seq = ++quoteSeq;
    const timer = setTimeout(() => {
      fetchWorkOrderQuote(snapshot).then((q) => {
        if (seq === quoteSeq) liveQuote = q;
      });
    }, 200);
    return () => clearTimeout(timer);
  });

  const liveWarning = $derived(
    liveQuote && liveQuote.belowCost ? liveQuote : null,
  );

  const fmt = (v: string) => Big(v).toFixed(2);

  let priceConfirmOpen = $state(false);
  let pendingSubmitData = $state<z.infer<typeof workOrderSchema> | null>(null);
  let pendingQuote = $state<PreviewWorkOrderQuote | null>(null);

  export const workOrderSchema = z
    .object({
      ownerIsPayer: z.boolean().default(true),
      payerClientId: z.number().int().nullable().default(null),
      ownerIsCustomer: z.boolean().default(true),
      customerClientId: z.number().int().nullable().default(null),
      workerId: z.number().int().nullable().default(null),
      odometerIn: z
        .number({ error: "Вкажіть пробіг" })
        .nonnegative("Неправильний пробіг")
        .default(0),
      odometerOut: z
        .number({ error: "Неправильний пробіг" })
        .nonnegative("Неправильний пробіг")
        .nullable()
        .default(0),
      driverSurname: z.string().default(""),
      driverFirstName: z.string().default(""),
      driverPatronymic: z.string().default(""),
      driverPhone: z.string().default(""),
      description: z
        .string()
        .nullable()
        .default(null)
        .transform((val) => (val === "" ? null : val)),
      conditionReport: z
        .string()
        .nullable()
        .default(null)
        .transform((val) => (val === "" ? null : val)),
      notes: z
        .string()
        .nullable()
        .default(null)
        .transform((val) => (val === "" ? null : val)),
      rate: zBigString({
        requiredError: "Вкажіть тариф",
        min: { value: 0, message: "Тариф не може бути від'ємним числом" },
      }).default("0"),
      vatRate: zBigString({
        requiredError: "Вкажіть ставку ПДВ",
        min: { value: 0, message: "Ставка ПДВ не може бути від'ємним числом" },
        max: { value: 100, message: "Ставка ПДВ має бути в межах 100%" },
      }).default("20"),
      vatScheme: z
        .enum<
          VatScheme[]
        >(["standard", "zero", "exempt", "not_object", "export"])
        .default("standard"),
      pricesIncludeVat: z.boolean().default(true),
      taxInvoiceNumber: z
        .string()
        .trim()
        .max(60, { error: "Номер не може бути довшим за 60 символів" })
        .default(""),
      taxInvoiceDate: z
        .union([z.literal(""), z.string().regex(/^\d{4}-\d{2}-\d{2}$/)])
        .default(""),
      exemptionCode: z
        .string()
        .trim()
        .max(30, { error: "Код не може бути довшим за 30 символів" })
        .default(""),
      ...laborCostFields,
      ...discountFields,
    })
    .superRefine(refineDiscount);

  const form = superForm(defaults(zod4(workOrderSchema)), {
    SPA: true,
    validators: zod4Client(workOrderSchema),
    validationMethod: "oninput",
    resetForm: false,
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (isLocked) return;
      if (!nativeForm.valid) return;

      if (!isEdit) {
        await submitWorkOrder(nativeForm.data, cancel);
        return;
      }

      // Fresh server-side quote at submit
      const quote = await fetchWorkOrderQuote(nativeForm.data);

      if (quote && quote.belowCost) {
        pendingSubmitData = nativeForm.data;
        pendingQuote = quote;
        priceConfirmOpen = true;
        return;
      }

      await submitWorkOrder(nativeForm.data, cancel);
    },
  });

  async function submitWorkOrder(
    data: z.infer<typeof workOrderSchema>,
    cancel?: () => void,
  ) {
    const laborCostPayload = laborCostToPayload(
      data.laborCostType,
      data.laborCost,
    );

    const discountPayload = discountToPayload(
      data.discountType,
      data.discountValue,
    );

    const vatRate = !isEdit && businessIsNotVatPayer ? "0" : data.vatRate;
    const vatScheme: VatScheme =
      !isEdit && businessIsNotVatPayer ? "standard" : data.vatScheme;
    const taxInvoiceNumber = data.taxInvoiceNumber.trim() || null;
    const taxInvoiceDate = data.taxInvoiceDate || null;
    const exemptionCode =
      data.vatScheme === "exempt" ? data.exemptionCode.trim() || null : null;

    if (!isEdit && !carId && !workOrderDetails) {
      toast.error("Виникла помилка", {
        position: "bottom-left",
      });
    }

    const result =
      isEdit && workOrderId
        ? await updateWorkOrderMutation
            .mutateAsync({
              id: workOrderDetails!.workOrder.id,
              carId: workOrderDetails!.workOrder.carId,
              payerClientId: data.payerClientId,
              customerClientId: data.customerClientId,
              workerId: data.workerId,
              odometerIn: data.odometerIn,
              odometerOut: data.odometerOut,
              driverSurname: data.driverSurname.trim() || null,
              driverFirstName: data.driverFirstName.trim() || null,
              driverPatronymic: data.driverPatronymic.trim() || null,
              driverPhone: data.driverPhone.trim() || null,
              description: data.description?.trim() || null,
              conditionReport: data.conditionReport?.trim() || null,
              notes: data.notes?.trim() || null,
              status: workOrderDetails!.workOrder.status,
              rate: data.rate,
              vatRate: data.vatRate,
              vatScheme,
              pricesIncludeVat: data.pricesIncludeVat,
              taxInvoiceNumber,
              taxInvoiceDate,
              exemptionCode,
              ...laborCostPayload,
              ...discountPayload,
              startedAt: workOrderDetails!.workOrder.startedAt,
              completedAt: workOrderDetails!.workOrder.completedAt,
              paidAt: workOrderDetails!.workOrder.paidAt,
              paymentMethod: workOrderDetails!.workOrder.paymentMethod,
            })
            .catch(() => null)
        : await createWorkOrderMutation
            .mutateAsync({
              carId: carId!,
              payerClientId: data.payerClientId,
              customerClientId: data.customerClientId,
              workerId: data.workerId,
              odometerIn: data.odometerIn,
              odometerOut: data.odometerOut,
              driverSurname: data.driverSurname.trim() || null,
              driverFirstName: data.driverFirstName.trim() || null,
              driverPatronymic: data.driverPatronymic.trim() || null,
              driverPhone: data.driverPhone.trim() || null,
              status: "Received",
              description: data.description?.trim() || null,
              conditionReport: data.conditionReport?.trim() || null,
              notes: data.notes?.trim() || null,
              rate: data.rate,
              vatRate,
              vatScheme,
              pricesIncludeVat: data.pricesIncludeVat,
              taxInvoiceNumber,
              taxInvoiceDate,
              exemptionCode,
              ...laborCostPayload,
              ...discountPayload,
            })
            .catch(() => null);

    if (!result) return;
    if (!isEdit) await goto(`/work_orders/${result.id}`);
    // Prevent superforms from re-applying the submitted data to the form
    // after the handler, which would overwrite the reset below and leave
    // stale values in the fields for the next time the sheet opens.
    cancel?.();
    open = false;
    form.reset();
  }

  async function confirmPriceAndSubmit() {
    priceConfirmOpen = false;
    if (pendingSubmitData) {
      await submitWorkOrder(pendingSubmitData);
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

  const zeroRateStandardWarning = $derived.by(() => {
    if ($formData.vatScheme !== "standard") return false;
    if (!$formData.vatRate.trim()) return false;
    try {
      return Big($formData.vatRate.replace(",", ".")).eq(0);
    } catch {
      return false;
    }
  });

  $effect(() => {
    if (open) {
      priceConfirmOpen = false;
      pendingSubmitData = null;
      pendingQuote = null;

      if (workOrderDetails) {
        $formData.payerClientId =
          workOrderDetails.workOrder.payerClientId ?? null;
        $formData.ownerIsPayer =
          workOrderDetails.workOrder.payerClientId == null;
        $formData.customerClientId =
          workOrderDetails.workOrder.customerClientId ?? null;
        $formData.ownerIsCustomer =
          workOrderDetails.workOrder.customerClientId == null;
        $formData.workerId = workOrderDetails.workOrder.workerId ?? null;
        $formData.odometerIn = workOrderDetails.workOrder.odometerIn;
        $formData.odometerOut = workOrderDetails.workOrder.odometerOut;
        $formData.driverSurname =
          workOrderDetails.workOrder.driverSurname ?? "";
        $formData.driverFirstName =
          workOrderDetails.workOrder.driverFirstName ?? "";
        $formData.driverPatronymic =
          workOrderDetails.workOrder.driverPatronymic ?? "";
        $formData.driverPhone = workOrderDetails.workOrder.driverPhone ?? "";
        $formData.description = workOrderDetails.workOrder.description;
        $formData.conditionReport = workOrderDetails.workOrder.conditionReport;
        $formData.notes = workOrderDetails.workOrder.notes;
        $formData.rate = workOrderDetails.workOrder.rate;
        $formData.vatRate = workOrderDetails.workOrder.vatRate;
        $formData.vatScheme = workOrderDetails.workOrder.vatScheme;
        $formData.pricesIncludeVat =
          workOrderDetails.workOrder.pricesIncludeVat;
        $formData.taxInvoiceNumber =
          workOrderDetails.workOrder.taxInvoiceNumber ?? "";
        $formData.taxInvoiceDate =
          workOrderDetails.workOrder.taxInvoiceDate ?? "";
        $formData.exemptionCode =
          workOrderDetails.workOrder.exemptionCode ?? "";
        $formData.laborCostType = workOrderDetails.workOrder.laborCostType;
        $formData.laborCost = laborCostFromEntity(workOrderDetails.workOrder);
        $formData.discountType = workOrderDetails.workOrder.discountType;
        $formData.discountValue = discountFromEntity(
          workOrderDetails.workOrder,
        );
      } else {
        form.reset();
        $formData.vatScheme = "standard";
        $formData.taxInvoiceNumber = "";
        $formData.taxInvoiceDate = "";
        $formData.exemptionCode = "";
        if (businessIsNotVatPayer) {
          $formData.vatRate = "0";
          $formData.pricesIncludeVat = true;
        } else {
          // New orders take the VAT defaults from app settings.
          $formData.vatRate = settingsState.settings?.defaultVatRate ?? "20";
          $formData.pricesIncludeVat =
            settingsState.settings?.defaultPricesIncludeVat ?? true;
        }
        $formData.rate = defaultRate?.value ?? "0";
      }
    }
  });
</script>

{#snippet requiredHint()}
  <span
    class="text-destructive inline-flex align-middle"
    title="Обов'язкове поле"
    aria-label="Обов'язкове поле"
  >
    *
  </span>
{/snippet}

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати наряд-замовлення" : "Створити наряд-замовлення"}
  description={isEdit
    ? "Змініть дані наряду-замовлення"
    : "Внесіть дані нового наряду-замовлення"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  {#if isLocked}
    <div
      class="flex items-center gap-2 rounded border border-border px-3 py-2 text-sm text-muted-foreground"
    >
      <LockIcon class="size-4 shrink-0" />
      <span>Замовлення заблоковано від змін</span>
    </div>
  {/if}

  <fieldset disabled={isLocked} class="contents">
    <MutationSheetSectionTitle text="Виконавець" icon={WrenchIcon} />

    <Form.Field {form} name="workerId">
      <Form.Control>
        {#snippet children({ props: formProps })}
          <ComboSelect
            items={workersQuery.data ?? []}
            bind:value={$formData.workerId}
            clearable
            getKey={(w) => w.id}
            getLabel={workerFullName}
            getSearchValue={(w) => `${workerFullName(w)} ${w.position ?? ""}`}
            placeholder="Оберіть виконавця..."
            searchPlaceholder="Пошук працівників..."
            emptyText="Працівника не знайдено."
            triggerClass={cn($formData.workerId != null && "pl-2")}
            {formProps}
          >
            {#snippet trigger(selected)}
              {#if selected}
                <WorkerOption worker={selected} short />
              {:else}
                Оберіть виконавця...
              {/if}
            {/snippet}
            {#snippet item(worker)}
              <WorkerOption {worker} />
            {/snippet}
          </ComboSelect>
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <MutationSheetSectionTitle text="Опис" icon={ClipboardListIcon} />

    <Form.Field {form} name="description">
      <Form.Control>
        {#snippet children({ props })}
          <Textarea
            {...props}
            value={$formData.description ?? ""}
            oninput={(e) => {
              $formData.description = e.currentTarget.value;
            }}
            placeholder="Короткий опис звернення: тип роботи, попередня діагностика, скарги клієнта тощо..."
            rows={2}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <MutationSheetSectionTitle text="Автомобіль" icon={CarIcon} />

    <Form.Field {form} name="conditionReport">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Огляд при прийнятті</Label>
          <Textarea
            {...props}
            value={$formData.conditionReport ?? ""}
            oninput={(e) => {
              $formData.conditionReport = e.currentTarget.value;
            }}
            placeholder="Опис зовнішнього стану кузова на момент прийняття: наявні пошкодження, подряпини, вм'ятини, тощо..."
            rows={2}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="odometerIn">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Пробіг (на момент прийняття) {@render requiredHint()}</Label>
          <div class="flex gap-2 items-baseline">
            <Input
              type="number"
              min="0"
              {...props}
              bind:value={$formData.odometerIn}
              class="flex-1"
            />
            <p>км.</p>
          </div>
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="odometerOut">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Пробіг (на момент видачі)</Label>
          <div class="flex gap-2 items-baseline">
            <Input
              type="number"
              min="0"
              {...props}
              bind:value={$formData.odometerOut}
              class="flex-1"
            />
            <p>км.</p>
          </div>
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <MutationSheetSectionTitle text="Сторони" icon={UsersIcon} />

    <div class="flex flex-col gap-3">
      <Label>Власник</Label>
      {#if ownerClient}
        <ClientBadge name={ownerClient.name} iconSize="size-5" />
      {:else}
        <span class="text-sm text-muted-foreground">Невідомий власник</span>
      {/if}
    </div>

    <Form.Field {form} name="customerClientId">
      <Form.Control>
        {#snippet children({ props: formProps })}
          <Label>Замовник</Label>
          <div
            class={cn(
              "flex items-center gap-2",
              !$formData.ownerIsCustomer && "pb-1",
            )}
          >
            <Checkbox
              id="ownerIsCustomer"
              checked={$formData.ownerIsCustomer}
              onCheckedChange={(checked) => {
                $formData.ownerIsCustomer = checked === true;
                if ($formData.ownerIsCustomer) {
                  $formData.customerClientId = null;
                }
              }}
            />
            <Label for="ownerIsCustomer" class="font-normal">
              Власник є замовником
            </Label>
          </div>
          {#if !$formData.ownerIsCustomer}
            <ComboSelect
              items={clientsQuery.data ?? []}
              bind:value={$formData.customerClientId}
              clearable
              getKey={(c) => c.id}
              getLabel={(c) => c.name}
              getSearchValue={(c) => c.name}
              placeholder="Оберіть замовника..."
              searchPlaceholder="Пошук клієнтів..."
              emptyText="Клієнта не знайдено."
              triggerClass={cn($formData.customerClientId != null && "pl-2")}
              {formProps}
            >
              {#snippet trigger(selected)}
                {#if selected}
                  <ClientBadge name={selected.name} iconSize="size-5" />
                {:else}
                  Оберіть замовника...
                {/if}
              {/snippet}
              {#snippet item(client)}
                <ClientBadge name={client.name} />
              {/snippet}
            </ComboSelect>
          {/if}
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="payerClientId">
      <Form.Control>
        {#snippet children({ props: formProps })}
          <Label>Платник</Label>
          <div
            class={cn(
              "flex items-center gap-2",
              !$formData.ownerIsPayer && "pb-1",
            )}
          >
            <Checkbox
              id="ownerIsPayer"
              checked={$formData.ownerIsPayer}
              onCheckedChange={(checked) => {
                $formData.ownerIsPayer = checked === true;
                if ($formData.ownerIsPayer) {
                  $formData.payerClientId = null;
                }
              }}
            />
            <Label for="ownerIsPayer" class="font-normal">
              Власник є платником
            </Label>
          </div>
          {#if !$formData.ownerIsPayer}
            <ComboSelect
              items={clientsQuery.data ?? []}
              bind:value={$formData.payerClientId}
              clearable
              getKey={(c) => c.id}
              getLabel={(c) => c.name}
              getSearchValue={(c) => c.name}
              placeholder="Оберіть платника..."
              searchPlaceholder="Пошук клієнтів..."
              emptyText="Клієнта не знайдено."
              triggerClass={cn($formData.payerClientId != null && "pl-2")}
              {formProps}
            >
              {#snippet trigger(selected)}
                {#if selected}
                  <ClientBadge name={selected.name} iconSize="size-5" />
                {:else}
                  Оберіть платника...
                {/if}
              {/snippet}
              {#snippet item(client)}
                <ClientBadge name={client.name} />
              {/snippet}
            </ComboSelect>
          {/if}
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    {#if driverFieldsEnabled}
      <MutationSheetSectionTitle text="Водій / представник" icon={IdCardIcon} />

      <div class="grid grid-cols-1 gap-2 sm:grid-cols-3">
        <Form.Field {form} name="driverSurname">
          <Form.Control>
            {#snippet children({ props })}
              <Label>Прізвище</Label>
              <Input {...props} bind:value={$formData.driverSurname} />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
        <Form.Field {form} name="driverFirstName">
          <Form.Control>
            {#snippet children({ props })}
              <Label>Ім'я</Label>
              <Input {...props} bind:value={$formData.driverFirstName} />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
        <Form.Field {form} name="driverPatronymic">
          <Form.Control>
            {#snippet children({ props })}
              <Label>По-батькові</Label>
              <Input {...props} bind:value={$formData.driverPatronymic} />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>

      <Form.Field {form} name="driverPhone">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Телефон</Label>
            <PhoneInput {...props} bind:value={$formData.driverPhone} />
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    {/if}

    <MutationSheetSectionTitle text="Оплата" icon={CreditCardIcon} />

    <Form.Field {form} name="rate">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Тариф {@render requiredHint()}</Label>
          <div class="flex gap-2 items-baseline" {...props}>
            <CurrencyInput
              bind:value={$formData.rate}
              onkeydown={(e: KeyboardEvent) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  const input = e.currentTarget as HTMLInputElement;
                  input.blur();
                  input.closest("form")?.requestSubmit();
                }
              }}
            />
            <p>грн./год.</p>
          </div>
          {#if rateVariants.length > 0}
            <div class="flex flex-wrap gap-1.5 pt-1.5">
              {#each rateVariants as variant (variant.name)}
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  class="px-2.5 text-xs"
                  onclick={() => ($formData.rate = variant.value)}
                >
                  {variant.name}
                  <span class="font-normal text-muted-foreground">
                    {variant.value}
                  </span>
                </Button>
              {/each}
            </div>
          {/if}
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <Form.Field {form} name="discountValue">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Знижка</Label>
          <DiscountField
            bind:type={$formData.discountType}
            bind:value={$formData.discountValue}
            inputProps={props}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    {#if businessIsVatPayer}
      <Form.Field {form} name="vatRate">
        <Form.Control>
          {#snippet children({ props })}
            <Label>ПДВ</Label>
            <div class="flex items-center gap-2">
              <Checkbox
                id="pricesIncludeVat"
                checked={$formData.pricesIncludeVat}
                onCheckedChange={(checked) =>
                  ($formData.pricesIncludeVat = checked === true)}
              />
              <Label for="pricesIncludeVat" class="font-normal">
                Ціни вже включають ПДВ
              </Label>
            </div>
            <div class="flex gap-2 items-baseline pt-1" {...props}>
              <Input
                bind:value={$formData.vatRate}
                type="text"
                inputmode="decimal"
                class="flex-1"
              />
              <p>%</p>
            </div>
            <div class="flex items-center gap-2 pt-1">
              <Label for="vatScheme" class="font-normal">Режим ПДВ</Label>
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
                <Select.Trigger id="vatScheme" class="w-44">
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
            </div>
            {#if $formData.vatScheme === "exempt"}
              <div class="grid gap-1.5 pt-1">
                <Label for="exemptionCode" class="font-normal">Код пільги</Label
                >
                <Input
                  id="exemptionCode"
                  bind:value={$formData.exemptionCode}
                  placeholder="З Довідника податкових пільг"
                />
                <p class="text-xs text-muted-foreground">
                  Потрібен для податкової накладної та декларації зі звільнених
                  операцій.
                </p>
              </div>
            {/if}
            <div class="grid grid-cols-1 gap-2 pt-1 sm:grid-cols-2">
              <div class="grid gap-1.5">
                <Label for="taxInvoiceNumber" class="font-normal">
                  Номер податкової накладної
                </Label>
                <Input
                  id="taxInvoiceNumber"
                  bind:value={$formData.taxInvoiceNumber}
                  placeholder="ПН-000123"
                />
              </div>
              <div class="grid gap-1.5">
                <Label class="font-normal">Дата податкової накладної</Label>
                <DateTimePicker bind:value={$formData.taxInvoiceDate} />
              </div>
            </div>
            {#if zeroRateStandardWarning}
              <p class="text-sm text-amber-600 dark:text-amber-500 mt-1.5">
                0 % зі стандартним режимом не потрапить у реєстр ПДВ. Для
                нульової поставки оберіть «Нульова», «Експорт» або «Звільнена».
              </p>
            {/if}
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    {/if}

    <Form.Field {form} name="laborCost">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Собівартість робіт</Label>
          <LaborCostField
            bind:type={$formData.laborCostType}
            bind:value={$formData.laborCost}
            inputProps={props}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
      {#if liveWarning}
        {#if Big(liveWarning.vatAmount).gt(0)}
          <p class="text-sm text-amber-600 dark:text-amber-500 mt-1.5">
            Увага: після сплати ПДВ залишиться {fmt(liveWarning.vatBase)} грн., що
            нижче за собівартість робіт і запчастин{" "}
            {fmt(liveWarning.combinedCost)} грн.
          </p>
        {:else}
          <p class="text-sm text-amber-600 dark:text-amber-500 mt-1.5">
            Увага: кінцева сума {fmt(liveWarning.totalWithVat)} грн. нижча за собівартість
            робіт і запчастин{" "}
            {fmt(liveWarning.combinedCost)} грн.
          </p>
        {/if}
      {/if}
    </Form.Field>

    <MutationSheetSectionTitle text="Нотатки" icon={FileTextIcon} />

    <Form.Field {form} name="notes">
      <Form.Control>
        {#snippet children({ props })}
          <Textarea
            {...props}
            value={$formData.notes ?? ""}
            oninput={(e) => {
              $formData.notes = e.currentTarget.value;
            }}
            placeholder="Загальні нотатки щодо замовлення..."
            rows={3}
          />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>
  </fieldset>
</MutationSheetShell>

<AlertDialog.Root bind:open={priceConfirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Ціна нижча за собівартість</AlertDialog.Title>
      <AlertDialog.Description>
        {#if pendingQuote && pendingQuote.belowCost}
          {#if Big(pendingQuote.vatAmount).gt(0)}
            Після сплати ПДВ у вас залишиться {fmt(pendingQuote.vatBase)} грн., що
            нижче за собівартість робіт і запчастин{" "}
            {fmt(pendingQuote.combinedCost)} грн. (клієнт заплатить{" "}
            {fmt(pendingQuote.totalWithVat)} грн.). Зберегти наряд-замовлення з такою
            ціною?
          {:else}
            Кінцева сума становить {fmt(pendingQuote.totalWithVat)}{" "}
            грн., що нижче за собівартість робіт і запчастин{" "}
            {fmt(pendingQuote.combinedCost)} грн. Зберегти?
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
