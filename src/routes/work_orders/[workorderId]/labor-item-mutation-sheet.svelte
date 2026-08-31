<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import {
    type DiscountType,
    type LaborPriceType,
    type WorkOrder,
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

  import DiscountField from "../discount-field.svelte";
  import {
    createCreateLaborItemMutation,
    createLaborItemByIdQuery,
    createUpdateLaborItemMutation,
    createWorkOrderDetailsQuery,
  } from "../queries";
  import LaborPriceField from "./labor-price-field.svelte";

  let {
    laborItemId,
    workOrderId,
    open = $bindable(false),
  }: {
    laborItemId: number | null;
    workOrderId: string | null;
    open: boolean;
  } = $props();

  const formID = "labor-item-mutation-form";
  const isEdit = $derived(laborItemId != null);

  /**
   * Computes the labor cost basis and the final discounted price.
   *
   * Price (subtotal) depends on the item's own pricing mode:
   *  - Fixed: subtotal = laborPrice (the fixed price itself)
   *  - Hourly: subtotal = laborPrice (hours) * workOrder.rate
   *
   * Cost depends on the work order's labor cost settings:
   *  - Hourly: cost = hours worked * laborCostValue
   *  - Percentage: cost = subtotal * laborCostValue / 100
   *
   * When laborCostType is Hourly but this item is priced Fixed,
   * there's no hours-based cost basis so 'cost' comes out to 0.
   * This is when costBasisKnown becomes false.
   *
   * Returns null if any of the inputs aren't parseable numbers yet.
   */
  function computeLaborPricing(
    laborPriceType: LaborPriceType,
    laborPriceStr: string,
    discountType: DiscountType,
    discountStr: string,
    workOrder: WorkOrder | null | undefined,
  ): { cost: Big; finalPrice: Big; costBasisKnown: boolean } | null {
    if (!workOrder) return null;

    try {
      const laborPrice = new Big(laborPriceStr || "0");

      let subtotal: Big;
      if (laborPriceType === "Fixed") {
        subtotal = laborPrice;
      } else {
        subtotal = laborPrice.times(new Big(workOrder.rate || "0"));
      }

      const costBasisKnown = !(
        workOrder.laborCostType === "Hourly" && laborPriceType !== "Hourly"
      );

      let cost: Big;
      if (workOrder.laborCostType === "Hourly") {
        const hours = laborPriceType === "Hourly" ? laborPrice : new Big(0);
        cost = hours.times(new Big(workOrder.laborCostValue || "0"));
      } else {
        cost = subtotal.times(
          new Big(workOrder.laborCostValue || "0").div(100),
        );
      }

      let finalPrice = subtotal;
      if (discountType === "Fixed") {
        finalPrice = finalPrice.minus(new Big(discountStr || "0"));
      } else if (discountType === "Percentage") {
        finalPrice = finalPrice.times(
          Big(1).minus(new Big(discountStr || "0").div(100)),
        );
      }
      if (finalPrice.lt(0)) finalPrice = new Big(0);

      return { cost, finalPrice, costBasisKnown };
    } catch {
      return null;
    }
  }

  let priceConfirmOpen = $state(false);
  let pendingSubmitData = $state<z.infer<typeof laborItemSchema> | null>(null);

  const queryClient = useQueryClient();

  const createLaborItemMutation = createCreateLaborItemMutation(queryClient);
  const updatLaborItemMutation = createUpdateLaborItemMutation(queryClient);

  let laborItemByIdQuery = createLaborItemByIdQuery(() => laborItemId);
  let laborItem = $derived(laborItemByIdQuery.data);

  let workOrderDetailsQuery = createWorkOrderDetailsQuery(() => workOrderId);
  let workOrderDetails = $derived(workOrderDetailsQuery.data?.workOrder);

  const laborItemSchema = z
    .object({
      code: z.string().nullable(),
      name: z.string().min(1, { error: "Вкажіть назву" }),
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

      const pricing = computeLaborPricing(
        nativeForm.data.laborPriceType,
        nativeForm.data.laborPrice,
        nativeForm.data.discountType,
        nativeForm.data.discountValue,
        workOrderDetails,
      );

      if (
        pricing &&
        pricing.costBasisKnown &&
        pricing.finalPrice.lt(pricing.cost)
      ) {
        // Final price undercuts labor cost, halt submit
        pendingSubmitData = nativeForm.data;
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

    if (isEdit && laborItem) {
      await updatLaborItemMutation
        .mutateAsync({
          id: laborItem.id,
          payload: {
            serviceId: null,
            code: data.code,
            name: data.name,
            ...laborPricePayload,
            ...discountPayload,
          },
        })
        .catch(() => {});
    } else {
      await createLaborItemMutation
        .mutateAsync({
          workOrderId: workOrderId!,
          serviceId: null,
          code: data.code,
          name: data.name,
          ...laborPricePayload,
          ...discountPayload,
        })
        .catch(() => {});
    }
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
      await submitLaborItem(pendingSubmitData);
      pendingSubmitData = null;
    }
  }

  function cancelPriceConfirm() {
    priceConfirmOpen = false;
    pendingSubmitData = null;
  }

  const { form: formData, enhance, submitting } = form;

  const pricingWarning = $derived.by(() => {
    const pricing = computeLaborPricing(
      $formData.laborPriceType as LaborPriceType,
      $formData.laborPrice,
      $formData.discountType as DiscountType,
      $formData.discountValue,
      workOrderDetails,
    );
    if (
      pricing &&
      pricing.costBasisKnown &&
      pricing.finalPrice.lt(pricing.cost)
    )
      return pricing;
    return null;
  });

  const costBasisUnknown = $derived(
    workOrderDetails?.laborCostType === "Hourly" &&
      $formData.laborPriceType === "Fixed",
  );

  $effect(() => {
    if (open) {
      priceConfirmOpen = false;
      pendingSubmitData = null;

      if (laborItem) {
        $formData.code = laborItem.code;
        $formData.name = laborItem.name;
        $formData.laborPriceType = laborItem.priceType;
        $formData.laborPrice = laborPriceFromEntity(laborItem);
        $formData.discountType = laborItem.discountType;
        $formData.discountValue = discountFromEntity(laborItem);
      } else {
        form.reset();
      }
    }
  });
</script>

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати сервіс" : "Створити сервіс"}
  description={isEdit ? "Змініть дані сервісу" : "Внесіть дані нового сервісу"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <Form.Field {form} name="code">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Код</Label>
        <Input {...props} bind:value={$formData.code} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="name">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Назва</Label>
        <Input {...props} bind:value={$formData.name} />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="laborPrice">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Вартість</Label>
        <LaborPriceField
          bind:type={$formData.laborPriceType}
          bind:value={$formData.laborPrice}
          inputProps={props}
        />
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
    {#if pricingWarning}
      <p class="text-sm text-amber-600 dark:text-amber-500 mt-1.5">
        Увага: кінцева ціна {pricingWarning.finalPrice.toFixed(2)} грн. нижча за собівартість
        {pricingWarning.cost.toFixed(2)} грн.
      </p>
    {:else if costBasisUnknown}
      <p class="text-sm text-muted-foreground mt-1.5">
        Собівартість не відстежується для фіксованої ціни при погодинному типі
        витрат на роботу — перевірка на ціну нижче собівартості не виконується.
      </p>
    {/if}
  </Form.Field>
</MutationSheetShell>

<AlertDialog.Root bind:open={priceConfirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Ціна нижча за собівартість</AlertDialog.Title>
      <AlertDialog.Description>
        {#if pendingSubmitData}
          {@const pricing = computeLaborPricing(
            pendingSubmitData.laborPriceType as LaborPriceType,
            pendingSubmitData.laborPrice,
            pendingSubmitData.discountType as DiscountType,
            pendingSubmitData.discountValue,
            workOrderDetails,
          )}
          {#if pricing && pricing.costBasisKnown}
            Кінцева ціна зі знижкою становить {pricing.finalPrice.toFixed(2)} грн.,
            що нижче за собівартість {pricing.cost.toFixed(2)} грн. Зберегти роботу
            з такою ціною?
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
