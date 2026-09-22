<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import PencilIcon from "@lucide/svelte/icons/pencil";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import * as Item from "$lib/components/ui/item/index.js";
  import { Label } from "$lib/components/ui/label";
  import Spinner from "$lib/components/ui/spinner/spinner.svelte";

  import ArchivedBadge from "$lib/components/archived-badge.svelte";
  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { cn } from "$lib/utils";

  import {
    createArchiveCarBrandMutation,
    createCreateCarBrandMutation,
    createCustomCarBrandsQuery,
    createRestoreCarBrandMutation,
    createUpdateCarBrandMutation,
  } from "../../../cars/queries";
  import { SETTINGS_CONTENT_WIDTH_CLASS } from "../../settings-context";

  const queryClient = useQueryClient();

  const customBrandsQuery = createCustomCarBrandsQuery();
  const createBrandMutation = createCreateCarBrandMutation(queryClient);
  const updateBrandMutation = createUpdateCarBrandMutation(queryClient);
  const archiveBrandMutation = createArchiveCarBrandMutation(queryClient);
  const restoreBrandMutation = createRestoreCarBrandMutation(queryClient);

  const brands = $derived(customBrandsQuery.data ?? []);
  const saving = $derived(
    createBrandMutation.isPending || updateBrandMutation.isPending,
  );

  let dialogOpen = $state(false);
  let editingId = $state<number | null>(null);
  let draftName = $state("");

  function openCreate() {
    editingId = null;
    draftName = "";
    dialogOpen = true;
  }

  function openRename(id: number, name: string) {
    editingId = id;
    draftName = name;
    dialogOpen = true;
  }

  async function save() {
    const name = draftName.trim();
    if (!name || saving) return;

    const saved =
      editingId === null
        ? await createBrandMutation.mutateAsync(name).catch(() => null)
        : await updateBrandMutation
            .mutateAsync({ id: editingId, name })
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
        aria-label="Назад до налаштувань автомобілів"
        onclick={() => goto("/settings/cars")}
      >
        <ArrowLeftIcon class="size-4" />
      </Button>
      <h1 class="font-medium">Марки автомобілів</h1>
    </div>
    <Button variant="link" class="text-base" onclick={openCreate}>
      Додати +
    </Button>
  </div>

  <div
    id="settings-car-brands"
    class="mt-4 border border-border rounded bg-card"
  >
    {#if customBrandsQuery.isPending}
      <div class="flex items-center justify-center px-4 py-10">
        <Spinner />
      </div>
    {:else if customBrandsQuery.isError}
      <p class="px-4 py-10 text-sm text-destructive text-center">
        Не вдалося завантажити марки ({customBrandsQuery.error?.message})
      </p>
    {:else if brands.length === 0}
      <p class="px-4 py-10 text-sm text-muted-foreground text-center">
        Власних марок ще немає. Додайте її тут або просто введіть назву у формі
        автомобіля — якщо пошук нічого не знайде, застосунок запропонує створити
        марку.
      </p>
    {:else}
      <div class="flex flex-col gap-0 first:mt-3 last:mb-3">
        {#each brands as brand, i (brand.id)}
          <Item.Root class={cn("py-1 px-4", brand.archivedAt && "opacity-60")}>
            <Item.Media>
              <CarLogoLoader
                slug={brand.iconSlug}
                name={brand.name}
                class="size-8"
              />
            </Item.Media>
            <Item.Content>
              <Item.Title class="flex items-center gap-2">
                {brand.name}
                {#if brand.archivedAt}
                  <ArchivedBadge date={brand.archivedAt} />
                {/if}
              </Item.Title>
            </Item.Content>
            <Item.Actions>
              <Button
                variant="ghost"
                size="icon"
                aria-label="Перейменувати"
                onclick={() => openRename(brand.id, brand.name)}
              >
                <PencilIcon class="size-4 text-muted-foreground" />
              </Button>
              {#if brand.archivedAt}
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Відновити"
                  disabled={restoreBrandMutation.isPending}
                  onclick={() => restoreBrandMutation.mutate(brand.id)}
                >
                  <ArchiveRestoreIcon class="size-4 text-muted-foreground" />
                </Button>
              {:else}
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Архівувати"
                  disabled={archiveBrandMutation.isPending}
                  onclick={() => archiveBrandMutation.mutate(brand.id)}
                >
                  <ArchiveIcon class="size-4 text-muted-foreground" />
                </Button>
              {/if}
            </Item.Actions>
          </Item.Root>
          {#if i !== brands.length - 1}
            <Item.Separator />
          {/if}
        {/each}
      </div>
    {/if}
  </div>

  {#if brands.some((brand) => brand.archivedAt)}
    <p class="mt-3 px-1 text-xs text-muted-foreground">
      Архівовані марки не доступні для вибору, але залишаються на автомобілях,
      які вже їх використовують.
    </p>
  {/if}
</PageLayout>

<Dialog.Root bind:open={dialogOpen}>
  <Dialog.Content class="sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>
        {editingId === null ? "Нова марка" : "Перейменування марки"}
      </Dialog.Title>
    </Dialog.Header>

    <div class="space-y-2">
      <Label for="brand-name">Назва</Label>
      <Input
        id="brand-name"
        bind:value={draftName}
        placeholder="Введіть назву марки"
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            save();
          }
        }}
      />
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
