<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import FunnelXIcon from "@lucide/svelte/icons/funnel-x";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import DataTableFacetedFilter from "$lib/components/data-table-faceted-filter/data-table-faceted-filter.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { carBrandsQueryOptions, carsQueryOptions } from "../cars/queries";
  import { clientsQueryOptions } from "../clients/queries";
  import { columns } from "./columns";
  import {
    workOrderLaborItemsQueryOptions,
    workOrderPartItemsQueryOptions,
    workOrdersQueryOptions,
    workOrderTotalsQueryOptions,
  } from "./queries";
  import { buildWorkOrderListItems } from "./types";
  import { statusFilterOptions } from "./work-order-status";

  let inputValue: string = $state("");
  let searchInputEl = $state<HTMLInputElement | null>(null);

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
    get searchTerm() {
      return inputValue;
    },
    initialSorting: [{ id: "created_at", desc: true }],
  });

  const isFiltered = $derived(table.getState().columnFilters.length > 0);
  const statusCol = $derived(table.getColumn("status"));

  $effect(() => searchInputEl?.focus());
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full flex items-center gap-2">
      <Input
        placeholder="Пошук за кодом, клієнтом, автомобілем, статусом, або датою..."
        bind:value={inputValue}
        bind:ref={searchInputEl}
        disabled={!combinedQueries.isReady || workOrderListItems.length === 0}
      />
      {#if statusCol}
        <DataTableFacetedFilter
          column={statusCol}
          title="Статус"
          options={statusFilterOptions}
          disabled={!combinedQueries.isReady || workOrderListItems.length === 0}
        />
      {/if}
      {#if isFiltered}
        <Button
          size="default"
          variant="ghost"
          onclick={() => table.resetColumnFilters()}
          class="px-2 py-4"
          disabled={!combinedQueries.isReady || workOrderListItems.length === 0}
        >
          <FunnelXIcon />
          Прибрати фільтри
        </Button>
      {/if}
    </div>
  {/snippet}
  <div class="p-4">
    {#if combinedQueries.isError}
      <FetchErrorDisplay
        message={combinedQueries.firstError?.message}
        onRetry={combinedQueries.refetchAll}
        class="mt-20"
      />
    {:else if combinedQueries.isLoading}
      <Skeleton class="h-50 w-full rounded-lg" />
    {:else}
      <DataTable
        {table}
        emptyMessage="Замовлень не знайдено..."
        onRowClick={(row) => goto(`/work_orders/${row.id}`)}
        class="rounded-sm shadow-xs"
      />
    {/if}
  </div>
</PageLayout>
