<script lang="ts">
  import { getContext } from "svelte";

  import GripVerticalIcon from "@lucide/svelte/icons/grip-vertical";

  import {
    ROW_REORDER_CONTEXT,
    type RowReorderHandle,
  } from "$lib/components/data-table/data-table-reorder";

  let { label }: { label?: string } = $props();

  const reorder = getContext<RowReorderHandle | undefined>(ROW_REORDER_CONTEXT);
</script>

{#if reorder}
  <button
    type="button"
    class="flex h-10 w-6 shrink-0 cursor-grab touch-none items-center justify-center rounded text-muted-foreground transition-colors select-none hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none active:cursor-grabbing active:translate-y-0 disabled:cursor-not-allowed disabled:opacity-40"
    aria-label={label ? `Перемістити: ${label}` : "Перемістити рядок"}
    disabled={reorder.disabled}
    {@attach reorder.attachHandle}
  >
    <GripVerticalIcon class="size-4" />
  </button>
{/if}
