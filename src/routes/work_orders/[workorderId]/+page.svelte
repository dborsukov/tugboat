<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import { NumericText } from "svelte-number-format";

  import DiscountIcon from "@lucide/svelte/icons/badge-percent";
  import CalendarCheckIcon from "@lucide/svelte/icons/calendar-check";
  import CalendarClockIcon from "@lucide/svelte/icons/calendar-clock";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import RateIcon from "@lucide/svelte/icons/hand-coins";
  import PackageIcon from "@lucide/svelte/icons/package";
  import ReceiptTextIcon from "@lucide/svelte/icons/receipt-text";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card/index";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import * as Details from "$lib/components/details/index";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { TableActionState } from "$lib/states/table-actions.svelte";
  import { prettifyIsoDateString } from "$lib/tools";

  import CarCard from "../../cars/car-card.svelte";
  import ClientCard from "../../clients/client-card.svelte";
  import { servicesQueryOptions } from "../../services/queries";
  import {
    createCreateLaborItemMutation,
    createDeleteLaborItemMutation,
    createDeletePartItemMutation,
    workOrderDetailsQueryOptions,
  } from "../queries";
  import StatusSelector from "../work-order-status-selector.svelte";
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
</script>

<PageLayout>
  {#snippet header()}
    {#if combinedQueries.isLoading}
      <div class="flex gap-4 py-2 w-full h-full">
        <Skeleton class="w-1/7 h-full" />
        <Skeleton class="w-1/7 h-full" />
        <Skeleton class="w-1/7 h-full" />
        <Skeleton class="w-1/7 h-full" />
        <Skeleton class="ml-auto h-full aspect-square" />
      </div>
    {:else if !combinedQueries.isError && workOrderDetails}
      <div class="h-full flex items-center gap-4">
        <div class="flex flex-col justify-center leading-tight shrink-0">
          <span class="text-xs text-muted-foreground">Наряд-замовлення</span>
          <h2 class="text-lg font-mono font-medium">
            {workOrderDetails.workOrder?.id}
          </h2>
        </div>

        <div class="hidden md:block h-9/12 w-px bg-border shrink-0"></div>

        <div
          class="h-full flex items-center gap-4 flex-1 min-w-0"
          hidden={!combinedQueries.isReady}
        >
          <div class="hidden md:flex items-center gap-2">
            <WrenchIcon class="size-5 text-muted-foreground" />
            <div class="flex flex-col leading-tight">
              <span class="text-xs text-muted-foreground">Роботи</span>
              <CurrencyDisplay
                cents={workOrderDetails.totals?.laborTotalCents ?? 0}
                suffix="грн."
              />
            </div>
          </div>

          <div class="hidden md:flex items-center gap-2">
            <PackageIcon class="size-5 text-muted-foreground" />
            <div class="flex flex-col leading-tight">
              <span class="text-xs text-muted-foreground">Запчастини</span>
              <CurrencyDisplay
                cents={workOrderDetails.totals?.partsTotalCents ?? 0}
                suffix="грн."
              />
            </div>
          </div>

          {#if workOrderDetails.workOrder?.discountCents != 0 || workOrderDetails.workOrder?.discountPercent != 0}
            <div class="h-9/12 w-px bg-border shrink-0"></div>

            <div
              class="hidden md:flex items-center gap-2 rounded-md px-2 py-1 bg-linear-to-r from-amber-400/25 dark:from-amber-700/35 to-transparent"
            >
              <DiscountIcon class="size-5 text-amber-700 dark:text-amber-200" />
              <div class="flex flex-col leading-tight">
                <span class="text-xs text-amber-700 dark:text-amber-200"
                  >Знижка</span
                >
                {#if workOrderDetails.workOrder?.discountType === "Fixed"}
                  <CurrencyDisplay
                    cents={workOrderDetails.workOrder?.discountCents}
                    suffix="грн."
                  />
                {:else}
                  {workOrderDetails.workOrder?.discountPercent}%
                {/if}
              </div>
            </div>
          {/if}

          <div class="h-9/12 w-px bg-border shrink-0"></div>

          <div
            class="flex items-center gap-2 rounded-md px-2 py-1 bg-linear-to-r from-emerald-400/25 dark:from-emerald-700/35 to-transparent"
          >
            <ReceiptTextIcon
              class="size-5 text-emerald-700 dark:text-emerald-200"
            />
            <div class="flex flex-col leading-tight">
              <span class="text-xs text-emerald-700 dark:text-emerald-200"
                >Разом</span
              >
              <CurrencyDisplay
                cents={workOrderDetails.totals?.grandTotalCents ?? 0}
                class="font-semibold"
                suffix="грн."
              />
            </div>
          </div>
        </div>

        <div class="flex gap-2 items-center" hidden={!combinedQueries.isReady}>
          <StatusSelector workOrder={workOrderDetails.workOrder} />
          <DropdownMenu id={workOrderDetails.workOrder?.id ?? ""} />
        </div>
      </div>
    {/if}
  {/snippet}

  <div class="h-full w-full p-4">
    {#if combinedQueries.isError}
      <FetchErrorDisplay
        message={combinedQueries.firstError?.message}
        onRetry={combinedQueries.refetchAll}
        class="mt-20"
      />
    {:else if combinedQueries.isLoading}
      <div class="flex w-full h-30 items-stretch gap-4">
        <Skeleton class="flex-1" />
        <Skeleton class="flex-1" />
      </div>
      <Skeleton class="mt-4 w-full h-35" />
      <Skeleton class="mt-4 w-full h-50" />
      <Skeleton class="mt-4 w-full h-50" />
    {:else if !workOrderDetails?.workOrder}
      <NotFound
        message={`Замовлення не знайдено (ID: ${params.workorderId})`}
        class="mt-20"
      />
    {:else}
      <div class="flex flex-col sm:flex-row w-full items-stretch gap-4">
        <a href="/cars/{workOrderDetails?.carDto?.car?.id}" class="flex flex-1">
          <CarCard
            car={workOrderDetails?.carDto.car}
            client={workOrderDetails?.client}
            brand={workOrderDetails?.carDto?.brand}
            short
            class="w-full h-full flex flex-col justify-center"
          />
        </a>
        <a href="/clients/{workOrderDetails?.client?.id}" class="flex flex-1">
          <ClientCard
            client={workOrderDetails?.client}
            short
            class="w-full h-full flex flex-col justify-center"
          />
        </a>
      </div>

      <Details.Card class="mt-4">
        <Details.Item>
          <Details.Title>
            <Details.Icon icon={GaugeIcon} /> Одометр
          </Details.Title>
          <Details.Content>
            <div class="flex gap-2">
              <NumericText
                value={workOrderDetails?.workOrder.odometerIn}
                class="text-end select-none!"
              />
              <p class="text-muted-foreground">км.</p>
            </div>
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={RateIcon} /> Тариф
          </Details.Title>
          <Details.Content>
            <CurrencyDisplay
              cents={workOrderDetails?.workOrder.rateCents}
              suffix="грн./год."
              suffix_muted
            />
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={CalendarPlusIcon} /> Створено
          </Details.Title>
          <Details.Content>
            {prettifyIsoDateString(
              workOrderDetails?.workOrder.createdAt,
              "datetime",
            )}
          </Details.Content>
        </Details.Item>

        {#if workOrderDetails?.workOrder.startedAt}
          <Details.Item>
            <Details.Title>
              <Details.Icon icon={CalendarClockIcon} /> Почато
            </Details.Title>
            <Details.Content>
              {prettifyIsoDateString(
                workOrderDetails?.workOrder.startedAt,
                "datetime",
              )}
            </Details.Content>
          </Details.Item>
        {/if}

        {#if workOrderDetails?.workOrder.completedAt}
          <Details.Item>
            <Details.Title>
              <Details.Icon icon={CalendarCheckIcon} /> Завершено
            </Details.Title>
            <Details.Content>
              {prettifyIsoDateString(
                workOrderDetails?.workOrder.completedAt,
                "datetime",
              )}
            </Details.Content>
          </Details.Item>
        {/if}
      </Details.Card>

      <Card.Root class="mt-4">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Сервіси</p>
            <p class="text-muted-foreground">
              {workOrderDetails?.laborItems.length}
            </p>
          </Card.Title>
          <Card.Action>
            <ButtonGroup.Root>
              <Button variant="outline" onclick={laborTableActions.openCreate}
                >Створити</Button
              >
              <Popover.Root bind:open={isPopoverOpen}>
                <Popover.Trigger>
                  {#snippet child({ props })}
                    <Button {...props}>
                      Додати з бази
                      <ChevronDownIcon />
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
                            onSelect={async () => {
                              createLaborItemMutation.mutateAsync({
                                workOrderId: params.workorderId,
                                serviceId: service.id,
                                code: service.code,
                                name: service.name,
                                laborHours: service.laborHours,
                                discountType: "Fixed",
                                discountCents: 0,
                                discountPercent: 0,
                              });
                              isPopoverOpen = false;
                            }}
                          >
                            {service.name}
                          </Command.Item>
                        {/each}
                      </Command.Group>
                    </Command.List>
                  </Command.Root>
                </Popover.Content>
              </Popover.Root>
            </ButtonGroup.Root>
          </Card.Action>
        </Card.Header>
        <Card.Content>
          <DataTable
            table={laborTable}
            emptyMessage="Сервісів не знайдено..."
            headerClasses={{ name: "w-full" }}
          />
        </Card.Content>
      </Card.Root>

      <Card.Root class="mt-4">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Запчастини</p>
            <p class="text-muted-foreground">
              {workOrderDetails?.partItems.length}
            </p>
          </Card.Title>
          <Card.Action>
            <Button onclick={partsTableActions.openCreate}>Створити</Button>
          </Card.Action>
        </Card.Header>
        <Card.Content>
          <DataTable
            table={partsTable}
            emptyMessage="Запчастин не знайдено..."
            headerClasses={{ name: "w-full" }}
          />
        </Card.Content>
      </Card.Root>

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
    {/if}
  </div></PageLayout
>
