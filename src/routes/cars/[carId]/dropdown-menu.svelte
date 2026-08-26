<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { toast } from "svelte-sonner";

  import { goto } from "$app/navigation";

  import CopyIcon from "@lucide/svelte/icons/copy";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import SearchIcon from "@lucide/svelte/icons/search";
  import Settings from "@lucide/svelte/icons/settings";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import { copyText } from "$lib/hooks/use-clipboard.svelte";

  import { createSettingsQuery } from "../../settings/queries";
  import { settingsSearchIndex } from "../../settings/settings-search-index";
  import { settingsSearch } from "../../settings/settings-search.svelte";
  import CarMutationSheet from "../car-mutation-sheet.svelte";
  import { createDeleteCarMutation } from "../queries";

  let {
    carId,
    vin,
    licensePlate,
  }: {
    carId: number;
    vin: string | null;
    licensePlate: string | null;
  } = $props();

  let isSheetOpen = $state(false);
  let isDeleteDialogOpen = $state(false);

  const queryClient = useQueryClient();

  const deleteCarMutation = createDeleteCarMutation(queryClient);

  const settingsQuery = createSettingsQuery();
  const vinTemplates = $derived(settingsQuery.data?.vinLookupTemplates ?? []);
  const plateTemplates = $derived(
    settingsQuery.data?.licensePlateLookupTemplates ?? [],
  );

  function handleDelete() {
    deleteCarMutation.mutate(carId, {
      onSuccess: async () => {
        await goto("/cars");
      },
    });
  }

  async function copyValue(value: string, label: string) {
    const status = await copyText(value);
    if (status === "success") {
      toast.success(`${label} скопійовано`, { position: "bottom-right" });
    } else {
      toast.error("Не вдалося скопіювати", {
        description: label,
        position: "bottom-right",
      });
    }
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
  <DropdownMenu.Content class="w-40" align="end">
    <DropdownMenu.Item onSelect={() => (isSheetOpen = true)}>
      <EditIcon />
      Редагувати
    </DropdownMenu.Item>

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

    <DropdownMenu.Item
      variant="destructive"
      onSelect={() => (isDeleteDialogOpen = true)}
    >
      <TrashIcon />
      Видалити
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>

<CarMutationSheet bind:open={isSheetOpen} {carId} />
<DeleteDialog
  bind:open={isDeleteDialogOpen}
  onDelete={handleDelete}
  description="Цю дію неможливо відмінити. Усі пов'язані з цим автомобілем дані будуть видалені."
/>
