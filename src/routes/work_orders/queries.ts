import {
  createMutation,
  createQuery,
  type QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import {
  commands,
  type WorkOrderPartItem,
  type WorkOrderTotal,
} from "$lib/bindings";
import type {
  CreateWorkOrderLaborItemPayload,
  UpdateWorkOrderPayload,
  WorkOrder,
  WorkOrderLaborItem,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

export function workOrdersQueryOptions() {
  return {
    queryKey: keys.workOrders.lists(),
    queryFn: () => unwrap(commands.fetchWorkOrdersList()),
  };
}

export function createWorkOrdersQuery() {
  return createQuery<WorkOrder[]>(() => workOrdersQueryOptions());
}

export function workOrdersCountQueryOptions() {
  return {
    queryKey: keys.workOrders.count(),
    queryFn: () => unwrap(commands.fetchWorkOrdersCount()),
  };
}

export function createWorkOrdersCountQuery() {
  return createQuery<number>(() => workOrdersCountQueryOptions());
}

export function workOrderLaborItemsQueryOptions() {
  return {
    queryKey: keys.workOrders.laborItems(),
    queryFn: () => unwrap(commands.fetchWorkOrderLaborItemsList()),
  };
}

export function createWorkOrderLaborItemsQuery() {
  return createQuery<WorkOrderLaborItem[]>(() =>
    workOrderLaborItemsQueryOptions(),
  );
}

export function workOrderPartItemsQueryOptions() {
  return {
    queryKey: keys.workOrders.partItems(),
    queryFn: () => unwrap(commands.fetchWorkOrderPartItemsList()),
  };
}

export function createWorkOrderPartItemsQuery() {
  return createQuery<WorkOrderPartItem[]>(() =>
    workOrderPartItemsQueryOptions(),
  );
}

export function workOrderTotalsQueryOptions() {
  return {
    queryKey: keys.workOrders.totals(),
    queryFn: () => unwrap(commands.fetchWorkOrderTotals()),
  };
}

export function createWorkOrderTotalsQuery() {
  return createQuery<WorkOrderTotal[]>(() => workOrderTotalsQueryOptions());
}

export function createUpdateWorkOrderMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: UpdateWorkOrderPayload) =>
        unwrap(commands.updateWorkOrder(payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: (error) => {
        toast.error("Виникла помилка", {
          description: error.message,
          position: "bottom-left",
        });
      },
    }),
    () => queryClient,
  );
}

export function createCreateWorkOrderLaborItemMutation(
  queryClient: QueryClient,
) {
  return createMutation(
    () => ({
      mutationFn: async (payload: CreateWorkOrderLaborItemPayload) =>
        unwrap(commands.createWorkOrderLaborItem(payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: (error) => {
        toast.error("Виникла помилка", {
          description: error.message,
          position: "bottom-left",
        });
      },
    }),
    () => queryClient,
  );
}

export function invalidateWorkOrderLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.workOrders.all });
}
