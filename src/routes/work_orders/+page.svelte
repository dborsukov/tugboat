<script lang="ts">
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

  import { columns } from "./columns";
  import { createWorkOrderDetailsDtoListQuery } from "./queries";
  import { statusFilterOptions } from "./work-order-status";

  let inputValue: string = $state("");
  let searchInputEl = $state<HTMLInputElement | null>(null);

  let workOrderDetailsDtoListQuery = createWorkOrderDetailsDtoListQuery();
  let workOrderDetailsDtoList = $derived(
    workOrderDetailsDtoListQuery.data ?? [],
  );

  let interactionDisabled = $derived(
    !workOrderDetailsDtoListQuery.isSuccess ||
      workOrderDetailsDtoList.length === 0,
  );

  let table = buildTable({
    get data() {
      return workOrderDetailsDtoList;
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
        disabled={interactionDisabled}
      />
      {#if statusCol}
        <DataTableFacetedFilter
          column={statusCol}
          title="Статус"
          options={statusFilterOptions}
          disabled={interactionDisabled}
        />
      {/if}
      {#if isFiltered}
        <Button
          size="default"
          variant="ghost"
          onclick={() => table.resetColumnFilters()}
          class="px-2 py-4"
          disabled={interactionDisabled}
        >
          <FunnelXIcon />
          Прибрати фільтри
        </Button>
      {/if}
    </div>
  {/snippet}
  <div class="p-4">
    {#if workOrderDetailsDtoListQuery.isError}
      <FetchErrorDisplay
        message={workOrderDetailsDtoListQuery.error?.message}
        onRetry={workOrderDetailsDtoListQuery.refetch}
        class="mt-20"
      />
    {:else if workOrderDetailsDtoListQuery.isLoading}
      <Skeleton class="h-50 w-full rounded-lg" />
    {:else}
      <DataTable
        {table}
        emptyMessage="Замовлень не знайдено..."
        onRowClick={(row) => goto(`/work_orders/${row.workOrder.id}`)}
        class="rounded-sm shadow-xs"
      />
    {/if}
  </div>
</PageLayout>
