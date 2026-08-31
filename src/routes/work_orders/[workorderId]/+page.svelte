<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Info from "@lucide/svelte/icons/info";
  import LockIcon from "@lucide/svelte/icons/lock";
  import LockOpenIcon from "@lucide/svelte/icons/lock-open";
  import QuoteIcon from "@lucide/svelte/icons/message-square-quote";
  import StickyNoteIcon from "@lucide/svelte/icons/sticky-note";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTableBulkActions from "$lib/components/data-table/data-table-bulk-actions.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";
  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import type { PartStatus, Service, WorkOrderStatus } from "$lib/bindings";
  import { privacyState } from "$lib/states/privacy.svelte";
  import { TableActionState } from "$lib/states/table-actions.svelte";
  import { pluralizeUk, prettifyIsoDateString } from "$lib/tools";

  import ClientAvatar from "../../clients/client-avatar.svelte";
  import { servicesQueryOptions } from "../../services/queries";
  import {
    createCreateLaborItemMutation,
    createDeleteLaborItemMutation,
    createDeleteLaborItemsMutation,
    createDeletePartItemMutation,
    createDeletePartItemsMutation,
    createRestoreWorkOrderMutation,
    createUpdatePartItemsStatusMutation,
    workOrderDetailsQueryOptions,
  } from "../queries";
  import StatusSelector from "../work-order-status-selector.svelte";
  import WorkOrderTotalsPopover from "../work-order-totals-popover.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";
  import {
    columns as laborColumns,
    type LaborTableMeta,
  } from "./labor-columns";
  import WorkOrderLaborItemMutationSheet from "./labor-item-mutation-sheet.svelte";
  import PartBulkStatusSubmenu from "./part-bulk-status-submenu.svelte";
  import { columns as partsColumns, type PartsTableMeta } from "./part-columns";
  import WorkOrderPartsItemMutationSheet from "./part-item-mutation-sheet.svelte";

  let { params } = $props();

  let isPopoverOpen: boolean = $state(false);

  const queryClient = useQueryClient();

  const deletePartItemMutation = createDeletePartItemMutation(queryClient);
  const deletePartItemsMutation = createDeletePartItemsMutation(queryClient);
  const createLaborItemMutation = createCreateLaborItemMutation(queryClient);
  const deleteLaborItemMutation = createDeleteLaborItemMutation(queryClient);
  const deleteLaborItemsMutation = createDeleteLaborItemsMutation(queryClient);
  const updatePartItemsStatusMutation =
    createUpdatePartItemsStatusMutation(queryClient);
  const restoreWorkOrderMutation = createRestoreWorkOrderMutation(queryClient);

  const combinedQueries = createQueries(() => ({
    queries: [
      workOrderDetailsQueryOptions(params.workorderId),
      servicesQueryOptions(),
    ],
    combine: (results) => {
      return {
        workOrderDetails: results[0],
        services: results[1],

        isReady: results.every((r) => r.isSuccess),
        isLoading: results.some((r) => r.isPending),
        isError: results.some((r) => r.isError),
        firstError: results.find((r) => r.isError)?.error,

        refetchAll: () => {
          results.forEach((r) => r.refetch());
        },
      };
    },
  }));

  let services = $derived(combinedQueries.services.data);
  let workOrderDetails = $derived(combinedQueries.workOrderDetails.data);

  const isArchived = $derived(workOrderDetails?.workOrder?.archivedAt != null);
  const isPaid = $derived(workOrderDetails?.workOrder?.status === "Paid");

  // Locks the whole order (status + details) once it's Paid.
  let isLocked = $state(true);
  let previousStatus: WorkOrderStatus | undefined;

  // Archived orders are read-only regardless of the paid-lock state.
  const isReadOnly = $derived(isLocked || isArchived);

  $effect(() => {
    const currentStatus = workOrderDetails?.workOrder?.status;
    if (currentStatus === undefined) return;

    if (previousStatus === undefined) {
      isLocked = currentStatus === "Paid";
    } else if (currentStatus === "Paid" && previousStatus !== "Paid") {
      isLocked = true;
    }

    previousStatus = currentStatus;
  });

  let [datetime_label, datetime_iso_string] = $derived.by(() => {
    if (!combinedQueries.isReady || !workOrderDetails) return ["", ""];

    if (workOrderDetails.workOrder.paidAt) {
      return ["Сплачено", workOrderDetails.workOrder.paidAt];
    } else if (workOrderDetails.workOrder.completedAt) {
      return ["Завершено", workOrderDetails.workOrder.completedAt];
    } else if (workOrderDetails.workOrder.startedAt) {
      return ["Почато", workOrderDetails.workOrder.startedAt];
    } else {
      return ["Створено", workOrderDetails.workOrder.createdAt];
    }
  });

  let partsTableActions = new TableActionState<number>();
  let laborTableActions = new TableActionState<number>();

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

  function handleRestore() {
    restoreWorkOrderMutation.mutate(params.workorderId);
  }

  function carSecondaryLine(wod: typeof workOrderDetails) {
    return [wod?.carDto?.car?.licensePlate, wod?.carDto?.car?.year]
      .filter(Boolean)
      .join(" · ");
  }
</script>

<PageLayout contentWidthClass="max-w-fit">
  {#if combinedQueries.isError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-10"
    />
  {:else if combinedQueries.isLoading}
    <div class="mt-4 border border-border rounded bg-card">
      <!-- Info bar: id / date / odometer / rate -->
      <div class="h-15 flex flex-row items-center border-b border-border">
        <!-- Order ID -->
        <div
          class="p-4 flex flex-col justify-start items-start gap-1.5 min-w-30"
        >
          <Skeleton class="h-3.5 w-24" />
          <Skeleton class="h-5 w-16" />
        </div>

        <div class="block h-8/12 w-px bg-border"></div>

        <!-- Date -->
        <div
          class="p-4 flex flex-col justify-start items-start gap-1.5 min-w-40"
        >
          <div class="flex items-center gap-1">
            <Skeleton class="h-3.5 w-16" />
            <Skeleton class="size-3.5 rounded-full" />
          </div>
          <Skeleton class="h-5 w-32" />
        </div>

        <div class="block h-8/12 w-px bg-border"></div>

        <!-- Odometer -->
        <div
          class="p-4 flex flex-col justify-start items-start gap-1.5 min-w-30"
        >
          <Skeleton class="h-3.5 w-12" />
          <Skeleton class="h-5 w-24" />
        </div>

        <div class="block h-8/12 w-px bg-border"></div>

        <!-- Rate -->
        <div
          class="p-4 flex flex-col justify-start items-start gap-1.5 min-w-30"
        >
          <Skeleton class="h-3.5 w-14" />
          <Skeleton class="h-5 w-20" />
        </div>

        <!-- Actions -->
        <div class="flex items-center gap-2 ml-auto mr-2">
          <Skeleton class="h-9 w-24 rounded-md" />
          <!-- Totals popover -->
          <Skeleton class="h-9 w-32 rounded-md" />
          <!-- Status selector -->
          <Skeleton class="h-9 w-9 rounded-md shrink-0" />
          <!-- Dropdown menu -->
        </div>
      </div>

      <!-- Car / client navigation cards -->
      <div class="flex flex-row divide-x divide-border border-b border-border">
        <div class="flex-1 flex items-center gap-2 p-2 pl-4 h-11">
          <Skeleton class="size-6 rounded-full shrink-0" />
          <Skeleton class="h-5 w-48" />
          <Skeleton class="h-4 w-24 ml-auto" />
          <Skeleton class="size-4 shrink-0 rounded-sm" />
        </div>
        <div class="flex-1 flex items-center gap-2 p-2 h-11">
          <Skeleton class="size-6 rounded-full shrink-0" />
          <Skeleton class="h-5 w-40" />
          <Skeleton class="h-4 w-32 ml-auto" />
          <Skeleton class="size-4 shrink-0 rounded-sm" />
        </div>
      </div>

      <!-- Description / Notes (stacked, matching the real conditional layout) -->
      <div
        class="flex flex-col divide-y divide-border rounded-b overflow-hidden"
      >
        <div class="flex flex-col items-start gap-1.5 px-4 py-2 bg-muted/40">
          <div class="flex items-center gap-2">
            <Skeleton class="size-4 rounded-sm" />
            <Skeleton class="h-4 w-12" />
          </div>
          <Skeleton class="h-4 w-full max-w-md" />
        </div>
        <div class="flex flex-col items-start gap-1.5 px-4 py-2 bg-muted/40">
          <div class="flex items-center gap-2">
            <Skeleton class="size-4 rounded-sm" />
            <Skeleton class="h-4 w-16" />
          </div>
          <Skeleton class="h-4 w-full max-w-md" />
        </div>
      </div>
    </div>

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
    <div class="mt-4 border border-border rounded bg-card">
      {#if isArchived}
        <div
          class="flex items-center justify-between gap-2 rounded-t px-4 py-2 border-b border-border bg-muted/40 text-sm"
        >
          <span class="flex items-center gap-2 text-muted-foreground">
            <ArchiveIcon class="size-4" />
            Замовлення архівоване
            {prettifyIsoDateString(
              workOrderDetails?.workOrder?.archivedAt ?? "",
              "datetime",
            )}
          </span>
          <Button
            variant="link"
            class="p-0"
            onclick={handleRestore}
            disabled={restoreWorkOrderMutation.isPending}
          >
            <ArchiveRestoreIcon />
            Відновити
          </Button>
        </div>
      {/if}
      {#if isPaid && !isArchived}
        <div
          class="flex items-center justify-between gap-2 rounded-t px-4 py-2 border-b border-border bg-muted/40 text-sm"
        >
          <span class="flex items-center gap-2 text-muted-foreground">
            {#if isLocked}
              <LockIcon class="size-4" />
              Замовлення оплачено і заблоковано від змін
            {:else}
              <LockOpenIcon class="size-4" />
              Замовлення тимчасово розблоковано для редагування
            {/if}
          </span>
          <Button
            variant="link"
            class="p-0"
            onclick={() => (isLocked = !isLocked)}
            aria-label={isLocked
              ? "Розблокувати замовлення"
              : "Заблокувати замовлення"}
            title={isLocked
              ? "Розблокувати замовлення"
              : "Заблокувати замовлення"}
          >
            {#if isLocked}
              Розблокувати
            {:else}
              Заблокувати
            {/if}
          </Button>
        </div>
      {/if}

      <div class="h-15 flex flex-row items-center border-b border-border">
        <div class="p-4 flex flex-col justify-start items-start">
          <span class="text-sm text-muted-foreground">Наряд-замовлення</span>
          <p class="text-base font-medium">
            {workOrderDetails.workOrder?.id}
          </p>
        </div>

        <div class="block h-8/12 w-px bg-border"></div>

        <div class="p-4 flex flex-col justify-start items-start">
          <div class="flex items-center gap-1 text-muted-foreground">
            <span class="text-sm">{datetime_label}</span>
            <Popover.Root>
              <Popover.Trigger class="hover:cursor-pointer">
                <Info class="size-3.5" />
              </Popover.Trigger>
              <Popover.Content class="p-4 w-fit">
                <div class="flex flex-col gap-2">
                  <div class="flex gap-2 justify-between">
                    <p>Створено:</p>
                    <p>
                      {workOrderDetails.workOrder.createdAt
                        ? prettifyIsoDateString(
                            workOrderDetails.workOrder.createdAt,
                            "datetime",
                          )
                        : "—"}
                    </p>
                  </div>
                  <div class="flex gap-2 justify-between">
                    <p>Почато:</p>
                    <p>
                      {workOrderDetails.workOrder.startedAt
                        ? prettifyIsoDateString(
                            workOrderDetails.workOrder.startedAt,
                            "datetime",
                          )
                        : "—"}
                    </p>
                  </div>
                  <div class="flex gap-2 justify-between">
                    <p>Завершено:</p>
                    <p>
                      {workOrderDetails.workOrder.completedAt
                        ? prettifyIsoDateString(
                            workOrderDetails.workOrder.completedAt,
                            "datetime",
                          )
                        : "—"}
                    </p>
                  </div>
                  <div class="flex gap-2 justify-between">
                    <p>Сплачено:</p>
                    <p>
                      {workOrderDetails.workOrder.paidAt
                        ? prettifyIsoDateString(
                            workOrderDetails.workOrder.paidAt,
                            "datetime",
                          )
                        : "—"}
                    </p>
                  </div>
                </div>
              </Popover.Content>
            </Popover.Root>
          </div>
          <p class="text-base">
            {prettifyIsoDateString(datetime_iso_string, "datetime")}
          </p>
        </div>

        <div class="block h-8/12 w-px bg-border"></div>

        <div class="p-4 flex flex-col justify-start items-start">
          <span class="text-sm text-muted-foreground">Пробіг</span>
          <div class="flex gap-2 text-base">
            <p>{workOrderDetails?.workOrder?.odometerIn}</p>
            <p class="text-muted-foreground">км.</p>
          </div>
        </div>

        <div class="block h-8/12 w-px bg-border"></div>

        <div class="p-4 flex flex-col justify-start items-start">
          <span class="text-sm text-muted-foreground">Тариф</span>
          <CurrencyDisplay
            amount={workOrderDetails?.workOrder.rate}
            suffix="грн./год."
            suffix_muted
          />
        </div>

        <div class="flex items-center ml-auto mr-2 text-base">
          <WorkOrderTotalsPopover {workOrderDetails} class="mr-2" />
          <StatusSelector
            workOrder={workOrderDetails?.workOrder}
            partItems={workOrderDetails?.partItems ?? []}
            isLocked={isReadOnly}
          />
          <DropdownMenu
            workOrderId={workOrderDetails.workOrder?.id ?? ""}
            {isLocked}
            {isArchived}
          />
        </div>
      </div>

      <div class="flex flex-row divide-x divide-border">
        <div class="flex-1">
          <Button
            variant="ghost"
            class="w-full h-auto gap-2 items-center rounded-none p-2 pl-4 text-base font-normal"
            onclick={async () => {
              await goto(`/cars/${workOrderDetails?.carDto?.car?.id}`);
            }}
          >
            <CarLogoLoader
              slug={workOrderDetails?.carDto?.brand?.iconSlug}
              class="size-6"
            />
            <p class="text-base font-medium">
              {workOrderDetails?.carDto?.brand?.name}
              {workOrderDetails?.carDto?.car?.model}
            </p>
            <p class="ml-auto text-muted-foreground">
              {carSecondaryLine(workOrderDetails)}
            </p>
            <ChevronRight class="size-4 text-muted-foreground" />
          </Button>
        </div>
        <div class="flex-1">
          <Button
            variant="ghost"
            class="w-full h-auto gap-2 items-center rounded-none p-2 text-base font-normal"
            onclick={async () => {
              await goto(`/clients/${workOrderDetails?.client?.id}`);
            }}
          >
            <ClientAvatar
              name={workOrderDetails?.client?.name}
              class="size-6"
            />
            <p class="text-base font-medium">
              {workOrderDetails?.client?.name}
            </p>
            <PhoneDisplay
              value={workOrderDetails?.client?.clientType !== "individual"
                ? (workOrderDetails?.clientDetails?.contactPhone ??
                  workOrderDetails?.client?.phone)
                : workOrderDetails?.client?.phone}
              class="ml-auto select-none! text-muted-foreground"
            />
            <ChevronRight class="size-4 text-muted-foreground" />
          </Button>
        </div>
      </div>

      {#if workOrderDetails?.workOrder?.description || (!privacyState.hideNotes && workOrderDetails?.workOrder?.notes)}
        <div
          class="flex flex-col divide-y divide-border rounded-b overflow-hidden border-t border-border"
        >
          {#if workOrderDetails?.workOrder?.description}
            <div
              class="flex-1 flex flex-col items-start gap-1 px-4 py-2 bg-muted/40"
            >
              <span
                class="flex items-center justify-center gap-2 text-base text-muted-foreground"
              >
                <QuoteIcon class="size-4" />
                <span>Опис</span>
              </span>
              <p
                class="text-base italic text-foreground/90 whitespace-pre-wrap"
              >
                {workOrderDetails.workOrder.description}
              </p>
            </div>
          {/if}
          {#if !privacyState.hideNotes && workOrderDetails?.workOrder?.notes}
            <div
              class="flex-1 flex flex-col items-start gap-1 px-4 py-2 bg-muted/40"
            >
              <span
                class="flex items-center justify-center gap-2 text-base text-muted-foreground"
              >
                <StickyNoteIcon class="size-4" />
                <span>Нотатки</span>
              </span>
              <p
                class="text-base italic text-foreground/90 whitespace-pre-wrap"
              >
                {workOrderDetails.workOrder.notes}
              </p>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="mt-4 border border-border rounded bg-card">
      <div class="p-4">
        <div class="flex flex-row items-center justify-between">
          <div class="flex flex-row gap-2 text-xl font-medium">
            <p>Робота</p>
            <p class="text-muted-foreground">
              {workOrderDetails?.laborItems.length}
            </p>
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
        <DataTable
          table={laborTable}
          emptyMessage="Сервісів не знайдено..."
          class="mt-4"
          headerClasses={{ name: "w-full", select: "w-10 p-0" }}
          cellClasses={{ actions: "p-0", sum: "p-0", select: "p-0" }}
        >
          {#snippet selectionActions()}
            <DataTableBulkActions
              onDelete={() =>
                laborTableActions.openBulkDelete(selectedLaborIds)}
            />
          {/snippet}
        </DataTable>
      </div>

      <div class="p-4 pt-0">
        <div class="flex flex-row items-center justify-between">
          <div class="flex flex-row gap-2 text-xl font-medium">
            <p>Запчастини</p>
            <p class="text-muted-foreground">
              {workOrderDetails?.partItems.length}
            </p>
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
    </div>
  {/if}
</PageLayout>
