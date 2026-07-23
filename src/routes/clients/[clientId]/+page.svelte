<script lang="ts">
  import { MaskPatterns, PatternFormat } from "svelte-number-format";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import { goto } from "$app/navigation";

  import type { Client } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/misc";

  import Button from "$lib/components/ui/button/button.svelte";
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
  let cars = $derived(appStore.cars.filter((c) => c.client_id === client.id));
  let clientWorkOrders = $derived.by(() => {
    const carIds = new Set(cars.map((c) => c.id));
    return appStore.work_orders
      .filter((wo) => carIds.has(wo.car_id))
      .sort(
        (a, b) =>
          new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
      );
  });

  const tableData = $derived(
    clientWorkOrders.map((order) => {
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
    }),
  );

  let table = buildTable({
    get data() {
      return tableData;
    },
    get columns() {
      return columns;
    },
    disableSorting: true,
    disableFiltering: true,
    disablePagination: true,
    initialSorting: [{ id: "created_at", desc: true }],
    hiddenColumns: ["client"],
  });
</script>

{#if client}
  <PageLayout>
    {#snippet header()}
      <div class="h-full px-4 flex items-center justify-between">
        <Button variant="secondary" onclick={() => goto("/clients")}>
          <ChevronLeftIcon /> Список
        </Button>
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
            <PatternFormat
              bind:value={client.phone}
              format={MaskPatterns.PHONE_INTERNATIONAL}
              class="text-end"
            />
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={CalendarPlusIcon} /> Створено
          </Details.Title>
          <Details.Content>
            {prettifyIsoDateString(client.created_at, "long")}
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
