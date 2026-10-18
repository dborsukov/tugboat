<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";

  import {
    createCloseVatPeriodMutation,
    createReopenVatPeriodMutation,
    createVatPeriodsQuery,
  } from "../queries";

  let { open = $bindable(false) }: { open: boolean } = $props();

  const queryClient = useQueryClient();
  const periodsQuery = createVatPeriodsQuery();
  const closeMutation = createCloseVatPeriodMutation(queryClient);
  const reopenMutation = createReopenVatPeriodMutation(queryClient);

  const MONTH_NAMES = [
    "Січень",
    "Лютий",
    "Березень",
    "Квітень",
    "Травень",
    "Червень",
    "Липень",
    "Серпень",
    "Вересень",
    "Жовтень",
    "Листопад",
    "Грудень",
  ];

  function monthLabel(value: string): string {
    const [year, month] = value.split("-");
    const index = Number(month) - 1;
    return `${MONTH_NAMES[index] ?? month} ${year}`;
  }

  function monthValue(date: Date): string {
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
  }

  let month = $state("");
  let note = $state("");

  const periods = $derived(periodsQuery.data ?? []);
  const closedMonths = $derived(new Set(periods.map((period) => period.month)));

  const monthOptions = $derived.by(() => {
    const now = new Date();
    const options: { value: string; label: string; closed: boolean }[] = [];
    // One month ahead, down to two years back, newest first. The current month
    // sits at the top; the month being filed is usually the previous one.
    for (let offset = 1; offset >= -24; offset -= 1) {
      const value = monthValue(
        new Date(now.getFullYear(), now.getMonth() + offset, 1),
      );
      options.push({
        value,
        label: monthLabel(value),
        closed: closedMonths.has(value),
      });
    }
    return options;
  });

  const selectedIsClosed = $derived(month !== "" && closedMonths.has(month));

  function selectMonth(value: string | undefined) {
    if (!value) return;
    month = value;
    // Selecting a month that is already closed edits its note rather than
    // closing it again, so start from the note it already has.
    note = periods.find((period) => period.month === value)?.note ?? "";
  }

  async function closePeriod() {
    if (!month) return;
    const wasClosed = selectedIsClosed;
    try {
      await closeMutation.mutateAsync({
        month,
        note: note.trim() || null,
      });
      month = "";
      note = "";
      toast.success(wasClosed ? "Примітку оновлено" : "Період закрито", {
        description: wasClosed
          ? "Місяць уже був закритий, змінено лише примітку. Записи ПДВ за ним так само захищені."
          : "Нові записи ПДВ за цей місяць більше не приймаються. Відкрити його можна тут.",
        position: "bottom-left",
      });
    } catch {
      // The mutation already reported the failure.
    }
  }

  async function reopenPeriod(value: string) {
    try {
      await reopenMutation.mutateAsync(value);
      toast.success("Період відкрито", { position: "bottom-left" });
    } catch {
      // The mutation already reported the failure.
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="gap-4 sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Періоди ПДВ</Dialog.Title>
      <Dialog.Description>
        Закритий місяць не приймає нових записів ПДВ — ні нарахувань, ні
        кредиту, ні коригувань. Це захищає вже подану декларацію.
      </Dialog.Description>
    </Dialog.Header>

    <div class="grid gap-3">
      <div class="grid gap-1.5">
        <Label for="vat-period-month">Місяць</Label>
        <Select.Root type="single" value={month} onValueChange={selectMonth}>
          <Select.Trigger id="vat-period-month" class="w-full">
            {month ? monthLabel(month) : "Оберіть місяць"}
          </Select.Trigger>
          <Select.Content class="max-h-72">
            {#each monthOptions as option (option.value)}
              <Select.Item value={option.value}>
                {option.label}{option.closed ? " · закрито" : ""}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
        {#if selectedIsClosed}
          <p class="text-xs text-muted-foreground">
            Цей місяць уже закрито. Повторне закриття оновить лише примітку.
          </p>
        {:else}
          <p class="text-xs text-muted-foreground">
            Оберіть місяць, який уже подано.
          </p>
        {/if}
      </div>
      <div class="grid gap-1.5">
        <Label for="vat-period-note">Примітка</Label>
        <Input
          id="vat-period-note"
          bind:value={note}
          placeholder="Наприклад, декларація за вересень"
        />
      </div>
      <Button
        onclick={closePeriod}
        disabled={!month || closeMutation.isPending}
      >
        {selectedIsClosed ? "Оновити примітку" : "Закрити місяць"}
      </Button>
    </div>

    <div class="grid gap-2">
      <p class="text-sm font-medium">Закриті періоди</p>
      {#if periodsQuery.isPending}
        <p class="text-sm text-muted-foreground">Завантаження...</p>
      {:else if periods.length === 0}
        <p class="text-sm text-muted-foreground">Жодного закритого місяця.</p>
      {:else}
        {#each periods as period (period.month)}
          <div
            class="flex items-center justify-between gap-3 rounded border border-border px-3 py-2 text-sm"
          >
            <div class="min-w-0">
              <p class="font-medium">{monthLabel(period.month)}</p>
              <p class="truncate text-xs text-muted-foreground">
                {period.month}{period.note ? ` · ${period.note}` : ""}
              </p>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onclick={() => reopenPeriod(period.month)}
              disabled={reopenMutation.isPending}
            >
              Відкрити
            </Button>
          </div>
        {/each}
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
