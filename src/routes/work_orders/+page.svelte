<script lang="ts">
  import { goto } from "$app/navigation";

  import { appStore } from "$lib/stores/app.svelte";

  import { Input } from "$lib/components/ui/input";

  import DataTable from "$lib/components/data-table.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { columns } from "./columns";

  const data = appStore.work_orders.map((order) => {
    const car = appStore.cars.find((c) => c.id === order.car_id);
    const carBrand = appStore.car_brands.find((c) => c.id === car?.brand_id);
    const client = appStore.clients.find((c) => c.id === car?.client_id);
    if (car === undefined || client === undefined || carBrand === undefined) {
      throw new Error("Elements missing from appStore");
    }
    return {
      ...order,
      car,
      carBrand,
      client,
    };
  });

  let inputValue: string = $state("");
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center gap-2">
      <Input placeholder="Пошук..." bind:value={inputValue} class="w-full" />
    </div>
  {/snippet}
  <DataTable
    {data}
    {columns}
    bind:searchTerm={inputValue}
    emptyMessage="Замовлень не знайдено..."
    onRowClick={(row) => goto(`/work_orders/${row.id}`)}
  />
</PageLayout>
