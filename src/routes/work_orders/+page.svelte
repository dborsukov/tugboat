<script lang="ts">
  import FunnelXIcon from "@lucide/svelte/icons/funnel-x";

  import { goto } from "$app/navigation";

  import { appStore } from "$lib/stores/app.svelte";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import DataTableFacetedFilter from "$lib/components/data-table-faceted-filter/data-table-faceted-filter.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { columns } from "./columns";
  import { statusFilterOptions } from "./work-order-status";

  let inputValue: string = $state("");

  const tableData = $derived(
    appStore.workOrders.map((order) => {
      const car = appStore.cars.find((c) => c.id === order.carId);
      const carBrand = appStore.carBrands.find((c) => c.id === car?.brandId);
      const client = appStore.clients.find((c) => c.id === car?.clientId);
      const labor = appStore.workOrderLaborItems.filter(
        (i) => i.workOrderId === order.id,
      );
      const parts = appStore.workOrderPartItems.filter(
        (i) => i.workOrderId === order.id,
      );
      if (car === undefined || client === undefined || carBrand === undefined) {
        throw new Error("Elements missing from appStore");
      }
      return {
        ...order,
        car,
        carBrand,
        client,
        labor,
        parts,
      };
    }),
  );

  let table = buildTable({
    get data() {
      return tableData;
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
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full pl-1 flex items-center gap-2">
      <Input
        placeholder="Пошук за кодом, клієнтом, автомобілем, статусом, або датою..."
        bind:value={inputValue}
      />
      {#if statusCol}
        <DataTableFacetedFilter
          column={statusCol}
          title="Статус"
          options={statusFilterOptions}
        />
      {/if}
      {#if isFiltered}
        <Button
          size="default"
          variant="ghost"
          onclick={() => table.resetColumnFilters()}
          class="px-2 py-4"
        >
          <FunnelXIcon />
          Прибрати фільтри
        </Button>
      {/if}
    </div>
  {/snippet}
  <div class="p-4">
    <DataTable
      {table}
      emptyMessage="Замовлень не знайдено..."
      onRowClick={(row) => goto(`/work_orders/${row.id}`)}
      class="rounded-sm shadow-xs"
    />
  </div>
</PageLayout>
