<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import BarcodeIcon from "@lucide/svelte/icons/barcode";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import DotIcon from "@lucide/svelte/icons/dot";
  import FuelIcon from "@lucide/svelte/icons/fuel";
  import RectangleEllipsisIcon from "@lucide/svelte/icons/rectangle-ellipsis";
  import StickyNoteIcon from "@lucide/svelte/icons/sticky-note";
  import UserIcon from "@lucide/svelte/icons/user";
  import DriveTypeIcon from "~icons/tabler/car-4wd";
  import EngineIcon from "~icons/tabler/engine";
  import RulerIcon from "~icons/tabler/ruler-measure";

  import { Button } from "$lib/components/ui/button";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { prettifyIsoDateString } from "$lib/tools";

  import { invalidateClients } from "../../clients/queries";
  import { columns } from "../../work_orders/columns";
  import {
    invalidateWorkOrderLists,
    workOrderDetailsListByCarIdQueryOptions,
  } from "../../work_orders/queries";
  import WorkOrderMutationSheet from "../../work_orders/work-order-mutation-sheet.svelte";
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
    hiddenColumns: ["client", "car_id"],
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
    <div class="mt-4 border border-border rounded rounded-b-none bg-card">
      <!-- Header Skeleton -->
      <div class="p-4 border-b border-border">
        <div class="flex flex-row items-center gap-4">
          <div class="h-16 w-16 shrink-0">
            <Skeleton class="h-full w-full rounded-md" />
          </div>
          <div class="flex flex-col gap-1.5">
            <Skeleton class="h-6 w-48" />
            <div class="flex flex-row items-center gap-2">
              <Skeleton class="h-4 w-24" />
              <Skeleton class="size-1 rounded-full" />
              <Skeleton class="h-4 w-32" />
            </div>
          </div>
          <div class="h-full ml-auto flex flex-col items-start">
            <Skeleton class="size-8 rounded-md" />
          </div>
        </div>
      </div>

      <!-- Car Details Grid Skeleton -->
      <div class="w-full flex flex-wrap gap-px bg-border">
        {#each Array(6) as _}
          <div class="flex-[1_1_max-content] p-3 bg-card whitespace-nowrap">
            <div class="flex flex-col gap-1">
              <div class="flex flex-row gap-2 items-center">
                <Skeleton class="size-4 rounded-sm" />
                <Skeleton class="h-5 w-24" />
              </div>
              <div>
                <Skeleton class="h-5 w-16" />
              </div>
            </div>
          </div>
        {/each}
      </div>

      <!-- Notes Skeleton -->
      <div class="w-full flex flex-wrap border-t border-border bg-muted/40">
        <div class="flex-1 flex flex-col items-start gap-1.5 px-4 py-2">
          <div class="flex items-center gap-2">
            <Skeleton class="size-4 rounded-sm" />
            <Skeleton class="h-4 w-16" />
          </div>
          <Skeleton class="h-4 w-full max-w-md" />
        </div>
      </div>
    </div>

    <div class="mt-4 border border-border rounded rounded-t-none bg-card">
      <!-- Work Orders Section Skeleton -->
      <div class="p-4">
        <div class="flex flex-row items-center gap-2">
          <Skeleton class="h-7 w-28" />
          <Skeleton class="h-7 w-8" />
          <Skeleton class="ml-auto h-9 w-24" />
        </div>

        <!-- Table Skeleton -->
        <div class="mt-4 border border-border rounded-md">
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
    <div class="mt-4 border border-border rounded rounded-b-none bg-card">
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
      <div class="p-4 border-b border-border">
        <div class="flex flex-row items-center gap-4">
          <div class="h-16">
            <LogoLoader slug={carDetails?.carDto?.brand?.iconSlug} />
          </div>
          <div class="flex flex-col gap-1.5">
            <p class="text-xl font-semibold">
              {carDetails.carDto?.brand?.id != 0
                ? carDetails.carDto?.brand?.name + " "
                : ""}{carDetails?.carDto?.car?.model}
            </p>
            <div class="flex flex-row items-center">
              <UserIcon class="size-4 mr-2 text-muted-foreground" />
              <Button
                size="sm"
                variant="link"
                class="p-0 font-normal text-md"
                onclick={async () =>
                  await goto(`/clients/${carDetails?.client?.id}`)}
                >{carDetails?.client?.name}</Button
              >
              <DotIcon class="mx-1 size-4 text-muted-foreground" />
              <CalendarPlusIcon class="size-4 mr-2 text-muted-foreground" />
              <p>
                {prettifyIsoDateString(
                  carDetails?.carDto?.car?.createdAt,
                  "long",
                )}
              </p>
            </div>
          </div>
          <div class="h-full ml-auto flex flex-col items-start">
            <DropdownMenu
              carId={carDetails?.carDto?.car?.id}
              isArchived={carDetails?.carDto?.car?.archivedAt != null}
              vin={carDetails?.carDto?.car?.vin ?? null}
              licensePlate={carDetails?.carDto?.car?.licensePlate ?? null}
            />
          </div>
        </div>
      </div>

      <div class="w-full flex flex-wrap gap-px bg-border">
        <div class="flex-[1_1_max-content] p-3 bg-card whitespace-nowrap">
          <div class="flex flex-col gap-1">
            <div class="flex flex-row gap-2 items-center font-medium">
              <EngineIcon class="text-muted-foreground size-5" /> Код двигуна
            </div>
            <div>
              {#if carDetails?.carDto?.car?.engineCode}
                <p>{carDetails?.carDto?.car?.engineCode}</p>
              {:else}
                <p class="text-muted-foreground">Не вказано</p>
              {/if}
            </div>
          </div>
        </div>

        <div class="flex-[1_1_max-content] p-3 bg-card whitespace-nowrap">
          <div class="flex flex-col gap-1">
            <div class="flex flex-row gap-2 items-center font-medium">
              <RulerIcon class="text-muted-foreground size-4" /> Об'єм двигуна
            </div>
            <div>
              {#if carDetails?.carDto?.car?.engineDisplacement}
                <p>{carDetails?.carDto?.car?.engineDisplacement} л.</p>
              {:else}
                <p class="text-muted-foreground text-sm">Не вказано</p>
              {/if}
            </div>
          </div>
        </div>

        <div class="flex-[1_1_max-content] p-3 bg-card whitespace-nowrap">
          <div class="flex flex-col gap-1">
            <div class="flex flex-row gap-2 items-center font-medium">
              <FuelIcon class="text-muted-foreground size-4" /> Тип палива
            </div>
            <div>
              {#if carDetails?.carDto?.car?.fuelType}
                <p>{fuelTypeLabels[carDetails.carDto.car.fuelType]}</p>
              {:else}
                <p class="text-muted-foreground">Не вказано</p>
              {/if}
            </div>
          </div>
        </div>

        <div class="flex-[1_1_max-content] p-3 bg-card whitespace-nowrap">
          <div class="flex flex-col gap-1">
            <div class="flex flex-row gap-2 items-center font-medium">
              <DriveTypeIcon class="text-muted-foreground size-4" /> Тип приводу
            </div>
            <div>
              {#if carDetails?.carDto?.car?.driveType}
                <p>{driveTypeLabels[carDetails.carDto.car.driveType]}</p>
              {:else}
                <p class="text-muted-foreground">Не вказано</p>
              {/if}
            </div>
          </div>
        </div>

        <div class="flex-[1_1_max-content] p-3 bg-card whitespace-nowrap">
          <div class="flex flex-col gap-1">
            <div class="flex flex-row gap-2 items-center font-medium">
              <RectangleEllipsisIcon class="text-muted-foreground size-4" /> Номерний
              знак
            </div>
            <div>
              {#if carDetails?.carDto?.car?.licensePlate}
                <p>{carDetails?.carDto?.car?.licensePlate}</p>
              {:else}
                <p class="text-muted-foreground">Не вказано</p>
              {/if}
            </div>
          </div>
        </div>

        <div class="flex-[1_1_max-content] p-3 bg-card whitespace-nowrap">
          <div class="flex flex-col gap-1">
            <div class="flex flex-row gap-2 items-center font-medium">
              <BarcodeIcon class="text-muted-foreground size-4" /> VIN-код
            </div>
            <div>
              {#if carDetails?.carDto?.car?.vin}
                <p class="font-mono">{carDetails?.carDto?.car?.vin}</p>
              {:else}
                <p class="text-muted-foreground">Не вказано</p>
              {/if}
            </div>
          </div>
        </div>
      </div>

      {#if carDetails?.carDto?.car?.notes}
        <div class="w-full flex flex-wrap border-t border-border bg-muted/40">
          <div class="flex-1 flex flex-col items-start gap-1 px-4 py-2">
            <span
              class="flex items-center justify-center gap-2 text-base text-muted-foreground"
            >
              <StickyNoteIcon class="size-4" />
              <span>Нотатки</span>
            </span>
            <p class="text-base italic text-foreground/90 whitespace-pre-wrap">
              {carDetails.carDto.car.notes}
            </p>
          </div>
        </div>
      {/if}
    </div>

    <div class="mt-4 border border-border rounded rounded-t-none bg-card">
      <div class="p-4">
        <div class="flex flex-row gap-2 text-xl font-semibold">
          <p>Замовлення</p>
          <p class="text-muted-foreground">
            {workOrderListItems.length}
          </p>
          {#if !carDetails?.carDto?.car?.archivedAt}
            <Button
              variant="link"
              class="ml-auto text-base"
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
        {#if workOrderListItems.length != 0}
          <DataTable
            {table}
            class="mt-4"
            emptyMessage="Пусто..."
            onRowClick={(row) => goto(`/work_orders/${row.workOrder.id}`)}
            rowClassName={(row) =>
              row.workOrder.archivedAt ? "opacity-60" : undefined}
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
