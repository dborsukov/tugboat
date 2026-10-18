<script lang="ts">
  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let {
    id,
    archived,
    onEdit,
    onArchive,
    onRestore,
    onDelete,
  }: {
    id: number;
    archived: boolean;
    onEdit: (id: number) => void;
    onArchive: (id: number) => void;
    onRestore: (id: number) => void;
    onDelete: (id: number) => void;
  } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" class="h-full rounded-none">
        <EllipsisIcon />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-max whitespace-nowrap" align="end">
    <DropdownMenu.Group>
      {#if archived}
        <DropdownMenu.Item onclick={() => onRestore(id)}>
          <ArchiveRestoreIcon />
          Відновити
        </DropdownMenu.Item>
        <DropdownMenu.Item variant="destructive" onclick={() => onDelete(id)}>
          <TrashIcon />
          Видалити назавжди
        </DropdownMenu.Item>
      {:else}
        <DropdownMenu.Item onclick={() => onEdit(id)}>
          <EditIcon />
          Редагувати
        </DropdownMenu.Item>
        <DropdownMenu.Item variant="warning" onclick={() => onArchive(id)}>
          <ArchiveIcon />
          Архівувати
        </DropdownMenu.Item>
      {/if}
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
