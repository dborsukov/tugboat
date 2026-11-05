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
  DocumentFormat,
  DocumentKind,
  GenerateDocumentOptions,
  LaborItem,
  PartItem,
  PartStatus,
  UpdateLaborItemPayload,
  UpdatePartItemPayload,
  UpdateWorkOrderPayload,
  VatEntry,
  WorkOrderDeleteImpact,
  WorkOrderDetailsDto,
  WorkOrderDocuments,
} from "$lib/bindings";
import { keys } from "$lib/query-keys";
import {
  blockDocumentGeneration,
  SetupBlockedError,
} from "$lib/states/setup-attention.svelte";
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
      mutationKey: ["workOrders", "createWorkOrder"],
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
      mutationKey: ["workOrders", "createWarrantyWorkOrder"],
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
      mutationKey: ["workOrders", "updateWorkOrder"],
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
      mutationKey: ["workOrders", "deleteWorkOrder"],
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
      mutationKey: ["workOrders", "archiveWorkOrder"],
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
      mutationKey: ["workOrders", "restoreWorkOrder"],
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
      mutationKey: ["workOrders", "createLaborItem"],
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
      mutationKey: ["workOrders", "updateLaborItem"],
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
      mutationKey: ["workOrders", "deleteLaborItem"],
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
      mutationKey: ["workOrders", "deleteLaborItems"],
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
      mutationKey: ["workOrders", "updateLaborItemsDiscount"],
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

export function createReorderLaborItemsMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "reorderLaborItems"],
      mutationFn: (variables: { workOrderId: string; orderedIds: number[] }) =>
        unwrap(
          commands.reorderLaborItems(
            variables.workOrderId,
            variables.orderedIds,
          ),
        ),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: (error) => {
        showMutationErrorToast(error);
        invalidateWorkOrderLists(queryClient);
      },
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
      mutationKey: ["workOrders", "createPartItem"],
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
      mutationKey: ["workOrders", "updatePartItem"],
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
      mutationKey: ["workOrders", "deletePartItem"],
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
      mutationKey: ["workOrders", "deletePartItems"],
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
      mutationKey: ["workOrders", "updatePartItemsStatus"],
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

export function createUpdatePartItemsSupplierMutation(
  queryClient: QueryClient,
) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "updatePartItemsSupplier"],
      mutationFn: (variables: {
        ids: number[];
        supplier: string | null;
        supplierOrderId: string | null;
      }) =>
        unwrap(
          commands.updatePartItemsSupplier(
            variables.ids,
            variables.supplier,
            variables.supplierOrderId,
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

export function createUpdatePartItemsDiscountMutation(
  queryClient: QueryClient,
) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "updatePartItemsDiscount"],
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

export function createReorderPartItemsMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "reorderPartItems"],
      mutationFn: (variables: { workOrderId: string; orderedIds: number[] }) =>
        unwrap(
          commands.reorderPartItems(
            variables.workOrderId,
            variables.orderedIds,
          ),
        ),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: (error) => {
        showMutationErrorToast(error);
        invalidateWorkOrderLists(queryClient);
      },
    }),
    () => queryClient,
  );
}

export function invalidateWorkOrderLists(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: keys.workOrders.all });
  queryClient.invalidateQueries({ queryKey: keys.dashboard.all });
}

// ---------------------------------------------------------------------------
// VAT
// ---------------------------------------------------------------------------

export function workOrderVatQueryOptions(id: string | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.vat(id!),
    queryFn: () => unwrap(commands.fetchWorkOrderVat(id!)),
  };
}

export function createWorkOrderVatQuery(getId: () => string | null) {
  return createQuery<VatEntry[]>(() => workOrderVatQueryOptions(getId()));
}

/** The entry in force: neither a reversal nor a voided original. */
export function liveVatEntry(entries: VatEntry[] | undefined) {
  return entries?.find(
    (entry) => entry.reversesId == null && entry.voidedAt == null,
  );
}

export function createVoidWorkOrderVatMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "voidWorkOrderVat"],
      mutationFn: (variables: {
        id: string;
        creditNoteNumber: string | null;
        creditNoteDate: string | null;
      }) =>
        unwrap(
          commands.voidWorkOrderVat(
            variables.id,
            variables.creditNoteNumber,
            variables.creditNoteDate,
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

export function createVoidPartItemVatMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "voidPartItemVat"],
      mutationFn: (variables: {
        id: number;
        creditNoteNumber: string | null;
        creditNoteDate: string | null;
      }) =>
        unwrap(
          commands.voidPartItemVat(
            variables.id,
            variables.creditNoteNumber,
            variables.creditNoteDate,
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

export function partItemVatQueryOptions(id: number | null) {
  return {
    enabled: id != null,
    queryKey: keys.workOrders.vat(`part-${id}`),
    queryFn: () => unwrap(commands.fetchPartItemVat(id!)),
  };
}

export function createPartItemVatQuery(getId: () => number | null) {
  return createQuery<VatEntry[]>(() => partItemVatQueryOptions(getId()));
}

export function createRecordWorkOrderVatMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "recordWorkOrderVat"],
      mutationFn: (id: string) => unwrap(commands.recordWorkOrderVat(id)),
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
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
      mutationKey: ["workOrders", "generateDocument"],
      mutationFn: (variables: {
        workOrderId: string;
        kind: DocumentKind;
        options: GenerateDocumentOptions;
      }) => {
        // Guards background generation too: no entry point can write a
        // document while the seller's details are incomplete.
        if (blockDocumentGeneration()) throw new SetupBlockedError();

        return unwrap(
          commands.generateDocument(
            variables.workOrderId,
            variables.kind,
            variables.options,
          ),
        );
      },
      onSuccess: () => {
        invalidateWorkOrderLists(queryClient);
      },
      onError: (error) => {
        if (error instanceof SetupBlockedError) return;
        showMutationErrorToast(error);
      },
    }),
    () => queryClient,
  );
}

export function createOpenWorkOrderDocumentMutation(queryClient: QueryClient) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "openWorkOrderDocument"],
      mutationFn: (variables: { id: number; format: DocumentFormat }) =>
        unwrap(commands.openWorkOrderDocument(variables.id, variables.format)),
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}

export function createDownloadWorkOrderDocumentMutation(
  queryClient: QueryClient,
) {
  return createMutation(
    () => ({
      mutationKey: ["workOrders", "downloadWorkOrderDocument"],
      mutationFn: (variables: {
        id: number;
        format: DocumentFormat;
        fileName: string;
      }) =>
        unwrap(
          commands.downloadWorkOrderDocument(
            variables.id,
            variables.format,
            variables.fileName,
          ),
        ),
      onError: showMutationErrorToast,
    }),
    () => queryClient,
  );
}
