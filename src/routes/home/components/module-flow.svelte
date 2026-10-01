<script lang="ts">
  import { DragDropProvider, PointerSensor } from "@dnd-kit/svelte";
  import { onMount, type ComponentProps } from "svelte";

  import type {
    DashboardModuleHeight,
    DashboardModuleSize,
  } from "$lib/bindings";

  import {
    cellFromPoint,
    DASHBOARD_COLUMNS,
    planGrid,
    type GridCell,
  } from "../dashboard-grid";
  import {
    DASHBOARD_STACK_BELOW,
    dashboardWidthColumns,
    entryRect,
    withPosition,
    type LayoutEntry,
  } from "../dashboard-layout";
  import ModuleFrame from "./module-frame.svelte";

  type DragDropProviderProps = ComponentProps<typeof DragDropProvider>;
  type DragStartEvent = Parameters<
    NonNullable<DragDropProviderProps["onDragStart"]>
  >[0];
  type DragEndEvent = Parameters<
    NonNullable<DragDropProviderProps["onDragEnd"]>
  >[0];
  type DragMoveEvent = Parameters<
    NonNullable<DragDropProviderProps["onDragMove"]>
  >[0];

  /**
   * Wraps the drag-and-drop provider, measures its own width with a ResizeObserver,
   * runs planGrid over entries.map(entryRect), and absolutely places one ModuleFrame per module
   */

  let {
    entries,
    editing,
    onMove,
    onWidth,
    onHeight,
  }: {
    entries: readonly LayoutEntry[];
    editing: boolean;
    onMove: (id: string, col: number, row: number) => Promise<unknown>;
    onWidth: (id: string, width: DashboardModuleSize) => void;
    onHeight: (id: string, height: DashboardModuleHeight) => void;
  } = $props();

  /** The module being carried, and the corner of it the pointer took hold of. */
  let carried = $state<{
    id: string;
    left: number;
    top: number;
    pointerX: number;
    pointerY: number;
  } | null>(null);

  /** The cell the carried module will land in, and which module it is. */
  let preview = $state<{ id: string; cell: GridCell } | null>(null);

  /**
   * The last pointer position seen.
   *
   * Only a fallback: the drag reports its own coordinates on every move, and this
   * is here for the case where it does not.
   */
  let pointer = { x: 0, y: 0 };

  /** How wide the flow is, which is all the grid needs from the browser. */
  let measuredWidth = $state(0);
  let container = $state<HTMLElement | null>(null);

  onMount(() => {
    const track = (event: PointerEvent) => {
      pointer = { x: event.clientX, y: event.clientY };
    };

    window.addEventListener("pointermove", track);

    if (!container) {
      return () => window.removeEventListener("pointermove", track);
    }

    const observer = new ResizeObserver(([entry]) => {
      measuredWidth = entry.contentRect.width;
    });
    observer.observe(container);
    measuredWidth = container.clientWidth;

    return () => {
      window.removeEventListener("pointermove", track);
      observer.disconnect();
    };
  });

  /** What the grid needs to know about the space it is drawn into. */
  const metrics = $derived({
    width: measuredWidth,
    stackBelow: DASHBOARD_STACK_BELOW,
  });

  /** Where an arrow key has put a module, until that write lands. */
  let nudged = $state<{ id: string; cell: GridCell } | null>(null);

  /** Settled arrangement: what the pointer is previewing, or what the keys have asked for. */
  const settled = $derived.by(() => {
    const moved = preview ?? nudged;
    if (!moved) return entries;

    return withPosition(entries, moved.id, moved.cell.col, moved.cell.row);
  });

  const plan = $derived(planGrid(settled.map(entryRect), metrics));

  /** The arrangement the drag began with, for the module that is in the hand. */
  const rest = $derived(
    preview ? planGrid(entries.map(entryRect), metrics) : plan,
  );

  /** The height the flow takes, zero until its width is known. */
  const flowHeight = $derived(measuredWidth > 0 ? plan.height : 0);

  /**
   * Only the pointer needs a sensor of its own: moving a module by keyboard is a change of
   * cell rather than a drag that has to be pointed at one, so the arrows are read by the
   * handle and the grid does the rest.
   */
  const sensors = [PointerSensor];

  /** Moves a module one cell, as the arrow keys ask. */
  function nudge(id: string, dCol: number, dRow: number) {
    const placement = placements.get(id);
    if (!placement || measuredWidth <= 0) return;

    const span = spanOf(id);
    const from = cellFromPoint(placement.x, placement.y, span, {
      width: measuredWidth,
    });
    const cell = {
      col: Math.max(0, Math.min(from.col + dCol, DASHBOARD_COLUMNS - span)),
      row: Math.max(0, from.row + dRow),
    };
    if (cell.col === from.col && cell.row === from.row) return;

    nudged = { id, cell };
    void onMove(id, cell.col, cell.row).finally(() => {
      // Not if the module has been sent somewhere else in the meantime.
      if (
        nudged?.id === id &&
        nudged.cell.col === cell.col &&
        nudged.cell.row === cell.row
      ) {
        nudged = null;
      }
    });
  }

  const placements = $derived(
    new Map(plan.placements.map((placement) => [placement.id, placement])),
  );
  const restPlacements = $derived(
    new Map(rest.placements.map((placement) => [placement.id, placement])),
  );

  /** Where the carried module will land, drawn as the drop target. */
  const landing = $derived.by(() => {
    const carried = preview;
    if (!carried) return null;

    return (
      plan.placements.find((placement) => placement.id === carried.id) ?? null
    );
  });

  /** Module's rectangle. */
  function placementOf(entry: LayoutEntry) {
    // In the hand, not merely being previewed: once the module is dropped it belongs
    // at the cell it was dropped on, straight away..
    const inHand = carried !== null && entry.id === carried.id;
    const source = inHand ? restPlacements : placements;
    return source.get(entry.id) ?? placements.get(entry.id) ?? null;
  }

  /** The slot element of a module, which is what knows where it sits in the flow. */
  function slotElement(id: string): HTMLElement | null {
    return (
      container?.querySelector<HTMLElement>(
        `[data-module="${CSS.escape(id)}"]`,
      ) ?? null
    );
  }

  function spanOf(id: string): number {
    const entry = entries.find((candidate) => candidate.id === id);
    return entry ? dashboardWidthColumns[entry.slot.width] : 1;
  }

  function handleDragStart(event: DragStartEvent) {
    // Reffuse DND when in stacked layout
    if (measuredWidth < DASHBOARD_STACK_BELOW) return;

    const id = event.operation.source
      ? String(event.operation.source.id)
      : null;
    const slot = id ? slotElement(id) : null;
    if (!id || !slot) return;

    carried = {
      id,
      left: slot.offsetLeft,
      top: slot.offsetTop,
      pointerX: pointer.x,
      pointerY: pointer.y,
    };
    preview = {
      id,
      cell: cellFromPoint(slot.offsetLeft, slot.offsetTop, spanOf(id), {
        width: measuredWidth,
      }),
    };
  }

  /** Moves the preview to the cell the carried module is over now. */
  function handleDragMove(event: DragMoveEvent) {
    const held = carried;
    if (!held || measuredWidth <= 0) return;

    const moved = event.operation.transform;
    const dx = moved ? moved.x : pointer.x - held.pointerX;
    const dy = moved ? moved.y : pointer.y - held.pointerY;
    movePreview(held.left + dx, held.top + dy);
  }

  /** Moves the preview to the cell a corner is over, if that is a different cell. */
  function movePreview(left: number, top: number) {
    const held = carried;
    if (!held) return;

    const cell = cellFromPoint(left, top, spanOf(held.id), {
      width: measuredWidth,
    });
    if (cell.col === preview?.cell.col && cell.row === preview?.cell.row)
      return;

    preview = { id: held.id, cell };
  }

  function handleDragEnd(event: DragEndEvent) {
    const dropped = preview;
    carried = null;
    pointer = { x: pointer.x, y: pointer.y };

    if (event.canceled || !dropped) {
      preview = null;
      return;
    }

    /**
     * Keep drawing the module at the cell it was dropped on until the write lands.
     *
     * The library animates a dropped module to where it now sits in the document,
     * so the document has to already hold the new cell while that animation runs.
     */
    void onMove(dropped.id, dropped.cell.col, dropped.cell.row).finally(() => {
      // Not if another drag has started in the meantime.
      if (preview === dropped) preview = null;
    });
  }
</script>

<DragDropProvider
  {sensors}
  onDragStart={handleDragStart}
  onDragMove={handleDragMove}
  onDragEnd={handleDragEnd}
>
  <div
    bind:this={container}
    class="relative mt-4"
    style="height: {flowHeight}px"
  >
    {#if carried && landing}
      <div
        class="border-primary/60 bg-primary/10 pointer-events-none absolute rounded-2xl border-2 border-dashed"
        style="left: {landing.x}px; top: {landing.y}px; width: {landing.width}px; height: {landing.height}px"
      ></div>
    {/if}

    <!-- Modules are only drawn once the flow has been measured, so the first frame
         that contains them is already laid out at the real width. -->
    {#if measuredWidth > 0}
      {#each settled as entry, index (entry.id)}
        {@const placement = placementOf(entry)}
        {#if placement}
          <ModuleFrame
            {entry}
            {index}
            {placement}
            {editing}
            reflowing={carried !== null || preview !== null}
            {onWidth}
            {onHeight}
            onNudge={nudge}
          />
        {/if}
      {/each}
    {/if}
  </div>
</DragDropProvider>
