import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type CreateWorkerPayload,
  type UpdateWorkerPayload,
  type Worker,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

function showMutationErrorToast(error: Error) {
  toast.error("Виникла помилка", {
    description: error.message,
    position: "bottom-left",
  });
}

export function workersCountQueryOptions() {
  return {
    queryKey: keys.workers.count(),
    queryFn: () => unwrap(commands.fetchWorkersCount()),
  };
}

export function createWorkersCountQuery() {
  return createQuery<number>(() => workersCountQueryOptions());
}

export function workersQueryOptions() {
  return {
    queryKey: keys.workers.list(),
    queryFn: () => unwrap(commands.fetchWorkersList()),
  };
}

export function createWorkersQuery() {
  return createQuery<Worker[]>(() => workersQueryOptions());
}

export function workerByIdQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.workers.detail(id!),
    queryFn: () => unwrap(commands.fetchWorkerById(id!)),
  };
}

export function createWorkerByIdQuery(getId: () => number | null) {
  return createQuery<Worker>(() => workerByIdQueryOptions(getId()));
}

export function createCreateWorkerMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: CreateWorkerPayload) =>
        unwrap(commands.createWorker(payload)),
      onSuccess: () => {
        invalidateWorkerLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateWorkerMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: { id: number; payload: UpdateWorkerPayload }) =>
        unwrap(commands.updateWorker(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateWorkerLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteWorkerMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.deleteWorker(id)),
      onSuccess: () => {
        invalidateWorkerLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteWorkersMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (ids: number[]) => unwrap(commands.deleteWorkers(ids)),
      onSuccess: () => {
        invalidateWorkerLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function invalidateWorkerLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.workers.list() });
  queryClient.invalidateQueries({ queryKey: keys.workers.count() });
}
