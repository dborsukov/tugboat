<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { toast } from "svelte-sonner";
  import { defaults, superForm } from "sveltekit-superforms";
  import { zod4, zod4Client } from "sveltekit-superforms/adapters";
  import z from "zod";

  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";

  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
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

  import type { PaymentMethod, VatScheme } from "$lib/bindings";
  import {
    expenseFields,
    expenseToForm,
    expenseToPayload,
    expenseVatAmount,
    expenseVatBase,
    refineExpense,
  } from "$lib/schemas/zexpense";
  import { settingsState } from "$lib/states/settings.svelte";
  import {
    schemeNeedsZeroRate,
    VAT_SCHEMES,
    vatSchemeLabels,
  } from "$lib/vat-scheme";

  import { paymentMethodLabels } from "../work_orders/payment-method";
  import { expenseCategoryIndicatorClass } from "./categories";
  import {
    createCreateExpenseMutation,
    createExpenseCategoriesQuery,
    createExpensesQuery,
    createExpenseVatQuery,
    createUpdateExpenseMutation,
    createVoidExpenseVatMutation,
    liveExpenseVatEntry,
  } from "./queries";

  let {
    expenseId = null,
    open = $bindable(false),
  }: { expenseId?: number | null; open: boolean } = $props();

  const formID = "expense-mutation-form";
  const isEdit = $derived(expenseId != null);

  const queryClient = useQueryClient();

  const createExpenseMutation = createCreateExpenseMutation(queryClient);
  const updateExpenseMutation = createUpdateExpenseMutation(queryClient);

  const expensesQuery = createExpensesQuery();
  const categoriesQuery = createExpenseCategoriesQuery();
  const expenseVatQuery = createExpenseVatQuery(() => expenseId);

  const expense = $derived(
    (expensesQuery.data ?? []).find((e) => e.id === expenseId) ?? null,
  );

  const liveVat = $derived(liveExpenseVatEntry(expenseVatQuery.data));
  const vatLocked = $derived(Boolean(liveVat));

  const businessIsVatPayer = $derived(settingsState.isVatPayer);

  const voidVatMutation = createVoidExpenseVatMutation(queryClient);
  let voidOpen = $state(false);
  let creditNoteNumber = $state("");
  let creditNoteDate = $state("");

  async function voidVat() {
    if (!expenseId) return;
    try {
      await voidVatMutation.mutateAsync({
        id: expenseId,
        creditNoteNumber: creditNoteNumber.trim() || null,
        creditNoteDate: creditNoteDate || null,
      });
      voidOpen = false;
      creditNoteNumber = "";
      creditNoteDate = "";
      toast.success("ПДВ сторновано", {
        description:
          "Додано сторно-запис із сьогоднішньою датою. Витрату знову можна редагувати.",
        position: "bottom-left",
      });
    } catch {
      // The mutation already reported the failure.
    }
  }

  function dayText(iso: string): string {
    const [year, month, day] = iso.split("-");
    return year && month && day ? `${day}.${month}.${year}` : iso;
  }

  const categories = $derived.by(() => {
    const all = categoriesQuery.data ?? [];
    const currentId = expense?.category?.id ?? null;
    return all.filter(
      (category) => category.archivedAt == null || category.id === currentId,
    );
  });

  const paymentMethods: PaymentMethod[] = [
    "Cash",
    "Card",
    "BankTransfer",
    "CardTransfer",
  ];

  const schema = z.object(expenseFields).superRefine(refineExpense);
  const form = superForm(defaults(zod4(schema)), {
    SPA: true,
    validators: zod4Client(schema),
    validationMethod: "oninput",
    resetForm: false,
    // svelte-ignore state_referenced_locally
    id: formID,
    async onUpdate({ form: nativeForm, cancel }) {
      if (!nativeForm.valid) return;

      const payload = expenseToPayload(nativeForm.data);

      if (isEdit && expenseId) {
        await updateExpenseMutation
          .mutateAsync({ id: expenseId, payload })
          .catch(() => {});
      } else {
        await createExpenseMutation.mutateAsync(payload).catch(() => {});
      }
      // Prevent superforms from re-applying the submitted data to the form
      // after this handler, which would overwrite the reset below and leave
      // stale values in the fields for the next time the sheet opens.
      cancel();
      open = false;
      form.reset();
    },
  });

  const { form: formData, enhance, submitting } = form;

  $effect(() => {
    if (!open) return;
    if (expense) {
      form.reset({ data: expenseToForm(expense) });
    } else {
      form.reset({
        data: { ...defaults(zod4(schema)), date: today() },
      });
    }
  });

  /** Local calendar day, which is the day an expense is recorded on. */
  function today(): string {
    const now = new Date();
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
  }
</script>

<MutationSheetShell
  bind:open
  {formID}
  title={isEdit ? "Редагувати витрату" : "Створити витрату"}
  description={isEdit ? "Змініть дані витрати" : "Внесіть дані нової витрати"}
  submitLabel={isEdit ? "Зберегти зміни" : "Створити"}
  submitting={$submitting}
  enhanceAction={enhance}
  onCancel={() => {
    form.reset();
    open = false;
  }}
>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
    <Form.Field {form} name="date">
      <Form.Control>
        {#snippet children({ props })}
          <Label>Дата</Label>
          <DateTimePicker bind:value={$formData.date} {...props} />
        {/snippet}
      </Form.Control>
      <Form.FieldErrors />
    </Form.Field>

    <fieldset disabled={vatLocked} class="contents">
      <Form.Field {form} name="amount">
        <Form.Control>
          {#snippet children({ props })}
            <Label>Сума</Label>
            <div class="flex items-baseline gap-2" {...props}>
              <CurrencyInput bind:value={$formData.amount} />
              <p>грн.</p>
            </div>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </fieldset>
  </div>

  <Form.Field {form} name="description">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Опис</Label>
        <Textarea
          {...props}
          bind:value={$formData.description}
          placeholder="На що витрачено..."
          class="min-h-20"
        />
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="categoryId">
    <Form.Control>
      {#snippet children({ props: formProps })}
        <Label>Категорія</Label>
        <ComboSelect
          items={categories}
          bind:value={$formData.categoryId}
          clearable
          getKey={(category) => category.id}
          getLabel={(category) => category.name}
          getSearchValue={(category) => category.name}
          placeholder="Без категорії"
          searchPlaceholder="Пошук категорій..."
          emptyText="Категорій не знайдено."
          triggerClass={$formData.categoryId ? "pl-2" : undefined}
          {formProps}
        >
          {#snippet trigger(selected)}
            {#if selected}
              <span class="flex min-w-0 items-center gap-2">
                <span class={expenseCategoryIndicatorClass(selected.color)}
                ></span>
                <span class="truncate">{selected.name}</span>
              </span>
            {:else}
              Без категорії
            {/if}
          {/snippet}
          {#snippet item(category)}
            <span
              class="flex min-w-0 flex-1 items-center gap-2 whitespace-nowrap"
            >
              <span class={expenseCategoryIndicatorClass(category.color)}
              ></span>
              <span class="truncate">{category.name}</span>
              {#if category.archivedAt}
                <span class="text-xs text-muted-foreground">(архів)</span>
              {/if}
            </span>
          {/snippet}
        </ComboSelect>
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  <Form.Field {form} name="paymentMethod">
    <Form.Control>
      {#snippet children({ props })}
        <Label>Спосіб оплати</Label>
        <Select.Root
          type="single"
          value={$formData.paymentMethod ?? ""}
          onValueChange={(value) =>
            ($formData.paymentMethod = (value || null) as PaymentMethod | null)}
        >
          <Select.Trigger {...props} class="w-full">
            {$formData.paymentMethod
              ? paymentMethodLabels[$formData.paymentMethod]
              : "Не вказано"}
          </Select.Trigger>
          <Select.Content>
            {#each paymentMethods as method (method)}
              <Select.Item value={method}
                >{paymentMethodLabels[method]}</Select.Item
              >
            {/each}
          </Select.Content>
        </Select.Root>
        {#if $formData.paymentMethod}
          <button
            type="button"
            class="self-start text-xs text-muted-foreground hover:text-foreground"
            onclick={() => ($formData.paymentMethod = null)}
          >
            Прибрати спосіб оплати
          </button>
        {/if}
      {/snippet}
    </Form.Control>
    <Form.FieldErrors />
  </Form.Field>

  {#if liveVat}
    <div class="rounded border border-border bg-card p-3">
      <div class="flex items-start justify-between gap-3">
        <div class="flex items-start gap-2">
          <ShieldCheckIcon class="mt-0.5 size-4 shrink-0 text-emerald-600" />
          <div class="space-y-0.5">
            <p class="text-sm font-medium">ПДВ зафіксовано</p>
            <p class="text-xs text-muted-foreground">
              {dayText(liveVat.taxPoint)} · ПН {liveVat.documentNumber ?? "—"}
            </p>
            <p class="text-xs text-muted-foreground">
              База {Big(liveVat.base).toFixed(2)} грн · ПДВ {Big(
                liveVat.amount,
              ).toFixed(2)} грн
            </p>
          </div>
        </div>
        {#if businessIsVatPayer}
          <Button
            variant="outline"
            size="sm"
            onclick={() => (voidOpen = true)}
            disabled={voidVatMutation.isPending}
          >
            <RotateCcwIcon class="size-4" />
            Сторнувати
          </Button>
        {/if}
      </div>
      {#if businessIsVatPayer}
        <p class="mt-2 text-xs text-muted-foreground">
          Сума, ставка й податкова накладна зафіксовані. Сторнуйте запис, щоб їх
          змінити.
        </p>
      {:else}
        <p class="mt-2 text-xs text-muted-foreground">
          Історичний запис ПДВ. Бізнес зараз не є платником ПДВ, тому запис
          доступний лише для перегляду.
        </p>
      {/if}
    </div>
  {/if}

  {#if businessIsVatPayer}
    <fieldset disabled={vatLocked} class="contents">
      <Form.Field {form} name="vatRate">
        <Form.Control>
          {#snippet children({ props })}
            <Label>ПДВ</Label>
            <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
              <div class="flex items-baseline gap-2" {...props}>
                <Input
                  bind:value={$formData.vatRate}
                  type="text"
                  inputmode="decimal"
                  aria-label="Ставка ПДВ"
                  class="w-20 text-right"
                />
                <p>%</p>
              </div>
              <div class="flex items-center gap-2">
                <Checkbox
                  id="expense-price-includes-vat"
                  checked={$formData.priceIncludesVat}
                  onCheckedChange={(checked) =>
                    ($formData.priceIncludesVat = checked === true)}
                />
                <Label for="expense-price-includes-vat" class="font-normal">
                  Сума вже включає ПДВ
                </Label>
              </div>
              <div class="flex items-center gap-2">
                <Label for="expense-vat-scheme" class="font-normal"
                  >Режим ПДВ</Label
                >
                <Select.Root
                  type="single"
                  value={$formData.vatScheme}
                  onValueChange={(value) => {
                    if (!value) return;
                    $formData.vatScheme = value as VatScheme;
                    if (schemeNeedsZeroRate($formData.vatScheme)) {
                      $formData.vatRate = "0";
                      // There is no VAT to credit on a zero-rated/exempt purchase.
                      $formData.vatCredited = false;
                    }
                  }}
                >
                  <Select.Trigger id="expense-vat-scheme" class="w-40">
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
            </div>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>

      <MutationSheetSectionTitle
        text="Постачальник і податкова накладна"
        icon={FileTextIcon}
        hint="Потрібні, коли ПДВ витрати включається до податкового кредиту."
      />

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Form.Field {form} name="supplierName">
          <Form.Control>
            {#snippet children({ props })}
              <Label>Постачальник</Label>
              <Input
                {...props}
                bind:value={$formData.supplierName}
                placeholder="ТОВ «Постач»"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="supplierCode">
          <Form.Control>
            {#snippet children({ props })}
              <Label>Код (ЄДРПОУ / РНОКПП)</Label>
              <Input
                {...props}
                bind:value={$formData.supplierCode}
                placeholder="12345678"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Form.Field {form} name="invoiceNumber">
          <Form.Control>
            {#snippet children({ props })}
              <Label>Номер податкової накладної</Label>
              <Input
                {...props}
                bind:value={$formData.invoiceNumber}
                placeholder="ПН-000123"
              />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>

        <Form.Field {form} name="invoiceDate">
          <Form.Control>
            {#snippet children({ props })}
              <Label>Дата податкової накладної</Label>
              <DateTimePicker bind:value={$formData.invoiceDate} {...props} />
            {/snippet}
          </Form.Control>
          <Form.FieldErrors />
        </Form.Field>
      </div>

      <Form.Field {form} name="vatCredited">
        <Form.Control>
          {#snippet children()}
            <div class="flex items-start gap-2">
              <Checkbox
                id="expense-vat-credited"
                checked={$formData.vatCredited}
                onCheckedChange={(checked) =>
                  ($formData.vatCredited = checked === true)}
              />
              <div class="space-y-0.5">
                <Label for="expense-vat-credited" class="font-normal">
                  Включити ПДВ до податкового кредиту
                </Label>
                {#if $formData.vatCredited}
                  <p class="text-xs text-muted-foreground">
                    До кредиту: {expenseVatAmount(
                      $formData.amount,
                      $formData.vatRate,
                      $formData.priceIncludesVat,
                    ).toFixed(2)} грн (база {expenseVatBase(
                      $formData.amount,
                      $formData.vatRate,
                      $formData.priceIncludesVat,
                    ).toFixed(2)} грн)
                  </p>
                {/if}
              </div>
            </div>
          {/snippet}
        </Form.Control>
        <Form.FieldErrors />
      </Form.Field>
    </fieldset>
  {/if}

  <AlertDialog.Root bind:open={voidOpen}>
    <AlertDialog.Content>
      <AlertDialog.Header>
        <AlertDialog.Title>Сторнувати ПДВ?</AlertDialog.Title>
        <AlertDialog.Description>
          Буде додано сторно-запис із сьогоднішньою датою, а витрату знову можна
          буде редагувати. Початковий запис залишиться в реєстрі — історія
          періоду не переписується.
        </AlertDialog.Description>
      </AlertDialog.Header>
      <div class="grid gap-3 py-2">
        <div class="grid gap-1.5">
          <Label for="expense-credit-note">Номер коригування</Label>
          <Input
            id="expense-credit-note"
            bind:value={creditNoteNumber}
            placeholder="РК-000123"
          />
        </div>
        <div class="grid gap-1.5">
          <Label>Дата коригування</Label>
          <DateTimePicker bind:value={creditNoteDate} />
        </div>
      </div>
      <AlertDialog.Footer>
        <AlertDialog.Cancel>Скасувати</AlertDialog.Cancel>
        <AlertDialog.Action onclick={voidVat}>Сторнувати</AlertDialog.Action>
      </AlertDialog.Footer>
    </AlertDialog.Content>
  </AlertDialog.Root>
</MutationSheetShell>
