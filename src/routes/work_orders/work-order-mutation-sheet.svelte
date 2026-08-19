<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { goto } from "$app/navigation";

  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import type {
    DiscountType,
    LaborCostType,
    LaborItemDto,
    WorkOrderDetailsDto,
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
  import { sqidsEncode } from "$lib/tools";
  import type { CreateOrEditProps } from "$lib/types";

  import DiscountField from "./discount-field.svelte";
  import LaborCostField from "./labor-cost-field.svelte";
  import {
    createCreateWorkOrderMutation,
    createUpdateWorkOrderMutation,
    createWorkOrderDetailsQuery,
  } from "./queries";

  type Props = CreateOrEditProps<{ carId: number }, { workOrderId: string }> & {
    open: boolean;
  };

  let { open = $bindable(false), ...rest }: Props = $props();

  const formID = "work-order-mutation-form";
  const isEdit = $derived(rest.mode === "edit");
  const carId = $derived(rest.mode === "create" ? rest.carId : null);
  const workOrderId = $derived(rest.mode === "edit" ? rest.workOrderId : null);

  const queryClient = useQueryClient();

  const updateWorkOrderMutation = createUpdateWorkOrderMutation(queryClient);
  const createWorkOrderMutation = createCreateWorkOrderMutation(queryClient);

  let workOrderDetailsQuery = createWorkOrderDetailsQuery(() => workOrderId);
  let workOrderDetails = $derived(workOrderDetailsQuery.data);

  /**
   * Recomputes one labor item's subtotal/total/cost using a possibly-edited
   * rate and labor cost setting, mirroring calculate_labor_item_totals in pricing.rs
   */
  function computeLaborItemLive(
    item: LaborItemDto,
    rate: Big,
    laborCostType: LaborCostType,
    laborCostValue: Big,
  ): { total: Big; cost: Big } {
    const subtotal =
      item.priceType === "Hourly"
        ? new Big(item.laborHours).times(rate)
        : new Big(item.fixedPrice);

    let total = subtotal;
    if (item.discountType === "Fixed") {
      total = total.minus(new Big(item.discountAmount));
    } else if (item.discountType === "Percentage") {
      total = total.times(Big(1).minus(new Big(item.discountPercent).div(100)));
    }
    if (total.lt(0)) total = new Big(0);

    const cost =
      laborCostType === "Hourly"
        ? new Big(item.laborHours).times(laborCostValue)
        : subtotal.times(laborCostValue.div(100));

    return { total, cost };
  }

  /**
   * Combined (labor + parts) grand total after the work order's own
   * discount, and combined (labor + parts) cost. Labor items are recomputed
   * live since rate/laborCostType/laborCostValue are edited in this very form.
   *
   * Returns null if any of the inputs aren't parseable numbers yet or in create mode.
   */
  function computeWorkOrderPricing(
    rateStr: string,
    laborCostType: LaborCostType,
    laborCostStr: string,
    discountType: DiscountType,
    discountStr: string,
    workOrderDetails: WorkOrderDetailsDto | null | undefined,
  ): { grandTotal: Big; combinedCost: Big } | null {
    if (!workOrderDetails) return null;

    try {
      const rate = new Big(rateStr || "0");
      const laborCostValue = new Big(laborCostStr || "0");

      let laborTotal = new Big(0);
      let laborCost = new Big(0);
      for (const item of workOrderDetails.laborItems) {
        const { total, cost } = computeLaborItemLive(
          item,
          rate,
          laborCostType,
          laborCostValue,
        );
        laborTotal = laborTotal.plus(total);
        laborCost = laborCost.plus(cost);
      }

      const partsTotal = new Big(workOrderDetails.totals.partsTotal);
      const partsCost = new Big(workOrderDetails.totals.partsCost);

      const subtotal = laborTotal.plus(partsTotal);
      const combinedCost = laborCost.plus(partsCost);

      let grandTotal = subtotal;
      if (discountType === "Fixed") {
        grandTotal = grandTotal.minus(new Big(discountStr || "0"));
      } else if (discountType === "Percentage") {
        grandTotal = grandTotal.times(
          Big(1).minus(new Big(discountStr || "0").div(100)),
        );
      }
      console.log(
        laborTotal.toString(),
        laborCost.toString(),
        partsTotal.toString(),
        partsCost.toString(),
        subtotal.toString(),
        combinedCost.toString(),
        grandTotal.toString(),
      );
      if (grandTotal.lt(0)) grandTotal = new Big(0);

      return { grandTotal, combinedCost };
    } catch {
      return null;
    }
  }

  let priceConfirmOpen = $state(false);
  let pendingSubmitData = $state<z.infer<typeof workOrderSchema> | null>(null);

  export const workOrderSchema = z
    .object({
      odometerIn: z
        .number({ error: "Вкажіть пробіг" })
        .nonnegative("Неправильний пробіг")
        .default(0),
      rate: zBigString({
        requiredError: "Вкажіть тариф",
        min: { value: 0, message: "Тариф не може бути від'ємним числом" },
      }).default("0"),
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
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const pricing = computeWorkOrderPricing(
        nativeForm.data.rate,
        nativeForm.data.laborCostType,
        nativeForm.data.laborCost,
        nativeForm.data.discountType,
        nativeForm.data.discount,
        workOrderDetails,
      );

      if (pricing && pricing.grandTotal.lt(pricing.combinedCost)) {
        // Discounted grand total undercuts combined labor + parts cost, halt submit
        pendingSubmitData = nativeForm.data;
        priceConfirmOpen = true;
        return;
      }

      await submitWorkOrder(nativeForm.data);
    },
  });

  async function submitWorkOrder(data: z.infer<typeof workOrderSchema>) {
    let new_id = `WO-${sqidsEncode(Date.now())}`;

    const laborCostPayload = laborCostToPayload(
      data.laborCostType,
      data.laborCost,
    );

    const discountPayload = discountToPayload(data.discountType, data.discount);

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
              odometerIn: data.odometerIn,
              customerComplaint: workOrderDetails!.workOrder.customerComplaint,
              mechanicNotes: workOrderDetails!.workOrder.mechanicNotes,
              status: workOrderDetails!.workOrder.status,
              rate: data.rate,
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
              id: new_id,
              carId: carId!,
              odometerIn: data.odometerIn,
              status: "Received",
              customerComplaint: null,
              mechanicNotes: null,
              rate: data.rate,
              ...laborCostPayload,
              ...discountPayload,
            })
            .catch(() => null);

    if (!result) return;
    if (!isEdit) await goto(`/work_orders/${new_id}`);
    open = false;
    form.reset();
  }

  async function confirmPriceAndSubmit() {
    priceConfirmOpen = false;
    if (pendingSubmitData) {
      await submitWorkOrder(pendingSubmitData);
      pendingSubmitData = null;
    }
  }

  function cancelPriceConfirm() {
    priceConfirmOpen = false;
    pendingSubmitData = null;
  }

  const { form: formData, enhance, submitting } = form;

  const pricingWarning = $derived.by(() => {
    const pricing = computeWorkOrderPricing(
      $formData.rate,
      $formData.laborCostType,
      $formData.laborCost,
      $formData.discountType,
      $formData.discount,
      workOrderDetails,
    );
    if (pricing && pricing.grandTotal.lt(pricing.combinedCost)) return pricing;
    return null;
  });

  $effect(() => {
    if (open) {
      priceConfirmOpen = false;
      pendingSubmitData = null;

      if (workOrderDetails) {
        $formData.odometerIn = workOrderDetails.workOrder.odometerIn;
        $formData.rate = workOrderDetails.workOrder.rate;
        $formData.laborCostType = workOrderDetails.workOrder.laborCostType;
        $formData.laborCost = laborCostFromEntity(workOrderDetails.workOrder);
        $formData.discountType = workOrderDetails.workOrder.discountType;
        $formData.discount = discountFromEntity(workOrderDetails.workOrder);
      } else {
        form.reset();
      }
    }
  });
</script>

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
  <Form.Field {form} name="odometerIn">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Одометр(пробіг)</Label>
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

  <Form.Field {form} name="rate">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Тариф</Label>
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
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="discount">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Знижка</Label>
        <DiscountField
          bind:type={$formData.discountType}
          bind:value={$formData.discount}
          inputProps={props}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

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
    {#if pricingWarning}
      <p class="text-sm text-amber-600 dark:text-amber-500 mt-1.5">
        Увага: кінцева сума {pricingWarning.grandTotal.toFixed(2)} грн. нижча за собівартість
        робіт і запчастин {pricingWarning.combinedCost.toFixed(2)} грн.
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
          {@const pricing = computeWorkOrderPricing(
            pendingSubmitData.rate,
            pendingSubmitData.laborCostType,
            pendingSubmitData.laborCost,
            pendingSubmitData.discountType,
            pendingSubmitData.discount,
            workOrderDetails,
          )}
          {#if pricing}
            Кінцева сума зі знижкою становить {pricing.grandTotal.toFixed(2)} грн.,
            що нижче за собівартість робіт і запчастин {pricing.combinedCost.toFixed(
              2,
            )} грн. Зберегти наряд-замовлення з такою ціною?
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
