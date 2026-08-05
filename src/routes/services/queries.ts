import { createQuery, QueryClient } from "@tanstack/svelte-query";

import { commands, type Service } from "$lib/bindings";

import { unwrap } from "$lib/misc";
import { keys } from "$lib/query-keys";

export function servicesQueryOptions() {
  return {
    queryKey: keys.services.list(),
    queryFn: () => unwrap(commands.fetchServicesList()),
  };
}

export function createServicesQuery() {
  return createQuery<Service[]>(() => servicesQueryOptions());
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

export function invalidateServiceLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.services.list() });
  queryClient.invalidateQueries({ queryKey: keys.services.count() });
}
