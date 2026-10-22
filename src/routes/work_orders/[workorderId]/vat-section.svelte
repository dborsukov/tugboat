<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";
  import { toast } from "svelte-sonner";

  import BadgeCheckIcon from "@lucide/svelte/icons/badge-check";
  import CircleHelpIcon from "@lucide/svelte/icons/circle-help";
  import RotateCcwIcon from "@lucide/svelte/icons/rotate-ccw";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";

  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DateTimePicker from "$lib/components/date-time-picker.svelte";

  import type { VatEntry } from "$lib/bindings";
  import { vatSchemeLabels } from "$lib/vat-scheme";

  import {
    createRecordWorkOrderVatMutation,
    createVoidWorkOrderVatMutation,
  } from "../queries";

  let {
    workOrderId,
    entries = undefined,
    isSupplied = false,
    expectedVat = "0",
    canManage = true,
    onVoided,
    onRecorded,
  }: {
    workOrderId: string;
    entries?: VatEntry[];
    /** The work has been handed over, so VAT can be fixed. */
    isSupplied?: boolean;
    /** What the order would charge in VAT right now. */
    expectedVat?: string;
    /** Whether the business can record or reverse VAT right now. */
    canManage?: boolean;
    onVoided?: () => void;
    onRecorded?: () => void;
  } = $props();

  const queryClient = useQueryClient();
  const voidMutation = createVoidWorkOrderVatMutation(queryClient);
  const recordMutation = createRecordWorkOrderVatMutation(queryClient);

  const liveEntries = $derived(
    (entries ?? []).filter(
      (entry) => entry.reversesId == null && entry.voidedAt == null,
    ),
  );
  const voided = $derived(
    (entries ?? []).filter((entry) => entry.voidedAt != null),
  );
  const canRecord = $derived(
    liveEntries.length === 0 && isSupplied && Big(expectedVat || "0").gt(0),
  );
  const visible = $derived(
    liveEntries.length > 0 || voided.length > 0 || (canManage && canRecord),
  );

  let voidOpen = $state(false);
  let creditNoteNumber = $state("");
  let creditNoteDate = $state("");

  function dayText(iso: string | null | undefined): string {
    if (!iso) return "—";
    const [year, month, day] = iso.split("-");
    if (!year || !month || !day) return iso;
    return `${day}.${month}.${year}`;
  }

  function schemeText(entry: VatEntry): string {
    return entry.scheme === "standard"
      ? ""
      : ` · ${vatSchemeLabels[entry.scheme]}`;
  }

  async function voidEntry() {
    try {
      await voidMutation.mutateAsync({
        id: workOrderId,
        creditNoteNumber: creditNoteNumber.trim() || null,
        creditNoteDate: creditNoteDate || null,
      });
      voidOpen = false;
      creditNoteNumber = "";
      creditNoteDate = "";
      onVoided?.();
      toast.success("ПДВ сторновано", {
        description:
          "Додано сторно-запис із датою коригування. Замовлення знову можна редагувати.",
        position: "bottom-left",
      });
    } catch {
      // The mutation already reported the failure.
    }
  }

  async function recordEntry() {
    try {
      await recordMutation.mutateAsync(workOrderId);
      onRecorded?.();
      toast.success("ПДВ зафіксовано", {
        description: "Суми записано в реєстр ПДВ за датою податкової події.",
        position: "bottom-left",
      });
    } catch {
      // The mutation already reported the failure.
    }
  }
</script>

{#if visible}
  <div class="mt-4 rounded border border-border bg-card">
    <div class="flex min-h-12 flex-wrap items-center gap-3 px-4 py-2 mb-0.5">
      <div class="flex min-w-0 items-center gap-2">
        {#if liveEntries.length > 0}
          <ShieldCheckIcon class="size-5 shrink-0 text-emerald-600" />
        {:else}
          <CircleHelpIcon class="size-5 shrink-0 text-muted-foreground" />
        {/if}
        <p class="text-xl font-medium">ПДВ</p>
      </div>

      <div class="ml-auto flex items-center gap-2">
        {#if canManage}
          {#if liveEntries.length > 0}
            <Button
              variant="outline"
              size="sm"
              onclick={() => (voidOpen = true)}
              disabled={voidMutation.isPending}
            >
              <RotateCcwIcon class="size-4" />
              Сторнувати ПДВ
            </Button>
          {:else if canRecord}
            <Button
              variant="outline"
              size="sm"
              onclick={recordEntry}
              disabled={recordMutation.isPending}
            >
              <BadgeCheckIcon class="size-4" />
              Зафіксувати ПДВ
            </Button>
          {/if}
        {/if}
      </div>
    </div>

    <div class="px-4 pb-4">
      {#if liveEntries.length > 0}
        <p class="text-sm text-muted-foreground">
          Зафіксовано {dayText(liveEntries[0].taxPoint)} · документ {liveEntries[0]
            .documentNumber ?? liveEntries[0].sourceId}
        </p>
      {:else if canRecord}
        <p class="text-sm text-muted-foreground">
          ПДВ ще не зафіксовано. Зафіксуйте його, коли замовлення виконано або
          оплачено.
        </p>
      {:else}
        <p class="text-sm text-muted-foreground">
          ПДВ за цим замовленням сторновано.
        </p>
      {/if}

      {#if liveEntries.length > 0}
        <div
          class="mt-3 grid grid-cols-[repeat(auto-fit,minmax(12rem,1fr))] gap-3 text-sm"
        >
          {#each liveEntries as entry (entry.id)}
            <div class="rounded border border-border/70 px-3 py-2">
              <p class="text-muted-foreground">
                ПДВ {Big(entry.rate).toString()}%{schemeText(entry)}
              </p>
              <div class="font-medium">
                <CurrencyDisplay amount={entry.amount} />
              </div>
              <div class="text-xs text-muted-foreground">
                База <CurrencyDisplay amount={entry.base} class="inline-flex" />
              </div>
              {#if entry.exemptionCode}
                <p class="text-xs text-muted-foreground">
                  Код пільги {entry.exemptionCode}
                </p>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      {#if voided.length > 0}
        <details class="mt-3">
          <summary class="cursor-pointer text-sm text-muted-foreground">
            Історія: сторновано {voided.length}
          </summary>
          <ul class="mt-2 space-y-1 text-sm">
            {#each voided as entry (entry.id)}
              <li
                class="flex flex-wrap items-center gap-x-2 text-muted-foreground"
              >
                <span>{dayText(entry.taxPoint)}</span>
                <span>·</span>
                <CurrencyDisplay amount={entry.base} />
                <span>база,</span>
                <CurrencyDisplay amount={entry.amount} />
                <span>ПДВ</span>
              </li>
            {/each}
          </ul>
        </details>
      {/if}
    </div>
  </div>

  <AlertDialog.Root bind:open={voidOpen}>
    <AlertDialog.Content>
      <AlertDialog.Header>
        <AlertDialog.Title>Сторнувати ПДВ?</AlertDialog.Title>
        <AlertDialog.Description>
          Буде додано сторно-записи з датою коригування, а замовлення знову
          стане доступним для редагування. Початкові записи залишаться в
          реєстрі.
        </AlertDialog.Description>
      </AlertDialog.Header>
      <div class="grid gap-3 py-2">
        <div class="grid gap-1.5">
          <Label for="work-order-credit-note">Номер коригування</Label>
          <Input
            id="work-order-credit-note"
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
        <AlertDialog.Action onclick={voidEntry}>Сторнувати</AlertDialog.Action>
      </AlertDialog.Footer>
    </AlertDialog.Content>
  </AlertDialog.Root>
{/if}
