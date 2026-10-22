<script lang="ts" generics="TData">
  import { RestrictToVerticalAxis } from "@dnd-kit/abstract/modifiers";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import type { Row } from "@tanstack/table-core";
  import { setContext } from "svelte";

  import { FlexRender } from "$lib/components/ui/data-table";
  import * as TableUI from "$lib/components/ui/table/index.js";

  import { cn } from "$lib/utils";

  import {
    ROW_REORDER_CONTEXT,
    type RowReorderHandle,
    type RowReorderOptions,
  } from "./data-table-reorder";

  let {
    row,
    reorder,
    cellClasses = {},
    rowClassName,
    onRowClick,
  }: {
    row: Row<TData>;
    reorder: RowReorderOptions;
    cellClasses?: Record<string, string>;
    rowClassName?: (row: TData) => string | undefined;
    onRowClick?: (row: TData) => void;
  } = $props();

  // dnd-kit wants one sortable per row, created in the row's own component and
  // fed through getters: a reorder then updates the same instance instead of
  // replacing it, which is what keeps the transition animations alive.
  const sortable = createSortable({
    get id() {
      return row.id;
    },
    get index() {
      return row.index;
    },
    get type() {
      return reorder.type;
    },
    get accept() {
      return reorder.type;
    },
    get disabled() {
      return reorder.disabled ?? false;
    },
    get transition() {
      return reorder.transition;
    },
    // A table row only ever reorders up and down
    modifiers: [RestrictToVerticalAxis],
  });

  setContext<RowReorderHandle>(ROW_REORDER_CONTEXT, {
    attachHandle: sortable.attachHandle,
    get disabled() {
      return reorder.disabled ?? false;
    },
  });

  let rowElement = $state<HTMLElement | null>(null);

  /**
   * Attaches the row to its sortable and keeps the element around: keeping the
   * dragged row on screen (below) needs it.
   */
  function attachRow(node: HTMLElement) {
    rowElement = node;
    return sortable.attach(node);
  }

  /**
   * Keeps the dragged row visible. The library lifts it out of the table with
   * `popover`, and a popover is closed when its element is detached and
   * re-inserted — which is exactly how a keyed `{#each}` moves a row once the
   * host applies a reorder. Chromium does that silently (no `beforetoggle`, so
   * the library's own re-show never runs) and the UA stylesheet hides a closed
   * popover, leaving the row `display: none` for the rest of the drag: the row is
   * being dragged but cannot be seen. Re-open it as soon as the reorder reaches
   * the DOM; the observer callback runs after the move, which is what
   * `showPopover()` requires of the element.
   */
  $effect(() => {
    const element = rowElement;
    // Only while a drag is in flight: during the drop animation the library
    // takes the popover back for itself.
    if (!element || !sortable.isDragging) return;
    const parent = element.parentElement;
    if (!parent) return;

    const reopen = () => {
      if (
        !element.hasAttribute("popover") ||
        element.matches(":popover-open")
      ) {
        return;
      }
      try {
        element.showPopover();
      } catch {
        // Mid-reparent, or the library already put it back: nothing to do.
      }
    };

    const observer = new MutationObserver(reopen);
    observer.observe(parent, { childList: true });
    reopen();
    return () => observer.disconnect();
  });

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onRowClick?.(row.original);
    }
  }
</script>

<TableUI.Row
  tabindex={onRowClick ? 0 : undefined}
  data-state={row.getIsSelected() && "selected"}
  data-dragging={sortable.isDragSource ? "" : undefined}
  class={cn(
    onRowClick &&
      "hover:cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset focus-visible:bg-muted/60",
    // The dragged row floats above the table while it follows the pointer.
    sortable.isDragSource && "relative z-10 bg-card shadow-lg",
    rowClassName?.(row.original),
  )}
  {@attach attachRow}
  onclick={() => onRowClick?.(row.original)}
  onkeydown={handleKeyDown}
>
  {#each row.getVisibleCells() as cell (cell.id)}
    <TableUI.Cell class={cellClasses[cell.column.id]}>
      <FlexRender
        content={cell.column.columnDef.cell}
        context={cell.getContext()}
      />
    </TableUI.Cell>
  {/each}
</TableUI.Row>

<style>
  /*
   * While a row is being dragged the pointer travels across the other rows, and
   * their hover highlight would paint over the animation. The selector is global
   * because <tbody> and <tr> belong to the table component, not to this row.
   */
  :global(tbody:has([data-dragging]) tr:hover) {
    background-color: transparent;
  }
</style>
