<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import CheckIcon from "@lucide/svelte/icons/check";
  import PencilIcon from "@lucide/svelte/icons/pencil";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import * as Item from "$lib/components/ui/item/index.js";
  import { Label } from "$lib/components/ui/label";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import ArchivedBadge from "$lib/components/archived-badge.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import type { ExpenseCategoryColor } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import {
    EXPENSE_CATEGORY_COLORS,
    expenseCategoryColorClasses,
    expenseCategoryColorLabels,
    expenseCategoryIndicatorClass,
  } from "../../../expenses/categories";
  import {
    createArchiveExpenseCategoryMutation,
    createCreateExpenseCategoryMutation,
    createExpenseCategoriesQuery,
    createRestoreExpenseCategoryMutation,
    createUpdateExpenseCategoryMutation,
  } from "../../../expenses/queries";
  import { SETTINGS_CONTENT_WIDTH_CLASS } from "../../settings-context";

  const queryClient = useQueryClient();

  const categoriesQuery = createExpenseCategoriesQuery();
  const createCategoryMutation =
    createCreateExpenseCategoryMutation(queryClient);
  const updateCategoryMutation =
    createUpdateExpenseCategoryMutation(queryClient);
  const archiveCategoryMutation =
    createArchiveExpenseCategoryMutation(queryClient);
  const restoreCategoryMutation =
    createRestoreExpenseCategoryMutation(queryClient);

  const categories = $derived(categoriesQuery.data ?? []);
  const saving = $derived(
    createCategoryMutation.isPending || updateCategoryMutation.isPending,
  );

  let dialogOpen = $state(false);
  let editingId = $state<number | null>(null);
  let draftName = $state("");
  let draftColor = $state<ExpenseCategoryColor>("slate");

  function openCreate() {
    editingId = null;
    draftName = "";
    draftColor = "slate";
    dialogOpen = true;
  }

  function openEdit(id: number, name: string, color: ExpenseCategoryColor) {
    editingId = id;
    draftName = name;
    draftColor = color;
    dialogOpen = true;
  }

  async function save() {
    const name = draftName.trim();
    if (!name || saving) return;

    const payload = { name, color: draftColor };
    const saved =
      editingId === null
        ? await createCategoryMutation.mutateAsync(payload).catch(() => null)
        : await updateCategoryMutation
            .mutateAsync({ id: editingId, payload })
            .catch(() => null);

    if (saved) dialogOpen = false;
  }
</script>

<PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
  <div class="mt-4 flex items-center justify-between gap-3">
    <div class="flex items-center gap-3">
      <Button
        variant="ghost"
        size="icon-lg"
        aria-label="Назад до налаштувань фінансів"
        onclick={() => goto("/settings/finance")}
      >
        <ArrowLeftIcon class="size-4" />
      </Button>
      <h1 class="font-medium">Категорії витрат</h1>
    </div>
    <Button variant="link" class="text-base" onclick={openCreate}>
      Додати +
    </Button>
  </div>

  <div
    id="settings-expense-categories"
    class="mt-4 border border-border rounded bg-card"
  >
    {#if categoriesQuery.isPending}
      <div class="flex items-center justify-center px-4 py-10">
        <Spinner />
      </div>
    {:else if categoriesQuery.isError}
      <p class="px-4 py-10 text-sm text-destructive text-center">
        Не вдалося завантажити категорії ({categoriesQuery.error?.message})
      </p>
    {:else if categories.length === 0}
      <p class="px-4 py-10 text-sm text-muted-foreground text-center">
        Категорій ще немає. Натисніть "Додати +" щоб створити нову.
      </p>
    {:else}
      <div class="flex flex-col gap-0 first:mt-3 last:mb-3">
        {#each categories as category, i (category.id)}
          <Item.Root
            class={cn("py-1 px-4", category.archivedAt && "opacity-60")}
          >
            <Item.Media>
              <span class={expenseCategoryIndicatorClass(category.color)}
              ></span>
            </Item.Media>
            <Item.Content>
              <Item.Title class="flex items-center gap-2">
                {category.name}
                {#if category.archivedAt}
                  <ArchivedBadge date={category.archivedAt} />
                {/if}
              </Item.Title>
            </Item.Content>
            <Item.Actions>
              <Button
                variant="ghost"
                size="icon"
                aria-label="Редагувати"
                onclick={() =>
                  openEdit(category.id, category.name, category.color)}
              >
                <PencilIcon class="size-4 text-muted-foreground" />
              </Button>
              {#if category.archivedAt}
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Відновити"
                  disabled={restoreCategoryMutation.isPending}
                  onclick={() => restoreCategoryMutation.mutate(category.id)}
                >
                  <ArchiveRestoreIcon class="size-4 text-muted-foreground" />
                </Button>
              {:else}
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Архівувати"
                  disabled={archiveCategoryMutation.isPending}
                  onclick={() => archiveCategoryMutation.mutate(category.id)}
                >
                  <ArchiveIcon class="size-4 text-muted-foreground" />
                </Button>
              {/if}
            </Item.Actions>
          </Item.Root>
          {#if i !== categories.length - 1}
            <Item.Separator />
          {/if}
        {/each}
      </div>
    {/if}
  </div>

  {#if categories.some((category) => category.archivedAt)}
    <p class="mt-3 px-1 text-xs text-muted-foreground">
      Архівовані категорії не доступні для вибору у нових витратах, але
      залишаються на витратах, які вже їх використовують.
    </p>
  {/if}
</PageLayout>

<Dialog.Root bind:open={dialogOpen}>
  <Dialog.Content class="sm:max-w-xl">
    <Dialog.Header>
      <Dialog.Title>
        {editingId === null ? "Нова категорія" : "Редагування категорії"}
      </Dialog.Title>
    </Dialog.Header>

    <div class="grid gap-4 py-2">
      <div class="space-y-2">
        <Label for="category-name">Назва</Label>
        <Input
          id="category-name"
          bind:value={draftName}
          placeholder="Паливо, Оренда, Зарплата..."
          onkeydown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              save();
            }
          }}
        />
      </div>

      <div class="space-y-2">
        <Label>Колір</Label>
        <div class="grid w-full grid-cols-5 gap-2 sm:grid-cols-10">
          {#each EXPENSE_CATEGORY_COLORS as color (color)}
            {@const classes = expenseCategoryColorClasses(color)}
            <button
              type="button"
              class={cn(
                "aspect-square w-full rounded-full transition-shadow outline-none focus-visible:ring-3 focus-visible:ring-ring/30",
                classes.dot,
                color === draftColor &&
                  cn(
                    "ring-2 ring-offset-2 ring-offset-background",
                    classes.ring,
                  ),
              )}
              aria-label={expenseCategoryColorLabels[color]}
              aria-pressed={color === draftColor}
              title={expenseCategoryColorLabels[color]}
              onclick={() => (draftColor = color)}
            >
              {#if color === draftColor}
                <CheckIcon class="mx-auto size-4 text-white sm:size-6" />
              {/if}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (dialogOpen = false)}>
        Скасувати
      </Button>
      <Button onclick={save} disabled={!draftName.trim() || saving}>
        Зберегти
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
