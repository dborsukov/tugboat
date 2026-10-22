<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { toast } from "svelte-sonner";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import SearchIcon from "@lucide/svelte/icons/search";
  import Settings from "@lucide/svelte/icons/settings";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import type { CarDeleteImpact } from "$lib/bindings";
  import { copyValue } from "$lib/clipboard";
  import { settingsState } from "$lib/states/settings.svelte";
  import { pluralizeUk } from "$lib/tools";

  import { invalidateClients } from "../../clients/queries";
  import { settingsSearchIndex } from "../../settings/settings-search-index";
  import { settingsSearch } from "../../settings/settings-search.svelte";
  import { invalidateWorkOrderLists } from "../../work_orders/queries";
  import CarMutationSheet from "../car-mutation-sheet.svelte";
  import {
    createArchiveCarMutation,
    createCarDeleteImpactQuery,
    createDeleteCarMutation,
    createRestoreCarMutation,
  } from "../queries";

  let {
    carId,
    isArchived = false,
    vin,
    licensePlate,
  }: {
    carId: number;
    isArchived?: boolean;
    vin: string | null;
    licensePlate: string | null;
  } = $props();

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);
  let isArchiveDialogOpen = $state(false);

  const queryClient = useQueryClient();

  const deleteCarMutation = createDeleteCarMutation(queryClient);
  const archiveCarMutation = createArchiveCarMutation(queryClient);
  const restoreCarMutation = createRestoreCarMutation(queryClient);

  const deleteImpactQuery = createCarDeleteImpactQuery(
    () => carId,
    () => isDeleteDialogOpen,
  );

  function describeDeleteImpact(impact: CarDeleteImpact | undefined) {
    if (!impact) {
      return "Цю дію неможливо відмінити. Автомобіль буде видалено безповоротно разом з усіма пов'язаними даними.";
    }
    const parts: string[] = [];
    if (impact.workOrderCount > 0) {
      parts.push(
        `${impact.workOrderCount} ${pluralizeUk(impact.workOrderCount, ["замовлення", "замовлення", "замовлень"])}`,
      );
    }
    if (impact.laborItemCount > 0) {
      parts.push(
        `${impact.laborItemCount} ${pluralizeUk(impact.laborItemCount, ["робота", "роботи", "робіт"])}`,
      );
    }
    if (impact.partItemCount > 0) {
      parts.push(
        `${impact.partItemCount} ${pluralizeUk(impact.partItemCount, ["запчастина", "запчастини", "запчастин"])}`,
      );
    }
    if (parts.length === 0) {
      return "Цю дію неможливо відмінити. Автомобіль буде видалено безповоротно.";
    }
    return `Буде видалено: ${parts.join(", ")}. Цю дію неможливо відмінити.`;
  }

  const vinTemplates = $derived(
    settingsState.settings?.vinLookupTemplates ?? [],
  );
  const plateTemplates = $derived(
    settingsState.settings?.licensePlateLookupTemplates ?? [],
  );

  function invalidateCascades() {
    invalidateClients(queryClient);
    invalidateWorkOrderLists(queryClient);
  }

  function handleDelete() {
    deleteCarMutation.mutate(carId, {
      onSuccess: async () => {
        await goto("/cars");
      },
    });
  }

  function handleArchive() {
    archiveCarMutation.mutate(carId, {
      onSuccess: invalidateCascades,
    });
  }

  function handleRestore() {
    restoreCarMutation.mutate(carId, {
      onSuccess: invalidateCascades,
    });
  }

  function buildLookupUrl(urlTemplate: string, value: string): string {
    return urlTemplate.replace("%s", encodeURIComponent(value.trim()));
  }

  async function openLookup(urlTemplate: string, value: string | null) {
    if (!value) return;
    try {
      await openUrl(buildLookupUrl(urlTemplate, value));
    } catch (error) {
      toast.error("Не вдалося відкрити посилання", {
        description: (error as Error).message,
        position: "bottom-left",
      });
    }
  }

  /** Navigate to a settings page and flash-highlight its section, like the
   * settings search does. */
  function openSettings(id: string) {
    const item = settingsSearchIndex.find((i) => i.id === id);
    if (item) settingsSearch.select(item);
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon" })}
  >
    <EllipsisIcon />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-max whitespace-nowrap" align="end">
    {#if isArchived}
      <DropdownMenu.Item onSelect={handleRestore}>
        <ArchiveRestoreIcon />
        Відновити
      </DropdownMenu.Item>
    {:else}
      <DropdownMenu.Item onSelect={() => (isSheetOpen = true)}>
        <EditIcon />
        Редагувати
      </DropdownMenu.Item>
    {/if}

    <DropdownMenu.Separator />

    <DropdownMenu.Label>VIN-код</DropdownMenu.Label>
    <DropdownMenu.Sub>
      <DropdownMenu.SubTrigger>
        <SearchIcon />
        Пошук
      </DropdownMenu.SubTrigger>
      <DropdownMenu.SubContent class="w-max max-w-72 p-1">
        {#if vinTemplates.length !== 0}
          {#each vinTemplates as template (template.label)}
            <DropdownMenu.Item
              disabled={!vin}
              onSelect={() => openLookup(template.urlTemplate, vin)}
            >
              {template.label}
            </DropdownMenu.Item>
          {/each}
          <DropdownMenu.Separator />
        {/if}
        <DropdownMenu.Item
          onSelect={() => openSettings("settings-vin-templates-entry")}
        >
          <Settings />
          Налаштувати
        </DropdownMenu.Item>
      </DropdownMenu.SubContent>
    </DropdownMenu.Sub>
    <DropdownMenu.Item
      disabled={!vin}
      onSelect={() => vin && copyValue(vin, "VIN-код")}
    >
      <CopyIcon />
      Копіювати
    </DropdownMenu.Item>

    <DropdownMenu.Separator />

    <DropdownMenu.Label>Номерний знак</DropdownMenu.Label>
    <DropdownMenu.Sub>
      <DropdownMenu.SubTrigger>
        <SearchIcon />
        Пошук
      </DropdownMenu.SubTrigger>
      <DropdownMenu.SubContent class="w-max max-w-72 p-1">
        {#if plateTemplates.length !== 0}
          {#each plateTemplates as template (template.label)}
            <DropdownMenu.Item
              disabled={!licensePlate}
              onSelect={() => openLookup(template.urlTemplate, licensePlate)}
            >
              {template.label}
            </DropdownMenu.Item>
          {/each}
          <DropdownMenu.Separator />
        {/if}
        <DropdownMenu.Item
          onSelect={() => openSettings("settings-plate-templates-entry")}
        >
          <Settings />
          Налаштувати
        </DropdownMenu.Item>
      </DropdownMenu.SubContent>
    </DropdownMenu.Sub>
    <DropdownMenu.Item
      disabled={!licensePlate}
      onSelect={() => licensePlate && copyValue(licensePlate, "Номерний знак")}
    >
      <CopyIcon />
      Копіювати
    </DropdownMenu.Item>

    <DropdownMenu.Separator />

    {#if isArchived}
      <DropdownMenu.Item
        variant="destructive"
        onSelect={() => (isDeleteDialogOpen = true)}
      >
        <TrashIcon />
        Видалити назавжди
      </DropdownMenu.Item>
    {:else}
      <DropdownMenu.Item
        variant="warning"
        onSelect={() => (isArchiveDialogOpen = true)}
      >
        <ArchiveIcon />
        Архівувати
      </DropdownMenu.Item>
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>

<CarMutationSheet bind:open={isSheetOpen} {carId} />
<DeleteDialog
  bind:open={isArchiveDialogOpen}
  onDelete={handleArchive}
  title="Підтвердіть архівацію"
  confirmLabel="Архівувати"
  actionVariant="warning"
  description="Автомобіль буде архівований разом з його замовленнями. Ви зможете відновити його пізніше."
/>
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  title="Видалити назавжди"
  description={deleteImpactQuery.isPending
    ? "Підрахунок пов'язаних даних…"
    : describeDeleteImpact(deleteImpactQuery.data)}
/>
