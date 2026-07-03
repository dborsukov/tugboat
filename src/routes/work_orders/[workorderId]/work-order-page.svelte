<script lang="ts">
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import GaugeIcon from "@lucide/svelte/icons/gauge";

  import { goto } from "$app/navigation";

  import { type WorkOrder } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/utils";

  import Button from "$lib/components/ui/button/button.svelte";

  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "../../cars/car-card.svelte";
  import ClientCard from "../../clients/client-card.svelte";
  import StatusSelector from "../status-selector.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { work_order }: { work_order: WorkOrder } = $props();

  const car = appStore.cars.find((c) => c.id === work_order.car_id);
  const carBrand = appStore.car_brands.find((c) => c.id === car?.brand_id);
  const client = appStore.clients.find((c) => c.id === car?.client_id);
  if (car === undefined || client === undefined || carBrand === undefined) {
    throw new Error("Elements missing from appStore");
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="h-full px-4 flex items-center justify-between">
      <Button variant="secondary" onclick={() => goto("/work_orders")}>
        <ChevronLeftIcon /> Список
      </Button>
      <div class="flex gap-2 items-center">
        <StatusSelector {work_order} />
        <DropdownMenu {work_order} />
      </div>
    </div>
  {/snippet}
  <div class="h-full w-full p-4">
    <div class="border p-4 rounded">
      <h2 class="text-xl text-center font-semibold">
        Замовлення {work_order.id}
      </h2>
    </div>
    <div class="grid grid-cols-2 gap-4 mt-4">
      <a href="/cars/{car.id}">
        <CarCard {car} short class="h-full" />
      </a>
      <a href="/clients/{client.id}">
        <ClientCard {client} short class="h-full" />
      </a>
    </div>
    <div class="border p-4 rounded grid mt-4">
      <div class="flex items-center justify-between border-b border-muted pb-2">
        <span class="flex items-center gap-2 text-muted-foreground">
          <CalendarPlusIcon class="size-4 text-muted-foreground/70" /> Створено
        </span>
        <span class="max-w-45 truncate">
          {prettifyIsoDateString(work_order.created_at, "long")}
        </span>
      </div>

      {#if work_order.started_at}
        <div
          class="flex items-center justify-between mt-2 border-b border-muted pb-2"
        >
          <span class="flex items-center gap-2 text-muted-foreground">
            <CalendarPlusIcon class="size-4 text-muted-foreground/70" /> Почато
          </span>
          <span class="max-w-45 truncate">
            {prettifyIsoDateString(work_order.started_at, "long")}
          </span>
        </div>
      {/if}

      {#if work_order.completed_at}
        <div
          class="flex items-center justify-between mt-2 border-b border-muted pb-2"
        >
          <span class="flex items-center gap-2 text-muted-foreground">
            <CalendarPlusIcon class="size-4 text-muted-foreground/70" /> Завершено
          </span>
          <span class="max-w-45 truncate">
            {prettifyIsoDateString(work_order.completed_at, "long")}
          </span>
        </div>
      {/if}

      <div class="flex items-center justify-between mt-2">
        <span class="flex items-center gap-2 text-muted-foreground">
          <GaugeIcon class="size-4 text-muted-foreground/70" />Одометр
        </span>
        {#if work_order.odometer_in}
          <span class="max-w-45 truncate font-mono border rounded px-2">
            {work_order.odometer_in}
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>
    </div>
  </div></PageLayout
>
