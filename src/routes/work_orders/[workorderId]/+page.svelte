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
    WorkOrderStatus,
  } from "$lib/bindings";
  import { commands } from "$lib/bindings";
  import { privacyState } from "$lib/states/privacy.svelte";
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
    createUpdateLaborItemsDiscountMutation,
    createUpdatePartItemsDiscountMutation,
    createUpdatePartItemsStatusMutation,
    workOrderDetailsQueryOptions,
  } from "../queries";
  import BulkDiscountDialog from "./bulk-discount-dialog.svelte";
  import BulkDiscountMenuItem from "./bulk-discount-menu-item.svelte";
  import DocumentGenerateDialog from "./document-generate-dialog.svelte";
  import DocumentsSection from "./documents-section.svelte";
  import {
    columns as laborColumns,
    type LaborTableMeta,
  } from "./labor-columns";
  import WorkOrderLaborItemMutationSheet from "./labor-item-mutation-sheet.svelte";
  import PartBulkStatusSubmenu from "./part-bulk-status-submenu.svelte";
  import { columns as partsColumns, type PartsTableMeta } from "./part-columns";
  import WorkOrderPartsItemMutationSheet from "./part-item-mutation-sheet.svelte";
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

  const combinedQueries = createQueries(() => ({
    queries: [
      workOrderDetailsQueryOptions(params.workorderId),
      servicesQueryOptions(),
      clientsQueryOptions(),
    ],
    combine: (results) => {
      return {
        workOrderDetails: results[0],
        services: results[1],
        clients: results[2],

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

  const orderIsPending = $derived(
    combinedQueries.workOrderDetails.isPending ||
      combinedQueries.services.isPending,
  );
  const orderIsError = $derived(
    combinedQueries.workOrderDetails.isError ||
      combinedQueries.services.isError,
  );

  const isArchived = $derived(workOrderDetails?.workOrder?.archivedAt != null);

  let isLocked = $state(true);
  let previousStatus: WorkOrderStatus | undefined;

  // Archived orders are read-only regardless of the paid-lock state.
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

    const currentTerminal =
      currentStatus === "Paid" || currentStatus === "Completed";

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
        ? partsColumns.filter((c) => c.id !== "actions" && c.id !== "select")
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
        ? laborColumns.filter((c) => c.id !== "actions" && c.id !== "select")
        : laborColumns;
    },
    get meta() {
      return {
        onEdit: laborTableActions.openEdit,
        onDelete: laborTableActions.openDelete,
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
          headerClasses={{ name: "w-full", select: "w-10 p-0" }}
          cellClasses={{
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
          headerClasses={{ name: "w-full", select: "w-10 p-0" }}
          cellClasses={{
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
