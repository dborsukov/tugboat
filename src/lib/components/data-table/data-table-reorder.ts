import type { DragDropProvider } from "@dnd-kit/svelte";
import type { ComponentProps } from "svelte";
import type { Attachment } from "svelte/attachments";

type DragDropProviderProps = ComponentProps<typeof DragDropProvider>;

/**
 * The `dragover` payload of the drag provider. Derived from the component
 * instead of importing `@dnd-kit/dom`, which is a transitive dependency and not
 * resolvable from this package.
 */
export type RowDragOverEvent = Parameters<
  NonNullable<DragDropProviderProps["onDragOver"]>
>[0];

/** The `dragend` payload of the drag provider. */
export type RowDragEndEvent = Parameters<
  NonNullable<DragDropProviderProps["onDragEnd"]>
>[0];

/** Everything `DataTable` needs to make its rows reorderable by drag. */
export type RowReorderOptions = {
  /**
   * Identifies this table's rows to the drag system. A row can only be dropped
   * on rows of a table that accepts the same type.
   */
  type: string;
  /** Blocks dragging, e.g. on a locked or archived order. */
  disabled?: boolean;
  /** Timing of the rows making room for the dragged one. */
  transition?: { duration?: number; easing?: string };
  /** A drag started: snapshot the order here so `onCancel` can restore it. */
  onStart?: () => void;
  /** The dragged row changed position: apply this order to follow the pointer. */
  onMove?: (orderedRowIds: string[]) => void;
  /**
   * The row was dropped: the order `onMove` last applied is the one to keep.
   * Nothing is passed because the drag event describes the move that led here,
   * not the order that resulted from it, the host already holds the latter.
   */
  onCommit?: () => void;
  /** The drag was aborted: restore the order snapshotted in `onStart`. */
  onCancel?: () => void;
};

/** What a drag handle cell needs from the row it belongs to. */
export type RowReorderHandle = {
  /**
   * Attachment for the handle element. The library only starts a drag from the
   * handle, so the row's own controls (checkbox, menu) keep working normally.
   */
  attachHandle: Attachment<HTMLElement>;
  disabled: boolean;
};

/** Context key a reorderable row uses to hand its drag handle to the handle cell. */
export const ROW_REORDER_CONTEXT = Symbol("row-reorder");

/**
 * Moves one entry of an array, returning a new array. Returns the input array
 * unchanged when the move is a no-op or out of bounds, so callers can use
 * identity to detect "nothing happened".
 */
export function moveEntry<T>(entries: T[], from: number, to: number): T[] {
  if (
    from === to ||
    from < 0 ||
    to < 0 ||
    from >= entries.length ||
    to >= entries.length
  ) {
    return entries;
  }
  const next = entries.slice();
  next.splice(to, 0, next.splice(from, 1)[0]);
  return next;
}

/**
 * The order a drag asks for: the dragged row takes the place of the row it is
 * dropped on. Returns `rowIds` unchanged when there is nothing to move, so a
 * caller can skip the update by identity.
 *
 * The destination is read from the drop *target*, not from `source.index`. The
 * Svelte adapter filters `OptimisticSortingPlugin` out of the sortable defaults
 * (it reorders DOM nodes behind Svelte's back), so nothing projects the dragged
 * row's index while the pointer travels: `source.index` is kept equal to the
 * row's rendered index by `createSortable`, which makes moving to it a no-op and
 * the whole drag silently do nothing. The library's own `move()` helper reads
 * `source.index` safely only because that plugin has already reordered the DOM
 * by the time it runs.
 */
export function moveRowIds(
  rowIds: string[],
  event: RowDragOverEvent | RowDragEndEvent,
): string[] {
  const { source, target, canceled } = event.operation;
  if (!source || !target || canceled) return rowIds;
  const from = rowIds.indexOf(String(source.id));
  const to = rowIds.indexOf(String(target.id));
  // A target that is not one of the rendered rows (a stale registration, or a
  // droppable that is not a row) has nowhere to put the row.
  if (from === -1 || to === -1) return rowIds;
  return moveEntry(rowIds, from, to);
}
