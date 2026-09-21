<script lang="ts">
  import ArrowDownIcon from "@lucide/svelte/icons/arrow-down";
  import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import EditIcon from "@lucide/svelte/icons/square-pen";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let {
    id,
    onEdit,
    onDelete,
    onMoveUp,
    onMoveDown,
    canMoveUp = false,
    canMoveDown = false,
  }: {
    id: number;
    onEdit: (id: number) => void;
    onDelete: (id: number) => void;
    onMoveUp: (id: number) => void;
    onMoveDown: (id: number) => void;
    canMoveUp?: boolean;
    canMoveDown?: boolean;
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
      <DropdownMenu.Item onclick={() => onEdit(id)}>
        <EditIcon />
        Редагувати
      </DropdownMenu.Item>
      <DropdownMenu.Separator />
      <DropdownMenu.Label>Перемістити</DropdownMenu.Label>
      <DropdownMenu.Item disabled={!canMoveUp} onclick={() => onMoveUp(id)}>
        <ArrowUpIcon />
        Вгору
      </DropdownMenu.Item>
      <DropdownMenu.Item disabled={!canMoveDown} onclick={() => onMoveDown(id)}>
        <ArrowDownIcon />
        Вниз
      </DropdownMenu.Item>
      <DropdownMenu.Separator />
      <DropdownMenu.Item variant="destructive" onclick={() => onDelete(id)}>
        <TrashIcon />
        Видалити
      </DropdownMenu.Item>
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
