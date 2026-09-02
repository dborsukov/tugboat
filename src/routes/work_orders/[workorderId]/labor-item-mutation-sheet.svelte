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
    commands,
    type DiscountType,
    type LaborPriceType,
    type PreviewLaborItemQuote,
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
  import { unwrap } from "$lib/tools";

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
    open = $bindable(false),
  }: {
    laborItemId: number | null;
    workOrderId: string | null;
    open: boolean;
  } = $props();

  const formID = "labor-item-mutation-form";
  const isEdit = $derived(laborItemId != null);

  async function fetchLaborQuote(data: {
    laborPriceType: LaborPriceType;
    laborPrice: string;
    discountType: DiscountType;
    discountValue: string;
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
      pendingQuote = null;
    }
  }

  function cancelPriceConfirm() {
    priceConfirmOpen = false;
    pendingSubmitData = null;
    pendingQuote = null;
  }

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (open) {
      priceConfirmOpen = false;
      pendingSubmitData = null;
      pendingQuote = null;

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
    {#if liveWarning}
      {#if liveWarning.kept !== liveWarning.total}
        <p class="text-sm text-amber-600 dark:text-amber-500 mt-1.5">
          Увага: після сплати ПДВ залишиться {fmt(liveWarning.kept)} грн. (ціна {fmt(
            liveWarning.total,
          )} грн.), що нижче за собівартість{" "}
          {fmt(liveWarning.cost)} грн.
        </p>
      {:else}
        <p class="text-sm text-amber-600 dark:text-amber-500 mt-1.5">
          Увага: кінцева ціна {fmt(liveWarning.total)} грн. нижча за собівартість
          {fmt(liveWarning.cost)} грн.
        </p>
      {/if}
    {:else if liveQuote && !liveQuote.costBasisKnown}
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
        {#if pendingQuote && pendingQuote.belowCost}
          {#if pendingQuote.kept !== pendingQuote.total}
            Після сплати ПДВ залишиться {fmt(pendingQuote.kept)} грн., що нижче за
            собівартість {fmt(pendingQuote.cost)} грн. (клієнт заплатить{" "}
            {fmt(pendingQuote.total)} грн.). Зберегти?
          {:else}
            Кінцева ціна становить {fmt(pendingQuote.total)} грн., що нижче за собівартість
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
