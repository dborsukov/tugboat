<script lang="ts">
  import { MaskPatterns, PatternText } from "svelte-number-format";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import { goto } from "$app/navigation";

  import type { Client } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/misc";

  import * as Card from "$lib/components/ui/card/index.js";
  import * as Details from "$lib/components/ui/details/index";

  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "../../cars/car-card.svelte";
  import { columns } from "../../work_orders/columns";
  import ClientBadge from "../client-badge.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { data } = $props();

  let client = $derived(
    appStore.clients.find((c) => c.id.toString() === data.id) as Client,
  );
  let cars = $derived(appStore.cars.filter((c) => c.clientId === client.id));
  let clientWorkOrders = $derived.by(() => {
    const carIds = new Set(cars.map((c) => c.id));
    return appStore.workOrders
      .filter((wo) => carIds.has(wo.carId))
      .sort(
        (a, b) =>
          new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
      );
  });

  const tableData = $derived(
    clientWorkOrders.map((order) => {
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
    disableFiltering: true,
    disablePagination: true,
    initialSorting: [{ id: "created_at", desc: true }],
    hiddenColumns: ["client"],
  });
</script>

{#if client}
  <PageLayout>
    {#snippet header()}
      <div class="h-full flex items-center justify-end">
        <DropdownMenu {client} />
      </div>
    {/snippet}
    <div class="p-4">
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <ClientBadge
            {client}
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
              {#each cars as car (car.id)}
                <a href="/cars/{car.id}" class="w-full">
                  <CarCard {car} short />
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
            <p class="text-muted-foreground">{clientWorkOrders.length}</p>
          </Card.Title>
        </Card.Header>
        {#if clientWorkOrders.length != 0}
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
  </PageLayout>
{/if}
