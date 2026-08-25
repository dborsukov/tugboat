import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type AppSettings,
  type RateVariants,
  type ThemeMode,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

export function settingsQueryOptions() {
  return {
    queryKey: keys.settings.all,
    queryFn: () => unwrap(commands.fetchSettings()),
    staleTime: Infinity,
    gcTime: Infinity,
  };
}

export function createSettingsQuery() {
  return createQuery<AppSettings>(() => settingsQueryOptions());
}

function patchSettingsCache<K extends keyof AppSettings>(
  queryClient: QueryClient,
  key: K,
  value: AppSettings[K],
) {
  queryClient.setQueryData<AppSettings>(keys.settings.all, (prev) =>
    prev ? { ...prev, [key]: value } : prev,
  );
}

/**
 * Optimistic update -> re-sync cache with server
 * Rollback on error
 */
function createSettingsFieldMutation<K extends keyof AppSettings>(
  queryClient: QueryClient,
  key: K,
  mutationFn: (value: AppSettings[K]) => Promise<AppSettings[K]>,
) {
  return createMutation(
    () => ({
      mutationFn,
      onMutate: async (next: AppSettings[K]) => {
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
        // Invalid input is rejected client-side and surfaced in the UI
        if ((error as { type?: string }).type === "ConstraintViolation") {
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

export function createUpdateThemeMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(queryClient, "theme", (theme: ThemeMode) =>
    unwrap(commands.updateTheme(theme)),
  );
}

export function createUpdateUiScaleMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "uiScale",
    // FIXME: figure out why specta generates invalid bindings
    // specta generates "number | null" for some reason, even though
    // Rust always returns f64 - ignoring for now
    async (uiScale: number) =>
      (await unwrap(commands.updateUiScale(uiScale))) ?? uiScale,
  );
}

export function createUpdateDefaultMarkupMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "defaultMarkup",
    (defaultMarkup: string) =>
      unwrap(commands.updateDefaultMarkup(defaultMarkup)),
  );
}

export function createUpdateRateVariantsMutation(queryClient: QueryClient) {
  return createSettingsFieldMutation(
    queryClient,
    "rateVariants",
    (rateVariants: RateVariants[]) =>
      unwrap(commands.updateRateVariants(rateVariants)),
  );
}
