<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import { sheetInput, sheetLabel } from "$lib/components/sheet/field-styles";
  import SuggestInput from "$lib/components/suggest-input.svelte";

  import { commands, type PartItemDto } from "$lib/bindings";

  let {
    open = $bindable(false),
    rows,
    onApply,
  }: {
    open: boolean;
    rows: PartItemDto[];
    onApply: (
      supplier: string | null,
      supplierOrderId: string | null,
    ) => Promise<void>;
  } = $props();

  let supplier = $state("");
  let supplierOrderId = $state("");
  let suggestions = $state<string[]>([]);
  let error = $state<string | null>(null);
  let applying = $state(false);
  let overwriteOpen = $state(false);
  let overwriteTimer: ReturnType<typeof setTimeout> | null = null;
  let pending = $state<{
    supplier: string | null;
    supplierOrderId: string | null;
  } | null>(null);

  $effect(() => {
    if (!open) return;
    if (overwriteTimer) {
      clearTimeout(overwriteTimer);
      overwriteTimer = null;
    }
    supplier = "";
    supplierOrderId = "";
    error = null;
    applying = false;
    overwriteOpen = false;
    pending = null;
    void commands.fetchPartSuppliers().then((result) => {
      if (result.status === "ok") suggestions = result.data;
    });
  });

  const hasExistingValue = $derived(
    rows.some(
      (row) =>
        Boolean(row.supplier?.trim()) || Boolean(row.supplierOrderId?.trim()),
    ),
  );

  function normalize(value: string): string | null {
    const trimmed = value.trim();
    return trimmed === "" ? null : trimmed;
  }

  function requestApply() {
    const nextSupplier = normalize(supplier);
    const nextOrderId = normalize(supplierOrderId);

    if ((nextSupplier?.length ?? 0) > 200) {
      error = "Назва постачальника має містити не більше 200 символів";
      return;
    }
    if ((nextOrderId?.length ?? 0) > 120) {
      error = "Номер замовлення має містити не більше 120 символів";
      return;
    }

    error = null;
    if (hasExistingValue) {
      pending = {
        supplier: nextSupplier,
        supplierOrderId: nextOrderId,
      };
      open = false;
      overwriteTimer = setTimeout(() => {
        overwriteTimer = null;
        if (pending) overwriteOpen = true;
      }, 200);
      return;
    }
    void submit(nextSupplier, nextOrderId);
  }

  async function submit(
    nextSupplier: string | null,
    nextOrderId: string | null,
  ) {
    applying = true;
    error = null;
    try {
      await onApply(nextSupplier, nextOrderId);
      open = false;
    } catch (cause) {
      error =
        cause instanceof Error
          ? cause.message
          : "Не вдалося змінити постачальника";
    } finally {
      applying = false;
    }
  }

  async function confirmOverwrite() {
    if (!pending) return;
    overwriteOpen = false;
    const next = pending;
    pending = null;
    await submit(next.supplier, next.supplierOrderId);
  }

  function cancelOverwrite() {
    overwriteOpen = false;
    pending = null;
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="gap-4 sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Змінити постачальника</Dialog.Title>
      <Dialog.Description>
        Для {rows.length}
        {rows.length === 1 ? "вибраної позиції" : "вибраних позицій"}.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-3">
      <div class="space-y-1.5">
        <Label class={sheetLabel}>Постачальник</Label>
        <SuggestInput
          bind:value={supplier}
          items={suggestions}
          placeholder="Назва постачальника"
          class={sheetInput}
        />
      </div>
      <div class="space-y-1.5">
        <Label class={sheetLabel}>№ замовлення у постачальника</Label>
        <Input
          bind:value={supplierOrderId}
          placeholder="Наприклад, 4821"
          class={sheetInput}
        />
      </div>
      {#if error}
        <p class="text-sm text-destructive">{error}</p>
      {/if}
    </div>

    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (open = false)}
        disabled={applying}
      >
        Відмінити
      </Button>
      <Button onclick={requestApply} disabled={applying}>
        {#if applying}
          <Spinner class="size-4" />
        {/if}
        Застосувати
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<AlertDialog.Root bind:open={overwriteOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Перезаписати дані постачальника?</AlertDialog.Title>
      <AlertDialog.Description>
        Деякі вибрані позиції вже мають постачальника або номер замовлення.
        Поточні значення буде замінено.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={cancelOverwrite}
        >Відмінити</AlertDialog.Cancel
      >
      <AlertDialog.Action onclick={confirmOverwrite}>
        Перезаписати
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
