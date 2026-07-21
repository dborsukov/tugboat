<script lang="ts">
  import { NumberFormatStyle, NumericText } from "svelte-number-format";
  import { toast } from "svelte-sonner";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import RateIcon from "@lucide/svelte/icons/hand-coins";

  import { commands, type WorkOrder } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/utils";

  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Details from "$lib/components/ui/details/index";
  import * as Popover from "$lib/components/ui/popover/index.js";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTable from "$lib/components/data-table.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import CarCard from "../../cars/car-card.svelte";
  import ClientCard from "../../clients/client-card.svelte";
  import StatusSelector from "../status-selector.svelte";
  import DropdownMenu from "./dropdown-menu.svelte";
  import { columns as laborColumns } from "./labor-columns";
  import { columns as partsColumns } from "./parts-columns";
  import WorkOrderLaborItemMutationSheet from "./work-order-labor-item-mutation-sheet.svelte";
  import WorkOrderPartItemMutationSheet from "./work-order-parts-item-mutation-sheet.svelte";

  let { data } = $props();

  let work_order = $derived(
    appStore.work_orders.find((w) => w.id === data.id) as WorkOrder,
  );

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

  const parts_data = $derived(
    appStore.work_order_part_items.filter(
      (i) => i.work_order_id === work_order.id,
    ),
  );
</script>

{#if work_order}
  <PageLayout>
    {#snippet header()}
      <div class="h-full px-4 flex items-center justify-between">
        <h2 class="text-xl font-mono">
          {work_order.id}
        </h2>
        <div class="flex gap-2 items-center">
          <StatusSelector {work_order} />
          <DropdownMenu {work_order} />
        </div>
      </div>
    {/snippet}
    <div class="h-full w-full p-4">
      <div class="flex flex-row w-full items-stretch gap-4">
        <a href="/cars/{car.id}" class="flex flex-1">
          <CarCard
            {car}
            short
            class="w-full h-full flex flex-col justify-center"
          />
        </a>
        <a href="/clients/{client.id}" class="flex flex-1">
          <ClientCard
            {client}
            short
            class="w-full h-full flex flex-col justify-center"
          />
        </a>
      </div>

      <Details.Card class="mt-4">
        <Details.Item>
          <Details.Title>
            <Details.Icon icon={CalendarPlusIcon} /> Створено
          </Details.Title>
          <Details.Content>
            {prettifyIsoDateString(work_order.created_at, "long")}
          </Details.Content>
        </Details.Item>

        {#if work_order.started_at}
          <Details.Item>
            <Details.Title>
              <Details.Icon icon={CalendarPlusIcon} /> Почато
            </Details.Title>
            <Details.Content>
              {prettifyIsoDateString(work_order.started_at, "long")}
            </Details.Content>
          </Details.Item>
        {/if}

        {#if work_order.completed_at}
          <Details.Item>
            <Details.Title>
              <Details.Icon icon={CalendarPlusIcon} /> Завершено
            </Details.Title>
            <Details.Content>
              {prettifyIsoDateString(work_order.completed_at, "long")}
            </Details.Content>
          </Details.Item>
        {/if}

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={GaugeIcon} /> Одометр
          </Details.Title>
          <Details.Content>
            <div class="flex gap-2">
              <p>{work_order.odometer_in}</p>
              <p class="text-muted-foreground">км.</p>
            </div>
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={RateIcon} /> Тариф
          </Details.Title>
          <Details.Content>
            <CurrencyDisplay
              cents={work_order.rate}
              suffix="грн./год."
              suffix_muted
            />
          </Details.Content>
        </Details.Item>
      </Details.Card>

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

      <div class="mt-4">
        <DataTable
          columns={laborColumns}
          data={labor_data}
          emptyMessage="Сервісів не знайдено..."
          showPagination={false}
          headerClasses={{ name: "w-full" }}
        />
      </div>

      <div class="flex gap-4 my-4">
        <h2 class="text-xl font-semibold mr-auto">Запчастини</h2>
        <Button onclick={() => (isPartsSheetOpen = !isPartsSheetOpen)}
          >Створити</Button
        >
      </div>

      <div class="mt-4">
        <DataTable
          columns={partsColumns}
          data={parts_data}
          emptyMessage="Запчастин не знайдено..."
          showPagination={false}
          headerClasses={{ name: "w-full" }}
        />
      </div>

      <WorkOrderLaborItemMutationSheet
        bind:open={isLaborSheetOpen}
        work_order_id={work_order.id}
      />
      <WorkOrderPartItemMutationSheet
        bind:open={isPartsSheetOpen}
        work_order_id={work_order.id}
      />
    </div></PageLayout
  >
{/if}
