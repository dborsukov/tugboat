<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import LockIcon from "@lucide/svelte/icons/lock";
  import SearchIcon from "@lucide/svelte/icons/search";

  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { Textarea } from "$lib/components/ui/textarea";

  import ComboSelect from "$lib/components/combo-select.svelte";
  import CurrencyInput from "$lib/components/currency-input.svelte";
  import DateTimePicker from "$lib/components/date-time-picker.svelte";
  import PhoneInput from "$lib/components/phone-input.svelte";
  import CarCard from "$lib/components/sheet/car-card.svelte";
  import CarPicker from "$lib/components/sheet/car-picker.svelte";
  import {
    sheetHint,
    sheetInput,
    sheetLabel,
    sheetPresetButton,
    sheetSecondaryButton,
    sheetSelectTrigger,
    sheetTextarea,
  } from "$lib/components/sheet/field-styles";
  import NotesDisclosure from "$lib/components/sheet/notes-disclosure.svelte";
  import RequiredMark from "$lib/components/sheet/required-mark.svelte";
  import SheetFrame from "$lib/components/sheet/sheet-frame.svelte";
  import SheetSection from "$lib/components/sheet/sheet-section.svelte";

  import {
    commands,
    type DiscountType,
    type LaborCostType,
    type PreviewWorkOrderQuote,
    type VatScheme,
  } from "$lib/bindings";
  import { createRecentItemsQuery } from "$lib/recent-items";
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
  import { cn } from "$lib/utils";
  import {
    schemeNeedsZeroRate,
    VAT_SCHEMES,
    vatSchemeLabels,
  } from "$lib/vat-scheme";

  import {
    createCarDetailsListQuery,
    createCarDetailsQuery,
  } from "../cars/queries";
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
    createWorkOrderDetailsListByCarIdQuery,
    createWorkOrderDetailsQuery,
  } from "./queries";

  let {
    active = true,
    session = 1,
    mode = "create",
    initialCarId = null,
    workOrderId = null,
    isLocked = false,
    allowPicker = false,
    allowCreateActions = false,
    scopedClientId = null,
    selectedCarId = $bindable<number | null>(null),
    pickerQuery = $bindable(""),
    onCancel,
    onCreated,
    onNewCarForClient,
    onNewClientAndCar,
    onBack,
    backLabel,
    stepLabel,
  }: {
    active?: boolean;
    session?: number;
    mode?: "create" | "edit";
    initialCarId?: number | null;
    workOrderId?: string | null;
    isLocked?: boolean;
    allowPicker?: boolean;
    allowCreateActions?: boolean;
    scopedClientId?: number | null;
    selectedCarId?: number | null;
    pickerQuery?: string;
    onCancel: () => void;
    onCreated: (id: string) => void;
    onNewCarForClient?: (clientId: number, clientName: string) => void;
    onNewClientAndCar?: () => void;
    onBack?: () => void;
    backLabel?: string;
    stepLabel?: string;
  } = $props();

  const formID = "work-order-sheet-form";
  const isEdit = $derived(mode === "edit");

  const queryClient = useQueryClient();
  const createWorkOrderMutation = createCreateWorkOrderMutation(queryClient);
  const updateWorkOrderMutation = createUpdateWorkOrderMutation(queryClient);

  const clientsQuery = createClientsQuery();
  const workersQuery = createWorkersQuery();
  const carDetailsListQuery = createCarDetailsListQuery();
  const recentItemsQuery = createRecentItemsQuery();
  const selectedCarQuery = createCarDetailsQuery(() => selectedCarId);
  const workOrderDetailsQuery = createWorkOrderDetailsQuery(() =>
    isEdit ? workOrderId : null,
  );
  const carOrdersQuery = createWorkOrderDetailsListByCarIdQuery(
    () => selectedCarId,
  );

  const clients = $derived(clientsQuery.data ?? []);
  const workers = $derived(workersQuery.data ?? []);
  const carDetailsList = $derived(carDetailsListQuery.data ?? []);
  const recentItems = $derived(recentItemsQuery.data ?? []);
  const recentCarIds = $derived(
    recentItems
      .filter((item) => item.kind === "car")
      .map((item) => Number(item.id)),
  );

  const workOrderDetails = $derived(workOrderDetailsQuery.data);
  const selectedCarDetails = $derived.by(() => {
    if (selectedCarId == null) return null;
    if (
      isEdit &&
      workOrderDetails &&
      workOrderDetails.workOrder.carId === selectedCarId
    ) {
      return {
        carDto: workOrderDetails.carDto,
        client: workOrderDetails.client,
      };
    }
    return (
      carDetailsList.find((item) => item.carDto.car.id === selectedCarId) ??
      selectedCarQuery.data ??
      null
    );
  });

  const clientsById = $derived(
    new Map(clients.map((client) => [client.id, client])),
  );

  const ownerClient = $derived(
    selectedCarDetails
      ? (clientsById.get(selectedCarDetails.carDto.car.clientId) ??
          selectedCarDetails.client ??
          null)
      : null,
  );

  const ownerName = $derived(ownerClient?.name ?? "");

  const driverFieldsEnabled = $derived(
    ownerClient?.clientType === "fop" || ownerClient?.clientType === "company",
  );

  const rateVariants = $derived(settingsState.settings?.rateVariants ?? []);
  const defaultRate = $derived(
    rateVariants.find((variant) => variant.isDefault) ?? null,
  );

  const businessIsNotVatPayer = $derived(
    settingsState.settings?.companyInfo == null ||
      settingsState.settings.companyInfo.vatPayer === false,
  );
  const businessIsVatPayer = $derived(!businessIsNotVatPayer);

  const previousMileage = $derived.by(() => {
    const orders = (carOrdersQuery.data ?? [])
      .filter(
        (item) =>
          item.workOrder.id !== workOrderId &&
          item.workOrder.archivedAt == null,
      )
      .sort(
        (a, b) =>
          new Date(b.workOrder.createdAt).getTime() -
          new Date(a.workOrder.createdAt).getTime(),
      );
    const latest = orders[0];
    if (!latest) return null;
    return latest.workOrder.odometerOut ?? latest.workOrder.odometerIn;
  });

  function formatMileage(value: number) {
    return new Intl.NumberFormat("uk-UA").format(value);
  }

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

  let liveQuote = $state<PreviewWorkOrderQuote | null>(null);
  let quoteSeq = 0;

  const workOrderSchema = z
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

  const { form: formData, enhance, submitting } = form;

  let baseline = $state<string | null>(null);
  let loadedKey = $state<string | null>(null);
  let appliedCarId = $state<number | null>(null);
  let hasSelectedCar = $state(false);
  let pickerOpen = $state(false);
  let partiesOpen = $state(false);
  let notesOpen = $state(false);
  let descriptionEl = $state<HTMLTextAreaElement | null>(null);
  let carSelectionDirty = $state(false);

  const dirty = $derived(
    selectedCarDetails != null &&
      baseline !== null &&
      (JSON.stringify($formData) !== baseline || carSelectionDirty),
  );

  let priceConfirmOpen = $state(false);
  let pendingSubmitData = $state<z.infer<typeof workOrderSchema> | null>(null);
  let pendingQuote = $state<PreviewWorkOrderQuote | null>(null);
  let priceCancelButton = $state<HTMLButtonElement | null>(null);

  $effect(() => {
    if (!priceConfirmOpen) return;
    void tick().then(() => priceCancelButton?.focus());
  });

  function markBaseline() {
    baseline = JSON.stringify($formData);
  }

  function applyCarDefaults() {
    $formData.ownerIsCustomer = true;
    $formData.ownerIsPayer = true;
    $formData.customerClientId = null;
    $formData.payerClientId = null;
  }

  function resetFormToDefaults() {
    form.reset();
    $formData.vatScheme = "standard";
    $formData.taxInvoiceNumber = "";
    $formData.taxInvoiceDate = "";
    $formData.exemptionCode = "";
    if (businessIsNotVatPayer) {
      $formData.vatRate = "0";
      $formData.pricesIncludeVat = true;
    } else {
      $formData.vatRate = settingsState.settings?.defaultVatRate ?? "20";
      $formData.pricesIncludeVat =
        settingsState.settings?.defaultPricesIncludeVat ?? true;
    }
    $formData.rate = defaultRate?.value ?? "0";
  }

  $effect(() => {
    if (session === 0) return;

    const key = `${session}:${mode}:${workOrderId ?? ""}:${initialCarId ?? ""}`;
    if (loadedKey === key) return;

    if (isEdit && workOrderDetailsQuery.isPending) return;
    if (!isEdit && clientsQuery.isLoading) return;

    loadedKey = key;
    partiesOpen = false;
    carSelectionDirty = false;
    priceConfirmOpen = false;
    pendingSubmitData = null;
    pendingQuote = null;

    if (isEdit && workOrderDetails) {
      const workOrder = workOrderDetails.workOrder;
      selectedCarId = workOrder.carId;
      appliedCarId = workOrder.carId;
      hasSelectedCar = true;
      pickerOpen = false;
      $formData.payerClientId = workOrder.payerClientId ?? null;
      $formData.ownerIsPayer = workOrder.payerClientId == null;
      $formData.customerClientId = workOrder.customerClientId ?? null;
      $formData.ownerIsCustomer = workOrder.customerClientId == null;
      $formData.workerId = workOrder.workerId ?? null;
      $formData.odometerIn = workOrder.odometerIn;
      $formData.odometerOut = workOrder.odometerOut;
      $formData.driverSurname = workOrder.driverSurname ?? "";
      $formData.driverFirstName = workOrder.driverFirstName ?? "";
      $formData.driverPatronymic = workOrder.driverPatronymic ?? "";
      $formData.driverPhone = workOrder.driverPhone ?? "";
      $formData.description = workOrder.description;
      $formData.conditionReport = workOrder.conditionReport;
      $formData.notes = workOrder.notes;
      $formData.rate = workOrder.rate;
      $formData.vatRate = workOrder.vatRate;
      $formData.vatScheme = workOrder.vatScheme;
      $formData.pricesIncludeVat = workOrder.pricesIncludeVat;
      $formData.taxInvoiceNumber = workOrder.taxInvoiceNumber ?? "";
      $formData.taxInvoiceDate = workOrder.taxInvoiceDate ?? "";
      $formData.exemptionCode = workOrder.exemptionCode ?? "";
      $formData.laborCostType = workOrder.laborCostType;
      $formData.laborCost = laborCostFromEntity(workOrder);
      $formData.discountType = workOrder.discountType;
      $formData.discountValue = discountFromEntity(workOrder);
      notesOpen = (workOrder.notes ?? "").trim() !== "";
    } else {
      resetFormToDefaults();
      notesOpen = false;
      selectedCarId = initialCarId ?? selectedCarId ?? null;
      appliedCarId = selectedCarId;
      hasSelectedCar = selectedCarId != null;
      pickerOpen = allowPicker && selectedCarId == null;
      if (selectedCarId != null) {
        applyCarDefaults();
        $formData.rate = defaultRate?.value ?? "0";
      }
    }

    let baselineTimer: ReturnType<typeof setTimeout> | undefined;

    void tick().then(() => {
      markBaseline();
      // Form controls (notably the formatted rate field) can emit their
      // display value one frame after mounting; take the baseline again once
      // they have settled so an untouched form is not considered dirty.
      baselineTimer = setTimeout(() => markBaseline(), 250);
      if (pickerOpen) return;
      descriptionEl?.focus();
      setTimeout(() => descriptionEl?.focus(), 120);
    });

    return () => clearTimeout(baselineTimer);
  });

  // A car can arrive from the picker or from a chained car-creation sheet.
  $effect(() => {
    const id = selectedCarId;
    if (id == null) {
      appliedCarId = null;
      return;
    }
    if (id === appliedCarId) return;
    appliedCarId = id;
    carSelectionDirty = true;
    pickerOpen = false;
    applyCarDefaults();
    if (active) void tick().then(() => descriptionEl?.focus());
    if (!isEdit && !hasSelectedCar) {
      $formData.rate = defaultRate?.value ?? "0";
      hasSelectedCar = true;
    }
  });

  $effect(() => {
    if (!active || !isEdit || !workOrderId) return;
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
      fetchWorkOrderQuote(snapshot).then((quote) => {
        if (seq === quoteSeq) liveQuote = quote;
      });
    }, 200);
    return () => clearTimeout(timer);
  });

  const liveWarning = $derived(
    liveQuote && liveQuote.belowCost ? liveQuote : null,
  );

  const fmt = (value: string) => Big(value).toFixed(2);

  const zeroRateStandardWarning = $derived.by(() => {
    if ($formData.vatScheme !== "standard") return false;
    if (!$formData.vatRate.trim()) return false;
    try {
      return Big($formData.vatRate.replace(",", ".")).eq(0);
    } catch {
      return false;
    }
  });

  const partyCustomer = $derived(
    $formData.ownerIsCustomer
      ? ownerClient
      : (clientsById.get($formData.customerClientId ?? -1) ?? null),
  );
  const partyPayer = $derived(
    $formData.ownerIsPayer
      ? ownerClient
      : (clientsById.get($formData.payerClientId ?? -1) ?? null),
  );
  const partiesAreSame = $derived(
    ownerClient != null &&
      partyCustomer?.id === ownerClient.id &&
      partyPayer?.id === ownerClient.id,
  );

  async function submitWorkOrder(
    data: z.infer<typeof workOrderSchema>,
    cancel?: () => void,
  ) {
    if (selectedCarId == null) {
      toast.error("Виберіть автомобіль", { position: "bottom-left" });
      return;
    }

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

    const result =
      isEdit && workOrderId
        ? await updateWorkOrderMutation
            .mutateAsync({
              id: workOrderId,
              carId: selectedCarId,
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
              carId: selectedCarId,
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

    cancel?.();
    baseline = null;
    loadedKey = null;
    form.reset();
    onCreated(result.id);
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

  function handleCancel() {
    form.reset();
    baseline = null;
    loadedKey = null;
    onCancel();
  }

  function selectCar(id: number) {
    selectedCarId = id;
    pickerOpen = false;
    void tick().then(() => descriptionEl?.focus());
  }
</script>

{#snippet requiredMark()}
  <RequiredMark />
{/snippet}

<div class="relative flex min-h-0 flex-1 flex-col">
  <SheetFrame
    {formID}
    active={active && !priceConfirmOpen}
    {dirty}
    title={isEdit ? "Редагувати наряд-замовлення" : "Створити наряд-замовлення"}
    subtitle={isEdit
      ? "Змініть дані наряду-замовлення"
      : "Внесіть дані нового наряду-замовлення"}
    submitLabel={isEdit ? "Зберегти" : "Створити"}
    shortcutLabel={isEdit ? "зберегти" : "створити"}
    submitting={$submitting}
    submitDisabled={isLocked || selectedCarDetails == null}
    enhanceAction={enhance}
    onRequestClose={handleCancel}
    {onBack}
    {backLabel}
    {stepLabel}
    class="gap-0"
  >
    {#if isLocked}
      <div
        class="mb-4 flex items-center gap-2 rounded-[8px] border border-border bg-muted/30 px-3 py-2 text-xs text-muted-foreground"
      >
        <LockIcon class="size-3.5 shrink-0" />
        <span>Замовлення заблоковано від змін</span>
      </div>
    {/if}

    <SheetSection title="Автомобіль і клієнт" class="pt-0">
      {#if selectedCarDetails}
        <CarCard
          car={selectedCarDetails.carDto}
          {ownerName}
          onEdit={allowPicker && !isLocked
            ? () => (pickerOpen = !pickerOpen)
            : undefined}
        />
        {#if allowPicker && pickerOpen}
          <div class="mt-3">
            <CarPicker
              cars={carDetailsList}
              {clients}
              {recentCarIds}
              {scopedClientId}
              bind:query={pickerQuery}
              onSelect={selectCar}
              onClose={() => (pickerOpen = false)}
              onNewCarForClient={(client) =>
                onNewCarForClient?.(client.id, client.name)}
              {onNewClientAndCar}
              allowCreateActions={allowCreateActions && !isEdit}
              {active}
            />
          </div>
        {/if}
      {:else if allowPicker && pickerOpen}
        <CarPicker
          cars={carDetailsList}
          {clients}
          {recentCarIds}
          {scopedClientId}
          bind:query={pickerQuery}
          onSelect={selectCar}
          onClose={() => (pickerOpen = false)}
          onNewCarForClient={(client) =>
            onNewCarForClient?.(client.id, client.name)}
          {onNewClientAndCar}
          allowCreateActions={allowCreateActions && !isEdit}
          {active}
        />
      {:else if allowPicker}
        <button
          type="button"
          class="flex w-full items-center gap-3 rounded-[10px] border border-dashed border-border bg-card/60 px-3 py-2.5 text-left transition-colors hover:bg-muted/40"
          onclick={() => (pickerOpen = true)}
        >
          <span
            class="flex size-9 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
          >
            <SearchIcon class="size-4" />
          </span>
          <span class="min-w-0 flex-1">
            <span class="block text-sm font-medium">Обрати автомобіль</span>
            <span class="block text-xs text-muted-foreground">
              Номер, VIN, модель або власник
            </span>
          </span>
          <span class="shrink-0 text-xs font-medium text-brand"> Обрати </span>
        </button>
      {:else}
        <div class="rounded-[10px] border border-border bg-card px-3 py-3">
          <p class="text-xs text-muted-foreground">Автомобіль не вибрано</p>
        </div>
      {/if}
    </SheetSection>

    <div
      class={cn(
        "flex flex-col",
        selectedCarDetails == null &&
          "pointer-events-none opacity-40 select-none",
      )}
      inert={selectedCarDetails == null}
    >
      <fieldset disabled={isLocked} class="contents">
        <SheetSection title="Звернення">
          <div class="flex flex-col gap-4">
            <Form.Field {form} name="workerId" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props: formProps })}
                  <Label class={sheetLabel}>Виконавець</Label>
                  <ComboSelect
                    items={workers}
                    bind:value={$formData.workerId}
                    clearable
                    getKey={(worker) => worker.id}
                    getLabel={workerFullName}
                    getSearchValue={(worker) =>
                      `${workerFullName(worker)} ${worker.position ?? ""}`}
                    placeholder="Оберіть виконавця..."
                    searchPlaceholder="Пошук працівників..."
                    emptyText="Працівника не знайдено."
                    triggerClass={cn(sheetInput, "justify-between")}
                    contentClass="w-(--bits-floating-anchor-width) p-0"
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

            <Form.Field {form} name="description" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Опис</Label>
                  <Textarea
                    {...props}
                    bind:ref={descriptionEl}
                    data-sheet-autofocus
                    class={sheetTextarea}
                    value={$formData.description ?? ""}
                    oninput={(event) => {
                      $formData.description = event.currentTarget.value;
                    }}
                    placeholder="Короткий опис звернення: тип роботи, попередня діагностика, скарги клієнта тощо..."
                    rows={2}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>

            <Form.Field {form} name="conditionReport" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Огляд при прийнятті</Label>
                  <Textarea
                    {...props}
                    class={sheetTextarea}
                    value={$formData.conditionReport ?? ""}
                    oninput={(event) => {
                      $formData.conditionReport = event.currentTarget.value;
                    }}
                    placeholder="Опис зовнішнього стану кузова: пошкодження, подряпини, вм'ятини..."
                    rows={2}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </div>
        </SheetSection>

        <SheetSection title="Пробіг">
          <div class="grid grid-cols-2 gap-x-3 gap-y-4">
            <Form.Field {form} name="odometerIn" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>
                    При прийнятті{@render requiredMark()}
                  </Label>
                  <div class="relative">
                    <Input
                      type="number"
                      min="0"
                      {...props}
                      bind:value={$formData.odometerIn}
                      class={cn(sheetInput, "pr-10")}
                      placeholder="напр. 120 500"
                    />
                    <span
                      class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs whitespace-nowrap text-muted-foreground"
                    >
                      км
                    </span>
                  </div>
                  {#if previousMileage != null}
                    <p
                      class="text-[11.5px] leading-tight text-muted-foreground/80"
                    >
                      Минулого разу: {formatMileage(previousMileage)} км
                    </p>
                  {/if}
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>

            <Form.Field {form} name="odometerOut" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>При видачі</Label>
                  <div class="relative">
                    <Input
                      type="number"
                      min="0"
                      {...props}
                      bind:value={$formData.odometerOut}
                      class={cn(sheetInput, "pr-10")}
                      placeholder="напр. 120 800"
                    />
                    <span
                      class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs whitespace-nowrap text-muted-foreground"
                    >
                      км
                    </span>
                  </div>
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </div>
        </SheetSection>

        <SheetSection title="Оплата і сторони">
          <div class="flex flex-col gap-4">
            <Form.Field {form} name="rate" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>
                    Тариф{@render requiredMark()}
                  </Label>
                  <div class="relative">
                    <CurrencyInput
                      {...props}
                      bind:value={$formData.rate}
                      class={cn(sheetInput, "pr-20")}
                      onkeydown={(event: KeyboardEvent) => {
                        if (
                          event.key === "Enter" &&
                          !event.metaKey &&
                          !event.ctrlKey
                        ) {
                          event.preventDefault();
                          const input = event.currentTarget as HTMLInputElement;
                          input.blur();
                          input.closest("form")?.requestSubmit();
                        }
                      }}
                    />
                    <span
                      class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs whitespace-nowrap text-muted-foreground"
                    >
                      грн./год.
                    </span>
                  </div>
                  {#if rateVariants.length > 0}
                    <div class="flex flex-wrap gap-1.5 pt-0.5">
                      {#each rateVariants as variant (variant.name)}
                        <Button
                          type="button"
                          variant="outline"
                          size="xs"
                          class={sheetPresetButton}
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

            <div class="rounded-[10px] border border-border bg-card p-3">
              <div class="flex items-start gap-2">
                <div class="min-w-0 flex-1">
                  {#if partiesAreSame && ownerClient}
                    <p class="text-[12.5px] leading-snug">
                      <span class="text-muted-foreground"
                        >Замовник і платник:</span
                      >
                      <span class="font-medium"> {ownerClient.name}</span>
                      <span class="text-muted-foreground"> · власник</span>
                    </p>
                  {:else}
                    <div class="flex flex-col gap-1 text-[12.5px] leading-snug">
                      <p>
                        <span class="text-muted-foreground">Власник:</span>
                        {ownerClient?.name ?? "—"}
                      </p>
                      <p>
                        <span class="text-muted-foreground">Замовник:</span>
                        {partyCustomer?.name ?? "—"}
                      </p>
                      <p>
                        <span class="text-muted-foreground">Платник:</span>
                        {partyPayer?.name ?? "—"}
                      </p>
                    </div>
                  {/if}
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="xs"
                  class="h-7 shrink-0 rounded-[6px] px-2 text-xs font-medium text-brand hover:bg-brand/10 hover:text-brand"
                  onclick={() => (partiesOpen = !partiesOpen)}
                >
                  {partiesOpen ? "Готово" : "Змінити"}
                </Button>
              </div>

              {#if partiesOpen}
                <div
                  class="mt-3 flex flex-col gap-4 border-t border-border pt-3"
                >
                  <div class="flex flex-col gap-1.5">
                    <Label class={sheetLabel}>Власник</Label>
                    {#if ownerClient}
                      <ClientBadge name={ownerClient.name} iconSize="size-5" />
                    {:else}
                      <span class="text-xs text-muted-foreground">
                        Невідомий власник
                      </span>
                    {/if}
                  </div>

                  <Form.Field
                    {form}
                    name="customerClientId"
                    class="space-y-1.5"
                  >
                    <Form.Control>
                      {#snippet children({ props: formProps })}
                        <Label class={sheetLabel}>Замовник</Label>
                        <div class="flex items-center gap-2">
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
                          <Label
                            for="ownerIsCustomer"
                            class="text-[12.5px] font-normal"
                          >
                            Власник є замовником
                          </Label>
                        </div>
                        {#if !$formData.ownerIsCustomer}
                          <ComboSelect
                            items={clients}
                            bind:value={$formData.customerClientId}
                            clearable
                            getKey={(client) => client.id}
                            getLabel={(client) => client.name}
                            getSearchValue={(client) => client.name}
                            placeholder="Оберіть замовника..."
                            searchPlaceholder="Пошук клієнтів..."
                            emptyText="Клієнта не знайдено."
                            triggerClass={cn(sheetInput, "justify-between")}
                            contentClass="w-(--bits-floating-anchor-width) p-0"
                            {formProps}
                          >
                            {#snippet trigger(selected)}
                              {#if selected}
                                <ClientBadge
                                  name={selected.name}
                                  iconSize="size-5"
                                />
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

                  <Form.Field {form} name="payerClientId" class="space-y-1.5">
                    <Form.Control>
                      {#snippet children({ props: formProps })}
                        <Label class={sheetLabel}>Платник</Label>
                        <div class="flex items-center gap-2">
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
                          <Label
                            for="ownerIsPayer"
                            class="text-[12.5px] font-normal"
                          >
                            Власник є платником
                          </Label>
                        </div>
                        {#if !$formData.ownerIsPayer}
                          <ComboSelect
                            items={clients}
                            bind:value={$formData.payerClientId}
                            clearable
                            getKey={(client) => client.id}
                            getLabel={(client) => client.name}
                            getSearchValue={(client) => client.name}
                            placeholder="Оберіть платника..."
                            searchPlaceholder="Пошук клієнтів..."
                            emptyText="Клієнта не знайдено."
                            triggerClass={cn(sheetInput, "justify-between")}
                            contentClass="w-(--bits-floating-anchor-width) p-0"
                            {formProps}
                          >
                            {#snippet trigger(selected)}
                              {#if selected}
                                <ClientBadge
                                  name={selected.name}
                                  iconSize="size-5"
                                />
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
                </div>
              {/if}
            </div>

            {#if driverFieldsEnabled}
              <div class="flex flex-col gap-4">
                <p
                  class="flex items-center gap-2 text-[11.5px] font-bold tracking-[0.06em] text-muted-foreground/80 uppercase"
                >
                  Водій / представник
                </p>
                <div class="grid grid-cols-3 gap-x-3 gap-y-4">
                  <Form.Field {form} name="driverSurname" class="space-y-1.5">
                    <Form.Control>
                      {#snippet children({ props })}
                        <Label class={sheetLabel}>Прізвище</Label>
                        <Input
                          {...props}
                          class={sheetInput}
                          bind:value={$formData.driverSurname}
                        />
                      {/snippet}
                    </Form.Control>
                    <Form.FieldErrors />
                  </Form.Field>
                  <Form.Field {form} name="driverFirstName" class="space-y-1.5">
                    <Form.Control>
                      {#snippet children({ props })}
                        <Label class={sheetLabel}>Ім'я</Label>
                        <Input
                          {...props}
                          class={sheetInput}
                          bind:value={$formData.driverFirstName}
                        />
                      {/snippet}
                    </Form.Control>
                    <Form.FieldErrors />
                  </Form.Field>
                  <Form.Field
                    {form}
                    name="driverPatronymic"
                    class="space-y-1.5"
                  >
                    <Form.Control>
                      {#snippet children({ props })}
                        <Label class={sheetLabel}>По батькові</Label>
                        <Input
                          {...props}
                          class={sheetInput}
                          bind:value={$formData.driverPatronymic}
                        />
                      {/snippet}
                    </Form.Control>
                    <Form.FieldErrors />
                  </Form.Field>
                </div>
                <Form.Field {form} name="driverPhone" class="space-y-1.5">
                  <Form.Control>
                    {#snippet children({ props })}
                      <Label class={sheetLabel}>Телефон</Label>
                      <PhoneInput
                        {...props}
                        class={sheetInput}
                        triggerClass="h-9.5 rounded-[8px] border border-border bg-muted/40 px-2.5 focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20"
                        bind:value={$formData.driverPhone}
                      />
                    {/snippet}
                  </Form.Control>
                  <Form.FieldErrors />
                </Form.Field>
              </div>
            {/if}
          </div>
        </SheetSection>

        <SheetSection title="Знижка, ПДВ і собівартість">
          <div class="flex flex-col gap-4">
            <Form.Field {form} name="discountValue" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Знижка</Label>
                  <DiscountField
                    variant="sheet"
                    bind:type={$formData.discountType}
                    bind:value={$formData.discountValue}
                    inputProps={props}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>

            {#if businessIsVatPayer}
              <Form.Field {form} name="vatRate" class="space-y-1.5">
                <Form.Control>
                  {#snippet children({ props })}
                    <Label class={sheetLabel}>ПДВ</Label>
                    <div class="flex items-center gap-2">
                      <Checkbox
                        id="pricesIncludeVat"
                        checked={$formData.pricesIncludeVat}
                        onCheckedChange={(checked) =>
                          ($formData.pricesIncludeVat = checked === true)}
                      />
                      <Label
                        for="pricesIncludeVat"
                        class="text-[12.5px] font-normal"
                      >
                        Ціни вже включають ПДВ
                      </Label>
                    </div>
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
                    <div class="flex items-center gap-2 pt-0.5">
                      <Label for="vatScheme" class={sheetLabel}>Режим ПДВ</Label
                      >
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
                        <Select.Trigger
                          id="vatScheme"
                          class={cn(sheetSelectTrigger, "w-48")}
                        >
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
                      <div class="space-y-1.5">
                        <Label class={sheetLabel}>Код пільги</Label>
                        <Input
                          bind:value={$formData.exemptionCode}
                          class={sheetInput}
                          placeholder="З Довідника податкових пільг"
                        />
                        <p class={sheetHint}>
                          Потрібен для податкової накладної та декларації зі
                          звільнених операцій.
                        </p>
                      </div>
                    {/if}
                    <div class="grid grid-cols-2 gap-x-3 gap-y-4">
                      <div class="space-y-1.5">
                        <Label for="taxInvoiceNumber" class={sheetLabel}>
                          Номер податкової накладної
                        </Label>
                        <Input
                          id="taxInvoiceNumber"
                          bind:value={$formData.taxInvoiceNumber}
                          class={sheetInput}
                          placeholder="ПН-000123"
                        />
                      </div>
                      <div class="space-y-1.5">
                        <Label class={sheetLabel}>
                          Дата податкової накладної
                        </Label>
                        <DateTimePicker
                          bind:value={$formData.taxInvoiceDate}
                          class={cn(sheetInput, "justify-start")}
                        />
                      </div>
                    </div>
                    {#if zeroRateStandardWarning}
                      <p
                        class="text-xs leading-relaxed text-amber-600 dark:text-amber-500"
                      >
                        0 % зі стандартним режимом не потрапить у реєстр ПДВ.
                        Для нульової поставки оберіть «Нульова», «Експорт» або
                        «Звільнена».
                      </p>
                    {/if}
                  {/snippet}
                </Form.Control>
                <Form.FieldErrors />
              </Form.Field>
            {/if}

            <Form.Field {form} name="laborCost" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Собівартість робіт</Label>
                  <LaborCostField
                    variant="sheet"
                    bind:type={$formData.laborCostType}
                    bind:value={$formData.laborCost}
                    inputProps={props}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
              {#if liveWarning}
                <div
                  class="rounded-[8px] border border-rose-500/35 bg-rose-500/10 px-3 py-2.5"
                >
                  {#if Big(liveWarning.vatAmount).gt(0)}
                    <p
                      class="text-xs leading-relaxed text-rose-700 dark:text-rose-400"
                    >
                      Після сплати ПДВ залишиться {fmt(
                        liveWarning.vatBase,
                      )}{" "}
                      грн., що нижче за собівартість робіт і запчастин{" "}
                      {fmt(liveWarning.combinedCost)} грн.
                    </p>
                  {:else}
                    <p
                      class="text-xs leading-relaxed text-rose-700 dark:text-rose-400"
                    >
                      Кінцева сума {fmt(liveWarning.totalWithVat)} грн. нижча за собівартість
                      робіт і запчастин{" "}
                      {fmt(liveWarning.combinedCost)} грн.
                    </p>
                  {/if}
                </div>
              {/if}
            </Form.Field>
          </div>
        </SheetSection>

        <SheetSection title="Додатково">
          <NotesDisclosure bind:expanded={notesOpen}>
            <Form.Field {form} name="notes" class="space-y-1.5">
              <Form.Control>
                {#snippet children({ props })}
                  <Label class={sheetLabel}>Нотатки</Label>
                  <Textarea
                    {...props}
                    class={sheetTextarea}
                    value={$formData.notes ?? ""}
                    oninput={(event) => {
                      $formData.notes = event.currentTarget.value;
                    }}
                    placeholder="Загальні нотатки щодо замовлення..."
                    rows={3}
                  />
                {/snippet}
              </Form.Control>
              <Form.FieldErrors />
            </Form.Field>
          </NotesDisclosure>
        </SheetSection>
      </fieldset>
    </div>
  </SheetFrame>

  {#if priceConfirmOpen}
    <div
      class="absolute inset-0 z-20 flex items-center justify-center bg-background/70 p-6 backdrop-blur-sm"
      role="presentation"
      onclick={(event) => {
        if (event.target === event.currentTarget) cancelPriceConfirm();
      }}
      onkeydown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          cancelPriceConfirm();
          return;
        }
        if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
          event.preventDefault();
          void confirmPriceAndSubmit();
        }
      }}
    >
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="price-confirm-title"
        aria-describedby="price-confirm-description"
        class="w-full max-w-sm rounded-[12px] border border-border bg-popover p-5 shadow-2xl"
      >
        <h2
          id="price-confirm-title"
          class="text-[15px] font-semibold tracking-[-0.01em]"
        >
          Ціна нижча за собівартість
        </h2>
        <p
          id="price-confirm-description"
          class="mt-1.5 text-[13px] leading-relaxed text-muted-foreground"
        >
          {#if pendingQuote && pendingQuote.belowCost}
            {#if Big(pendingQuote.vatAmount).gt(0)}
              Після сплати ПДВ у вас залишиться {fmt(pendingQuote.vatBase)} грн.,
              що нижче за собівартість робіт і запчастин{" "}
              {fmt(pendingQuote.combinedCost)} грн. (клієнт заплатить{" "}
              {fmt(pendingQuote.totalWithVat)} грн.). Зберегти наряд-замовлення з
              такою ціною?
            {:else}
              Кінцева сума становить {fmt(pendingQuote.totalWithVat)} грн., що нижче
              за собівартість робіт і запчастин{" "}
              {fmt(pendingQuote.combinedCost)} грн. Зберегти?
            {/if}
          {/if}
        </p>
        <div class="mt-5 flex items-center justify-end gap-2">
          <Button
            bind:ref={priceCancelButton}
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
</div>
