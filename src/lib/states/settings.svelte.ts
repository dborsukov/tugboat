import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

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

/** Writes one settings field into the cache without waiting for the backend */
export function patchSettingsCache<K extends keyof AppSettings>(
  queryClient: QueryClient,
  key: K,
  value: AppSettings[K],
) {
  queryClient.setQueryData<AppSettings>(keys.settings.all, (prev) =>
    prev ? { ...prev, [key]: value } : prev,
  );
}

/** Optimistic update -> re-sync cache with server, rollback on error */
export function createSettingsFieldMutation<K extends keyof AppSettings>(
  queryClient: QueryClient,
  key: K,
  mutationFn: (value: NonNullable<AppSettings[K]>) => Promise<AppSettings[K]>,
) {
  return createMutation(
    () => ({
      mutationFn,
      onMutate: async (next) => {
        await queryClient.cancelQueries({ queryKey: keys.settings.all });
        const previous = queryClient.getQueryData<AppSettings>(
          keys.settings.all,
        )?.[key];
        patchSettingsCache(queryClient, key, next);
        return { previous };
      },
      onSuccess: (saved: AppSettings[K]) => {
        patchSettingsCache(queryClient, key, saved);
      },
      onError: (error: Error, _next, context) => {
        if (context?.previous !== undefined) {
          patchSettingsCache(queryClient, key, context.previous);
        }
        if ((error as { type?: string }).type === "validation") {
          return;
        }
        toast.error("Виникла помилка", {
          description: error.message,
          position: "bottom-left",
        });
      },
    }),
    () => queryClient,
  );
}

class AppSettingsState {
  settings = $state<AppSettings | null>(null);

  /**
   * UI scale (webview zoom) currently in effect.
   *
   * Until the settings query resolves this falls back to the scale the window
   * was created with (`data-ui-scale`, injected before the first paint by
   * `boot_script` in `src-tauri/src/lib.rs`).
   */
  get uiScale(): number {
    const bootScale = Number(document.documentElement.dataset.uiScale);
    return this.settings?.uiScale ?? (bootScale > 0 ? bootScale : 1);
  }

  /* Whether the business profile is explicitly marked as a VAT payer. */
  get isVatPayer(): boolean {
    return this.settings?.companyInfo?.vatPayer === true;
  }
}

export const settingsState = new AppSettingsState();
