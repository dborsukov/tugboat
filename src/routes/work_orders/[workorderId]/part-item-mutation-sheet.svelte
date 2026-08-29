<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Form from "$lib/components/ui/form";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import MutationSheetShell from "$lib/components/mutation-sheet-shell.svelte";

  import { type DiscountType, type PartStatus } from "$lib/bindings";
  import { zBigString } from "$lib/schemas/zbig";
  import {
    discountFields,
    discountFromEntity,
    discountToPayload,
    refineDiscount,
  } from "$lib/schemas/zdiscount";
  import { settingsState } from "$lib/states/settings.svelte";

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

  /**
   * Computes the final sale price: base cost + markup, minus discount.
   * Returns null if any of the inputs aren't parseable numbers yet.
   */
  function computeFinalPrice(
    costStr: string,
    markupStr: string,
    discountType: DiscountType,
    discountStr: string,
  ): { cost: Big; finalPrice: Big } | null {
    try {
      const cost = new Big(costStr || "0");
      const markup = new Big(markupStr || "0");
      let finalPrice = cost.times(Big(1).plus(markup.div(100)));

      if (discountType === "Fixed") {
        finalPrice = finalPrice.minus(new Big(discountStr || "0"));
      } else if (discountType === "Percentage") {
        finalPrice = finalPrice.times(
          Big(1).minus(new Big(discountStr || "0").div(100)),
        );
      }

      return { cost, finalPrice };
    } catch {
      return null;
    }
  }

  let priceConfirmOpen = $state(false);
  let pendingSubmitData = $state<z.infer<typeof partItemSchema> | null>(null);

  const queryClient = useQueryClient();

  const createPartItemMutation = createCreatePartItemMutation(queryClient);
  const updatePartItemMutation = createUpdatePartItemMutation(queryClient);

  // Default markup for new parts comes from the app settings (shared cache).
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
      cost: zBigString({
        requiredError: "Вкажіть ціну",
        min: { value: 0, message: "Ціна не може бути від'ємним числом" },
      }).default("0"),
      markup: zBigString({
        requiredError: "Вкажіть націнку у відсотках",
        min: { value: 0, message: "Націнка не може бути від'ємним числом" },
        max: { value: 300, message: "Націнка має бути від 0% до 300%" },
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
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      const pricing = computeFinalPrice(
        nativeForm.data.cost,
        nativeForm.data.markup,
        nativeForm.data.discountType,
        nativeForm.data.discountValue,
      );

      if (pricing && pricing.finalPrice.lt(pricing.cost)) {
        // Final price undercuts base cost, halt submit
        pendingSubmitData = nativeForm.data;
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

    const payload = {
      status: data.status as PartStatus,
      code: data.code,
      name: data.name,
      quantity: data.quantity,
      unitType: data.unitType,
      baseCost: data.cost,
      markup: data.markup,
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
          status: data.status as PartStatus,
          code: data.code,
          name: data.name,
          quantity: data.quantity,
          unitType: data.unitType,
          baseCost: data.cost,
          markup: data.markup,
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
      await submitPartItem(pendingSubmitData);
      pendingSubmitData = null;
    }
  }

  function cancelPriceConfirm() {
    priceConfirmOpen = false;
    pendingSubmitData = null;
  }

  const { form: formData, enhance, submitting } = form;

  const pricingWarning = $derived.by(() => {
    const pricing = computeFinalPrice(
      $formData.cost,
      $formData.markup,
      $formData.discountType as DiscountType,
      $formData.discountValue,
    );
    if (pricing && pricing.finalPrice.lt(pricing.cost)) return pricing;
    return null;
  });

  $effect(() => {
    if (open) {
      priceConfirmOpen = false;
      pendingSubmitData = null;

      if (partItem) {
        $formData.status = partItem.status;
        $formData.code = partItem.code;
        $formData.name = partItem.name;
        $formData.quantity = partItem.quantity;
        $formData.unitType = partItem.unitType;
        $formData.cost = partItem.baseCost;
        $formData.markup = partItem.markup;
        $formData.discountType = partItem.discountType;
        $formData.discountValue = discountFromEntity(partItem);
      } else {
        form.reset();
        $formData.unitType = "штука";
        $formData.markup = defaultMarkup;
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
    {/if}
  </Form.Field>
</MutationSheetShell>

<AlertDialog.Root bind:open={priceConfirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Ціна нижча за собівартість</AlertDialog.Title>
      <AlertDialog.Description>
        {#if pendingSubmitData}
          {@const pricing = computeFinalPrice(
            pendingSubmitData.cost,
            pendingSubmitData.markup,
            pendingSubmitData.discountType as DiscountType,
            pendingSubmitData.discountValue,
          )}
          {#if pricing}
            Кінцева ціна зі знижкою становить {pricing.finalPrice.toFixed(2)} грн.,
            що нижче за собівартість {pricing.cost.toFixed(2)} грн. Зберегти запчастину
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
