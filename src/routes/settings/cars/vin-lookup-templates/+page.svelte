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

  import { type VinLookupTemplates } from "$lib/bindings";
  import { createItemEditor } from "$lib/states/item-editor.svelte";

  import { createUpdateVinLookupTemplatesMutation } from "../../queries";
  import {
    getSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "../../settings-context";
  import {
    normalizeLookupTemplate,
    validateLookupTemplateEntry,
  } from "../lookup-template-validation";

  const queryClient = useQueryClient();
  const { draft } = getSettingsContext();

  const updateVinLookupTemplatesMutation =
    createUpdateVinLookupTemplatesMutation(queryClient);

  const itemEditor = createItemEditor<VinLookupTemplates>({
    items: () => draft?.vinLookupTemplates ?? [],
    makeEmpty: () => ({ label: "", urlTemplate: "" }),
    validate: (entry, others) => {
      const { labelError, urlTemplateError } = validateLookupTemplateEntry(
        entry,
        others,
      );
      return { label: labelError, urlTemplate: urlTemplateError };
    },
    commit: (items) => {
      if (!draft) return;
      draft.vinLookupTemplates = items;
      updateVinLookupTemplatesMutation.mutate(items);
    },
    transform: (entry) => normalizeLookupTemplate(entry),
  });
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 flex items-center justify-between gap-3">
    <div class="flex items-center gap-3">
      <Button
        variant="ghost"
        size="icon-lg"
        aria-label="Назад до налаштувань автомобілів"
        onclick={() => goto("/settings/cars")}
      >
        <ArrowLeftIcon class="size-4" />
      </Button>
      <h1 class="font-medium">Шаблони пошуку VIN-кодів</h1>
    </div>
    <Button variant="link" class="text-base" onclick={itemEditor.openCreate}>
      Додати +
    </Button>
  </div>

  {#if draft}
    <div class="mt-4">
      <div
        id="settings-vin-templates"
        class="border border-border rounded bg-card"
      >
        {#if draft.vinLookupTemplates.length === 0}
          <p class="px-4 py-10 text-sm text-muted-foreground text-center">
            Натисніть "Додати +" щоб додати новий шаблон...
          </p>
        {:else}
          <div class="flex flex-col gap-0 first:mt-3 last:mb-3">
            {#each draft.vinLookupTemplates as template, i (i)}
              <Item.Root class="py-1 px-4">
                <Item.Content>
                  <Item.Title>{template.label}</Item.Title>
                  <Item.Description class="break-all">
                    {template.urlTemplate}
                  </Item.Description>
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
              {#if i !== draft.vinLookupTemplates.length - 1}
                <Item.Separator />
              {/if}
            {/each}
          </div>
        {/if}
      </div>

      {#if updateVinLookupTemplatesMutation.isError}
        <p class="mt-3 px-1 text-xs text-destructive">
          Не вдалося зберегти шаблони, виправте помилки вище ({updateVinLookupTemplatesMutation
            .error?.message})
        </p>
      {/if}
    </div>
  {/if}
</PageLayout>

<Dialog.Root bind:open={itemEditor.isOpen}>
  <Dialog.Content class="sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>
        {itemEditor.editingIndex === null
          ? "Новий шаблон"
          : "Редагування шаблону"}
      </Dialog.Title>
    </Dialog.Header>

    <div class="grid gap-4 py-2">
      <div class="space-y-2">
        <Label for="template-label">Назва</Label>
        <Input
          id="template-label"
          value={itemEditor.draft.label}
          placeholder="Vindecoderz, Autozone, ..."
          aria-invalid={!!itemEditor.errors.label}
          oninput={(e) => (itemEditor.draft.label = e.currentTarget.value)}
        />
        {#if itemEditor.errors.label}
          <p class="text-xs text-destructive">{itemEditor.errors.label}</p>
        {/if}
      </div>

      <div class="space-y-2">
        <Label for="template-url">URL-шаблон</Label>
        <Input
          id="template-url"
          value={itemEditor.draft.urlTemplate}
          placeholder="https://example.com/vin?code=%s"
          aria-invalid={!!itemEditor.errors.urlTemplate}
          oninput={(e) =>
            (itemEditor.draft.urlTemplate = e.currentTarget.value)}
        />
        <p class="text-xs text-muted-foreground">
          Плейсхолдер %s замінюється на реальний VIN-код
        </p>
        {#if itemEditor.errors.urlTemplate}
          <p class="text-xs text-destructive">
            {itemEditor.errors.urlTemplate}
          </p>
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
