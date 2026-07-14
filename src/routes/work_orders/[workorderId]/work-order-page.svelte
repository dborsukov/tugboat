<script lang="ts">
  import { toast } from "svelte-sonner";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import RateIcon from "@lucide/svelte/icons/hand-coins";

  import { goto } from "$app/navigation";

  import { commands, type WorkOrder } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/utils";

  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Command from "$lib/components/ui/command/index.js";
  import Input from "$lib/components/ui/input/input.svelte";
  import * as Popover from "$lib/components/ui/popover/index.js";

  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "../../cars/car-card.svelte";
  import ClientCard from "../../clients/client-card.svelte";
  import StatusSelector from "../status-selector.svelte";
  import DataTable from "./data-table.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";
  import { columns as laborColumns } from "./labor-columns";
  import { columns as partsColumns } from "./parts-columns";
  import WorkOrderLaborItemMutationSheet from "./work-order-labor-item-mutation-sheet.svelte";
  import WorkOrderPartItemMutationSheet from "./work-order-parts-item-mutation-sheet.svelte";

  let { work_order }: { work_order: WorkOrder } = $props();
  let isLaborSheetOpen: boolean = $state(false);
  let isPartsSheetOpen: boolean = $state(false);
  let isPopoverOpen: boolean = $state(false);

  const car = appStore.cars.find((c) => c.id === work_order.car_id);
  const carBrand = appStore.car_brands.find((c) => c.id === car?.brand_id);
  const client = appStore.clients.find((c) => c.id === car?.client_id);
  if (car === undefined || client === undefined || carBrand === undefined) {
    throw new Error("Elements missing from appStore");
  }

  const labor_data = $derived(
    appStore.work_order_labor_items
      .filter((i) => i.work_order_id === work_order.id)
      .map((item) => {
        return {
          ...item,
          rate: work_order.rate,
        };
      }),
  );
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

      <div
        class="flex items-center justify-between mt-2 border-b border-muted pb-2"
      >
        <span class="flex items-center gap-2 text-muted-foreground">
          <GaugeIcon class="size-4 text-muted-foreground/70" />Одометр
        </span>
        {#if work_order.odometer_in}
          <span class="max-w-45 truncate font-mono border rounded px-2">
            {work_order.odometer_in} км
          </span>
        {:else}
          <span class="text-muted-foreground/50 italic text">Не вказано</span>
        {/if}
      </div>

      <div class="flex items-center justify-between mt-2">
        <span class="flex items-center gap-2 text-muted-foreground">
          <RateIcon class="size-4 text-muted-foreground/70" />Тариф
        </span>
        <span class="max-w-45 truncate font-mono border rounded px-2">
          {work_order.rate} грн/год.
        </span>
      </div>
    </div>
    <div class="flex gap-4 my-4">
      <h2 class="text-xl font-semibold mr-auto">Сервіси</h2>
      <ButtonGroup.Root>
        <Button
          variant="outline"
          onclick={() => (isLaborSheetOpen = !isLaborSheetOpen)}
          >Створити</Button
        >
        <Popover.Root bind:open={isPopoverOpen}>
          <Popover.Trigger>
            {#snippet child({ props })}
              <Button {...props}>
                Додати з бази
                <ChevronDownIcon />
              </Button>
            {/snippet}
          </Popover.Trigger>
          <Popover.Content class="w-80 p-0">
            <Command.Root>
              <Command.Input placeholder="Пошук сервісів..." />
              <Command.List>
                <Command.Empty>Сервісів не знайдено...</Command.Empty>
                <Command.Group value="services">
                  {#each appStore.services as service (service.id)}
                    <Command.Item
                      value={service.name}
                      onSelect={async () => {
                        let res = await commands.createWorkOrderLaborItem({
                          work_order_id: work_order.id,
                          service_id: service.id,
                          code: service.code,
                          name: service.name,
                          labor_hours: service.labor_hours,
                        });
                        isPopoverOpen = false;
                        if (res.status === "ok") {
                          appStore.createWorkOrderLaborItem(res.data);
                        } else {
                          return toast.error("Виникла помилка", {
                            description: res.error.message,
                            position: "bottom-left",
                          });
                        }
                      }}
                    >
                      {service.name}
                    </Command.Item>
                  {/each}
                </Command.Group>
              </Command.List>
            </Command.Root>
          </Popover.Content>
        </Popover.Root>
      </ButtonGroup.Root>
    </div>
    <div class="border rounded mt-4">
      <DataTable columns={laborColumns} data={labor_data} />
    </div>
    <div class="flex gap-4 my-4">
      <h2 class="text-xl font-semibold mr-auto">Запчастини</h2>
      <Button onclick={() => (isPartsSheetOpen = !isPartsSheetOpen)}
        >Створити</Button
      >
    </div>
    <div class="border rounded mt-4">
      <DataTable columns={partsColumns} data={appStore.work_order_part_items} />
    </div>
  </div>
  <WorkOrderLaborItemMutationSheet
    bind:open={isLaborSheetOpen}
    work_order_id={work_order.id}
  />
  <WorkOrderPartItemMutationSheet
    bind:open={isPartsSheetOpen}
    work_order_id={work_order.id}
  />
</PageLayout>
