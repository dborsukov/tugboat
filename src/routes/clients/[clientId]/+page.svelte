<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";
  import { MaskPatterns, PatternText } from "svelte-number-format";

  import { goto } from "$app/navigation";

  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import DotIcon from "@lucide/svelte/icons/dot";
  import PhoneIcon from "@lucide/svelte/icons/phone";

  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { prettifyIsoDateString } from "$lib/tools";

  import CarMutationSheet from "../../cars/car-mutation-sheet.svelte";
  import { columns } from "../../work_orders/columns";
  import { workOrderDetailsListByClientIdQueryOptions } from "../../work_orders/queries";
  import ClientAvatar from "../client-avatar.svelte";
  import { clientDetailsQueryOptions } from "../queries";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { params } = $props();
  let isCarMutationSheetOpen = $state(false);
  let clientId = $derived(Number(params.clientId));

  const combinedQueries = createQueries(() => ({
    queries: [
      clientDetailsQueryOptions(clientId),
      workOrderDetailsListByClientIdQueryOptions(clientId),
    ],
    combine: (results) => {
      return {
        clientDetails: results[0],
        workOrderDetailsDtoList: results[1],

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

  let clientDetails = $derived(combinedQueries.clientDetails.data);
  let workOrderListItems = $derived(
    combinedQueries.workOrderDetailsDtoList.data ?? [],
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
    initialSorting: [{ id: "createdAt", desc: true }],
    hiddenColumns: ["client"],
  });
</script>

<PageLayout>
  {#if combinedQueries.isError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-20"
    />
  {:else if combinedQueries.isLoading}
    <div class="mt-4 border border-border rounded bg-card">
      <!-- Header Skeleton -->
      <div class="p-4 border-b border-border">
        <div class="flex flex-row items-center gap-4">
          <Skeleton class="size-16 rounded-full shrink-0" />
          <div class="flex flex-col gap-2 w-full">
            <Skeleton class="h-7 w-48" />
            <div class="flex flex-row items-center gap-2">
              <Skeleton class="h-4 w-32" />
              <Skeleton class="size-1 rounded-full shrink-0" />
              <Skeleton class="h-4 w-32" />
            </div>
          </div>
          <div class="h-full ml-auto flex flex-col items-start">
            <Skeleton class="size-8 rounded-md" />
          </div>
        </div>
      </div>

      <!-- Cars Section Skeleton -->
      <div class="p-4 border-b border-border">
        <div class="flex flex-row items-center gap-2 pb-2">
          <Skeleton class="h-7 w-28" />
          <Skeleton class="h-7 w-8" />
          <Skeleton class="ml-auto h-9 w-24" />
        </div>
        <div class="flex flex-col items-start mt-2">
          {#each Array(2) as _}
            <div class="h-12 w-full px-4 flex flex-row items-center gap-4">
              <Skeleton class="h-8 w-12 rounded-sm" />
              <Skeleton class="h-5 w-40" />
              <Skeleton class="h-5 w-12 ml-auto" />
              <Skeleton class="size-5 rounded-sm shrink-0" />
            </div>
          {/each}
        </div>
      </div>

      <!-- Work Orders Section Skeleton -->
      <div class="p-4">
        <div class="flex flex-row items-center gap-2 mb-4">
          <Skeleton class="h-7 w-32" />
          <Skeleton class="h-7 w-8" />
        </div>

        <!-- Table Skeleton -->
        <div class="mt-4 flex flex-col gap-2">
          <Skeleton class="h-10 w-full rounded-md" />
          <Skeleton class="h-12 w-full rounded-md" />
          <Skeleton class="h-12 w-full rounded-md" />
        </div>
      </div>
    </div>
  {:else if !clientDetails}
    <NotFound
      message={`Клієнта не знайдено (ID: ${params.clientId})`}
      class="mt-10"
    />
  {:else}
    <div class="mt-4 border border-border rounded bg-card">
      <div class="p-4 border-b border-border">
        <div class="flex flex-row items-center gap-4">
          <ClientAvatar name={clientDetails?.client?.name} class="size-16" />
          <div class="flex flex-col gap-2">
            <p class="text-xl font-semibold">{clientDetails?.client?.name}</p>
            <div class="flex flex-row items-center">
              <PhoneIcon class="size-4 mr-2 text-muted-foreground" />
              <PatternText
                value={clientDetails?.client?.phone}
                format={MaskPatterns.PHONE_INTERNATIONAL}
                class="text-end select-none!"
              />
              <DotIcon class="mx-1 size-4 text-muted-foreground" />
              <CalendarPlusIcon class="size-4 mr-2 text-muted-foreground" />
              {prettifyIsoDateString(clientDetails?.client?.createdAt, "long")}
            </div>
          </div>
          <div class="h-full ml-auto flex flex-col items-start">
            <DropdownMenu clientId={clientDetails?.client?.id} />
          </div>
        </div>
      </div>

      <div class="p-4 border-b border-border">
        <div class="flex flex-row gap-2 items-center text-xl font-medium pb-2">
          <p>Автомобілі</p>
          <p class="text-muted-foreground">
            {clientDetails?.cars?.length}
          </p>
          <Button
            variant="link"
            class="ml-auto text-base"
            onclick={() => (isCarMutationSheetOpen = !isCarMutationSheetOpen)}
            >Додати +</Button
          >
          <CarMutationSheet
            initClientId={clientDetails?.client?.id}
            bind:open={isCarMutationSheetOpen}
          />
        </div>
        {#if clientDetails?.cars?.length != 0}
          <div class="flex flex-col items-start">
            {#each clientDetails?.cars as cli (cli.car.id)}
              <div class="h-12 w-full">
                <Button
                  variant="ghost"
                  class="h-12 w-full justify-start gap-4"
                  onclick={async () => await goto(`/cars/${cli.car?.id}`)}
                >
                  <LogoLoader slug={cli.brand.iconSlug} class="h-3/4" />
                  <p class="text-base font-normal">
                    {cli.brand?.id != 0 ? cli.brand?.name + " " : ""}{cli.car
                      ?.model}
                  </p>
                  <p class="text-muted-foreground ml-auto">{cli.car?.year}</p>
                  <ChevronRightIcon class="text-muted-foreground" />
                </Button>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-muted-foreground">Автомобілей ще не додано</p>
        {/if}
      </div>

      <div class="p-4">
        <div class="flex flex-row gap-2 text-xl font-medium">
          <p>Замовлення</p>
          <p class="text-muted-foreground">
            {workOrderListItems.length}
          </p>
        </div>
        {#if workOrderListItems.length != 0}
          <DataTable
            {table}
            class="mt-4"
            emptyMessage="Пусто..."
            onRowClick={(row) => goto(`/work_orders/${row.workOrder.id}`)}
            headerClasses={{
              total: "px-0 text-right",
              createdAt: "px-0 text-right",
            }}
            cellClasses={{
              status: "p-0",
              total: "p-0",
              createdAt: "px-2 text-right",
            }}
          />
        {:else}
          <p class="text-muted-foreground">Замовлень ще не було</p>
        {/if}
      </div>
    </div>
  {/if}
</PageLayout>
