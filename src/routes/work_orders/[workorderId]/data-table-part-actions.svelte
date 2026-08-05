<script lang="ts">
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let {
    id,
    onEdit,
    onDelete,
  }: {
    id: number;
    onEdit: (id: number) => void;
    onDelete: (id: number) => void;
  } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="icon">
        <EllipsisIcon />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-40" align="end">
    <DropdownMenu.Group>
      <DropdownMenu.Label>Дії</DropdownMenu.Label>
      <DropdownMenu.Item onclick={() => onEdit(id)}>
        Редагувати
        <EditIcon class="ml-auto" />
      </DropdownMenu.Item>
      <DropdownMenu.Item onclick={() => onDelete(id)}>
        Видалити
        <TrashIcon class="ml-auto" />
      </DropdownMenu.Item>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
