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
  import { columns } from "../../work_orders/columns";
  import { workOrderDetailsListByClientIdQueryOptions } from "../../work_orders/queries";
  import ClientBadge from "../client-badge.svelte";
  import { clientDetailsQueryOptions } from "../queries";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { params } = $props();
  let clientId = $derived(Number(params.clientId));

  const combinedQueries = createQueries(() => ({
    queries: [
      clientDetailsQueryOptions(clientId),
      workOrderDetailsListByClientIdQueryOptions(clientId),
    ],
    combine: (results) => {
      return {
        clientDetails: results[0],
        workOrderDetailsDtoList: results[1],

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

  let clientDetails = $derived(combinedQueries.clientDetails.data);
  let workOrderListItems = $derived(
    combinedQueries.workOrderDetailsDtoList.data ?? [],
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
    {#if clientDetails}
      <div class="h-full flex items-center justify-end">
        <DropdownMenu clientId={clientDetails?.client?.id} />
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
  {:else if !clientDetails}
    <NotFound
      message={`Клієнта не знайдено (ID: ${params.clientId})`}
      class="mt-20"
    />
  {:else}
    <div class="p-4">
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <ClientBadge
            name={clientDetails?.client?.name}
            class="text-xl font-semibold"
            iconSize="size-10"
          />
          <p class="text-muted-foreground">ID: {clientDetails?.client?.id}</p>
        </Card.Content>
      </Card.Root>
      <Details.Card class="mt-4">
        <Details.Item>
          <Details.Title>
            <Details.Icon icon={PhoneIcon} /> Номер
          </Details.Title>
          <Details.Content>
            <PatternText
              value={clientDetails?.client?.phone}
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
            {prettifyIsoDateString(clientDetails?.client?.createdAt, "long")}
          </Details.Content>
        </Details.Item>
      </Details.Card>

      <Card.Root class="mt-4 gap-2">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Автомобілі</p>
            <p class="text-muted-foreground">{clientDetails?.cars?.length}</p>
          </Card.Title>
        </Card.Header>
        {#if clientDetails?.cars?.length != 0}
          <Card.Content>
            <div
              class="grid gap-4 grid-cols-[repeat(auto-fit,minmax(100px,390px))] mt-4 w-full"
            >
              {#each clientDetails?.cars as cli (cli.car.id)}
                <a href="/cars/{cli.car.id}" class="w-full">
                  <CarCard
                    car={cli.car}
                    client={clientDetails.client}
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
              onRowClick={(row) => goto(`/work_orders/${row.workOrder.id}`)}
            />
          </Card.Content>
        {/if}
      </Card.Root>
    </div>
  {/if}
</PageLayout>
