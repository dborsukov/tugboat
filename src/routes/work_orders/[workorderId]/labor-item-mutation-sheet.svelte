<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import {
    discountFields,
    discountFromEntity,
    discountToPayload,
    refineDiscount,
  } from "../discount";
  import DiscountField from "../discount-field.svelte";
  import {
    createCreateLaborItemMutation,
    createLaborItemByIdQuery,
    createUpdateLaborItemMutation,
  } from "../queries";

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

  const queryClient = useQueryClient();

  const createLaborItemMutation = createCreateLaborItemMutation(queryClient);
  const updatLaborItemMutation = createUpdateLaborItemMutation(queryClient);

  let laborItemByIdQuery = createLaborItemByIdQuery(() => laborItemId);
  let laborItem = $derived(laborItemByIdQuery.data);

  const laborItemSchema = z
    .object({
      code: z.string().nullable(),
      name: z.string().min(1, { error: "Вкажіть назву" }),
      laborHours: z
        .number({ error: "Вкажіть кількість норм-годин" })
        .nonnegative("Неправильна кількість")
        .default(0),
      ...discountFields,
    })
    .superRefine(refineDiscount);

  const form = superForm(defaults(zod4(laborItemSchema)), {
    SPA: true,
    validators: zod4Client(laborItemSchema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm }) {
      if (!nativeForm.valid) return;

      const discountPayload = discountToPayload(
        nativeForm.data.discountType,
        nativeForm.data.discount,
      );

      if (isEdit && laborItem) {
        await updatLaborItemMutation
          .mutateAsync({
            id: laborItem.id,
            payload: {
              serviceId: null,
              code: nativeForm.data.code,
              name: nativeForm.data.name,
              laborHours: nativeForm.data.laborHours,
              ...discountPayload,
            },
          })
          .catch(() => {});
      } else {
        await createLaborItemMutation
          .mutateAsync({
            workOrderId: workOrderId!,
            serviceId: null,
            code: nativeForm.data.code,
            name: nativeForm.data.name,
            laborHours: nativeForm.data.laborHours,
            ...discountPayload,
          })
          .catch(() => {});
      }
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      if (laborItem) {
        $formData.code = laborItem.code;
        $formData.name = laborItem.name;
        $formData.laborHours = laborItem.laborHours;
        $formData.discountType = laborItem.discountType;
        $formData.discount = discountFromEntity(laborItem);
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

  <Form.Field {form} name="laborHours">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Норм-години</Label>
        <Input
          type="number"
          step="0.1"
          min="0"
          {...props}
          bind:value={$formData.laborHours}
        />
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
