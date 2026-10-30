<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import { onMount } from "svelte";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import DataTableBulkActions from "$lib/components/data-table/data-table-bulk-actions.svelte";
  import type { RowReorderOptions } from "$lib/components/data-table/data-table-reorder";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";
  import ServicePicker from "$lib/components/sheet/service-picker.svelte";

  import type {
    DiscountType,
    DocumentKind,
    GenerateDocumentOptions,
    LaborItemDto,
    PartItemDto,
    PartStatus,
    Service,
    WorkOrderDetailsDto,
    WorkOrderStatus,
  } from "$lib/bindings";
  import { commands } from "$lib/bindings";
  import { keys } from "$lib/query-keys";
  import { recordRecentItem } from "$lib/recent-items";
  import { privacyState } from "$lib/states/privacy.svelte";
  import { settingsState } from "$lib/states/settings.svelte";
  import { blockDocumentGeneration } from "$lib/states/setup-attention.svelte";
  import { TableActionState } from "$lib/states/table-actions.svelte";
  import { pluralizeUk, unwrap } from "$lib/tools";

  import { clientsQueryOptions } from "../../clients/queries";
  import { servicesQueryOptions } from "../../services/queries";
  import {
    createCreateLaborItemMutation,
    createDeleteLaborItemMutation,
    createDeleteLaborItemsMutation,
    createDeletePartItemMutation,
    createDeletePartItemsMutation,
    createReorderLaborItemsMutation,
    createReorderPartItemsMutation,
    createUpdateLaborItemsDiscountMutation,
    createUpdatePartItemsDiscountMutation,
    createUpdatePartItemsStatusMutation,
    createUpdatePartItemsSupplierMutation,
    liveVatEntry,
    workOrderDetailsQueryOptions,
    workOrderVatQueryOptions,
  } from "../queries";
  import BulkDiscountDialog from "./bulk-discount-dialog.svelte";
  import BulkDiscountMenuItem from "./bulk-discount-menu-item.svelte";
  import DocumentGenerateDialog from "./document-generate-dialog.svelte";
  import DocumentsSection from "./documents-section.svelte";
  import { applyItemOrder, isSameOrder, moveItemById } from "./item-order";
  import WorkOrderLaborItemMutationSheet from "./labor-item-mutation-sheet.svelte";
  import LaborItemsTable from "./labor-items-table.svelte";
  import PartBulkStatusSubmenu from "./part-bulk-status-submenu.svelte";
  import PartBulkSupplierDialog from "./part-bulk-supplier-dialog.svelte";
  import PartBulkSupplierMenuItem from "./part-bulk-supplier-menu-item.svelte";
  import WorkOrderPartsItemMutationSheet from "./part-item-mutation-sheet.svelte";
  import PartItemsTable from "./part-items-table.svelte";
  import VatSection from "./vat-section.svelte";
  import WorkOrderHeader from "./work-order-header.svelte";
  import { PURCHASE_COLUMN_KEY } from "./work-order-table";

  let { params } = $props();

  let isPopoverOpen: boolean = $state(false);

  let generateRequest = $state<{
    kind: DocumentKind;
    /** Choices of the version being updated, when it is an update. */
    seed: GenerateDocumentOptions | null;
  } | null>(null);
  let isGenerateDialogOpen = $state(false);

  function openGenerate(
    kind: DocumentKind,
    seed: GenerateDocumentOptions | null = null,
  ) {
    // Every generate/regenerate entry point passes through here.
    if (blockDocumentGeneration()) return;

    generateRequest = { kind, seed };
    isGenerateDialogOpen = true;
  }

  const queryClient = useQueryClient();

  const deletePartItemMutation = createDeletePartItemMutation(queryClient);
  const deletePartItemsMutation = createDeletePartItemsMutation(queryClient);
  const createLaborItemMutation = createCreateLaborItemMutation(queryClient);
  const deleteLaborItemMutation = createDeleteLaborItemMutation(queryClient);
  const deleteLaborItemsMutation = createDeleteLaborItemsMutation(queryClient);
  const updatePartItemsStatusMutation =
    createUpdatePartItemsStatusMutation(queryClient);
  const updateLaborItemsDiscountMutation =
    createUpdateLaborItemsDiscountMutation(queryClient);
  const updatePartItemsDiscountMutation =
    createUpdatePartItemsDiscountMutation(queryClient);
  const updatePartItemsSupplierMutation =
    createUpdatePartItemsSupplierMutation(queryClient);
  const reorderLaborItemsMutation =
    createReorderLaborItemsMutation(queryClient);
  const reorderPartItemsMutation = createReorderPartItemsMutation(queryClient);

  const combinedQueries = createQueries(() => ({
    queries: [
      workOrderDetailsQueryOptions(params.workorderId),
      servicesQueryOptions(),
      clientsQueryOptions(),
      workOrderVatQueryOptions(params.workorderId),
    ],
    combine: (results) => {
      return {
        workOrderDetails: results[0],
        services: results[1],
        clients: results[2],
        vat: results[3],

        isError: results.some((r) => r.isError),
        firstError: results.find((r) => r.isError)?.error,
        refetchAll: () => {
          results.forEach((r) => r.refetch());
        },
      };
    },
  }));

  let services = $derived(combinedQueries.services.data);
  let clients = $derived(combinedQueries.clients.data);
  let workOrderDetails = $derived(combinedQueries.workOrderDetails.data);
  let vatEntries = $derived(combinedQueries.vat.data);

  $effect(() => {
    const id = workOrderDetails?.workOrder?.id;
    if (id) void recordRecentItem("work_order", id);
  });

  const liveVat = $derived(liveVatEntry(vatEntries));
  const hasLiveVat = $derived(Boolean(liveVat));

  const orderIsPending = $derived(
    combinedQueries.workOrderDetails.isPending ||
      combinedQueries.services.isPending,
  );
  const orderIsError = $derived(
    combinedQueries.workOrderDetails.isError ||
      combinedQueries.services.isError,
  );

  const isArchived = $derived(workOrderDetails?.workOrder?.archivedAt != null);

  /** The work has been handed over, which is when VAT becomes due. */
  const statusIsSupplied = $derived(
    ["ReadyForPickup", "AwaitingPayment", "Completed", "Paid"].includes(
      workOrderDetails?.workOrder?.status ?? "",
    ),
  );

  let isLocked = $state(true);
  let previousStatus: WorkOrderStatus | undefined;

  // Archived orders are read-only regardless of the paid-lock state.
  const businessIsVatPayer = $derived(settingsState.isVatPayer);
  const isReadOnly = $derived(isLocked || isArchived);

  $effect(() => {
    const currentStatus = workOrderDetails?.workOrder?.status;
    if (currentStatus === undefined) return;

    // A recorded VAT entry freezes the figures just as a terminal status does,
    // even when the order has not been paid yet: the tax point already happened.
    const currentTerminal = isTerminal(currentStatus) || hasLiveVat;

    if (previousStatus === undefined) {
      isLocked = currentTerminal;
    } else if (currentTerminal && !isTerminal(previousStatus)) {
      isLocked = true;
    }

    previousStatus = currentStatus;
  });

  function isTerminal(status: WorkOrderStatus): boolean {
    return (
      status === "Paid" ||
      status === "AwaitingPayment" ||
      status === "Completed"
    );
  }

  let partsTableActions = new TableActionState<number>();
  let laborTableActions = new TableActionState<number>();

  let isLaborBulkDiscountOpen = $state(false);
  let isPartsBulkDiscountOpen = $state(false);
  let isPartsBulkSupplierOpen = $state(false);
  let partsBulkSupplierTargets = $state<PartItemDto[]>([]);
  // Row snapshots taken when the bulk discount dialog is opened, so the
  // preflight quotes and the mutation always target the same rows even if
  // the selection changes while the dialog is on screen.
  let laborBulkTargets = $state<{ id: number }[]>([]);
  let partsBulkTargets = $state<{ id: number }[]>([]);
  let laborBulkRowsById = new Map<number, LaborItemDto>();
  let partsBulkRowsById = new Map<number, PartItemDto>();

  let selectedLaborIds = $state<number[]>([]);
  let selectedPartIds = $state<number[]>([]);

  const selectedPartRows = $derived(
    (workOrderDetails?.partItems ?? []).filter((item) =>
      selectedPartIds.includes(item.id),
    ),
  );
  const canBulkChangeSupplier = $derived(
    selectedPartRows.length > 0 &&
      selectedPartRows.every((item) => item.providedBy === "shop"),
  );

  function openPartEdit(id: number) {
    partsTableActions.openEdit(id);
  }

  let tablesContainer = $state<HTMLDivElement | null>(null);
  let storedPurchase = $state<boolean | null>(null);
  let autoPurchase = $state(false);
  let purchasePreferred = $derived(storedPurchase ?? autoPurchase);
  let showPurchase = $derived(purchasePreferred && !privacyState.hidePricing);

  onMount(() => {
    try {
      const raw = localStorage.getItem(PURCHASE_COLUMN_KEY);
      if (raw === "1" || raw === "0") storedPurchase = raw === "1";
    } catch {
      // storage unavailable, default is used
    }
  });

  $effect(() => {
    const element = tablesContainer;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      if (storedPurchase === null)
        autoPurchase = entry.contentRect.width >= 900;
    });
    observer.observe(element);
    return () => observer.disconnect();
  });

  function setPurchaseColumn(visible: boolean) {
    storedPurchase = visible;
    try {
      localStorage.setItem(PURCHASE_COLUMN_KEY, visible ? "1" : "0");
    } catch {}
  }

  // ---------------------------------------------------------------------------
  // Item order
  // ---------------------------------------------------------------------------

  const ORDER_TRANSITION_MS = 180;

  const laborItemIds = $derived(
    (workOrderDetails?.laborItems ?? []).map((item) => item.id),
  );
  const partItemIds = $derived(
    (workOrderDetails?.partItems ?? []).map((item) => item.id),
  );

  let laborDragStartOrder: number[] | null = null;
  let partDragStartOrder: number[] | null = null;

  function setLaborItemsOrder(orderedIds: number[]) {
    queryClient.setQueryData<WorkOrderDetailsDto>(
      keys.workOrders.detail(params.workorderId),
      (current) =>
        current
          ? {
              ...current,
              laborItems: applyItemOrder(current.laborItems, orderedIds),
            }
          : current,
    );
  }

  function setPartItemsOrder(orderedIds: number[]) {
    queryClient.setQueryData<WorkOrderDetailsDto>(
      keys.workOrders.detail(params.workorderId),
      (current) =>
        current
          ? {
              ...current,
              partItems: applyItemOrder(current.partItems, orderedIds),
            }
          : current,
    );
  }

  function persistLaborOrder(orderedIds: number[]) {
    reorderLaborItemsMutation.mutate({
      workOrderId: params.workorderId,
      orderedIds,
    });
  }

  function persistPartOrder(orderedIds: number[]) {
    reorderPartItemsMutation.mutate({
      workOrderId: params.workorderId,
      orderedIds,
    });
  }

  function moveLaborItem(id: number, delta: number) {
    const items = workOrderDetails?.laborItems ?? [];
    const next = moveItemById(items, id, delta);
    if (next === items) return;
    const orderedIds = next.map((item) => item.id);
    setLaborItemsOrder(orderedIds);
    persistLaborOrder(orderedIds);
  }

  function movePartItem(id: number, delta: number) {
    const items = workOrderDetails?.partItems ?? [];
    const next = moveItemById(items, id, delta);
    if (next === items) return;
    const orderedIds = next.map((item) => item.id);
    setPartItemsOrder(orderedIds);
    persistPartOrder(orderedIds);
  }

  const toItemIds = (rowIds: string[]) => rowIds.map(Number);

  const laborRowReorder = $derived(
    isReadOnly
      ? undefined
      : ({
          type: "work-order-labor-items",
          transition: { duration: ORDER_TRANSITION_MS },
          onStart: () => {
            laborDragStartOrder = [...laborItemIds];
          },
          onMove: (rowIds: string[]) => setLaborItemsOrder(toItemIds(rowIds)),
          onCommit: () => {
            const startOrder = laborDragStartOrder;
            laborDragStartOrder = null;
            // Dropped where it started: the backend already has this order.
            if (startOrder && isSameOrder(startOrder, laborItemIds)) return;
            persistLaborOrder(laborItemIds);
          },
          onCancel: () => {
            if (laborDragStartOrder) setLaborItemsOrder(laborDragStartOrder);
            laborDragStartOrder = null;
          },
        } satisfies RowReorderOptions),
  );

  const partRowReorder = $derived(
    isReadOnly
      ? undefined
      : ({
          type: "work-order-part-items",
          transition: { duration: ORDER_TRANSITION_MS },
          onStart: () => {
            partDragStartOrder = [...partItemIds];
          },
          onMove: (rowIds: string[]) => setPartItemsOrder(toItemIds(rowIds)),
          onCommit: () => {
            const startOrder = partDragStartOrder;
            partDragStartOrder = null;
            // Dropped where it started: the backend already has this order.
            if (startOrder && isSameOrder(startOrder, partItemIds)) return;
            persistPartOrder(partItemIds);
          },
          onCancel: () => {
            if (partDragStartOrder) setPartItemsOrder(partDragStartOrder);
            partDragStartOrder = null;
          },
        } satisfies RowReorderOptions),
  );

  async function createCreateLaborItemFromService(
    service: Service,
    keepPickerOpen = false,
  ) {
    createLaborItemMutation.mutateAsync(
      {
        workOrderId: params.workorderId,
        serviceId: service.id,
        code: service.code,
        name: service.name,
        priceType: service.priceType,
        fixedPrice: service.fixedPrice,
        laborHours: service.laborHours,
        discountType: "None",
        discountValue: "0",
        isFree: workOrderDetails?.workOrder?.type === "Warranty",
        vatRate: null,
        vatScheme: null,
      },
      {
        onSuccess: () => {
          if (!keepPickerOpen) isPopoverOpen = false;
        },
      },
    );
  }

  function confirmPartDelete() {
    if (partsTableActions.deleteId !== null) {
      deletePartItemMutation.mutate(partsTableActions.deleteId, {
        onSuccess: partsTableActions.resetDelete,
      });
    }
  }

  function confirmLaborDelete() {
    if (laborTableActions.deleteId !== null) {
      deleteLaborItemMutation.mutate(laborTableActions.deleteId, {
        onSuccess: laborTableActions.resetDelete,
      });
    }
  }

  function confirmBulkLaborDelete() {
    if (laborTableActions.bulkDeleteIds.length > 0) {
      deleteLaborItemsMutation.mutate([...laborTableActions.bulkDeleteIds], {
        onSuccess: () => {
          laborTableActions.resetBulkDelete();
          selectedLaborIds = [];
        },
      });
    }
  }

  function confirmBulkPartDelete() {
    if (partsTableActions.bulkDeleteIds.length > 0) {
      deletePartItemsMutation.mutate([...partsTableActions.bulkDeleteIds], {
        onSuccess: () => {
          partsTableActions.resetBulkDelete();
          selectedPartIds = [];
        },
      });
    }
  }

  function confirmBulkPartStatusChange(status: PartStatus) {
    const ids = [...selectedPartIds];
    if (ids.length === 0) return;
    updatePartItemsStatusMutation.mutate(
      { ids, status },
      {
        onSuccess: () => {
          selectedPartIds = [];
        },
      },
    );
  }

  function openLaborBulkDiscount() {
    const selected = (workOrderDetails?.laborItems ?? []).filter((item) =>
      selectedLaborIds.includes(item.id),
    );
    laborBulkRowsById = new Map(selected.map((row) => [row.id, row]));
    laborBulkTargets = selected.map((row) => ({ id: row.id }));
    isLaborBulkDiscountOpen = true;
  }

  function openPartsBulkDiscount() {
    const selected = (workOrderDetails?.partItems ?? []).filter((item) =>
      selectedPartIds.includes(item.id),
    );
    partsBulkRowsById = new Map(selected.map((row) => [row.id, row]));
    partsBulkTargets = selected.map((row) => ({ id: row.id }));
    isPartsBulkDiscountOpen = true;
  }

  function openPartsBulkSupplier() {
    partsBulkSupplierTargets = [...selectedPartRows];
    isPartsBulkSupplierOpen = true;
  }

  async function applyPartsBulkSupplier(
    supplier: string | null,
    supplierOrderId: string | null,
  ) {
    const ids = partsBulkSupplierTargets.map((item) => item.id);
    if (ids.length === 0) return;
    await updatePartItemsSupplierMutation.mutateAsync({
      ids,
      supplier,
      supplierOrderId,
    });
    selectedPartIds = [];
    isPartsBulkSupplierOpen = false;
  }

  async function previewLaborBulkDiscount(
    id: number,
    discountType: DiscountType,
    discountValue: string,
  ) {
    const row = laborBulkRowsById.get(id);
    if (!row) return null;
    try {
      const quote = await unwrap(
        commands.previewLaborItem({
          workOrderId: row.workOrderId,
          priceType: row.priceType,
          fixedPrice: row.priceType === "Fixed" ? row.fixedPrice : "0",
          laborHours: row.priceType === "Hourly" ? row.laborHours : "0",
          discountType,
          discountValue,
          isFree: row.isFree,
        }),
      );
      return { belowCost: quote.belowCost, belowCostNow: row.totals.belowCost };
    } catch {
      return null;
    }
  }

  async function previewPartsBulkDiscount(
    id: number,
    discountType: DiscountType,
    discountValue: string,
  ) {
    const row = partsBulkRowsById.get(id);
    if (!row) return null;
    try {
      const quote = await unwrap(
        commands.previewPartItem({
          workOrderId: row.workOrderId,
          baseCost: row.baseCost,
          markup: row.markup,
          discountType,
          discountValue,
          isFree: row.isFree,
        }),
      );
      return { belowCost: quote.belowCost, belowCostNow: row.totals.belowCost };
    } catch {
      return null;
    }
  }

  async function applyBulkLaborDiscount(
    ids: number[],
    discountType: DiscountType,
    discountValue: string,
  ) {
    try {
      await updateLaborItemsDiscountMutation.mutateAsync({
        ids,
        discountType,
        discountValue,
      });
      selectedLaborIds = [];
    } catch {
      // Error toast is shown by the mutation layer.
    }
  }

  async function applyBulkPartsDiscount(
    ids: number[],
    discountType: DiscountType,
    discountValue: string,
  ) {
    try {
      await updatePartItemsDiscountMutation.mutateAsync({
        ids,
        discountType,
        discountValue,
      });
      selectedPartIds = [];
    } catch {
      // Error toast is shown by the mutation layer.
    }
  }
</script>

<PageLayout contentWidthClass="max-w-fit">
  {#if orderIsError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-10"
    />
  {:else if orderIsPending}
    <WorkOrderHeader isLoading onGenerate={openGenerate} />

    <div class="mt-4 border border-border rounded bg-card overflow-hidden">
      <!-- Services (labor) section -->
      <div class="px-4 pb-2">
        <div
          class="flex min-h-12 flex-wrap items-center justify-between gap-3 mb-0.5"
        >
          <div class="flex flex-row items-center gap-2">
            <Skeleton class="h-7 w-20" />
            <!-- Title -->
            <Skeleton class="h-6 w-5" />
            <!-- Counter -->
          </div>
          <div class="flex gap-4">
            <Skeleton class="h-5 w-24" />
            <!-- Create link -->
            <Skeleton class="h-5 w-32" />
            <!-- Add from DB link -->
          </div>
        </div>
        <!-- Simulated DataTable -->
        <div class="-mx-4 -mb-2 w-[calc(100%+2rem)] max-w-none">
          <div class="border-t border-border">
            <div
              class="h-10 border-b border-border flex items-center px-4 gap-4"
            >
              <Skeleton class="h-4 w-1/2" />
              <Skeleton class="h-4 w-16" />
              <Skeleton class="h-4 w-16" />
            </div>
            {#each { length: 3 } as _, i (i)}
              <div
                class="h-12 border-b border-border last:border-0 flex items-center px-4 gap-4"
              >
                <Skeleton class="h-4 w-2/3" />
                <Skeleton class="h-4 w-16" />
                <Skeleton class="h-4 w-16" />
                <Skeleton class="size-8 rounded-md ml-auto" />
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>

    <div class="mt-4 border border-border rounded bg-card overflow-hidden">
      <!-- Parts section -->
      <div class="px-4 pb-2">
        <div
          class="flex min-h-12 flex-wrap items-center justify-between gap-3 mb-0.5"
        >
          <div class="flex flex-row items-center gap-2">
            <Skeleton class="h-7 w-28" />
            <!-- Title -->
            <Skeleton class="h-6 w-5" />
            <!-- Counter -->
          </div>
          <Skeleton class="h-5 w-20" />
          <!-- Add link -->
        </div>
        <!-- Simulated DataTable -->
        <div class="-mx-4 -mb-2 w-[calc(100%+2rem)] max-w-none">
          <div class="border-t border-border">
            <div
              class="h-10 border-b border-border flex items-center px-4 gap-4"
            >
              <Skeleton class="h-4 w-1/2" />
              <Skeleton class="h-4 w-20" />
              <Skeleton class="h-4 w-20" />
            </div>
            {#each { length: 2 } as _, i (i)}
              <div
                class="h-12 border-b border-border last:border-0 flex items-center px-4 gap-4"
              >
                <Skeleton class="h-4 w-2/3" />
                <Skeleton class="h-4 w-20" />
                <Skeleton class="h-4 w-20" />
                <Skeleton class="size-8 rounded-md ml-auto" />
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {:else if !workOrderDetails?.workOrder}
    <NotFound
      message={`Замовлення не знайдено (ID: ${params.workorderId})`}
      class="mt-10"
    />
  {:else}
    <WorkOrderHeader
      bind:isLocked
      {isArchived}
      workOrderDetails={combinedQueries.workOrderDetails.data}
      clients={clients ?? []}
      hideNotes={privacyState.hideNotes}
      onGenerate={openGenerate}
    />

    <VatSection
      workOrderId={params.workorderId}
      entries={vatEntries}
      isSupplied={statusIsSupplied}
      expectedVat={workOrderDetails?.totals?.vatAmount ?? "0"}
      canManage={businessIsVatPayer}
      onVoided={() => (isLocked = false)}
      onRecorded={() => (isLocked = true)}
    />

    <div bind:this={tablesContainer} class="min-w-0">
      {#snippet laborActions()}
        {#if !isReadOnly}
          <Button
            variant="link"
            onclick={laborTableActions.openCreate}
            class="h-7 px-0 text-sm">Створити +</Button
          >
          <Popover.Root bind:open={isPopoverOpen}>
            <Popover.Trigger>
              {#snippet child({ props })}
                <Button variant="link" {...props} class="h-7 pr-0 text-sm">
                  Додати з бази
                  <ChevronDownIcon class="size-4" />
                </Button>
              {/snippet}
            </Popover.Trigger>
            <Popover.Content
              class="w-[400px] overflow-hidden rounded-[12px] border border-border bg-popover p-0 shadow-xl ring-0"
            >
              <ServicePicker
                services={services ?? []}
                onClose={() => (isPopoverOpen = false)}
                onSelect={createCreateLaborItemFromService}
              />
            </Popover.Content>
          </Popover.Root>
        {/if}
      {/snippet}

      {#snippet laborBulkItems()}
        <BulkDiscountMenuItem onclick={openLaborBulkDiscount} />
      {/snippet}

      {#snippet laborSelectionActions()}
        <DataTableBulkActions
          onDelete={() => laborTableActions.openBulkDelete(selectedLaborIds)}
          menuItems={laborBulkItems}
        />
      {/snippet}

      {#snippet partsActions()}
        {#if !isReadOnly}
          <Button
            variant="link"
            onclick={partsTableActions.openCreate}
            class="h-7 px-0 text-sm">Додати +</Button
          >
        {/if}
      {/snippet}

      {#snippet partsBulkItems()}
        <PartBulkStatusSubmenu onStatusChange={confirmBulkPartStatusChange} />
        <BulkDiscountMenuItem onclick={openPartsBulkDiscount} />
        {#if canBulkChangeSupplier}
          <PartBulkSupplierMenuItem onclick={openPartsBulkSupplier} />
        {/if}
      {/snippet}

      {#snippet partsSelectionActions()}
        <DataTableBulkActions
          onDelete={() => partsTableActions.openBulkDelete(selectedPartIds)}
          menuItems={partsBulkItems}
        />
      {/snippet}

      <LaborItemsTable
        items={workOrderDetails?.laborItems ?? []}
        total={workOrderDetails?.totals?.laborTotal ?? "0"}
        readOnly={isReadOnly}
        bind:selectedIds={selectedLaborIds}
        rowReorder={laborRowReorder}
        onEdit={(id) => laborTableActions.openEdit(id)}
        onDelete={(id) => laborTableActions.openDelete(id)}
        onMoveUp={(id) => moveLaborItem(id, -1)}
        onMoveDown={(id) => moveLaborItem(id, 1)}
        actions={laborActions}
        selectionActions={laborSelectionActions}
      />

      <PartItemsTable
        items={workOrderDetails?.partItems ?? []}
        total={workOrderDetails?.totals?.partsTotal ?? "0"}
        pendingTotal={workOrderDetails?.totals?.partsPendingTotal ?? "0"}
        readOnly={isReadOnly}
        {showPurchase}
        purchaseChecked={purchasePreferred}
        purchaseDisabled={privacyState.hidePricing}
        onPurchaseChange={setPurchaseColumn}
        bind:selectedIds={selectedPartIds}
        rowReorder={partRowReorder}
        onEdit={openPartEdit}
        onDelete={(id) => partsTableActions.openDelete(id)}
        onMoveUp={(id) => movePartItem(id, -1)}
        onMoveDown={(id) => movePartItem(id, 1)}
        actions={partsActions}
        selectionActions={partsSelectionActions}
      />
    </div>

    <WorkOrderLaborItemMutationSheet
      bind:open={laborTableActions.isMutationOpen}
      laborItemId={laborTableActions.editId}
      workOrderId={workOrderDetails?.workOrder?.id}
      workOrderType={workOrderDetails?.workOrder?.type}
    />
    <DeleteDialog
      bind:open={laborTableActions.isDeleteOpen}
      onDelete={confirmLaborDelete}
      description="Цю дію неможливо відмінити."
    />
    <DeleteDialog
      bind:open={laborTableActions.isBulkDeleteOpen}
      onDelete={confirmBulkLaborDelete}
      description={`Видалити ${laborTableActions.bulkDeleteIds.length} ${pluralizeUk(laborTableActions.bulkDeleteIds.length, ["сервіс", "сервіси", "сервісів"])}? Цю дію неможливо відмінити.`}
    />
    <WorkOrderPartsItemMutationSheet
      bind:open={partsTableActions.isMutationOpen}
      partItemId={partsTableActions.editId}
      workOrderId={workOrderDetails?.workOrder?.id}
      workOrderType={workOrderDetails?.workOrder?.type}
    />
    <DeleteDialog
      bind:open={partsTableActions.isDeleteOpen}
      onDelete={confirmPartDelete}
      description="Цю дію неможливо відмінити."
    />
    <DeleteDialog
      bind:open={partsTableActions.isBulkDeleteOpen}
      onDelete={confirmBulkPartDelete}
      description={`Видалити ${partsTableActions.bulkDeleteIds.length} ${pluralizeUk(partsTableActions.bulkDeleteIds.length, ["запчастину", "запчастини", "запчастин"])}? Цю дію неможливо відмінити.`}
    />
    <BulkDiscountDialog
      bind:open={isLaborBulkDiscountOpen}
      description={`Знижка застосується до ${laborBulkTargets.length} ${pluralizeUk(laborBulkTargets.length, ["сервісу", "сервісів", "сервісів"])}.`}
      rows={laborBulkTargets}
      preview={previewLaborBulkDiscount}
      applyDiscount={applyBulkLaborDiscount}
    />
    <BulkDiscountDialog
      bind:open={isPartsBulkDiscountOpen}
      description={`Знижка застосується до ${partsBulkTargets.length} ${pluralizeUk(partsBulkTargets.length, ["запчастини", "запчастин", "запчастин"])}.`}
      rows={partsBulkTargets}
      preview={previewPartsBulkDiscount}
      applyDiscount={applyBulkPartsDiscount}
    />
    <PartBulkSupplierDialog
      bind:open={isPartsBulkSupplierOpen}
      rows={partsBulkSupplierTargets}
      onApply={applyPartsBulkSupplier}
    />

    <DocumentsSection
      workOrderId={params.workorderId}
      canGenerate={!isArchived}
      onGenerate={openGenerate}
    />

    {#if generateRequest}
      <DocumentGenerateDialog
        workOrderId={params.workorderId}
        {workOrderDetails}
        kind={generateRequest.kind}
        seed={generateRequest.seed}
        bind:open={isGenerateDialogOpen}
      />
    {/if}
  {/if}
</PageLayout>
