import {
  createMutation,
  createQuery,
  type QueryClient,
} from "@tanstack/svelte-query";
import { toast } from "svelte-sonner";

import { commands } from "$lib/bindings";
import type {
  CreateLaborItemPayload,
  CreatePartItemPayload,
  CreateWorkOrderPayload,
  DiscountType,
  DocumentConflictAction,
  DocumentFormat,
  DocumentKind,
  GenerateDocumentOptions,
  LaborItem,
  PartItem,
  PartStatus,
  UpdateLaborItemPayload,
  UpdatePartItemPayload,
  UpdateWorkOrderPayload,
  WorkOrderDeleteImpact,
  WorkOrderDetailsDto,
  WorkOrderDocuments,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import { unwrap } from "$lib/tools";

function showMutationErrorToast(error: Error) {
  toast.error("Виникла помилка", {
    description: error.message,
    position: "bottom-left",
  });
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

export function workOrderDetailsQueryOptions(id: string | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.detail(id!),
    queryFn: () => unwrap(commands.fetchWorkOrderDetailsDto(id!)),
  };
}

export function createWorkOrderDetailsQuery(getId: () => string | null) {
  return createQuery<WorkOrderDetailsDto>(() =>
    workOrderDetailsQueryOptions(getId()),
  );
}

export function workOrderDetailsListByCarIdQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.detail(id!),
    queryFn: () => unwrap(commands.fetchWorkOrderDetailsDtoByCarId(id!)),
  };
}

export function createWorkOrderDetailsListByCarIdQuery(
  getId: () => number | null,
) {
  return createQuery<WorkOrderDetailsDto[]>(() =>
    workOrderDetailsListByCarIdQueryOptions(getId()),
  );
}

export function workOrderDetailsListByClientIdQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.detail(id!),
    queryFn: () => unwrap(commands.fetchWorkOrderDetailsDtoByClientId(id!)),
  };
}

export function createWorkOrderDetailsListByClientIdQuery(
  getId: () => number | null,
) {
  return createQuery<WorkOrderDetailsDto[]>(() =>
    workOrderDetailsListByClientIdQueryOptions(getId()),
  );
}

export function workOrderDetailsListQueryOptions() {
  return {
    queryKey: keys.workOrders.list(),
    queryFn: () => unwrap(commands.fetchWorkOrderDetailsDtoList()),
  };
}

export function createWorkOrderDetailsDtoListQuery() {
  return createQuery<WorkOrderDetailsDto[]>(() =>
    workOrderDetailsListQueryOptions(),
  );
}

export function workOrderDeleteImpactQueryOptions(
  id: string | null,
  enabled: boolean,
) {
  return {
    enabled: enabled && id != null,
    queryKey: keys.workOrders.deleteImpact(id!),
    queryFn: () => unwrap(commands.fetchWorkOrderDeleteImpact(id!)),
  };
}

export function createWorkOrderDeleteImpactQuery(
  getId: () => string | null,
  getEnabled: () => boolean,
) {
  return createQuery<WorkOrderDeleteImpact>(() =>
    workOrderDeleteImpactQueryOptions(getId(), getEnabled()),
  );
}

export function createCreateWorkOrderMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: CreateWorkOrderPayload) =>
        unwrap(commands.createWorkOrder(payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createCreateWarrantyWorkOrderMutation(
  queryClient: QueryClient,
) {
  return createMutation(
    () => ({
      mutationFn: (variables: {
        sourceWorkOrderId: string;
        odometerIn: number;
        description: string | null;
        notes: string | null;
      }) =>
        unwrap(
          commands.createWarrantyWorkOrder(
            variables.sourceWorkOrderId,
            variables.odometerIn,
            variables.description,
            variables.notes,
          ),
        ),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateWorkOrderMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: UpdateWorkOrderPayload) =>
        unwrap(commands.updateWorkOrder(payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteWorkOrderMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: string) => unwrap(commands.deleteWorkOrder(id)),
      onSuccess: async () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createArchiveWorkOrderMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: string) => unwrap(commands.archiveWorkOrder(id)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createRestoreWorkOrderMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: string) => unwrap(commands.restoreWorkOrder(id)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function laborItemByIdQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.laborItem(id!),
    queryFn: () => unwrap(commands.fetchLaborItemById(id!)),
  };
}

export function createLaborItemByIdQuery(getId: () => number | null) {
  return createQuery<LaborItem>(() => laborItemByIdQueryOptions(getId()));
}

export function laborItemsQueryOptions() {
  return {
    queryKey: keys.workOrders.laborItems(),
    queryFn: () => unwrap(commands.fetchLaborItemsList()),
  };
}

export function createLaborItemsQuery() {
  return createQuery<LaborItem[]>(() => laborItemsQueryOptions());
}

export function createCreateLaborItemMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: async (payload: CreateLaborItemPayload) =>
        unwrap(commands.createLaborItem(payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateLaborItemMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: async (variables: {
        id: number;
        payload: UpdateLaborItemPayload;
      }) => unwrap(commands.updateLaborItem(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteLaborItemMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.deleteLaborItem(id)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeleteLaborItemsMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (ids: number[]) => unwrap(commands.deleteLaborItems(ids)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdateLaborItemsDiscountMutation(
  queryClient: QueryClient,
) {
  return createMutation(
    () => ({
      mutationFn: (variables: {
        ids: number[];
        discountType: DiscountType;
        discountValue: string;
      }) =>
        unwrap(
          commands.updateLaborItemsDiscount(
            variables.ids,
            variables.discountType,
            variables.discountValue,
          ),
        ),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function partItemByIdQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.partItem(id!),
    queryFn: () => unwrap(commands.fetchPartItemById(id!)),
  };
}

export function createPartItemByIdQuery(getId: () => number | null) {
  return createQuery<PartItem>(() => partItemByIdQueryOptions(getId()));
}

export function partItemsQueryOptions() {
  return {
    queryKey: keys.workOrders.partItems(),
    queryFn: () => unwrap(commands.fetchPartItemsList()),
  };
}

export function createPartItemsQuery() {
  return createQuery<PartItem[]>(() => partItemsQueryOptions());
}

export function createCreatePartItemMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (payload: CreatePartItemPayload) =>
        unwrap(commands.createPartItem(payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdatePartItemMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: { id: number; payload: UpdatePartItemPayload }) =>
        unwrap(commands.updatePartItem(variables.id, variables.payload)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeletePartItemMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (id: number) => unwrap(commands.deletePartItem(id)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDeletePartItemsMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (ids: number[]) => unwrap(commands.deletePartItems(ids)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdatePartItemsStatusMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: { ids: number[]; status: PartStatus }) =>
        unwrap(commands.updatePartItemsStatus(variables.ids, variables.status)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createUpdatePartItemsDiscountMutation(
  queryClient: QueryClient,
) {
  return createMutation(
    () => ({
      mutationFn: (variables: {
        ids: number[];
        discountType: DiscountType;
        discountValue: string;
      }) =>
        unwrap(
          commands.updatePartItemsDiscount(
            variables.ids,
            variables.discountType,
            variables.discountValue,
          ),
        ),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function invalidateWorkOrderLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.workOrders.all });
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

export function workOrderDocumentsQueryOptions(id: string | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.documents(id!),
    queryFn: () => unwrap(commands.fetchWorkOrderDocuments(id!)),
  };
}

export function createWorkOrderDocumentsQuery(getId: () => string | null) {
  return createQuery<WorkOrderDocuments>(() =>
    workOrderDocumentsQueryOptions(getId()),
  );
}

export function createGenerateDocumentMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationFn: (variables: {
        workOrderId: string;
        kind: DocumentKind;
        format: DocumentFormat;
        options: GenerateDocumentOptions;
        conflict: DocumentConflictAction;
      }) =>
        unwrap(
          commands.generateDocument(
            variables.workOrderId,
            variables.kind,
            variables.format,
            variables.options,
            variables.conflict,
          ),
        ),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}
