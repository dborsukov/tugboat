<script lang="ts">
  import { getContext } from "svelte";

  import GripVerticalIcon from "@lucide/svelte/icons/grip-vertical";

  import { Button } from "$lib/components/ui/button/index.js";

  import {
    ROW_REORDER_CONTEXT,
    type RowReorderHandle,
  } from "./data-table-reorder";

  let { label }: { label?: string } = $props();

  // Set by the row this cell belongs to. Missing only if the column is used on a
  // table that has no row reordering, in which case there is nothing to render.
  const reorder = getContext<RowReorderHandle | undefined>(ROW_REORDER_CONTEXT);
</script>

{#if reorder}
  <Button
    variant="ghost"
    size="icon-sm"
    class="cursor-grab touch-none select-none active:translate-y-0 active:cursor-grabbing"
    aria-label={label ? `Перемістити: ${label}` : "Перемістити рядок"}
    disabled={reorder.disabled}
    {@attach reorder.attachHandle}
  >
    <GripVerticalIcon class="text-muted-foreground" />
  </Button>
{/if}
