import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type CarBrand,
  type CarDetailsDto,
  type CarDto,
  type CreateCarPayload,
  type UpdateCarPayload,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

function showMutationErrorToast(error: Error) {
  toast.error("Виникла помилка", {
    description: error.message,
    position: "bottom-left",
  });
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

export function carsListQueryOptions() {
  return {
    queryKey: keys.cars.list(),
    queryFn: async () => unwrap(commands.fetchCarsList()),
  };
}

export function createCarsListQuery() {
  return createQuery<CarDto[]>(() => carsListQueryOptions());
}

export function carDetailsQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.cars.detail(id!),
    queryFn: async () => unwrap(commands.fetchCarDetails(id!)),
  };
}

export function createCarDetailsQuery(getId: () => number | null) {
  return createQuery<CarDetailsDto>(() => carDetailsQueryOptions(getId()));
}

export function carDetailsListQueryOptions() {
  return {
    queryKey: keys.cars.listDetails(),
    queryFn: async () => unwrap(commands.fetchCarDetailsList()),
  };
}

export function createCarDetailsListQuery() {
  return createQuery<CarDetailsDto[]>(() => carDetailsListQueryOptions());
}

export function createCreateCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: async (payload: CreateCarPayload) =>
        unwrap(commands.createCar(payload)),
      onSuccess: () => {
        invalidateCars(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: async (variables: {
        id: number;
        payload: UpdateCarPayload;
      }) => unwrap(commands.updateCar(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateCars(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.deleteCar(id)),
      onSuccess: async () => {
        invalidateCars(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
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

export function invalidateCars(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.cars.all });
}
