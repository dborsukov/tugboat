<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import DotIcon from "@lucide/svelte/icons/dot";

  import { Button } from "$lib/components/ui/button";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import LicensePlate from "$lib/components/license-plate.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { isUnknownBrand } from "$lib/brands";
  import { copyValue } from "$lib/clipboard";
  import { recordRecentItem } from "$lib/recent-items";
  import { prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  import { createClientsQuery, invalidateClients } from "../../clients/queries";
  import { columns, type WorkOrderRow } from "../../work_orders/columns";
  import {
    invalidateWorkOrderLists,
    workOrderDetailsListByCarIdQueryOptions,
  } from "../../work_orders/queries";
  import { buildWorkOrderClientColumn } from "../../work_orders/work-order-client-column";
  import WorkOrderMutationSheet from "../../work_orders/work-order-mutation-sheet.svelte";
  import { isAwaitingPayment } from "../../work_orders/work-order-status";
  import { driveTypeLabels, fuelTypeLabels } from "../enum-labels";
  import { carDetailsQueryOptions, createRestoreCarMutation } from "../queries";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { params } = $props();
  let isSheetOpen: boolean = $state(false);
  let carId = $derived(Number(params.carId));

  const queryClient = useQueryClient();
  const restoreCarMutation = createRestoreCarMutation(queryClient);

  const combinedQueries = createQueries(() => ({
    queries: [
      carDetailsQueryOptions(carId),
      workOrderDetailsListByCarIdQueryOptions(carId),
    ],
    combine: (results) => {
      return {
        carDetailsDto: results[0],
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

  let carDetails = $derived(combinedQueries.carDetailsDto.data);
  let workOrderListItems = $derived(
    combinedQueries.workOrderDetailsDtoList.data ?? [],
  );

  $effect(() => {
    const id = carDetails?.carDto?.car?.id;
    if (id != null) void recordRecentItem("car", id);
  });

  let ordersCount = $derived(workOrderListItems.length);
  let totalSpent = $derived(
    workOrderListItems
      .filter((details) => details.workOrder.status === "Paid")
      .reduce(
        (total, details) => total.plus(details.totals?.totalWithVat ?? "0"),
        new Big(0),
      ),
  );
  let lastVisit = $derived(
    workOrderListItems.reduce<string | null>((latest, details) => {
      const createdAt = details.workOrder.createdAt;
      return !latest || new Date(createdAt) > new Date(latest)
        ? createdAt
        : latest;
    }, null),
  );
  const awaitingPaymentOrders = $derived(
    workOrderListItems.filter((details) =>
      isAwaitingPayment(details.workOrder.status),
    ),
  );
  const awaitingPaymentTotal = $derived(
    awaitingPaymentOrders.reduce(
      (sum, details) => sum.plus(details.totals?.totalWithVat ?? "0"),
      new Big(0),
    ),
  );

  const clientsQuery = createClientsQuery();
  const clientsById = $derived(
    new Map((clientsQuery.data ?? []).map((client) => [client.id, client])),
  );

  function clientName(clientId: number) {
    return clientsById.get(clientId)?.name ?? null;
  }

  let tableRows = $derived.by<WorkOrderRow[]>(() =>
    workOrderListItems.map((details) => ({
      ...details,
      clientColumn: buildWorkOrderClientColumn(
        details,
        clientName,
        !clientsQuery.isPending,
      ),
    })),
  );

  let table = buildTable({
    get data() {
      return tableRows;
    },
    get columns() {
      return columns;
    },
    disableFiltering: true,
    disablePagination: true,
    initialSorting: [{ id: "createdAt", desc: true }],
    hiddenColumns: ["carId"],
  });

  function handleRestore() {
    restoreCarMutation.mutate(carId, {
      onSuccess: () => {
        invalidateClients(queryClient);
        invalidateWorkOrderLists(queryClient);
      },
    });
  }
</script>

<PageLayout>
  {#if combinedQueries.isError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-20"
    />
  {:else if combinedQueries.isLoading}
    <div
      class="mt-4 border border-border rounded bg-card @container overflow-hidden"
    >
      <!-- Header Skeleton -->
      <div class="px-4 pt-4 @5xl:border-b @5xl:border-border @5xl:p-4">
        <div
          class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-6 gap-y-4 @5xl:grid-cols-[minmax(0,1fr)_auto_auto]"
        >
          <div
            class="col-start-1 row-start-1 flex min-w-0 flex-row items-center gap-4"
          >
            <div class="h-16 w-16 shrink-0">
              <Skeleton class="h-full w-full rounded-md" />
            </div>
            <div class="flex min-w-0 flex-col gap-1.5">
              <Skeleton class="h-6 w-48" />
              <div class="flex flex-row items-center gap-2">
                <Skeleton class="h-4 w-24" />
                <Skeleton class="size-1 rounded-full" />
                <Skeleton class="h-4 w-32" />
              </div>
            </div>
          </div>
          <div class="col-start-2 row-start-1 @5xl:col-start-3">
            <Skeleton class="size-8 rounded-md" />
          </div>
          <div
            class="-mx-4 col-span-2 row-start-2 grid grid-cols-4 gap-px border-y border-border bg-border @max-[900px]:grid-cols-2 @max-[600px]:grid-cols-1 @5xl:mx-0 @5xl:col-span-1 @5xl:col-start-2 @5xl:row-start-1 @5xl:flex @5xl:gap-8 @5xl:border-y-0 @5xl:bg-transparent"
          >
            {#each Array(4) as _}
              <div class="flex flex-col gap-1 bg-card p-3 @5xl:p-0">
                <Skeleton class="h-4 w-20" />
                <Skeleton class="h-5 w-16" />
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Car Details Grid Skeleton -->
      <div
        class="grid grid-cols-6 gap-px bg-border @max-[700px]:grid-cols-3 @max-[671px]:grid-cols-2"
      >
        {#each Array(6) as _}
          <div class="min-w-0 bg-card px-4 py-3">
            <div class="flex flex-col gap-1">
              <Skeleton class="h-4 w-24" />
              <Skeleton class="h-5 w-16" />
            </div>
          </div>
        {/each}
      </div>

      <!-- Notes Skeleton -->
      <div
        class="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-t border-border px-4 py-2"
      >
        <Skeleton class="h-4 w-16" />
        <Skeleton class="h-4 min-w-0 flex-1" />
      </div>
    </div>

    <div class="mt-4 border border-border rounded bg-card">
      <!-- Work Orders Section Skeleton -->
      <div class="flex min-h-12 items-center px-4 py-2 mb-0.5">
        <div class="flex flex-1 flex-row items-center gap-2">
          <Skeleton class="h-7 w-28" />
          <Skeleton class="h-7 w-8" />
          <Skeleton class="ml-auto h-8 w-24" />
        </div>

        <!-- Table Skeleton -->
        <div class="border border-border rounded-md">
          <div class="h-10 border-b border-border flex items-center px-4 gap-4">
            <Skeleton class="h-4 w-1/2" />
            <Skeleton class="h-4 w-14" />
            <Skeleton class="h-4 w-14 ml-auto" />
            <Skeleton class="h-4 w-20" />
          </div>
          {#each { length: 3 } as _, i (i)}
            <div
              class="h-12 border-b border-border last:border-0 flex items-center px-4 gap-4"
            >
              <Skeleton class="h-4 w-2/3" />
              <Skeleton class="h-5 w-14 rounded-full" />
              <Skeleton class="h-4 w-14 ml-auto" />
              <Skeleton class="h-4 w-20" />
            </div>
          {/each}
        </div>
      </div>
    </div>
  {:else if !carDetails}
    <NotFound
      message={`Автомобіль не знайдено (ID: ${params.carId})`}
      class="mt-10"
    />
  {:else}
    {#snippet copyableDetailCell(
      label: string,
      value: string | null,
      mono: boolean = false,
      plate: boolean = false,
    )}
      {#if value}
        <button
          type="button"
          class="group flex w-full min-w-0 cursor-pointer flex-col gap-1 bg-card px-4 py-3 text-left transition-colors outline-none hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset active:bg-muted/70"
          title="Копіювати"
          onclick={() => copyValue(value, label)}
        >
          <span class="flex items-center gap-1.5 text-sm text-muted-foreground">
            {label}
            <CopyIcon
              class="size-3.5 text-muted-foreground/70 transition-colors group-hover:text-foreground"
              aria-hidden="true"
            />
          </span>
          {#if plate}
            <LicensePlate {value} plain class="font-semibold" />
          {:else}
            <span
              class="font-semibold {mono
                ? 'font-mono text-sm wrap-anywhere @max-[700px]:text-base'
                : ''}"
            >
              {value}
            </span>
          {/if}
        </button>
      {:else}
        <div class="min-w-0 bg-card px-4 py-3">
          <div class="flex flex-col gap-1">
            <span class="text-sm text-muted-foreground">{label}</span>
            <p class="text-muted-foreground">—</p>
          </div>
        </div>
      {/if}
    {/snippet}

    <div
      class="sticky top-0 z-20 mt-4 border border-border rounded-t bg-card @container overflow-hidden"
    >
      {#if carDetails?.carDto?.car?.archivedAt}
        <div
          class="flex items-center justify-between gap-2 rounded-t px-4 py-2 border-b border-border bg-muted/40 text-sm"
        >
          <span class="flex items-center gap-2 text-muted-foreground">
            <ArchiveIcon class="size-4" />
            Автомобіль архівований
            {prettifyIsoDateString(
              carDetails.carDto.car.archivedAt,
              "datetime",
            )}
          </span>
          <Button
            variant="link"
            class="p-0"
            onclick={handleRestore}
            disabled={restoreCarMutation.isPending}
          >
            <ArchiveRestoreIcon />
            Відновити
          </Button>
        </div>
      {/if}
      <div class="px-4 pt-4 @5xl:p-4">
        <div
          class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-6 gap-y-4 @5xl:grid-cols-[minmax(0,1fr)_auto_auto]"
        >
          <div
            class="col-start-1 row-start-1 flex min-w-0 flex-row items-center gap-4"
          >
            <div class="h-16 shrink-0">
              <LogoLoader
                slug={carDetails?.carDto?.brand?.iconSlug ?? ""}
                name={carDetails?.carDto?.brand?.name}
              />
            </div>
            <div class="flex min-w-0 flex-col gap-1.5">
              <p
                class="flex flex-wrap items-baseline gap-x-2 text-xl font-semibold"
              >
                <span>
                  {isUnknownBrand(carDetails?.carDto?.brand)
                    ? ""
                    : carDetails?.carDto?.brand?.name + " "}{carDetails?.carDto
                    ?.car?.model}
                </span>
                <span class="text-muted-foreground">
                  {carDetails?.carDto?.car?.year}
                </span>
              </p>
              <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
                <Button
                  size="sm"
                  variant="link"
                  class="h-auto p-0 text-base font-normal"
                  onclick={async () =>
                    await goto(`/clients/${carDetails?.client?.id}`)}
                  >{carDetails?.client?.name}</Button
                >
                <DotIcon
                  class="size-4 shrink-0 text-muted-foreground @max-lg:hidden"
                />
                <p class="text-base text-muted-foreground">
                  Додано {prettifyIsoDateString(
                    carDetails?.carDto?.car?.createdAt,
                    "long",
                  )}
                </p>
              </div>
            </div>
          </div>

          <div class="col-start-2 row-start-1 @5xl:col-start-3">
            <DropdownMenu
              carId={carDetails?.carDto?.car?.id}
              isArchived={carDetails?.carDto?.car?.archivedAt != null}
              vin={carDetails?.carDto?.car?.vin ?? null}
              licensePlate={carDetails?.carDto?.car?.licensePlate ?? null}
            />
          </div>

          <div
            class={cn(
              "-mx-4 col-span-2 row-start-2 grid gap-px border-t border-border bg-border @5xl:mx-0 @5xl:col-span-1 @5xl:col-start-2 @5xl:row-start-1 @5xl:flex @5xl:gap-8 @5xl:border-t-0 @5xl:bg-transparent",
              awaitingPaymentOrders.length > 0 ? "grid-cols-4" : "grid-cols-3",
              "@max-[900px]:grid-cols-2 @max-[600px]:grid-cols-1",
            )}
          >
            <div class="flex flex-col gap-0.5 bg-card p-3 @5xl:p-0">
              <span class="text-sm text-muted-foreground">Замовлень</span>
              <span class="text-base font-semibold">{ordersCount}</span>
            </div>
            <div class="flex flex-col gap-0.5 bg-card p-3 @5xl:p-0">
              <span class="text-sm text-muted-foreground">
                <span class="@2xl:hidden">Витрачено</span>
                <span class="hidden @2xl:inline">Всього витрачено</span>
              </span>
              <CurrencyDisplay
                amount={totalSpent.toString()}
                suffix="грн."
                class="text-base font-semibold"
              />
            </div>
            {#if awaitingPaymentOrders.length > 0}
              <div class="flex flex-col gap-0.5 bg-card p-3 @5xl:p-0">
                <span class="text-sm text-muted-foreground">Очікує оплати</span>
                <CurrencyDisplay
                  amount={awaitingPaymentTotal.toString()}
                  suffix="грн."
                  class="text-base font-semibold text-primary"
                />
              </div>
            {/if}
            <div class="flex flex-col gap-0.5 bg-card p-3 @5xl:p-0">
              <span class="text-sm text-muted-foreground">
                <span class="@2xl:hidden">Візит</span>
                <span class="hidden @2xl:inline">Останній візит</span>
              </span>
              <span
                class="text-base font-semibold {lastVisit
                  ? ''
                  : 'text-muted-foreground'}"
              >
                {lastVisit ? prettifyIsoDateString(lastVisit, "short") : "—"}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      class="overflow-hidden rounded-b border-x border-b border-border bg-card @container"
    >
      <div
        class="grid grid-cols-6 gap-px bg-border @max-[700px]:grid-cols-3 @max-[671px]:grid-cols-2"
      >
        {@render copyableDetailCell(
          "Код двигуна",
          carDetails.carDto.car.engineCode,
        )}

        <div class="min-w-0 bg-card px-4 py-3">
          <div class="flex flex-col gap-1">
            <span class="text-sm text-muted-foreground">Об'єм двигуна</span>
            {#if carDetails?.carDto?.car?.engineDisplacement}
              <p class="font-semibold">
                {carDetails.carDto.car.engineDisplacement} л.
              </p>
            {:else}
              <p class="text-muted-foreground">—</p>
            {/if}
          </div>
        </div>

        <div class="min-w-0 bg-card px-4 py-3">
          <div class="flex flex-col gap-1">
            <span class="text-sm text-muted-foreground">Тип палива</span>
            {#if carDetails?.carDto?.car?.fuelType}
              <p class="font-semibold">
                {fuelTypeLabels[carDetails.carDto.car.fuelType]}
              </p>
            {:else}
              <p class="text-muted-foreground">—</p>
            {/if}
          </div>
        </div>

        <div class="min-w-0 bg-card px-4 py-3">
          <div class="flex flex-col gap-1">
            <span class="text-sm text-muted-foreground">Тип приводу</span>
            {#if carDetails?.carDto?.car?.driveType}
              <p class="font-semibold">
                {driveTypeLabels[carDetails.carDto.car.driveType]}
              </p>
            {:else}
              <p class="text-muted-foreground">—</p>
            {/if}
          </div>
        </div>

        {@render copyableDetailCell(
          "Номерний знак",
          carDetails.carDto.car.licensePlate,
          false,
          true,
        )}

        {@render copyableDetailCell("VIN-код", carDetails.carDto.car.vin, true)}
      </div>

      {#if carDetails?.carDto?.car?.notes}
        <div
          class="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-t border-border px-4 py-2"
        >
          <span class="shrink-0 text-base text-muted-foreground">Нотатки</span>
          <p
            class="min-w-0 text-base italic text-foreground/90 whitespace-pre-wrap"
          >
            {carDetails.carDto.car.notes}
          </p>
        </div>
      {/if}
    </div>

    <div class="mt-4 border border-border rounded bg-card overflow-hidden">
      <div class="flex min-h-12 items-center px-4 py-2 mb-0.5">
        <div
          class="flex min-w-0 flex-1 flex-row items-center gap-2 text-xl font-medium"
        >
          <div class="flex items-baseline gap-2">
            <p>Замовлення</p>
            <p class="text-muted-foreground">
              {workOrderListItems.length}
            </p>
          </div>
          {#if !carDetails?.carDto?.car?.archivedAt}
            <Button
              variant="link"
              class="ml-auto pr-0 text-base"
              onclick={() => (isSheetOpen = !isSheetOpen)}
            >
              Створити +
            </Button>
          {/if}
          <WorkOrderMutationSheet
            bind:open={isSheetOpen}
            mode="create"
            carId={carDetails?.carDto?.car?.id}
          />
        </div>
      </div>
      {#if workOrderListItems.length != 0}
        <DataTable
          {table}
          variant="embedded"
          emptyMessage="Пусто..."
          onRowClick={(row) => goto(`/work_orders/${row.workOrder.id}`)}
          rowClassName={(row) =>
            row.workOrder.archivedAt ? "opacity-60" : undefined}
          headerClasses={{
            total: "text-right",
            createdAt: "text-right",
          }}
          cellClasses={{
            total: "text-right",
            createdAt: "text-right",
          }}
        />
      {:else}
        <p class="px-4 pb-4 text-muted-foreground">Замовлень ще не було</p>
      {/if}
    </div>
  {/if}
</PageLayout>
