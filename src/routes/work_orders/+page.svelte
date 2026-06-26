<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";

  import { Input } from "$lib/components/ui/input";

  import PageLayout from "$lib/components/page-layout.svelte";

  import DropdownMenu from "./[workorderId]/dropdown-menu.svelte";
  import { columns } from "./columns";
  import DataTable from "./data-table.svelte";

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
  <DataTable {data} {columns} bind:searchTerm={inputValue} />
</PageLayout>
