import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import { commands, type AppSettings } from "$lib/bindings";
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

export function createUpdateSettingsMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: AppSettings) =>
        unwrap(commands.updateSettings(payload)),
      onMutate: async (next) => {
        await queryClient.cancelQueries({ queryKey: keys.settings.all });
        const previous = queryClient.getQueryData<AppSettings>(
          keys.settings.all,
        );
        queryClient.setQueryData(keys.settings.all, next);
        return { previous };
      },
      onError: (error, _next, context) => {
        if (context?.previous) {
          queryClient.setQueryData(keys.settings.all, context.previous);
        }
        toast.error("Помилка збереження налаштувань", {
          description: error.message,
          position: "bottom-center",
          duration: Infinity,
          closeButton: true,
        });
      },
      onSettled: () => {
        invalidateSettings(queryClient);
      },
    }),
    () => queryClient,
  );
}

export function invalidateSettings(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.settings.all });
}
