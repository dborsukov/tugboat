<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";
  import { MaskPatterns, PatternText } from "svelte-number-format";

  import { goto } from "$app/navigation";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import * as Card from "$lib/components/ui/card/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import * as Details from "$lib/components/details/index";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { prettifyIsoDateString } from "$lib/tools";

  import CarCard from "../../cars/car-card.svelte";
  import { carBrandsQueryOptions, carsQueryOptions } from "../../cars/queries";
  import { buildCarListItems } from "../../cars/types";
  import { columns } from "../../work_orders/columns";
  import {
    workOrderLaborItemsQueryOptions,
    workOrderPartItemsQueryOptions,
    workOrdersQueryOptions,
    workOrderTotalsQueryOptions,
  } from "../../work_orders/queries";
  import { buildWorkOrderListItems } from "../../work_orders/types";
  import ClientBadge from "../client-badge.svelte";
  import { clientsQueryOptions } from "../queries";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { params } = $props();

  const combinedQueries = createQueries(() => ({
    queries: [
      workOrdersQueryOptions(),
      carsQueryOptions(),
      clientsQueryOptions(),
      carBrandsQueryOptions(),
      workOrderLaborItemsQueryOptions(),
      workOrderPartItemsQueryOptions(),
      workOrderTotalsQueryOptions(),
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

  let client = $derived(
    combinedQueries.clients.data?.find(
      (c) => c.id.toString() === params.clientId,
    ),
  );
  let cars = $derived(
    combinedQueries.cars.data?.filter((car) => car.clientId === client?.id) ??
      [],
  );
  let clientCarIds = $derived(
    new Set(cars.filter((c) => c.clientId === client!.id).map((c) => c.id)),
  );

  let carListItems = $derived(
    combinedQueries.isReady
      ? buildCarListItems(
          cars,
          combinedQueries.clients.data!,
          combinedQueries.carBrands.data!,
        )
      : [],
  );

  let workOrderListItems = $derived(
    combinedQueries.isReady
      ? buildWorkOrderListItems(
          combinedQueries.workOrders.data!,
          combinedQueries.cars.data!,
          combinedQueries.carBrands.data!,
          combinedQueries.clients.data!,
          combinedQueries.laborItems.data!,
          combinedQueries.partItems.data!,
          combinedQueries.totals.data!,
          (wo) => clientCarIds.has(wo.carId),
        )
      : [],
  );

  let table = buildTable({
    get data() {
      return workOrderListItems;
    },
    get columns() {
      return columns;
    },
    disableFiltering: true,
    disablePagination: true,
    initialSorting: [{ id: "created_at", desc: true }],
    hiddenColumns: ["client"],
  });
</script>

<PageLayout>
  {#snippet header()}
    {#if client}
      <div class="h-full flex items-center justify-end">
        <DropdownMenu clientId={client.id} />
      </div>
    {/if}
  {/snippet}
  {#if combinedQueries.isError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-20"
    />
  {:else if combinedQueries.isLoading}
    <Skeleton class="w-full h-25" />
    <Skeleton class="mt-4 w-full h-40" />
    <Skeleton class="mt-4 w-full h-40" />
  {:else if !client}
    <NotFound
      message={`Клієнта не знайдено (ID: ${params.clientId})`}
      class="mt-20"
    />
  {:else}
    <div class="p-4">
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <ClientBadge
            name={client.name}
            class="text-xl font-semibold"
            iconSize="size-10"
          />
          <p class="text-muted-foreground">ID: {client.id}</p>
        </Card.Content>
      </Card.Root>
      <Details.Card class="mt-4">
        <Details.Item>
          <Details.Title>
            <Details.Icon icon={PhoneIcon} /> Номер
          </Details.Title>
          <Details.Content>
            <PatternText
              value={client.phone}
              format={MaskPatterns.PHONE_INTERNATIONAL}
              class="text-end select-none!"
            />
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={CalendarPlusIcon} /> Створено
          </Details.Title>
          <Details.Content>
            {prettifyIsoDateString(client.createdAt, "long")}
          </Details.Content>
        </Details.Item>
      </Details.Card>

      <Card.Root class="mt-4 gap-2">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Автомобілі</p>
            <p class="text-muted-foreground">{cars.length}</p>
          </Card.Title>
        </Card.Header>
        {#if cars.length != 0}
          <Card.Content>
            <div
              class="grid gap-4 grid-cols-[repeat(auto-fit,minmax(100px,390px))] mt-4 w-full"
            >
              {#each carListItems as cli (cli.car.id)}
                <a href="/cars/{cli.car.id}" class="w-full">
                  <CarCard
                    car={cli.car}
                    client={cli.client}
                    brand={cli.brand}
                    short
                  />
                </a>
              {/each}
            </div>
          </Card.Content>
        {/if}
      </Card.Root>

      <Card.Root class="mt-4">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Замовлення</p>
            <p class="text-muted-foreground">{workOrderListItems.length}</p>
          </Card.Title>
        </Card.Header>
        {#if workOrderListItems.length != 0}
          <Card.Content>
            <DataTable
              {table}
              emptyMessage="Замовлень не знайдено..."
              onRowClick={(row) => goto(`/work_orders/${row.id}`)}
            />
          </Card.Content>
        {/if}
      </Card.Root>
    </div>
  {/if}
</PageLayout>
