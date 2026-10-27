<script lang="ts">
  import { RestrictToVerticalAxis } from "@dnd-kit/abstract/modifiers";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import { setContext, type Snippet } from "svelte";

  import {
    ROW_REORDER_CONTEXT,
    type RowReorderHandle,
    type RowReorderOptions,
  } from "$lib/components/data-table/data-table-reorder";

  import { cn } from "$lib/utils";

  let {
    id,
    index,
    reorder,
    selected = false,
    class: className,
    style,
    children,
  }: {
    id: string;
    index: number;
    reorder?: RowReorderOptions;
    selected?: boolean;
    class?: string;
    style?: string;
    children: Snippet;
  } = $props();

  const sortable = createSortable({
    get id() {
      return id;
    },
    get index() {
      return index;
    },
    get type() {
      return reorder?.type ?? "work-order-item-disabled";
    },
    get accept() {
      return reorder?.type ?? "work-order-item-disabled";
    },
    get disabled() {
      return reorder === undefined ? true : (reorder.disabled ?? false);
    },
    get transition() {
      return reorder?.transition;
    },
    modifiers: [RestrictToVerticalAxis],
  });

  setContext<RowReorderHandle>(ROW_REORDER_CONTEXT, {
    attachHandle: sortable.attachHandle,
    get disabled() {
      return reorder === undefined ? true : (reorder.disabled ?? false);
    },
  });

  let rowElement = $state<HTMLElement | null>(null);

  function attachRow(node: HTMLElement) {
    rowElement = node;
    return sortable.attach(node);
  }

  // A keyed {#each} re-inserts the dragged row once the host applies a reorder.
  // Chromium closes the drag popover on detach, which would hide the row while
  // it is being dragged. Put it back as soon as the reorder reaches the DOM.
  $effect(() => {
    const element = rowElement;
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
        // The library already put the row back.
      }
    };

    const observer = new MutationObserver(reopen);
    observer.observe(parent, { childList: true });
    reopen();
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={rowElement}
  {@attach attachRow}
  data-dragging={sortable.isDragSource ? "" : undefined}
  data-selected={selected ? "" : undefined}
  class={cn(
    "relative grid h-[60px] min-w-0 items-center overflow-hidden border-b border-border pl-2 pr-4 last:border-b-0",
    selected ? "bg-muted/50" : "hover:bg-muted/30",
    sortable.isDragSource && "z-10 bg-card shadow-lg",
    className,
  )}
  {style}
>
  {@render children()}
</div>

<style>
  :global([data-dragging] ~ div:hover) {
    background-color: transparent;
  }
</style>
