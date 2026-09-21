import { moveEntry } from "$lib/components/data-table/data-table-reorder";

export function applyItemOrder<T extends { id: number }>(
  items: T[],
  order: number[] | null,
): T[] {
  if (!order || order.length === 0) return items;

  const remaining = new Map(items.map((item) => [item.id, item]));
  const ordered: T[] = [];
  for (const id of order) {
    const item = remaining.get(id);
    if (item) {
      ordered.push(item);
      remaining.delete(id);
    }
  }
  for (const item of items) {
    if (remaining.has(item.id)) ordered.push(item);
  }

  const unchanged =
    ordered.length === items.length &&
    ordered.every((item, index) => item === items[index]);
  return unchanged ? items : ordered;
}

export function moveItemById<T extends { id: number }>(
  items: T[],
  id: number,
  delta: number,
): T[] {
  const from = items.findIndex((item) => item.id === id);
  if (from === -1) return items;
  return moveEntry(items, from, from + delta);
}

export function isSameOrder(left: number[], right: number[]): boolean {
  return (
    left.length === right.length &&
    left.every((id, index) => id === right[index])
  );
}
