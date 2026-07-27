<script lang="ts">
  import WorkOrderIcon from "@lucide/svelte/icons/briefcase-business";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import FingerprintIcon from "@lucide/svelte/icons/fingerprint-pattern";
  import HashIcon from "@lucide/svelte/icons/hash";
  import GotoIcon from "@lucide/svelte/icons/square-arrow-out-up-right";
  import UserIcon from "@lucide/svelte/icons/user";

  import { goto } from "$app/navigation";

  import type { Car } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/misc";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card/index";
  import * as Details from "$lib/components/ui/details/index";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import CopyButton from "$lib/components/copy-button.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import ClientBadge from "../../clients/client-badge.svelte";
  import { columns } from "../../work_orders/columns";
  import WorkOrderMutationSheet from "../../work_orders/work-order-mutation-sheet.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { data } = $props();
  let isSheetOpen: boolean = $state(false);

  let car = $derived(
    appStore.cars.find((c) => c.id.toString() === data.id) as Car,
  );
  let owner = $derived(appStore.clients.find((c) => c.id === car.client_id));
  let brand = $derived(appStore.car_brands.find((b) => b.id === car.brand_id));
  let slug = $derived(brand != undefined ? brand.icon_slug : "");
  let carWorkOrders = $derived.by(() => {
    return appStore.work_orders.filter((wo) => wo.car_id === car.id);
  });

  const tableData = $derived(
    carWorkOrders.map((order) => {
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
    disableFiltering: true,
    disablePagination: true,
    initialSorting: [{ id: "created_at", desc: true }],
    hiddenColumns: ["client", "car_id"],
  });
</script>

{#if car}
  <PageLayout>
    {#snippet header()}
      <div class="h-full flex items-center">
        <Button
          variant="outline"
          class="ml-auto mr-2"
          onclick={() => (isSheetOpen = !isSheetOpen)}
        >
          <WorkOrderIcon /> Нове замовлення
        </Button>
        <DropdownMenu {car} />
        <WorkOrderMutationSheet bind:open={isSheetOpen} car_id={car.id} />
      </div>
    {/snippet}
    <div class="p-4">
      <Card.Root>
        <Card.Content class="flex flex-row items-center justify-center gap-4">
          <div class="h-15">
            <LogoLoader {slug} />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xl font-semibold tracking-tight truncate">
              {#if brand != undefined}
                {brand.name}
              {:else}
                ?
              {/if}
            </div>
            <div class="text-lg tracking-tight mt-0.5">
              {car.model}
            </div>
          </div>
          <div class="flex flex-col gap-1 justify-between items-end">
            <Badge variant="outline" class="text-lg px-3 py-3.5 bg-background">
              {car.year}
            </Badge>
            <p class="text-muted-foreground">ID: {car.id}</p>
          </div>
        </Card.Content>
      </Card.Root>

      <Details.Card class="mt-4">
        <Details.Item>
          <Details.Title>
            <Details.Icon icon={UserIcon} /> Власник
          </Details.Title>
          <Details.Content>
            {#if owner != undefined}
              <div class="flex justify-end gap-2">
                <ClientBadge client={owner} />
                <Button
                  size="icon"
                  variant="ghost"
                  onclick={() => goto(`/clients/${owner.id}`)}
                >
                  <GotoIcon />
                </Button>
              </div>
            {:else}
              Не вказано
            {/if}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={HashIcon} /> Номерний знак
          </Details.Title>
          <Details.Content>
            {#if car.registration != undefined}
              <div class="flex items-center gap-2">
                <p
                  class="px-2 py-0.5 font-mono uppercase tracking-wider bg-muted rounded border"
                >
                  {car.registration}
                </p>
                <CopyButton variant="ghost" text={car.registration} />
              </div>
            {:else}
              Не вказано
            {/if}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={FingerprintIcon} /> VIN-код
          </Details.Title>
          <Details.Content>
            {#if car.vin != undefined}
              <div class="flex items-center gap-2">
                <p
                  class="px-2 py-0.5 font-mono uppercase tracking-wider bg-muted rounded border"
                >
                  {car.vin}
                </p>
                <CopyButton text={car.vin} variant="ghost" />
              </div>
            {:else}
              Не вказано
            {/if}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={CalendarPlusIcon} /> Створено
          </Details.Title>
          <Details.Content>
            {prettifyIsoDateString(car.created_at, "long")}
          </Details.Content>
        </Details.Item>
      </Details.Card>

      <Card.Root class="mt-4">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Замовлення</p>
            <p class="text-muted-foreground">{carWorkOrders.length}</p>
          </Card.Title>
        </Card.Header>
        {#if carWorkOrders.length != 0}
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
