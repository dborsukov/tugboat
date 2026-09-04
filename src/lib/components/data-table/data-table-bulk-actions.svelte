<script lang="ts">
  import type { Component, Snippet } from "svelte";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let {
    onDelete,
    deleteLabel = "Видалити",
    deleteVariant = "destructive",
    deleteIcon = TrashIcon,
    menuItems,
  }: {
    onDelete?: () => void;
    deleteLabel?: string;
    deleteVariant?: "destructive" | "warning";
    deleteIcon?: Component;
    menuItems?: Snippet;
  } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="outline" size="sm">
        Дії
        <ChevronDownIcon />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-max whitespace-nowrap" align="start">
    {#if menuItems}
      <DropdownMenu.Group>
        {@render menuItems()}
      </DropdownMenu.Group>
    {/if}
    {#if menuItems && onDelete}
      <DropdownMenu.Separator />
    {/if}
    {#if onDelete}
      <DropdownMenu.Group>
        {@const DeleteIcon = deleteIcon}
        <DropdownMenu.Item variant={deleteVariant} onclick={onDelete}>
          <DeleteIcon />
          {deleteLabel}
        </DropdownMenu.Item>
      </DropdownMenu.Group>
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>
