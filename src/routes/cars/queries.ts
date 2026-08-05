import { createQuery, QueryClient } from "@tanstack/svelte-query";

import { commands, type Car, type CarBrand } from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

export function carsQueryOptions() {
  return {
    queryKey: keys.cars.list(),
    queryFn: async () => unwrap(commands.fetchCarsList()),
  };
}

export function createCarsQuery() {
  return createQuery<Car[]>(() => carsQueryOptions());
}

export function carsCountQueryOptions() {
  return {
    queryKey: keys.cars.count(),
    queryFn: () => unwrap(commands.fetchCarsCount()),
  };
}

export function createCarsCountQuery() {
  return createQuery<number>(() => carsCountQueryOptions());
}

export function carBrandsQueryOptions() {
  return {
    queryKey: keys.carBrand,
    queryFn: () => unwrap(commands.fetchCarBrands()),
  };
}

export function createCarBrandsQuery() {
  return createQuery<CarBrand[]>(() => carBrandsQueryOptions());
}

export function invalidateCarsLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.cars.list() });
  queryClient.invalidateQueries({ queryKey: keys.cars.count() });
}
