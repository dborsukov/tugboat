<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";

  import WorkOrderIcon from "@lucide/svelte/icons/briefcase-business";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import FingerprintIcon from "@lucide/svelte/icons/fingerprint-pattern";
  import HashIcon from "@lucide/svelte/icons/hash";
  import GotoIcon from "@lucide/svelte/icons/square-arrow-out-up-right";
  import UserIcon from "@lucide/svelte/icons/user";

  import { goto } from "$app/navigation";

  import { prettifyIsoDateString } from "$lib/misc";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card/index";
  import * as Details from "$lib/components/ui/details/index";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import CopyButton from "$lib/components/copy-button.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import ClientBadge from "../../clients/client-badge.svelte";
  import { clientsQueryOptions } from "../../clients/queries";
  import { columns } from "../../work_orders/columns";
  import {
    workOrderLaborItemsQueryOptions,
    workOrderPartItemsQueryOptions,
    workOrdersQueryOptions,
    workOrderTotalsQueryOptions,
  } from "../../work_orders/queries";
  import { buildWorkOrderListItems } from "../../work_orders/types";
  import WorkOrderMutationSheet from "../../work_orders/work-order-mutation-sheet.svelte";
  import { carBrandsQueryOptions, carsQueryOptions } from "../queries";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { params } = $props();
  let isSheetOpen: boolean = $state(false);

  const combinedQueries = createQueries(() => ({
    queries: [
      workOrdersQueryOptions(),
      carsQueryOptions(),
      clientsQueryOptions(),
      carBrandsQueryOptions(),
      workOrderLaborItemsQueryOptions(),
      workOrderPartItemsQueryOptions(),
      workOrderTotalsQueryOptions(),
    ],
    combine: (results) => {
      return {
        workOrders: results[0],
        cars: results[1],
        clients: results[2],
        carBrands: results[3],
        laborItems: results[4],
        partItems: results[5],
        totals: results[6],

        isReady: results.every((r) => r.isSuccess),
        isLoading: results.some((r) => r.isPending),
        isError: results.some((r) => r.isError),
        firstError: results.find((r) => r.isError)?.error,

        refetchAll: () => {
          results.forEach((r) => r.refetch());
        },
      };
    },
  }));

  let car = $derived(
    combinedQueries.cars.data?.find((c) => c.id.toString() === params.carId),
  );
  let brand = $derived(
    combinedQueries.carBrands.data?.find((c) => c.id === car?.brandId),
  );
  let owner = $derived(
    combinedQueries.clients.data?.find((c) => c.id === car?.clientId),
  );

  let workOrderListItems = $derived(
    combinedQueries.isReady
      ? buildWorkOrderListItems(
          combinedQueries.workOrders.data!,
          combinedQueries.cars.data!,
          combinedQueries.carBrands.data!,
          combinedQueries.clients.data!,
          combinedQueries.laborItems.data!,
          combinedQueries.partItems.data!,
          combinedQueries.totals.data!,
          (wo) => wo.carId === car!.id,
        )
      : [],
  );

  let table = buildTable({
    get data() {
      return workOrderListItems;
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

<PageLayout>
  {#snippet header()}
    {#if car}
      <div class="h-full flex items-center">
        <Button
          variant="outline"
          class="ml-auto mr-2"
          onclick={() => (isSheetOpen = !isSheetOpen)}
        >
          <WorkOrderIcon /> Нове замовлення
        </Button>
        <DropdownMenu carId={car.id} />
        <WorkOrderMutationSheet bind:open={isSheetOpen} carId={car.id} />
      </div>
    {/if}
  {/snippet}
  <div class="p-4">
    {#if combinedQueries.isError}
      <FetchErrorDisplay
        message={combinedQueries.firstError?.message}
        onRetry={combinedQueries.refetchAll}
        class="mt-20"
      />
    {:else if combinedQueries.isLoading}
      <Skeleton class="w-full h-25" />
      <Skeleton class="mt-4 w-full h-40" />
      <Skeleton class="mt-4 w-full h-40" />
    {:else if !car}
      <NotFound
        message={`Автомобіль не знайдено (ID: ${params.carId})`}
        class="mt-20"
      />
    {:else}
      <Card.Root>
        <Card.Content class="flex flex-row items-center justify-center gap-4">
          <div class="h-15">
            <LogoLoader slug={brand?.iconSlug} />
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
                <ClientBadge name={owner.name} />
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
            {#if car.licensePlate != undefined}
              <div class="flex items-center gap-2">
                <p
                  class="px-2 py-0.5 font-mono uppercase tracking-wider bg-muted rounded border"
                >
                  {car.licensePlate}
                </p>
                <CopyButton variant="ghost" text={car.licensePlate} />
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
            {prettifyIsoDateString(car.createdAt, "long")}
          </Details.Content>
        </Details.Item>
      </Details.Card>

      <Card.Root class="mt-4">
        <Card.Header>
          <Card.Title class="flex gap-2 text-xl">
            <p>Замовлення</p>
            <p class="text-muted-foreground">{workOrderListItems.length}</p>
          </Card.Title>
        </Card.Header>
        {#if workOrderListItems.length != 0}
          <Card.Content>
            <DataTable
              {table}
              emptyMessage="Замовлень не знайдено..."
              onRowClick={(row) => goto(`/work_orders/${row.id}`)}
            />
          </Card.Content>
        {/if}
      </Card.Root>
    {/if}
  </div>
</PageLayout>
