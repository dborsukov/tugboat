<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { goto } from "$app/navigation";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import { zBigString } from "$lib/schemas/zbig";
  import {
    discountFields,
    discountFromEntity,
    discountToPayload,
    refineDiscount,
  } from "$lib/schemas/zdiscount";
  import { sqidsEncode } from "$lib/tools";
  import type { CreateOrEditProps } from "$lib/types";

  import DiscountField from "./discount-field.svelte";
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

      let new_id = `WO-${sqidsEncode(Date.now())}`;

      const discountPayload = discountToPayload(
        nativeForm.data.discountType,
        nativeForm.data.discount,
      );

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
                odometerIn: nativeForm.data.odometerIn,
                customerComplaint:
                  workOrderDetails!.workOrder.customerComplaint,
                mechanicNotes: workOrderDetails!.workOrder.mechanicNotes,
                status: workOrderDetails!.workOrder.status,
                rate: nativeForm.data.rate,
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
                odometerIn: nativeForm.data.odometerIn,
                status: "Received",
                customerComplaint: null,
                mechanicNotes: null,
                rate: nativeForm.data.rate,
                ...discountPayload,
              })
              .catch(() => null);

      if (!result) return;
      if (!isEdit) await goto(`/work_orders/${new_id}`);
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (workOrderDetails) {
        $formData.odometerIn = workOrderDetails.workOrder.odometerIn;
        $formData.rate = workOrderDetails.workOrder.rate;
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
</MutationSheetShell>
