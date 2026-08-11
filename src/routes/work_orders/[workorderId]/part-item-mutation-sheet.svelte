<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import { type PartStatus } from "$lib/bindings";
  import { zBigString } from "$lib/schemas/zbig";
  import {
    discountFields,
    discountFromEntity,
    discountToPayload,
    refineDiscount,
  } from "$lib/schemas/zdiscount";

  import DiscountField from "../discount-field.svelte";
  import {
    createCreatePartItemMutation,
    createPartItemByIdQuery,
    createUpdatePartItemMutation,
  } from "../queries";
  import PartStatusSelector from "./part-status-selector.svelte";

  let {
    partItemId = null,
    workOrderId = null,
    open = $bindable(false),
  }: {
    partItemId?: number | null;
    workOrderId?: string | null;
    open: boolean;
  } = $props();

  const formID = "part-item-mutation-form";
  const isEdit = $derived(partItemId != null);

  const queryClient = useQueryClient();

  const createPartItemMutation = createCreatePartItemMutation(queryClient);
  const updatePartItemMutation = createUpdatePartItemMutation(queryClient);

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
      cost: zBigString({
        requiredError: "Вкажіть ціну",
        min: { value: 0, message: "Ціна не може бути від'ємним числом" },
      }).default("0"),
      markup: zBigString({
        requiredError: "Вкажіть націнку у відсотках",
        min: {
          value: 0,
          message: "Знижка у відсотках має бути від 0% до 100%",
        },
        max: {
          value: 100,
          message: "Знижка у відсотках має бути від 0% до 100%",
        },
      }).default("0"),
      ...discountFields,
    })
    .superRefine(refineDiscount);

  const form = superForm(defaults(zod4(partItemSchema)), {
    SPA: true,
    validators: zod4Client(partItemSchema),
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

      const payload = {
        status: nativeForm.data.status as PartStatus,
        code: nativeForm.data.code,
        name: nativeForm.data.name,
        quantity: nativeForm.data.quantity,
        unitType: nativeForm.data.unitType,
        baseCost: nativeForm.data.cost,
        markup: nativeForm.data.markup,
        ...discountPayload,
      };

      if (isEdit && partItemId) {
        await updatePartItemMutation
          .mutateAsync({ id: partItemId, payload })
          .catch(() => {});
      } else {
        await createPartItemMutation
          .mutateAsync({
            workOrderId: workOrderId!,
            status: nativeForm.data.status as PartStatus,
            code: nativeForm.data.code,
            name: nativeForm.data.name,
            quantity: nativeForm.data.quantity,
            unitType: nativeForm.data.unitType,
            baseCost: nativeForm.data.cost,
            markup: nativeForm.data.markup,
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
      if (partItem) {
        $formData.status = partItem.status;
        $formData.code = partItem.code;
        $formData.name = partItem.name;
        $formData.quantity = partItem.quantity;
        $formData.unitType = partItem.unitType;
        $formData.cost = partItem.baseCost;
        $formData.markup = partItem.markup;
        $formData.discountType = partItem.discountType;
        $formData.discount = discountFromEntity(partItem);
      } else {
        form.reset();
        $formData.unitType = "штука";
      }
    }
  });
</script>

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати запчастину" : "Створити запчастину"}
  description={isEdit
    ? "Змініть дані запчастини"
    : "Внесіть дані нової запчастини"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <Form.Field {form} name="status">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Статус</Label>
        <div class="block" {...props}>
          <PartStatusSelector bind:status={$formData.status as PartStatus} />
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

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

  <Form.Field {form} name="quantity">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Кількість</Label>
        <Input
          bind:value={$formData.quantity}
          type="text"
          inputmode="decimal"
          {...props}
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="unitType">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Одиниці вимірювання</Label>
        <Input {...props} bind:value={$formData.unitType} />
      {/snippet}
    </Form.Control>
    <div class="flex gap-2 items-center">
      <Button
        variant="outline"
        size="sm"
        onclick={() => {
          $formData.unitType = "штука";
        }}>штука</Button
      >
      <Button
        variant="outline"
        size="sm"
        onclick={() => {
          $formData.unitType = "літр";
        }}>літр</Button
      >
    </div>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="cost">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Ціна</Label>
        <div class="flex gap-2 items-baseline" {...props}>
          <CurrencyInput
            bind:value={$formData.cost}
            onkeydown={(e: KeyboardEvent) => {
              if (e.key === "Enter") {
                e.preventDefault();
                const input = e.currentTarget as HTMLInputElement;
                input.blur();
                input.closest("form")?.requestSubmit();
              }
            }}
          />
          <p>грн.</p>
        </div>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="markup">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Націнка</Label>
        <div class="flex gap-2 items-baseline">
          <Input
            bind:value={$formData.markup}
            type="text"
            inputmode="decimal"
            {...props}
          />
          <p>%</p>
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
