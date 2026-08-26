<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import * as Item from "$lib/components/ui/item/index.js";
  import { Label } from "$lib/components/ui/label";

  import PageLayout from "$lib/components/page-layout.svelte";

  import { type RateVariants } from "$lib/bindings";
  import { createItemEditor } from "$lib/states/item-editor.svelte";

  import { createUpdateRateVariantsMutation } from "../../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../../settings-context";
  import {
    normalizeRateVariantValue,
    validateRateVariantEntry,
  } from "./rate-variant-validation";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateRateVariantsMutation =
    createUpdateRateVariantsMutation(queryClient);

  const itemEditor = createItemEditor<RateVariants>({
    items: () => draft?.rateVariants ?? [],
    makeEmpty: () => ({ name: "", value: "0.00" }),
    validate: (entry, others) => {
      const { nameError, valueError } = validateRateVariantEntry(entry, others);
      return { name: nameError, value: valueError };
    },
    commit: (items) => {
      if (!draft) return;
      draft.rateVariants = items;
      updateRateVariantsMutation.mutate(items);
    },
    transform: (entry) => ({
      ...entry,
      value: normalizeRateVariantValue(entry.value),
    }),
  });
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 flex items-center justify-between gap-3">
    <div class="flex items-center gap-3">
      <Button
        variant="ghost"
        size="icon-lg"
        aria-label="Назад до налаштувань замовлень"
        onclick={() => goto("/settings/workorders")}
      >
        <ArrowLeftIcon class="size-4" />
      </Button>
      <h1 class="font-medium">Тарифи</h1>
    </div>
    <Button variant="link" class="text-base" onclick={itemEditor.openCreate}>
      Додати +
    </Button>
  </div>

  {#if draft}
    <div class="mt-4">
      <div
        id="settings-rate-variants"
        class="border border-border rounded bg-card"
      >
        {#if draft.rateVariants.length === 0}
          <p class="px-4 py-10 text-sm text-muted-foreground text-center">
            Натисніть "Додати +" щоб додати новий тариф...
          </p>
        {:else}
          <div class="flex flex-col gap-0 first:mt-3 last:mb-3">
            {#each draft.rateVariants as variant, i (i)}
              <Item.Root class="py-1 px-4">
                <Item.Content>
                  <Item.Title>{variant.name}</Item.Title>
                  <Item.Description>{variant.value} грн./год.</Item.Description>
                </Item.Content>
                <Item.Actions>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label="Редагувати"
                    onclick={() => itemEditor.openEdit(i)}
                  >
                    <PencilIcon class="size-4 text-muted-foreground" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label="Видалити"
                    onclick={() => itemEditor.remove(i)}
                  >
                    <Trash2Icon class="size-4 text-muted-foreground" />
                  </Button>
                </Item.Actions>
              </Item.Root>
              {#if i !== draft.rateVariants.length - 1}
                <Item.Separator />
              {/if}
            {/each}
          </div>
        {/if}
      </div>

      {#if updateRateVariantsMutation.isError}
        <p class="mt-3 px-1 text-xs text-destructive">
          Не вдалося зберегти тарифи, виправте помилки вище ({updateRateVariantsMutation
            .error?.message})
        </p>
      {/if}
    </div>
  {/if}
</PageLayout>

<Dialog.Root bind:open={itemEditor.isOpen}>
  <Dialog.Content class="sm:max-w-sm">
    <Dialog.Header>
      <Dialog.Title>
        {itemEditor.editingIndex === null
          ? "Новий тариф"
          : "Редагування тарифу"}
      </Dialog.Title>
    </Dialog.Header>

    <div class="grid gap-4 py-2">
      <div class="space-y-2">
        <Label for="variant-name">Назва</Label>
        <Input
          id="variant-name"
          value={itemEditor.draft.name}
          placeholder="Стандарт, Преміум, ..."
          aria-invalid={!!itemEditor.errors.name}
          oninput={(e) => (itemEditor.draft.name = e.currentTarget.value)}
        />
        {#if itemEditor.errors.name}
          <p class="text-xs text-destructive">{itemEditor.errors.name}</p>
        {/if}
      </div>

      <div class="space-y-2">
        <Label for="variant-value">Тариф</Label>
        <Input
          id="variant-value"
          value={itemEditor.draft.value}
          inputmode="decimal"
          placeholder="0.00"
          aria-invalid={!!itemEditor.errors.value}
          oninput={(e) => (itemEditor.draft.value = e.currentTarget.value)}
          onblur={() =>
            (itemEditor.draft.value = normalizeRateVariantValue(
              itemEditor.draft.value,
            ))}
        />
        {#if itemEditor.errors.value}
          <p class="text-xs text-destructive">{itemEditor.errors.value}</p>
        {/if}
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={itemEditor.close}>Скасувати</Button>
      <Button onclick={itemEditor.save} disabled={itemEditor.hasErrors}>
        Зберегти
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
