<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";

  import CarFrontIcon from "@lucide/svelte/icons/car-front";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import QuoteIcon from "@lucide/svelte/icons/message-square-quote";
  import StickyNoteIcon from "@lucide/svelte/icons/sticky-note";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTableBulkActions from "$lib/components/data-table/data-table-bulk-actions.svelte";
  import type { RowReorderOptions } from "$lib/components/data-table/data-table-reorder";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import type {
    DiscountType,
    DocumentFormat,
    DocumentKind,
    LaborItemDto,
    PartItemDto,
    PartStatus,
    Service,
    WorkOrderDetailsDto,
    WorkOrderStatus,
  } from "$lib/bindings";
  import { commands } from "$lib/bindings";
  import { keys } from "$lib/query-keys";
  import { privacyState } from "$lib/states/privacy.svelte";
  import { settingsState } from "$lib/states/settings.svelte";
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
    liveVatEntry,
    workOrderDetailsQueryOptions,
    workOrderVatQueryOptions,
  } from "../queries";
  import BulkDiscountDialog from "./bulk-discount-dialog.svelte";
  import BulkDiscountMenuItem from "./bulk-discount-menu-item.svelte";
  import DocumentGenerateDialog from "./document-generate-dialog.svelte";
  import DocumentsSection from "./documents-section.svelte";
  import { applyItemOrder, isSameOrder, moveItemById } from "./item-order";
  import {
    columns as laborColumns,
    type LaborTableMeta,
  } from "./labor-columns";
  import WorkOrderLaborItemMutationSheet from "./labor-item-mutation-sheet.svelte";
  import PartBulkStatusSubmenu from "./part-bulk-status-submenu.svelte";
  import { columns as partsColumns, type PartsTableMeta } from "./part-columns";
  import WorkOrderPartsItemMutationSheet from "./part-item-mutation-sheet.svelte";
  import VatSection from "./vat-section.svelte";
  import WorkOrderHeader from "./work-order-header.svelte";

  let { params } = $props();

  let isPopoverOpen: boolean = $state(false);

  let generateRequest = $state<{
    kind: DocumentKind;
    format: DocumentFormat;
  } | null>(null);
  let isGenerateDialogOpen = $state(false);

  function openGenerate(kind: DocumentKind, format: DocumentFormat) {
    generateRequest = { kind, format };
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
    ["ReadyForPickup", "Completed", "Paid"].includes(
      workOrderDetails?.workOrder?.status ?? "",
    ),
  );

  let isLocked = $state(true);
  let previousStatus: WorkOrderStatus | undefined;

  // Archived orders are read-only regardless of the paid-lock state.
  const businessIsVatPayer = $derived(settingsState.isVatPayer);
  const isReadOnly = $derived(isLocked || isArchived);

  let hasPendingParts = $derived(
    Big(workOrderDetails?.totals?.partsPendingTotal ?? "0").gt(0),
  );

  const hasNotes = $derived(
    Boolean(
      workOrderDetails?.workOrder?.description ||
      workOrderDetails?.workOrder?.conditionReport ||
      (!privacyState.hideNotes && workOrderDetails?.workOrder?.notes),
    ),
  );

  $effect(() => {
    const currentStatus = workOrderDetails?.workOrder?.status;
    if (currentStatus === undefined) return;

    // A recorded VAT entry freezes the figures just as a terminal status does,
    // even when the order has not been paid yet: the tax point already happened.
    const currentTerminal =
      currentStatus === "Paid" || currentStatus === "Completed" || hasLiveVat;

    if (previousStatus === undefined) {
      isLocked = currentTerminal;
    } else if (currentTerminal && !isTerminal(previousStatus)) {
      isLocked = true;
    }

    previousStatus = currentStatus;
  });

  function isTerminal(status: WorkOrderStatus): boolean {
    return status === "Paid" || status === "Completed";
  }

  let partsTableActions = new TableActionState<number>();
  let laborTableActions = new TableActionState<number>();

  let isLaborBulkDiscountOpen = $state(false);
  let isPartsBulkDiscountOpen = $state(false);
  // Row snapshots taken when the bulk discount dialog is opened, so the
  // preflight quotes and the mutation always target the same rows even if
  // the selection changes while the dialog is on screen.
  let laborBulkTargets = $state<{ id: number }[]>([]);
  let partsBulkTargets = $state<{ id: number }[]>([]);
  let laborBulkRowsById = new Map<number, LaborItemDto>();
  let partsBulkRowsById = new Map<number, PartItemDto>();

  let partsTable = buildTable({
    get data() {
      return combinedQueries.workOrderDetails.data?.partItems ?? [];
    },
    get columns() {
      const columns = isReadOnly
        ? partsColumns.filter(
            (c) => c.id !== "actions" && c.id !== "select" && c.id !== "drag",
          )
        : partsColumns;
      if (privacyState.hidePricing) {
        return columns.filter((c) => {
          const id = "accessorKey" in c ? (c.accessorKey ?? c.id) : c.id;
          return id !== "baseCost" && id !== "markup";
        });
      }
      return columns;
    },
    get meta() {
      return {
        onEdit: partsTableActions.openEdit,
        onDelete: partsTableActions.openDelete,
        onMoveUp: (id: number) => movePartItem(id, -1),
        onMoveDown: (id: number) => movePartItem(id, 1),
      } satisfies PartsTableMeta;
    },
    getRowId: (row) => String(row.id),
    disableSorting: true,
    disableFiltering: true,
    disablePagination: true,
  });

  let laborTable = buildTable({
    get data() {
      return combinedQueries.workOrderDetails.data?.laborItems ?? [];
    },
    get columns() {
      return isReadOnly
        ? laborColumns.filter(
            (c) => c.id !== "actions" && c.id !== "select" && c.id !== "drag",
          )
        : laborColumns;
    },
    get meta() {
      return {
        onEdit: laborTableActions.openEdit,
        onDelete: laborTableActions.openDelete,
        onMoveUp: (id: number) => moveLaborItem(id, -1),
        onMoveDown: (id: number) => moveLaborItem(id, 1),
      } satisfies LaborTableMeta;
    },
    getRowId: (row) => String(row.id),
    disableSorting: true,
    disableFiltering: true,
    disablePagination: true,
  });

  const selectedLaborIds = $derived(
    laborTable.getFilteredSelectedRowModel().rows.map((row) => row.original.id),
  );

  const selectedPartIds = $derived(
    partsTable.getFilteredSelectedRowModel().rows.map((row) => row.original.id),
  );

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

  async function createCreateLaborItemFromService(service: Service) {
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
          isPopoverOpen = false;
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
          laborTable.resetRowSelection();
        },
      });
    }
  }

  function confirmBulkPartDelete() {
    if (partsTableActions.bulkDeleteIds.length > 0) {
      deletePartItemsMutation.mutate([...partsTableActions.bulkDeleteIds], {
        onSuccess: () => {
          partsTableActions.resetBulkDelete();
          partsTable.resetRowSelection();
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
          partsTable.resetRowSelection();
        },
      },
    );
  }

  function openLaborBulkDiscount() {
    const selected = laborTable
      .getFilteredSelectedRowModel()
      .rows.map((row) => row.original);
    laborBulkRowsById = new Map(selected.map((row) => [row.id, row]));
    laborBulkTargets = selected.map((row) => ({ id: row.id }));
    isLaborBulkDiscountOpen = true;
  }

  function openPartsBulkDiscount() {
    const selected = partsTable
      .getFilteredSelectedRowModel()
      .rows.map((row) => row.original);
    partsBulkRowsById = new Map(selected.map((row) => [row.id, row]));
    partsBulkTargets = selected.map((row) => ({ id: row.id }));
    isPartsBulkDiscountOpen = true;
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
      laborTable.resetRowSelection();
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
      partsTable.resetRowSelection();
    } catch {
      // Error toast is shown by the mutation layer.
    }
  }
</script>

{#snippet noteCard(label: string, Icon: typeof QuoteIcon, value: string)}
  <div
    class="flex min-h-15 items-start gap-3 rounded border border-border bg-card px-3 py-2.5"
  >
    <span
      class="flex size-9 shrink-0 items-center justify-center rounded border border-border bg-muted/40 text-muted-foreground"
    >
      <Icon class="size-4.5" />
    </span>
    <div class="flex min-w-0 flex-col gap-1">
      <span class="text-xs text-muted-foreground">{label}</span>
      <p
        class="text-sm wrap-break-word italic text-foreground/90 whitespace-pre-wrap"
      >
        {value}
      </p>
    </div>
  </div>
{/snippet}

<PageLayout contentWidthClass="max-w-fit">
  {#if orderIsError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-10"
    />
  {:else if orderIsPending}
    <WorkOrderHeader isLoading onGenerate={openGenerate} />

    <div class="mt-4 border border-border rounded bg-card">
      <!-- Services (labor) section -->
      <div class="p-4">
        <div class="flex flex-row items-center justify-between">
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
        <div class="mt-4 border border-border rounded-md">
          <div class="h-10 border-b border-border flex items-center px-4 gap-4">
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

      <!-- Parts section -->
      <div class="p-4 pt-0">
        <div class="flex flex-row items-center justify-between">
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
        <div class="mt-4 border border-border rounded-md">
          <div class="h-10 border-b border-border flex items-center px-4 gap-4">
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

    {#if hasNotes}
      <div
        class="mt-4 grid grid-cols-[repeat(auto-fit,minmax(13rem,1fr))] gap-4"
      >
        {#if workOrderDetails?.workOrder?.description}
          {@render noteCard(
            "Опис",
            QuoteIcon,
            workOrderDetails.workOrder.description,
          )}
        {/if}
        {#if workOrderDetails?.workOrder?.conditionReport}
          {@render noteCard(
            "Огляд при прийнятті",
            CarFrontIcon,
            workOrderDetails.workOrder.conditionReport,
          )}
        {/if}
        {#if !privacyState.hideNotes && workOrderDetails?.workOrder?.notes}
          {@render noteCard(
            "Нотатки",
            StickyNoteIcon,
            workOrderDetails.workOrder.notes,
          )}
        {/if}
      </div>
    {/if}

    <div class="mt-4 border border-border rounded bg-card">
      <div class="p-4">
        <div class="flex flex-row items-center justify-between">
          <div class="flex flex-row items-baseline gap-2">
            <p class="text-xl font-medium">Робота</p>
            <CurrencyDisplay
              amount={workOrderDetails?.totals?.laborTotal ?? "0"}
              suffix="грн."
              class="items-baseline text-base font-normal text-muted-foreground"
            />
          </div>
          <div>
            {#if !isReadOnly}
              <Button
                variant="link"
                onclick={laborTableActions.openCreate}
                class="text-base">Створити +</Button
              >
              <Popover.Root bind:open={isPopoverOpen}>
                <Popover.Trigger>
                  {#snippet child({ props })}
                    <Button variant="link" {...props} class="text-base">
                      Додати з бази
                      <ChevronDownIcon class="size-4" />
                    </Button>
                  {/snippet}
                </Popover.Trigger>
                <Popover.Content class="w-80 p-0">
                  <Command.Root>
                    <Command.Input placeholder="Пошук сервісів..." />
                    <Command.List>
                      <Command.Empty>Сервісів не знайдено...</Command.Empty>
                      <Command.Group value="services">
                        {#each services ?? [] as service (service.id)}
                          <Command.Item
                            value={service.name}
                            class="hover:cursor-pointer"
                            onSelect={async () =>
                              await createCreateLaborItemFromService(service)}
                          >
                            {service.name}
                          </Command.Item>
                        {/each}
                      </Command.Group>
                    </Command.List>
                  </Command.Root>
                </Popover.Content>
              </Popover.Root>
            {/if}
          </div>
        </div>
        {#snippet laborBulkMenuItems()}
          <BulkDiscountMenuItem onclick={openLaborBulkDiscount} />
        {/snippet}
        <DataTable
          table={laborTable}
          emptyMessage="Сервісів не знайдено..."
          class="mt-4"
          rowReorder={laborRowReorder}
          headerClasses={{
            name: "w-full",
            drag: "w-8 p-0",
            select: "w-10 p-0",
          }}
          cellClasses={{
            drag: "p-0",
            actions: "p-0",
            discountedPrice: "p-0",
            sum: "p-0",
            select: "p-0",
          }}
        >
          {#snippet selectionActions()}
            <DataTableBulkActions
              onDelete={() =>
                laborTableActions.openBulkDelete(selectedLaborIds)}
              menuItems={laborBulkMenuItems}
            />
          {/snippet}
        </DataTable>
      </div>

      <div class="p-4 pt-0">
        <div class="flex flex-row items-center justify-between">
          <div class="flex flex-col gap-0.5">
            <div class="flex flex-row items-baseline gap-2">
              <p class="text-xl font-medium">Запчастини</p>
              <CurrencyDisplay
                amount={workOrderDetails?.totals?.partsTotal ?? "0"}
                suffix="грн."
                class="items-baseline text-base font-normal text-muted-foreground"
              />
            </div>
            {#if hasPendingParts}
              <p
                class="flex items-baseline gap-1 text-sm text-muted-foreground/70"
              >
                <span>+</span>
                <CurrencyDisplay
                  amount={workOrderDetails?.totals?.partsPendingTotal ?? "0"}
                  suffix="грн."
                  class="items-baseline"
                />
                <span>очікують встановлення</span>
              </p>
            {/if}
          </div>
          {#if !isReadOnly}
            <Button
              variant="link"
              onclick={partsTableActions.openCreate}
              class="text-base">Додати +</Button
            >
          {/if}
        </div>
        {#snippet partsBulkMenuItems()}
          <PartBulkStatusSubmenu onStatusChange={confirmBulkPartStatusChange} />
          <BulkDiscountMenuItem onclick={openPartsBulkDiscount} />
        {/snippet}
        <DataTable
          table={partsTable}
          emptyMessage="Запчастин не знайдено..."
          class="mt-4"
          rowReorder={partRowReorder}
          headerClasses={{
            name: "w-full",
            drag: "w-8 p-0",
            select: "w-10 p-0",
          }}
          cellClasses={{
            drag: "p-0",
            status: "p-0",
            actions: "p-0",
            discountedPrice: "p-0",
            sum: "p-0",
            select: "p-0",
          }}
        >
          {#snippet selectionActions()}
            <DataTableBulkActions
              onDelete={() => partsTableActions.openBulkDelete(selectedPartIds)}
              menuItems={partsBulkMenuItems}
            />
          {/snippet}
        </DataTable>
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
    </div>

    <DocumentsSection workOrderId={params.workorderId} />

    {#if generateRequest}
      <DocumentGenerateDialog
        workOrderId={params.workorderId}
        {workOrderDetails}
        kind={generateRequest.kind}
        format={generateRequest.format}
        bind:open={isGenerateDialogOpen}
      />
    {/if}
  {/if}
</PageLayout>
