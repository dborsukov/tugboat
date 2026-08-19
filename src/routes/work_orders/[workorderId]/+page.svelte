<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import { MaskPatterns, PatternText } from "svelte-number-format";

  import { goto } from "$app/navigation";

  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Info from "@lucide/svelte/icons/info";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card/index";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import type { Service } from "$lib/bindings";
  import { TableActionState } from "$lib/states/table-actions.svelte";
  import { prettifyIsoDateString } from "$lib/tools";

  import ClientAvatar from "../../clients/client-avatar.svelte";
  import { servicesQueryOptions } from "../../services/queries";
  import {
    createCreateLaborItemMutation,
    createDeleteLaborItemMutation,
    createDeletePartItemMutation,
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
  import { columns as partsColumns, type PartsTableMeta } from "./part-columns";
  import WorkOrderPartsItemMutationSheet from "./part-item-mutation-sheet.svelte";

  let { params } = $props();

  let isPopoverOpen: boolean = $state(false);

  const queryClient = useQueryClient();

  const deletePartItemMutation = createDeletePartItemMutation(queryClient);
  const createLaborItemMutation = createCreateLaborItemMutation(queryClient);
  const deleteLaborItemMutation = createDeleteLaborItemMutation(queryClient);

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
      return partsColumns;
    },
    get meta() {
      return {
        onEdit: partsTableActions.openEdit,
        onDelete: partsTableActions.openDelete,
      } satisfies PartsTableMeta;
    },
    disableSorting: true,
    disableFiltering: true,
    disablePagination: true,
  });

  let laborTable = buildTable({
    get data() {
      return combinedQueries.workOrderDetails.data?.laborItems ?? [];
    },
    get columns() {
      return laborColumns;
    },
    get meta() {
      return {
        onEdit: laborTableActions.openEdit,
        onDelete: laborTableActions.openDelete,
      } satisfies LaborTableMeta;
    },
    disableSorting: true,
    disableFiltering: true,
    disablePagination: true,
  });

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

  function carSecondaryLine(wod: typeof workOrderDetails) {
    return [wod?.carDto?.car?.licensePlate, wod?.carDto?.car?.year]
      .filter(Boolean)
      .join(" · ");
  }
</script>

<PageLayout headerWidthClass="max-w-fit" contentWidthClass="max-w-fit">
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
        {#each { length: 4 } as _, i (i)}
          <div class="p-4 flex flex-col justify-start items-start gap-1.5">
            <Skeleton class="h-3 w-20" />
            <Skeleton class="h-5 w-24" />
          </div>
          {#if i < 3}
            <div class="block h-8/12 w-px bg-border"></div>
          {/if}
        {/each}

        <div class="flex items-center gap-2 ml-auto mr-2">
          <Skeleton class="h-9 w-32 rounded-md" />
          <Skeleton class="h-9 w-9 rounded-md shrink-0" />
        </div>
      </div>

      <!-- Car / client cards -->
      <div class="flex flex-row divide-x divide-border border-b border-border">
        {#each { length: 2 } as _, i (i)}
          <div class="flex-1 flex items-center gap-4 p-4">
            <Skeleton class="size-14 rounded-full shrink-0" />
            <div class="flex flex-col items-start gap-2 flex-1 min-w-0">
              <Skeleton class="h-5 w-2/5 min-w-32" />
              <Skeleton class="h-4 w-1/3 min-w-24" />
            </div>
            <Skeleton class="size-5 shrink-0 ml-auto" />
          </div>
        {/each}
      </div>

      <!-- Totals bar: parts / labor / (discount) / grand total -->
      <div class="h-15 flex flex-row items-center border-b border-border">
        <div class="p-4 flex flex-col justify-start items-start gap-1.5">
          <Skeleton class="h-3 w-20" />
          <Skeleton class="h-5 w-16" />
        </div>

        <div class="block h-8/12 w-px bg-border"></div>

        <div class="p-4 flex flex-col justify-start items-start gap-1.5">
          <Skeleton class="h-3 w-14" />
          <Skeleton class="h-5 w-16" />
        </div>

        <div
          class="ml-auto h-full flex flex-col px-2 py-1 mr-4 justify-center items-end gap-1.5"
        >
          <Skeleton class="h-3 w-12" />
          <Skeleton class="h-5 w-20" />
        </div>
      </div>

      <!-- Services (labor) section -->
      <div class="p-4">
        <div class="flex flex-row items-center justify-between">
          <div class="flex flex-row items-center gap-2">
            <Skeleton class="h-7 w-20" />
            <Skeleton class="h-7 w-5" />
          </div>
          <div class="flex gap-4">
            <Skeleton class="h-5 w-24" />
            <Skeleton class="h-5 w-32" />
          </div>
        </div>
        <div class="mt-4 flex flex-col gap-2">
          <Skeleton class="h-9 w-full" />
          {#each { length: 3 } as _, i (i)}
            <Skeleton class="h-12 w-full" />
          {/each}
        </div>
      </div>

      <!-- Parts section -->
      <div class="p-4 pt-0">
        <div class="flex flex-row items-center justify-between">
          <div class="flex flex-row items-center gap-2">
            <Skeleton class="h-7 w-24" />
            <Skeleton class="h-7 w-5" />
          </div>
          <Skeleton class="h-5 w-20" />
        </div>
        <div class="mt-4 flex flex-col gap-2">
          <Skeleton class="h-9 w-full" />
          {#each { length: 2 } as _, i (i)}
            <Skeleton class="h-12 w-full" />
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
      <div class="h-15 flex flex-row items-center border-b border-border">
        <div class="p-4 flex flex-col justify-start items-start">
          <span class="text-sm text-muted-foreground">Наряд-замовлення</span>
          <p class="text-base font-mono font-medium">
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
          <StatusSelector workOrder={workOrderDetails?.workOrder} />
          <DropdownMenu workOrderId={workOrderDetails.workOrder?.id ?? ""} />
        </div>
      </div>

      <div class="flex flex-row divide-x divide-border border-b border-border">
        <div class="flex-1">
          <Button
            variant="ghost"
            class="w-full h-auto gap-2 items-center rounded-none p-2 text-base font-normal"
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
            <PatternText
              value={workOrderDetails?.client?.phone}
              format={MaskPatterns.PHONE_INTERNATIONAL}
              class="ml-auto select-none! text-muted-foreground"
            />
            <ChevronRight class="size-4 text-muted-foreground" />
          </Button>
        </div>
      </div>
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
          </div>
        </div>
        <DataTable
          table={laborTable}
          emptyMessage="Сервісів не знайдено..."
          class="mt-4"
          headerClasses={{ name: "w-full" }}
          cellClasses={{ actions: "p-0", sum: "p-0" }}
        />
      </div>

      <div class="p-4 pt-0">
        <div class="flex flex-row items-center justify-between">
          <div class="flex flex-row gap-2 text-xl font-medium">
            <p>Запчастини</p>
            <p class="text-muted-foreground">
              {workOrderDetails?.partItems.length}
            </p>
          </div>
          <Card.Action>
            <Button
              variant="link"
              onclick={partsTableActions.openCreate}
              class="text-base">Додати +</Button
            >
          </Card.Action>
        </div>
        <DataTable
          table={partsTable}
          emptyMessage="Запчастин не знайдено..."
          class="mt-4"
          headerClasses={{ name: "w-full" }}
          cellClasses={{
            status: "p-0",
            actions: "p-0",
            discountedPrice: "p-0",
          }}
        />
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
    </div>
  {/if}
</PageLayout>
