import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type CreateServicePayload,
  type Service,
  type UpdateServicePayload,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

function showMutationErrorToast(error: Error) {
  toast.error("Виникла помилка", {
    description: error.message,
    position: "bottom-left",
  });
}

export function servicesCountQueryOptions() {
  return {
    queryKey: keys.services.count(),
    queryFn: () => unwrap(commands.fetchServicesCount()),
  };
}

export function createServicesCountQuery() {
  return createQuery<number>(() => servicesCountQueryOptions());
}

export function servicesQueryOptions() {
  return {
    queryKey: keys.services.list(),
    queryFn: () => unwrap(commands.fetchServicesList()),
  };
}

export function createServicesQuery() {
  return createQuery<Service[]>(() => servicesQueryOptions());
}

export function serviceByIdQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.services.detail(id!),
    queryFn: () => unwrap(commands.fetchServiceById(id!)),
  };
}

export function createServiceByIdQuery(getId: () => number | null) {
  return createQuery<Service>(() => serviceByIdQueryOptions(getId()));
}

export function createCreateServiceMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: CreateServicePayload) =>
        unwrap(commands.createService(payload)),
      onSuccess: () => {
        invalidateServiceLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateServiceMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: { id: number; payload: UpdateServicePayload }) =>
        unwrap(commands.updateService(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateServiceLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteServiceMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.deleteService(id)),
      onSuccess: () => {
        invalidateServiceLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function invalidateServiceLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.services.list() });
  queryClient.invalidateQueries({ queryKey: keys.services.count() });
}
