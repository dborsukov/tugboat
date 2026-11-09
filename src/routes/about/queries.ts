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

export function createCommitQuery() {
  return createQuery<string>(() => ({
    queryKey: [...keys.version, "commit"],
    queryFn: () => commands.getAppBuildCommit(),
    staleTime: Infinity,
    gcTime: Infinity,
  }));
}
