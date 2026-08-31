import { createQuery } from "@tanstack/svelte-query";

import { commands } from "$lib/bindings";
import { keys } from "$lib/query-keys";

export function versionQueryOptions() {
  return {
    queryKey: keys.version,
    queryFn: () => commands.getAppVersion(),
    staleTime: Infinity,
    gcTime: Infinity,
  };
}

export function createVersionQuery() {
  return createQuery<string>(() => versionQueryOptions());
}
