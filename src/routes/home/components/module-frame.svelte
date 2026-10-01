<script lang="ts">
  import { createSortable } from "@dnd-kit/svelte/sortable";

  import GripVerticalIcon from "@lucide/svelte/icons/grip-vertical";

  import { Button } from "$lib/components/ui/button";

  import type {
    DashboardModuleHeight,
    DashboardModuleSize,
  } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import {
    DASHBOARD_COLUMNS,
    dashboardHeightInfo,
    dashboardHeightRows,
    dashboardSizeInfo,
    dashboardWidthColumns,
    type GridPlacement,
  } from "../dashboard-grid";
  import {
    allowedHeights,
    allowedWidths,
    type LayoutEntry,
  } from "../dashboard-layout";
  import SizeMenu from "./size-menu.svelte";

  /**
   * One dashboard module, placed and wrapped in everything the layout editor needs.
   *
   * Two elements: the outer one is the *slot* the grid placed,
   * absolutely positioned and sized, and the inner one is the element the
   * browser and the drag library move around. They have to be separate because the
   * library lifts a dragged module with `position: fixed` and then the `left`/`top`
   * that placed the slot are measured from the window instead of from the flow.
   */
  let {
    entry,
    index,
    placement,
    editing = false,
    reflowing = false,
    onWidth,
    onHeight,
    onNudge,
  }: {
    entry: LayoutEntry;
    /** Place in the flow. */
    index: number;
    /** The rectangle the grid gave this module, in pixels. */
    placement: GridPlacement;
    editing?: boolean;
    /** A module is being carried right now, move the others without animating. */
    reflowing?: boolean;
    onWidth: (id: string, width: DashboardModuleSize) => void;
    onHeight: (id: string, height: DashboardModuleHeight) => void;
    onNudge: (id: string, dCol: number, dRow: number) => void;
  } = $props();

  const ARROW_STEPS: Record<string, readonly [number, number]> = {
    ArrowLeft: [-1, 0],
    ArrowRight: [1, 0],
    ArrowUp: [0, -1],
    ArrowDown: [0, 1],
  };

  function handleNudgeKeys(event: KeyboardEvent) {
    const step = ARROW_STEPS[event.key];
    if (!step || event.repeat) return;

    event.preventDefault();
    onNudge(entry.id, step[0], step[1]);
  }

  /**
   * Whether this module is the one being dragged.
   *
   * Kept as its own flag rather than read straight off the sortable: library
   * reads every getter of the input while it is being constructed, so a getter
   * cannot reference the instance it belongs to.
   */
  let isDragSource = $state(false);

  const widthOptions = $derived(allowedWidths(entry));
  const heightOptions = $derived(allowedHeights(entry));

  /** A bar filled to the share of the row the width takes. */
  const widthShape = (option: DashboardModuleSize) => ({
    filled: dashboardWidthColumns[option] / DASHBOARD_COLUMNS,
    lines: 1,
  });

  /** Whole bars stacked, one per row the height takes. */
  const heightShape = (option: DashboardModuleHeight) => ({
    filled: 1,
    lines: dashboardHeightRows[option],
  });

  const sortable: ReturnType<typeof createSortable> = createSortable({
    get id() {
      return entry.id;
    },
    get index() {
      return index;
    },
    get type() {
      return "dashboard-module";
    },
    get accept() {
      return "dashboard-module";
    },
    get disabled() {
      return !editing;
    },
    /**
     * Siblings animate when the flow makes room for a module, but the module being
     * dragged must not animate at all - `null`, and `animate()` returns early on it.
     * Its drop animation would otherwise fly the module back to where it started.
     */
    get transition() {
      return isDragSource ? null : { duration: 200, easing: "ease" };
    },
  });

  $effect(() => {
    isDragSource = sortable.isDragSource;
  });

  let frameElement = $state<HTMLElement | null>(null);

  /** Attaches the module to its sortable, keeping the element for the fix below. */
  function attachFrame(node: HTMLElement) {
    frameElement = node;
    return sortable.attach(node);
  }

  /**
   * Keeps the dragged module visible.
   * The library lifts the dragged element into a `popover`, and a popover is closed
   * when its element is detached and re-inserted - which is exactly how a keyed
   * `{#each}` moves a module once the drag applies a new order. Chromium does that
   * silently (no `beforetoggle`, so the library's own re-show never runs) and the
   * UA stylesheet hides a closed popover, leaving the module `display: none` for the
   * rest of the drag: it is being dragged but cannot be seen. Re-open it as soon as
   * the new order reaches the DOM; the observer callback runs after the move, which
   * is what `showPopover()` requires of the element.
   */
  $effect(() => {
    const element = frameElement;
    // Only while a drag is in flight: during the drop animation the library takes
    // the popover back for itself.
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
</script>

<!-- The slot: where the packer put this module. -->
<div
  class={cn(
    "absolute",
    !reflowing && "transition-[left,top,width,height] duration-200 ease-out",
  )}
  style="left: {placement.x}px; top: {placement.y}px; width: {placement.width}px; height: {placement.height}px"
  data-module={entry.id}
  data-dragging={sortable.isDragSource ? "" : undefined}
>
  <!-- The module itself, and the element the drag library moves. -->
  <div
    class={cn(
      "relative h-full w-full",
      editing && "outline-dashed outline-border rounded-2xl outline-1",
      sortable.isDragSource && "z-10 rounded-2xl shadow-lg",
    )}
    data-drag-frame
    {@attach attachFrame}
  >
    {#if editing}
      <div
        class="border-border bg-background absolute top-1 right-1 z-20 flex items-center gap-1 rounded-xl border px-1 py-0.5 shadow-sm"
      >
        <Button
          variant="ghost"
          size="icon-xs"
          class="cursor-grab touch-none select-none active:translate-y-0 active:cursor-grabbing"
          aria-label="Перемістити модуль: {entry.module.title}"
          aria-keyshortcuts="ArrowLeft ArrowRight ArrowUp ArrowDown"
          title="Перетягніть, або пересувайте стрілками"
          onkeydown={handleNudgeKeys}
          {@attach sortable.attachHandle}
        >
          <GripVerticalIcon class="text-muted-foreground size-4" />
        </Button>

        {#if widthOptions.length > 1}
          <span class="bg-border h-4 w-px" aria-hidden="true"></span>
          <SizeMenu
            heading="Ширина"
            options={widthOptions}
            value={entry.slot.width}
            axis="width"
            info={dashboardSizeInfo}
            shape={widthShape}
            onSelect={(width) => onWidth(entry.id, width)}
          />
        {/if}

        {#if heightOptions.length > 1}
          <SizeMenu
            heading="Висота"
            options={heightOptions}
            value={entry.slot.height}
            axis="height"
            info={dashboardHeightInfo}
            shape={heightShape}
            onSelect={(height) => onHeight(entry.id, height)}
          />
        {/if}
      </div>
    {/if}

    <entry.module.component slot={entry.slot} />
  </div>
</div>

<style>
  /*
   * Keeps the module on screen through the drop animation.
   * `:global` because the `popover` attribute is added at runtime: Svelte cannot
   * see it in the markup and would drop the rule as unused.
   */
  :global([data-drag-frame][popover]:not(:popover-open)) {
    display: block;
  }
</style>
