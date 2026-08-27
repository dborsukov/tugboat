<script lang="ts">
  import type { Snippet } from "svelte";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { Button } from "$lib/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let {
    onDelete,
    menuItems,
  }: {
    onDelete?: () => void;
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
  <DropdownMenu.Content class="w-44 whitespace-nowrap" align="start">
    <DropdownMenu.Group>
      {@render menuItems?.()}
      {#if onDelete}
        <DropdownMenu.Item variant="destructive" onclick={onDelete}>
          <TrashIcon />
          Видалити
        </DropdownMenu.Item>
      {/if}
    </DropdownMenu.Group>
  </DropdownMenu.Content>
</DropdownMenu.Root>
