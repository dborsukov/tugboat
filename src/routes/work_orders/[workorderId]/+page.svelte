<script lang="ts">
  import { NumericText } from "svelte-number-format";
  import { toast } from "svelte-sonner";

  import DiscountIcon from "@lucide/svelte/icons/badge-percent";
  import CalendarCheckIcon from "@lucide/svelte/icons/calendar-check";
  import CalendarClockIcon from "@lucide/svelte/icons/calendar-clock";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import GaugeIcon from "@lucide/svelte/icons/gauge";
  import RateIcon from "@lucide/svelte/icons/hand-coins";
  import PackageIcon from "@lucide/svelte/icons/package";
  import ReceiptTextIcon from "@lucide/svelte/icons/receipt-text";
  import WrenchIcon from "@lucide/svelte/icons/wrench";

  import { commands, type WorkOrder } from "$lib/bindings";

  import { appStore } from "$lib/stores/app.svelte";

  import { prettifyIsoDateString } from "$lib/misc";

  import * as ButtonGroup from "$lib/components/ui/button-group/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card/index";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Details from "$lib/components/ui/details/index";
  import * as Popover from "$lib/components/ui/popover/index.js";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
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

  let workOrder = $derived(
    appStore.workOrders.find((w) => w.id === data.id) as WorkOrder,
  );

  let isLaborSheetOpen: boolean = $state(false);
  let isPartsSheetOpen: boolean = $state(false);
  let isPopoverOpen: boolean = $state(false);

  const car = appStore.cars.find((c) => c.id === workOrder.carId);
  const carBrand = appStore.carBrands.find((c) => c.id === car?.brandId);
  const client = appStore.clients.find((c) => c.id === car?.clientId);
  if (car === undefined || client === undefined || carBrand === undefined) {
    throw new Error("Elements missing from appStore");
  }

  const laborData = $derived(
    appStore.workOrderLaborItems
      .filter((i) => i.workOrderId === workOrder.id)
      .map((item) => {
        return {
          ...item,
          rate: workOrder.rateCents,
        };
      }),
  );

  let laborTable = buildTable({
    get data() {
      return laborData;
    },
    get columns() {
      return laborColumns;
    },
    disableSorting: true,
    disableFiltering: true,
    disablePagination: true,
  });

  const partsData = $derived(
    appStore.workOrderPartItems.filter((i) => i.workOrderId === workOrder.id),
  );

  let partsTable = buildTable({
    get data() {
      return partsData;
    },
    get columns() {
      return partsColumns;
    },
    disableSorting: true,
    disableFiltering: true,
    disablePagination: true,
  });

  const laborTotalCents = $derived(
    laborData.reduce(
      (sum, item) => sum + Math.round(item.laborHours * item.rate),
      0,
    ),
  );

  const partsTotalCents = $derived(
    partsData.reduce(
      (sum, item) =>
        sum +
        Math.round(item.quantity * (item.costCents * (1 + item.markup / 100))),
      0,
    ),
  );

  const grandTotalCents = $derived.by(() => {
    let flatTotal = laborTotalCents + partsTotalCents;
    if (workOrder.discountCents != 0 || workOrder.discountPercent != 0) {
      if (workOrder.discountType === "Fixed") {
        return flatTotal - workOrder.discountCents;
      } else {
        return flatTotal * (1 - workOrder.discountPercent / 100);
      }
    }
    return flatTotal;
  });
</script>

{#if workOrder}
  <PageLayout>
    {#snippet header()}
      <div class="ml-2 h-full flex items-center gap-4">
        <div class="hidden md:block h-9/12 w-px bg-border shrink-0"></div>

        <div class="flex flex-col justify-center leading-tight shrink-0">
          <span class="text-xs text-muted-foreground">Наряд-замовлення</span>
          <h2 class="text-lg font-mono font-medium">{workOrder.id}</h2>
        </div>

        <div class="hidden md:block h-9/12 w-px bg-border shrink-0"></div>

        <div class="h-full flex items-center gap-4 flex-1 min-w-0">
          <div class="hidden md:flex items-center gap-2">
            <WrenchIcon class="size-5 text-muted-foreground" />
            <div class="flex flex-col leading-tight">
              <span class="text-xs text-muted-foreground">Роботи</span>
              <CurrencyDisplay cents={laborTotalCents} suffix="грн." />
            </div>
          </div>

          <div class="hidden md:flex items-center gap-2">
            <PackageIcon class="size-5 text-muted-foreground" />
            <div class="flex flex-col leading-tight">
              <span class="text-xs text-muted-foreground">Запчастини</span>
              <CurrencyDisplay cents={partsTotalCents} suffix="грн." />
            </div>
          </div>

          {#if workOrder.discountCents != 0 || workOrder.discountPercent != 0}
            <div class="h-9/12 w-px bg-border shrink-0"></div>

            <div
              class="hidden md:flex items-center gap-2 rounded-md px-2 py-1 bg-linear-to-r from-amber-400/25 dark:from-amber-700/35 to-transparent"
            >
              <DiscountIcon class="size-5 text-amber-700 dark:text-amber-200" />
              <div class="flex flex-col leading-tight">
                <span class="text-xs text-amber-700 dark:text-amber-200"
                  >Знижка</span
                >
                {#if workOrder.discountType === "Fixed"}
                  <CurrencyDisplay
                    cents={workOrder.discountCents}
                    suffix="грн."
                  />
                {:else}
                  {workOrder.discountPercent}%
                {/if}
              </div>
            </div>
          {/if}

          <div class="h-9/12 w-px bg-border shrink-0"></div>

          <div
            class="flex items-center gap-2 rounded-md px-2 py-1 bg-linear-to-r from-emerald-400/25 dark:from-emerald-700/35 to-transparent"
          >
            <ReceiptTextIcon
              class="size-5 text-emerald-700 dark:text-emerald-200"
            />
            <div class="flex flex-col leading-tight">
              <span class="text-xs text-emerald-700 dark:text-emerald-200"
                >Разом</span
              >
              <CurrencyDisplay
                cents={grandTotalCents}
                class="font-semibold"
                suffix="грн."
              />
            </div>
          </div>
        </div>

        <div class="flex gap-2 items-center">
          <StatusSelector work_order={workOrder} />
          <DropdownMenu work_order={workOrder} />
        </div>
      </div>
    {/snippet}
    <div class="h-full w-full p-4">
      <div class="flex flex-col sm:flex-row w-full items-stretch gap-4">
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
            <Details.Icon icon={GaugeIcon} /> Одометр
          </Details.Title>
          <Details.Content>
            <div class="flex gap-2">
              <NumericText
                value={workOrder.odometerIn}
                class="text-end select-none!"
              />
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
              cents={workOrder.rateCents}
              suffix="грн./год."
              suffix_muted
            />
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={CalendarPlusIcon} /> Створено
          </Details.Title>
          <Details.Content>
            {prettifyIsoDateString(workOrder.createdAt, "datetime")}
          </Details.Content>
        </Details.Item>

        {#if workOrder.startedAt}
          <Details.Item>
            <Details.Title>
              <Details.Icon icon={CalendarClockIcon} /> Почато
            </Details.Title>
            <Details.Content>
              {prettifyIsoDateString(workOrder.startedAt, "datetime")}
            </Details.Content>
          </Details.Item>
        {/if}

        {#if workOrder.completedAt}
          <Details.Item>
            <Details.Title>
              <Details.Icon icon={CalendarCheckIcon} /> Завершено
            </Details.Title>
            <Details.Content>
              {prettifyIsoDateString(workOrder.completedAt, "datetime")}
            </Details.Content>
          </Details.Item>
        {/if}
      </Details.Card>

      <Card.Root class="mt-4">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Сервіси</p>
            <p class="text-muted-foreground">{laborData.length}</p>
          </Card.Title>
          <Card.Action>
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
                              let res = await commands.createWorkOrderLaborItem(
                                {
                                  workOrderId: workOrder.id,
                                  serviceId: service.id,
                                  code: service.code,
                                  name: service.name,
                                  laborHours: service.laborHours,
                                },
                              );
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
          </Card.Action>
        </Card.Header>
        <Card.Content>
          <DataTable
            table={laborTable}
            emptyMessage="Сервісів не знайдено..."
            headerClasses={{ name: "w-full" }}
          />
        </Card.Content>
      </Card.Root>

      <Card.Root class="mt-4">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Запчастини</p>
            <p class="text-muted-foreground">{partsData.length}</p>
          </Card.Title>
          <Card.Action>
            <Button onclick={() => (isPartsSheetOpen = !isPartsSheetOpen)}
              >Створити</Button
            >
          </Card.Action>
        </Card.Header>
        <Card.Content>
          <DataTable
            table={partsTable}
            emptyMessage="Запчастин не знайдено..."
            headerClasses={{ name: "w-full" }}
          />
        </Card.Content>
      </Card.Root>

      <WorkOrderLaborItemMutationSheet
        bind:open={isLaborSheetOpen}
        work_order_id={workOrder.id}
      />
      <WorkOrderPartItemMutationSheet
        bind:open={isPartsSheetOpen}
        work_order_id={workOrder.id}
      />
    </div></PageLayout
  >
{/if}
