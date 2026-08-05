import { createQuery, QueryClient } from "@tanstack/svelte-query";

import { commands, type Client } from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

export function clientsQueryOptions() {
  return {
    queryKey: keys.clients.list(),
    queryFn: () => unwrap(commands.fetchClientsList()),
  };
}

export function createClientsQuery() {
  return createQuery<Client[]>(() => clientsQueryOptions());
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

export function invalidateClientsLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.clients.list() });
  queryClient.invalidateQueries({ queryKey: keys.clients.count() });
}
