<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import { createCreateWarrantyWorkOrderMutation } from "../queries";

  let {
    sourceWorkOrderId,
    odometerIn = 0,
    open = $bindable(false),
  }: {
    sourceWorkOrderId: string | null;
    odometerIn?: number;
    open: boolean;
  } = $props();

  const queryClient = useQueryClient();
  const createWarrantyWorkOrderMutation =
    createCreateWarrantyWorkOrderMutation(queryClient);

  let odometer = $state<string>("");
  let description = $state<string>("");
  let notes = $state<string>("");
  let odometerError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      odometer = String(odometerIn);
      description = "";
      notes = "";
      odometerError = null;
    }
  });

  async function handleCreate() {
    if (!sourceWorkOrderId) return;

    const rawOdometer = String(odometer ?? "").trim();
    const parsed = Number(rawOdometer);
    if (rawOdometer === "" || !Number.isInteger(parsed) || parsed < 0) {
      odometerError = "Вкажіть пробіг";
      return;
    }
    odometerError = null;

    try {
      const created = await createWarrantyWorkOrderMutation.mutateAsync({
        sourceWorkOrderId,
        odometerIn: parsed,
        description: description.trim() === "" ? null : description.trim(),
        notes: notes.trim() === "" ? null : notes.trim(),
      });
      if (created) {
        open = false;
        await goto(`/work_orders/${created.id}`);
      }
    } catch {}
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="gap-4">
    <Dialog.Header>
      <Dialog.Title>Оформити гарантійний випадок</Dialog.Title>
      <Dialog.Description>
        Створюється окреме замовлення, пов'язане з {sourceWorkOrderId}.
        Запчастини та роботи додаються до нього звичайним чином.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1.5">
        <Label for="warranty-odometer">Пробіг</Label>
        <Input
          id="warranty-odometer"
          type="text"
          inputmode="numeric"
          bind:value={odometer}
          onkeydown={(e: KeyboardEvent) => {
            if (e.key === "Enter") {
              e.preventDefault();
              handleCreate();
            }
          }}
        />
        {#if odometerError}
          <p class="text-sm text-destructive">{odometerError}</p>
        {/if}
      </div>
      <div class="flex flex-col gap-1.5">
        <Label for="warranty-description">Опис</Label>
        <Input
          id="warranty-description"
          type="text"
          bind:value={description}
          placeholder="Наприклад: заміна запчастини за гарантією"
        />
      </div>
      <div class="flex flex-col gap-1.5">
        <Label for="warranty-notes">Нотатки</Label>
        <Input id="warranty-notes" type="text" bind:value={notes} />
      </div>
    </div>

    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (open = false)}
        disabled={createWarrantyWorkOrderMutation.isPending}
      >
        Відмінити
      </Button>
      <Button
        onclick={handleCreate}
        disabled={createWarrantyWorkOrderMutation.isPending ||
          !sourceWorkOrderId}
      >
        {#if createWarrantyWorkOrderMutation.isPending}
          <Spinner class="size-4" />
        {/if}
        Створити
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
