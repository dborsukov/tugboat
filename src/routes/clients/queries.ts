import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type ClientDeleteImpact,
  type ClientDetailsDto,
  type ClientListItem,
  type ClientPayload,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

function showMutationErrorToast(error: Error) {
  toast.error("Виникла помилка", {
    description: error.message,
    position: "bottom-left",
  });
}

export function clientsCountQueryOptions() {
  return {
    queryKey: keys.clients.count(),
    queryFn: () => unwrap(commands.fetchClientsCount()),
  };
}

export function createClientsCountQuery() {
  return createQuery<number>(() => clientsCountQueryOptions());
}

export function clientsQueryOptions() {
  return {
    queryKey: keys.clients.list(),
    queryFn: () => unwrap(commands.fetchClientsList()),
  };
}

export function createClientsQuery() {
  return createQuery<ClientListItem[]>(() => clientsQueryOptions());
}

export function clientDetailsQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.clients.detail(id!),
    queryFn: () => unwrap(commands.fetchClientDetailsDto(id!)),
  };
}

export function createClientDetailsQuery(getId: () => number | null) {
  return createQuery<ClientDetailsDto>(() =>
    clientDetailsQueryOptions(getId()),
  );
}

export function clientDeleteImpactQueryOptions(
  id: number | null,
  enabled: boolean,
) {
  return {
    enabled: enabled && id != null,
    queryKey: keys.clients.deleteImpact(id!),
    queryFn: () => unwrap(commands.fetchClientDeleteImpact(id!)),
  };
}

export function createClientDeleteImpactQuery(
  getId: () => number | null,
  getEnabled: () => boolean,
) {
  return createQuery<ClientDeleteImpact>(() =>
    clientDeleteImpactQueryOptions(getId(), getEnabled()),
  );
}

export function createCreateClientMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: ClientPayload) =>
        unwrap(commands.createClient(payload)),
      onSuccess: () => {
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateClientMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: { id: number; payload: ClientPayload }) =>
        unwrap(commands.updateClient(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteClientMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.deleteClient(id)),
      onSuccess: async () => {
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createArchiveClientMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.archiveClient(id)),
      onSuccess: () => {
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createRestoreClientMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.restoreClient(id)),
      onSuccess: () => {
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function invalidateClients(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.clients.all });
}
