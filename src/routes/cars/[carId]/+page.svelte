<script lang="ts">
  import { createQueries } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import WorkOrderIcon from "@lucide/svelte/icons/briefcase-business";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import FingerprintIcon from "@lucide/svelte/icons/fingerprint-pattern";
  import FuelIcon from "@lucide/svelte/icons/fuel";
  import RectangleEllipsisIcon from "@lucide/svelte/icons/rectangle-ellipsis";
  import RulerIcon from "@lucide/svelte/icons/ruler";
  import GotoIcon from "@lucide/svelte/icons/square-arrow-out-up-right";
  import UserIcon from "@lucide/svelte/icons/user";
  import DriveTypeIcon from "~icons/tabler/car-4wd";
  import EngineIcon from "~icons/tabler/engine";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card/index";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import CopyButton from "$lib/components/copy-button.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import * as Details from "$lib/components/details/index";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { prettifyIsoDateString } from "$lib/tools";

  import ClientBadge from "../../clients/client-badge.svelte";
  import { columns } from "../../work_orders/columns";
  import { workOrderDetailsListByCarIdQueryOptions } from "../../work_orders/queries";
  import WorkOrderMutationSheet from "../../work_orders/work-order-mutation-sheet.svelte";
  import { driveTypeLabels, fuelTypeLabels } from "../enum-labels";
  import { carDetailsQueryOptions } from "../queries";
  import DropdownMenu from "./dropdown-menu.svelte";

  let { params } = $props();
  let isSheetOpen: boolean = $state(false);
  let carId = $derived(Number(params.carId));

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
    initialSorting: [{ id: "created_at", desc: true }],
    hiddenColumns: ["client", "car_id"],
  });
</script>

<PageLayout>
  {#snippet header()}
    {#if carDetails}
      <div class="h-full flex items-center">
        <Button
          variant="outline"
          class="ml-auto mr-2"
          onclick={() => (isSheetOpen = !isSheetOpen)}
        >
          <WorkOrderIcon /> Нове замовлення
        </Button>
        <DropdownMenu carId={carDetails?.carDto?.car?.id} />
        <WorkOrderMutationSheet
          bind:open={isSheetOpen}
          mode="create"
          carId={carDetails?.carDto?.car?.id}
        />
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
    {:else if !carDetails}
      <NotFound
        message={`Автомобіль не знайдено (ID: ${params.carId})`}
        class="mt-20"
      />
    {:else}
      <Card.Root>
        <Card.Content class="flex flex-row items-center justify-center gap-4">
          <div class="h-15">
            <LogoLoader slug={carDetails?.carDto?.brand?.iconSlug} />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xl font-semibold tracking-tight truncate">
              {#if carDetails?.carDto?.brand != undefined}
                {carDetails?.carDto?.brand?.name}
              {:else}
                ?
              {/if}
            </div>
            <div class="text-lg tracking-tight mt-0.5">
              {carDetails.carDto.car.model}
            </div>
          </div>
          <div class="flex flex-col gap-1 justify-between items-end">
            <Badge variant="outline" class="text-lg px-3 py-3.5 bg-background">
              {carDetails.carDto.car.year}
            </Badge>
            <p class="text-muted-foreground">ID: {carDetails.carDto.car.id}</p>
          </div>
        </Card.Content>
      </Card.Root>

      <Details.Card class="mt-4">
        <Details.Item>
          <Details.Title>
            <Details.Icon icon={UserIcon} /> Власник
          </Details.Title>
          <Details.Content>
            {#if carDetails.client != undefined}
              <div class="flex justify-end gap-2">
                <ClientBadge name={carDetails.client.name} />
                <Button
                  size="icon"
                  variant="ghost"
                  onclick={() => goto(`/clients/${carDetails.client.id}`)}
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
            <Details.Icon icon={EngineIcon} class="size-4.5" /> Код двигуна
          </Details.Title>
          <Details.Content>
            {#if carDetails.carDto.car.engineCode != undefined}
              <p class="font-mono">
                {carDetails.carDto.car.engineCode}
              </p>
            {:else}
              Не вказано
            {/if}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={RulerIcon} /> Об'єм двигуна
          </Details.Title>
          <Details.Content>
            {#if carDetails.carDto.car.engineDisplacement != undefined}
              <p class="font-mono">
                {carDetails.carDto.car.engineDisplacement} л.
              </p>
            {:else}
              Не вказано
            {/if}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={FuelIcon} /> Тип палива
          </Details.Title>
          <Details.Content>
            {#if carDetails.carDto.car.fuelType != undefined}
              {fuelTypeLabels[carDetails.carDto.car.fuelType]}
            {:else}
              Не вказано
            {/if}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={DriveTypeIcon} class="size-5" /> Тип приводу
          </Details.Title>
          <Details.Content>
            {#if carDetails.carDto.car.driveType != undefined}
              {driveTypeLabels[carDetails.carDto.car.driveType]}
            {:else}
              Не вказано
            {/if}
          </Details.Content>
        </Details.Item>

        <Details.Item>
          <Details.Title>
            <Details.Icon icon={RectangleEllipsisIcon} /> Номерний знак
          </Details.Title>
          <Details.Content>
            {#if carDetails.carDto.car.licensePlate != undefined}
              <div class="flex items-center gap-2">
                <p
                  class="px-2 py-0.5 font-mono uppercase tracking-wider bg-muted rounded border"
                >
                  {carDetails.carDto.car.licensePlate}
                </p>
                <CopyButton
                  variant="ghost"
                  text={carDetails.carDto.car.licensePlate ?? ""}
                />
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
            {#if carDetails.carDto.car.vin != undefined}
              <div class="flex items-center gap-2">
                <p
                  class="px-2 py-0.5 font-mono uppercase tracking-wider bg-muted rounded border"
                >
                  {carDetails.carDto.car.vin}
                </p>
                <CopyButton
                  text={carDetails.carDto.car.vin ?? ""}
                  variant="ghost"
                />
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
            {prettifyIsoDateString(carDetails.carDto.car.createdAt, "long")}
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
              onRowClick={(row) => goto(`/work_orders/${row.workOrder.id}`)}
            />
          </Card.Content>
        {/if}
      </Card.Root>
    {/if}
  </div>
</PageLayout>
