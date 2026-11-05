import {
  createMutation,
  createQuery,
  QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type CarBrand,
  type CarDeleteImpact,
  type CarDetailsDto,
  type CarDto,
  type CarModelSuggestion,
  type CreateCarPayload,
  type UpdateCarPayload,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

import { invalidateClients } from "../clients/queries";

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

export function carDeleteImpactQueryOptions(
  id: number | null,
  enabled: boolean,
) {
  return {
    enabled: enabled && id != null,
    queryKey: keys.cars.deleteImpact(id!),
    queryFn: () => unwrap(commands.fetchCarDeleteImpact(id!)),
  };
}

export function createCarDeleteImpactQuery(
  getId: () => number | null,
  getEnabled: () => boolean,
) {
  return createQuery<CarDeleteImpact>(() =>
    carDeleteImpactQueryOptions(getId(), getEnabled()),
  );
}

export function createCreateCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "createCar"],
      mutationFn: async (payload: CreateCarPayload) =>
        unwrap(commands.createCar(payload)),
      onSuccess: () => {
        invalidateCars(queryClient);
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "updateCar"],
      mutationFn: async (variables: {
        id: number;
        payload: UpdateCarPayload;
      }) => unwrap(commands.updateCar(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateCars(queryClient);
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "deleteCar"],
      mutationFn: (id: number) => unwrap(commands.deleteCar(id)),
      onSuccess: async () => {
        invalidateCars(queryClient);
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createArchiveCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "archiveCar"],
      mutationFn: (id: number) => unwrap(commands.archiveCar(id)),
      onSuccess: () => {
        invalidateCars(queryClient);
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createRestoreCarMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "restoreCar"],
      mutationFn: (id: number) => unwrap(commands.restoreCar(id)),
      onSuccess: () => {
        invalidateCars(queryClient);
        invalidateClients(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function carBrandsQueryOptions() {
  return {
    queryKey: keys.carBrand.list(),
    queryFn: () => unwrap(commands.fetchCarBrands()),
  };
}

export function createCarBrandsQuery() {
  return createQuery<CarBrand[]>(() => carBrandsQueryOptions());
}

export function customCarBrandsQueryOptions() {
  return {
    queryKey: keys.carBrand.custom(),
    queryFn: () => unwrap(commands.fetchCustomCarBrands()),
  };
}

export function createCustomCarBrandsQuery() {
  return createQuery<CarBrand[]>(() => customCarBrandsQueryOptions());
}

export function carModelsQueryOptions(brandId: number) {
  return {
    enabled: brandId > 0,
    queryKey: keys.carModel.list(brandId),
    queryFn: () => unwrap(commands.fetchCarModels(brandId)),
  };
}

export function createCarModelsQuery(getBrandId: () => number) {
  return createQuery<CarModelSuggestion[]>(() =>
    carModelsQueryOptions(getBrandId()),
  );
}

export function createCreateCarBrandMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "createCarBrand"],
      mutationFn: async (name: string) => unwrap(commands.createCarBrand(name)),
      onSuccess: (brand) => {
        invalidateCarBrands(queryClient);
        toast.success(`Марку «${brand.name}» додано`, {
          position: "bottom-left",
        });
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateCarBrandMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "updateCarBrand"],
      mutationFn: async (variables: { id: number; name: string }) =>
        unwrap(commands.updateCarBrand(variables.id, variables.name)),
      onSuccess: () => {
        invalidateCarBrands(queryClient);
        toast.success("Марку перейменовано", { position: "bottom-left" });
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createArchiveCarBrandMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "archiveCarBrand"],
      mutationFn: (id: number) => unwrap(commands.archiveCarBrand(id)),
      onSuccess: () => {
        invalidateCarBrands(queryClient);
        toast.success("Марку архівовано", { position: "bottom-left" });
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createRestoreCarBrandMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["cars", "restoreCarBrand"],
      mutationFn: (id: number) => unwrap(commands.restoreCarBrand(id)),
      onSuccess: () => {
        invalidateCarBrands(queryClient);
        toast.success("Марку відновлено", { position: "bottom-left" });
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function invalidateCarBrands(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.carBrand.all });
  queryClient.invalidateQueries({ queryKey: keys.cars.all });
  queryClient.invalidateQueries({ queryKey: keys.carModel.all });
  // Client details carry their cars, and those carry the brand name.
  queryClient.invalidateQueries({ queryKey: keys.clients.all });
  queryClient.invalidateQueries({ queryKey: keys.workOrders.all });
}

export function invalidateCars(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.cars.all });
  // Model suggestions are the names already used on cars, so they move with them.
  queryClient.invalidateQueries({ queryKey: keys.carModel.all });
}
