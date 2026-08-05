<script lang="ts">
  import {
    createMutation,
    createQueries,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import { TableActionState } from "$lib/states/table-actions.svelte";
  import { NumericText } from "svelte-number-format";
  import { toast } from "svelte-sonner";

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

  import { commands } from "$lib/bindings";

  import { prettifyIsoDateString, unwrap } from "$lib/misc";

  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card/index";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Details from "$lib/components/ui/details/index";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import DeleteDialog from "$lib/components/delete-dialog.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "../../cars/car-card.svelte";
  import { carBrandsQueryOptions, carsQueryOptions } from "../../cars/queries";
  import ClientCard from "../../clients/client-card.svelte";
  import { clientsQueryOptions } from "../../clients/queries";
  import { servicesQueryOptions } from "../../services/queries";
  import {
    createCreateWorkOrderLaborItemMutation,
    invalidateWorkOrderLists,
    workOrderLaborItemsQueryOptions,
    workOrderPartItemsQueryOptions,
    workOrdersQueryOptions,
    workOrderTotalsQueryOptions,
  } from "../queries";
  import StatusSelector from "../status-selector.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";
  import {
    columns as laborColumns,
    type LaborTableMeta,
  } from "./labor-columns";
  import {
    columns as partsColumns,
    type PartsTableMeta,
  } from "./parts-columns";
  import WorkOrderLaborItemMutationSheet from "./work-order-labor-item-mutation-sheet.svelte";
  import WorkOrderPartItemMutationSheet from "./work-order-parts-item-mutation-sheet.svelte";
  import WorkOrderPartsItemMutationSheet from "./work-order-parts-item-mutation-sheet.svelte";

  let { params } = $props();

  let isPopoverOpen: boolean = $state(false);

  const queryClient = useQueryClient();

  const createItemMutation =
    createCreateWorkOrderLaborItemMutation(queryClient);

  const combinedQueries = createQueries(() => ({
    queries: [
      workOrdersQueryOptions(),
      carsQueryOptions(),
      clientsQueryOptions(),
      carBrandsQueryOptions(),
      workOrderLaborItemsQueryOptions(),
      workOrderPartItemsQueryOptions(),
      workOrderTotalsQueryOptions(),
      servicesQueryOptions(),
    ],
    combine: (results) => {
      return {
        workOrders: results[0],
        cars: results[1],
        clients: results[2],
        carBrands: results[3],
        laborItems: results[4],
        partItems: results[5],
        totals: results[6],
        services: results[7],

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

  let workOrder = $derived(
    combinedQueries.workOrders.data?.find((wo) => wo.id === params.workorderId),
  );
  let laborItems = $derived(
    combinedQueries.laborItems.data?.filter(
      (i) => i.workOrderId === workOrder?.id,
    ) ?? [],
  );
  let partItems = $derived(
    combinedQueries.partItems.data?.filter(
      (i) => i.workOrderId === workOrder?.id,
    ) ?? [],
  );
  let totals = $derived(
    combinedQueries.totals.data?.find((t) => t.workOrderId === workOrder?.id),
  );
  let car = $derived(
    combinedQueries.cars.data?.find((c) => c.id === workOrder?.carId),
  );
  let carBrand = $derived(
    combinedQueries.carBrands.data?.find((c) => c.id === car?.brandId),
  );
  let client = $derived(
    combinedQueries.clients.data?.find((c) => c.id === car?.clientId),
  );

  let partsTableActions = new TableActionState<number>();
  let laborTableActions = new TableActionState<number>();

  const partDeleteMutation = createMutation(() => ({
    mutationFn: (id: number) => unwrap(commands.deleteWorkOrderPartItem(id)),
    onSuccess: () => {
      invalidateWorkOrderLists(queryClient);
      partsTableActions.resetDelete();
    },
    onError: (error) => {
      toast.error("Виникла помилка", { description: error.message });
    },
  }));

  const laborDeleteMutation = createMutation(() => ({
    mutationFn: (id: number) => unwrap(commands.deleteWorkOrderLaborItem(id)),
    onSuccess: () => {
      invalidateWorkOrderLists(queryClient);
      laborTableActions.resetDelete();
    },
    onError: (error) => {
      toast.error("Виникла помилка", { description: error.message });
    },
  }));

  let partsTable = buildTable({
    get data() {
      return partItems;
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
      return laborItems;
    },
    get columns() {
      return laborColumns;
    },
    get meta() {
      return {
        rate: workOrder?.rateCents ?? 0,
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
      partDeleteMutation.mutate(partsTableActions.deleteId);
    }
  }

  function confirmLaborDelete() {
    if (laborTableActions.deleteId !== null) {
      laborDeleteMutation.mutate(laborTableActions.deleteId);
    }
  }
</script>

{#if workOrder}
  <PageLayout>
    {#snippet header()}
      <div class="ml-2 h-full flex items-center gap-4">
        <div class="hidden md:block h-9/12 w-px bg-border shrink-0"></div>

        <div class="flex flex-col justify-center leading-tight shrink-0">
          <span class="text-xs text-muted-foreground">Наряд-замовлення</span>
          <h2 class="text-lg font-mono font-medium">{workOrder.id}</h2>
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
                cents={totals?.laborTotalCents ?? 0}
                suffix="грн."
              />
            </div>
          </div>

          <div class="hidden md:flex items-center gap-2">
            <PackageIcon class="size-5 text-muted-foreground" />
            <div class="flex flex-col leading-tight">
              <span class="text-xs text-muted-foreground">Запчастини</span>
              <CurrencyDisplay
                cents={totals?.partsTotalCents ?? 0}
                suffix="грн."
              />
            </div>
          </div>

          {#if workOrder.discountCents != 0 || workOrder.discountPercent != 0}
            <div class="h-9/12 w-px bg-border shrink-0"></div>

            <div
              class="hidden md:flex items-center gap-2 rounded-md px-2 py-1 bg-linear-to-r from-amber-400/25 dark:from-amber-700/35 to-transparent"
            >
              <DiscountIcon class="size-5 text-amber-700 dark:text-amber-200" />
              <div class="flex flex-col leading-tight">
                <span class="text-xs text-amber-700 dark:text-amber-200"
                  >Знижка</span
                >
                {#if workOrder.discountType === "Fixed"}
                  <CurrencyDisplay
                    cents={workOrder.discountCents}
                    suffix="грн."
                  />
                {:else}
                  {workOrder.discountPercent}%
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
                cents={totals?.grandTotalCents ?? 0}
                class="font-semibold"
                suffix="грн."
              />
            </div>
          </div>
        </div>

        <div class="flex gap-2 items-center" hidden={!combinedQueries.isReady}>
          <StatusSelector {workOrder} />
          <DropdownMenu id={workOrder.id} />
        </div>
      </div>
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
      {:else if !workOrder}
        <NotFound
          message={`Замовлення не знайдено (ID: ${params.workorderId})`}
          class="mt-20"
        />
      {:else}
        <div class="flex flex-col sm:flex-row w-full items-stretch gap-4">
          <a href="/cars/{car?.id}" class="flex flex-1">
            <CarCard
              car={car!}
              client={client!}
              brand={carBrand!}
              short
              class="w-full h-full flex flex-col justify-center"
            />
          </a>
          <a href="/clients/{client?.id}" class="flex flex-1">
            <ClientCard
              client={client!}
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
                  value={workOrder.odometerIn}
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
                cents={workOrder.rateCents}
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
              {prettifyIsoDateString(workOrder.createdAt, "datetime")}
            </Details.Content>
          </Details.Item>

          {#if workOrder.startedAt}
            <Details.Item>
              <Details.Title>
                <Details.Icon icon={CalendarClockIcon} /> Почато
              </Details.Title>
              <Details.Content>
                {prettifyIsoDateString(workOrder.startedAt, "datetime")}
              </Details.Content>
            </Details.Item>
          {/if}

          {#if workOrder.completedAt}
            <Details.Item>
              <Details.Title>
                <Details.Icon icon={CalendarCheckIcon} /> Завершено
              </Details.Title>
              <Details.Content>
                {prettifyIsoDateString(workOrder.completedAt, "datetime")}
              </Details.Content>
            </Details.Item>
          {/if}
        </Details.Card>

        <Card.Root class="mt-4">
          <Card.Header>
            <Card.Title class="flex gap-2 text-xl">
              <p>Сервіси</p>
              <p class="text-muted-foreground">
                {laborItems.length}
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
                          {#each combinedQueries.services.data ?? [] as service (service.id)}
                            <Command.Item
                              value={service.name}
                              onSelect={async () => {
                                createItemMutation.mutateAsync({
                                  workOrderId: workOrder.id,
                                  serviceId: service.id,
                                  code: service.code,
                                  name: service.name,
                                  laborHours: service.laborHours,
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
                {partItems.length}
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
      {/if}
    </div></PageLayout
  >
  <WorkOrderLaborItemMutationSheet
    bind:open={laborTableActions.isMutationOpen}
    id={laborTableActions.editId}
    woId={workOrder.id}
  />
  <DeleteDialog
    bind:open={laborTableActions.isDeleteOpen}
    onDelete={confirmLaborDelete}
    description="Цю дію неможливо відмінити."
  />

  <WorkOrderPartsItemMutationSheet
    bind:open={partsTableActions.isMutationOpen}
    id={partsTableActions.editId}
    woId={workOrder.id}
  />
  <DeleteDialog
    bind:open={partsTableActions.isDeleteOpen}
    onDelete={confirmPartDelete}
    description="Цю дію неможливо відмінити."
  />
{/if}
