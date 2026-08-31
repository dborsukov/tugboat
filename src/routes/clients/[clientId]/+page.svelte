<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import CalendarPlusIcon from "@lucide/svelte/icons/calendar-plus";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import DotIcon from "@lucide/svelte/icons/dot";
  import MailIcon from "@lucide/svelte/icons/mail";
  import PhoneIcon from "@lucide/svelte/icons/phone";
  import StickyNoteIcon from "@lucide/svelte/icons/sticky-note";

  import * as Accordion from "$lib/components/ui/accordion";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import ArchivedBadge from "$lib/components/archived-badge.svelte";
  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";
  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import { prettifyIsoDateString } from "$lib/tools";

  import CarMutationSheet from "../../cars/car-mutation-sheet.svelte";
  import { invalidateCars } from "../../cars/queries";
  import { columns } from "../../work_orders/columns";
  import {
    invalidateWorkOrderLists,
    workOrderDetailsListByClientIdQueryOptions,
  } from "../../work_orders/queries";
  import ClientAvatar from "../client-avatar.svelte";
  import {
    clientDetailsQueryOptions,
    createRestoreClientMutation,
  } from "../queries";
  import DropdownMenu from "./dropdown-menu.svelte";

  function composeName(
    surname?: string | null,
    firstName?: string | null,
    patronymic?: string | null,
  ): string {
    return [surname, firstName, patronymic]
      .filter((part) => part && part.trim() !== "")
      .join(" ");
  }

  let { params } = $props();
  let isCarMutationSheetOpen = $state(false);
  let clientId = $derived(Number(params.clientId));

  const queryClient = useQueryClient();
  const restoreClientMutation = createRestoreClientMutation(queryClient);

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

  function handleRestore() {
    restoreClientMutation.mutate(clientId, {
      onSuccess: () => {
        invalidateCars(queryClient);
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
    <div class="mt-4 border border-border rounded bg-card">
      <!-- Header Skeleton -->
      <div class="p-4">
        <div class="flex flex-row items-center gap-4">
          <Skeleton class="size-16 rounded-full shrink-0" />
          <div class="flex flex-col gap-2">
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

    <div class="mt-4 border border-border rounded bg-card">
      <!-- Cars Section Skeleton -->
      <div class="p-0 border-b border-border">
        <div class="px-4 pt-4 pb-3 flex flex-row items-center gap-2">
          <Skeleton class="h-7 w-28" />
          <Skeleton class="h-7 w-8" />
          <Skeleton class="ml-auto h-5 w-20" />
        </div>
        <div class="px-4 pb-4">
          <div
            class="border border-border rounded bg-card flex flex-col divide-y divide-border"
          >
            {#each Array(2) as _}
              <div class="h-10 w-full px-3 flex flex-row items-center gap-2.5">
                <Skeleton class="size-6 rounded-full shrink-0" />
                <Skeleton class="h-4 w-32" />
                <Skeleton class="h-4 w-14 ml-auto" />
                <Skeleton class="h-4 w-8" />
                <Skeleton class="size-4 rounded-sm shrink-0" />
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Work Orders Section Skeleton -->
      <div class="p-4">
        <div class="flex flex-row items-center gap-2">
          <Skeleton class="h-7 w-32" />
          <Skeleton class="h-7 w-8" />
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
  {:else if !clientDetails}
    <NotFound
      message={`Клієнта не знайдено (ID: ${params.clientId})`}
      class="mt-10"
    />
  {:else}
    <div class="mt-4 border border-border rounded bg-card">
      {#if clientDetails?.client?.archivedAt}
        <div
          class="flex items-center justify-between gap-2 rounded-t px-4 py-2 border-b border-border bg-muted/40 text-sm"
        >
          <span class="flex items-center gap-2 text-muted-foreground">
            <ArchiveIcon class="size-4" />
            Клієнт архівований
            {prettifyIsoDateString(clientDetails.client.archivedAt, "datetime")}
          </span>
          <Button
            variant="link"
            class="p-0"
            onclick={handleRestore}
            disabled={restoreClientMutation.isPending}
          >
            <ArchiveRestoreIcon />
            Відновити
          </Button>
        </div>
      {/if}
      <div class="p-4">
        <div class="flex flex-row items-center gap-4">
          <ClientAvatar name={clientDetails?.client?.name} class="size-16" />
          <div class="flex flex-col gap-2">
            <p class="text-xl font-semibold leading-none">
              {clientDetails?.client?.name}
            </p>

            <div class="flex flex-row items-center">
              {#if clientDetails?.client?.clientType === "individual"}
                {#if clientDetails?.client?.phone}
                  <PhoneIcon class="size-4 mr-2 text-muted-foreground" />
                  <PhoneDisplay
                    value={clientDetails?.client?.phone}
                    class="text-end select-none!"
                  />
                {/if}
                {#if clientDetails?.client?.email}
                  {#if clientDetails?.client?.phone}
                    <DotIcon class="mx-1 size-4 text-muted-foreground" />
                  {/if}
                  <MailIcon class="size-4 mr-2 text-muted-foreground" />
                  {clientDetails?.client?.email}
                {/if}
                {#if clientDetails?.client?.phone || clientDetails?.client?.email}
                  <DotIcon class="mx-1 size-4 text-muted-foreground" />
                {/if}
              {:else}
                <Badge
                  variant="outline"
                  class="px-2 py-2.5 text-sm font-normal"
                >
                  {clientDetails?.client?.clientType === "company"
                    ? "Організація"
                    : "ФОП"}
                </Badge>
                <DotIcon class="mx-1 size-4 text-muted-foreground" />
                {#if clientDetails?.details?.contactPhone}
                  <PhoneIcon class="size-4 mr-2 text-muted-foreground" />
                  <PhoneDisplay
                    value={clientDetails?.details?.contactPhone}
                    class="text-end select-none!"
                  />
                {/if}
                {#if clientDetails?.details?.contactEmail}
                  {#if clientDetails?.details?.contactPhone}
                    <DotIcon class="mx-1 size-4 text-muted-foreground" />
                  {/if}
                  <MailIcon class="size-4 mr-2 text-muted-foreground" />
                  {clientDetails?.details?.contactEmail}
                {/if}
                {#if clientDetails?.details?.contactPhone || clientDetails?.details?.contactEmail}
                  <DotIcon class="mx-1 size-4 text-muted-foreground" />
                {/if}
              {/if}
              <CalendarPlusIcon class="size-4 mr-2 text-muted-foreground" />
              {prettifyIsoDateString(clientDetails?.client?.createdAt, "long")}
            </div>
          </div>
          <div class="h-full ml-auto flex flex-col items-start">
            <DropdownMenu
              clientId={clientDetails?.client?.id}
              isArchived={clientDetails?.client?.archivedAt != null}
              phone={(clientDetails?.client?.clientType === "individual"
                ? clientDetails?.client?.phone
                : clientDetails?.details?.contactPhone) ?? null}
              email={(clientDetails?.client?.clientType === "individual"
                ? clientDetails?.client?.email
                : clientDetails?.details?.contactEmail) ?? null}
            />
          </div>
        </div>
      </div>

      {#if clientDetails?.client?.notes}
        <div class="w-full flex flex-wrap border-t border-border bg-muted/40">
          <div class="flex-1 flex flex-col items-start gap-1 px-4 py-2">
            <span
              class="flex items-center justify-center gap-2 text-base text-muted-foreground"
            >
              <StickyNoteIcon class="size-4" />
              <span>Нотатки</span>
            </span>
            <p class="text-base italic text-foreground/90 whitespace-pre-wrap">
              {clientDetails.client.notes}
            </p>
          </div>
        </div>
      {/if}
    </div>

    {#if clientDetails?.details}
      {#snippet detailRow(label: string, value: string | null | undefined)}
        <div class="flex flex-col gap-0.5 min-w-0">
          <span class="text-sm text-muted-foreground">{label}</span>
          <span class="truncate {value ? '' : 'text-muted-foreground/50'}">
            {value || "—"}
          </span>
        </div>
      {/snippet}

      <div class="mt-4 border border-border rounded bg-card">
        <!-- Contact person: always visible, same header style as the rest
             of the page. Sits outside the accordion below. -->
        <div class="p-4 border-b border-border">
          <p class="text-xl font-medium mb-3">Контактна особа</p>
          <div
            class="grid grid-cols-1 sm:grid-cols-3 gap-x-8 gap-y-3 text-base"
          >
            {@render detailRow(
              "ПІБ",
              composeName(
                clientDetails?.details?.contactSurname,
                clientDetails?.details?.contactFirstName,
                clientDetails?.details?.contactPatronymic,
              ),
            )}
            {@render detailRow("Телефон", clientDetails?.details?.contactPhone)}
            {@render detailRow("Email", clientDetails?.details?.contactEmail)}
          </div>
        </div>

        <Accordion.Root type="single" class="w-full border-none">
          <Accordion.Item value="requisites" class="border-none">
            <Accordion.Trigger
              class="p-4 text-xl font-medium items-center hover:no-underline"
            >
              {clientDetails?.client?.clientType === "company"
                ? "Дані організації"
                : "Дані ФОП"}
            </Accordion.Trigger>
            <Accordion.Content>
              <div
                class="grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-3 text-base"
              >
                {#if clientDetails?.client?.clientType === "company"}
                  {@render detailRow(
                    "Юридична назва",
                    clientDetails?.details?.legalName,
                  )}
                  {@render detailRow("ЄДРПОУ", clientDetails?.details?.edrpou)}
                  {@render detailRow(
                    "Юридична адреса",
                    clientDetails?.details?.legalAddress,
                  )}
                  {@render detailRow(
                    "Керівник",
                    composeName(
                      clientDetails?.details?.directorSurname,
                      clientDetails?.details?.directorFirstName,
                      clientDetails?.details?.directorPatronymic,
                    ),
                  )}
                  {@render detailRow("IBAN", clientDetails?.details?.iban)}
                  {@render detailRow("Банк", clientDetails?.details?.bankName)}
                  {@render detailRow(
                    "Платник ПДВ",
                    clientDetails?.details?.vatPayer ? "Так" : "Ні",
                  )}
                {:else}
                  {@render detailRow("РНОКПП", clientDetails?.details?.rnokpp)}
                  {@render detailRow(
                    "Юридична адреса",
                    clientDetails?.details?.legalAddress,
                  )}
                  {@render detailRow("IBAN", clientDetails?.details?.iban)}
                  {@render detailRow("Банк", clientDetails?.details?.bankName)}
                {/if}
              </div>
            </Accordion.Content>
          </Accordion.Item>
        </Accordion.Root>
      </div>
    {/if}

    <div class="mt-4 border border-border rounded bg-card">
      <div class="p-0 border-b border-border">
        <div
          class="px-4 pt-4 pb-3 flex flex-row gap-2 items-center text-xl font-medium"
        >
          <p>Автомобілі</p>
          <p class="text-muted-foreground">
            {clientDetails?.cars?.length}
          </p>
          {#if !clientDetails?.client?.archivedAt}
            <Button
              variant="link"
              class="ml-auto text-base"
              onclick={() => (isCarMutationSheetOpen = !isCarMutationSheetOpen)}
              >Додати +</Button
            >
          {/if}
          <CarMutationSheet
            initClientId={clientDetails?.client?.id}
            bind:open={isCarMutationSheetOpen}
          />
        </div>
        <div class="px-4 pb-4">
          <div
            class="border border-border rounded bg-card flex flex-col divide-y divide-border"
          >
            {#if clientDetails?.cars?.length != 0}
              {#each clientDetails?.cars as cli (cli.car.id)}
                <Button
                  variant="ghost"
                  class="w-full h-10 justify-start rounded-none px-3 shrink-0 {cli
                    .car?.archivedAt
                    ? 'opacity-60'
                    : ''}"
                  onclick={async () => await goto(`/cars/${cli.car?.id}`)}
                >
                  <div class="flex w-full items-center gap-1">
                    <LogoLoader slug={cli.brand.iconSlug} class="size-5 mr-2" />
                    <p
                      class="truncate text-base font-medium text-left flex-1 min-w-0"
                    >
                      {cli.brand?.id != 0 ? cli.brand?.name + " " : ""}{cli.car
                        ?.model}
                    </p>
                    {#if cli.car?.archivedAt}
                      <ArchivedBadge date={cli.car.archivedAt} />
                    {/if}
                    {#if cli.car?.licensePlate}
                      <span
                        class="shrink-0 text-muted-foreground font-mono text-sm"
                      >
                        {cli.car.licensePlate}
                      </span>
                      <DotIcon class="size-4 text-muted-foreground" />
                    {/if}
                    <span class="shrink-0 text-muted-foreground text-sm">
                      {cli.car?.year}
                    </span>
                    <ChevronRightIcon
                      class="size-4 shrink-0 text-muted-foreground"
                    />
                  </div>
                </Button>
              {/each}
            {:else}
              <div class="p-4 flex items-center justify-center">
                <p class="text-muted-foreground">Автомобілей ще не додано</p>
              </div>
            {/if}
          </div>
        </div>
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
