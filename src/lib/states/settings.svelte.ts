import { createQuery } from "@tanstack/svelte-query";

import { commands, type AppSettings } from "$lib/bindings";
import { keys } from "$lib/query-keys";

export function settingsQueryOptions() {
  return {
    queryKey: keys.settings.all,
    queryFn: async () => {
      const result = await commands.fetchSettings();
      if (result.status === "error") throw result.error;
      return result.data;
    },
    staleTime: Infinity,
    gcTime: Infinity,
  };
}

export function createSettingsQuery() {
  return createQuery<AppSettings>(() => settingsQueryOptions());
}

class AppSettingsState {
  settings = $state<AppSettings | null>(null);
}

export const settingsState = new AppSettingsState();
