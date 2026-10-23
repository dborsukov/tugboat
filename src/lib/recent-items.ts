import { createQuery } from "@tanstack/svelte-query";
import { writable } from "svelte/store";

import { commands, type RecentItem, type RecentItemKind } from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

export const recentItemsVersion = writable(0);

export async function recordRecentItem(
  kind: RecentItemKind,
  id: string | number,
): Promise<void> {
  try {
    await unwrap(commands.recordRecentItem(kind, String(id)));
    recentItemsVersion.update((value) => value + 1);
  } catch {}
}

export async function clearRecentItems(): Promise<void> {
  try {
    await unwrap(commands.clearRecentItems());
    recentItemsVersion.update((value) => value + 1);
  } catch {}
}

export function recentItemsQueryOptions() {
  return {
    queryKey: keys.recent.list(),
    queryFn: () => unwrap(commands.fetchRecentItems()),
    staleTime: 10_000,
  };
}

export function createRecentItemsQuery() {
  return createQuery<RecentItem[]>(() => recentItemsQueryOptions());
}
