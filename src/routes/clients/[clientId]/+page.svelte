<script lang="ts">
  import { createQueries, useQueryClient } from "@tanstack/svelte-query";
  import Big from "big.js";

  import { goto } from "$app/navigation";

  import ArchiveIcon from "@lucide/svelte/icons/archive";
  import ArchiveRestoreIcon from "@lucide/svelte/icons/archive-restore";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CopyIcon from "@lucide/svelte/icons/copy";

  import * as Accordion from "$lib/components/ui/accordion";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import ArchivedBadge from "$lib/components/archived-badge.svelte";
  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import CurrencyDisplay from "$lib/components/currency-display.svelte";
  import DataTable from "$lib/components/data-table/data-table.svelte";
  import { buildTable } from "$lib/components/data-table/table-builder.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import LicensePlate from "$lib/components/license-plate.svelte";
  import NotFound from "$lib/components/not-found.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";
  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import { isUnknownBrand } from "$lib/brands";
  import { copyValue } from "$lib/clipboard";
  import { recordRecentItem } from "$lib/recent-items";
  import { pluralizeUk, prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  import CarMutationSheet from "../../cars/car-mutation-sheet.svelte";
  import { invalidateCars } from "../../cars/queries";
  import { columns, type WorkOrderRow } from "../../work_orders/columns";
  import {
    invalidateWorkOrderLists,
    workOrderDetailsListByClientIdQueryOptions,
  } from "../../work_orders/queries";
  import { buildWorkOrderClientColumn } from "../../work_orders/work-order-client-column";
  import WorkOrderMutationSheet from "../../work_orders/work-order-mutation-sheet.svelte";
  import { isAwaitingPayment } from "../../work_orders/work-order-status";
  import ClientAvatar from "../client-avatar.svelte";
  import {
    clientDetailsQueryOptions,
    createClientsQuery,
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
  let isWorkOrderCarPopoverOpen = $state(false);
  let selectedWorkOrderCarId = $state<number | null>(null);
  let isWorkOrderSheetOpen = $state(false);
  let orgDetailsOpen = $state<string | undefined>(undefined);
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

  $effect(() => {
    const id = clientDetails?.client?.id;
    if (id != null) void recordRecentItem("client", id);
  });

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
    hiddenColumns: ["client"],
    disableFiltering: true,
    disablePagination: true,
    initialSorting: [{ id: "createdAt", desc: true }],
  });

  const isIndividual = $derived(
    clientDetails?.client?.clientType === "individual",
  );
  const clientPhone = $derived(
    isIndividual
      ? clientDetails?.client?.phone
      : clientDetails?.details?.contactPhone,
  );
  const clientEmail = $derived(
    isIndividual
      ? clientDetails?.client?.email
      : clientDetails?.details?.contactEmail,
  );

  const ordersCount = $derived(workOrderListItems.length);
  const paidTotal = $derived(
    workOrderListItems
      .filter((details) => details.workOrder.status === "Paid")
      .reduce(
        (sum, details) => sum.plus(details.totals?.totalWithVat ?? "0"),
        new Big(0),
      ),
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
  const lastVisit = $derived(
    workOrderListItems.reduce<string | null>((latest, details) => {
      const createdAt = details.workOrder.createdAt;
      return !latest || new Date(createdAt) > new Date(latest)
        ? createdAt
        : latest;
    }, null),
  );

  const carOrderStats = $derived.by(() => {
    const map = new Map<number, { count: number; total: Big }>();
    for (const details of workOrderListItems) {
      const carId = details.carDto?.car?.id;
      if (carId == null) continue;
      const entry = map.get(carId) ?? { count: 0, total: new Big(0) };
      entry.count += 1;
      if (details.workOrder.status === "Paid") {
        entry.total = entry.total.plus(details.totals?.totalWithVat ?? "0");
      }
      map.set(carId, entry);
    }
    return map;
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

{#snippet copyCell(
  label: string,
  value: string | null | undefined,
  kind: "text" | "phone" = "text",
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
      {#if kind === "phone"}
        <PhoneDisplay {value} class="block w-full truncate" />
      {:else}
        <span class="truncate">{value}</span>
      {/if}
    </button>
  {:else}
    <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
      <span class="text-sm text-muted-foreground">{label}</span>
      <span class="text-muted-foreground/50">—</span>
    </div>
  {/if}
{/snippet}

{#snippet infoCell(
  label: string,
  value: string | null | undefined,
  wide: boolean = false,
)}
  <div
    class={cn(
      "flex min-w-0 flex-col gap-1 bg-card px-4 py-3",
      wide && "col-span-2 @max-[600px]:col-span-1",
    )}
  >
    <span class="text-sm text-muted-foreground">{label}</span>
    <span
      class="truncate {value ? '' : 'text-muted-foreground/50'}"
      title={value ?? undefined}>{value || "—"}</span
    >
  </div>
{/snippet}

<PageLayout>
  {#if combinedQueries.isError}
    <FetchErrorDisplay
      message={combinedQueries.firstError?.message}
      onRetry={combinedQueries.refetchAll}
      class="mt-20"
    />
  {:else if combinedQueries.isLoading}
    <div
      class="mt-4 overflow-hidden rounded border border-border bg-card @container"
    >
      <div class="border-b border-border p-4">
        <div class="flex flex-wrap items-center gap-4">
          <Skeleton class="size-16 rounded-full shrink-0" />
          <div class="flex min-w-0 flex-col gap-2">
            <Skeleton class="h-7 w-48" />
            <Skeleton class="h-4 w-40" />
          </div>
          <div
            class="ml-auto flex flex-wrap items-start gap-x-6 gap-y-3 @max-[900px]:hidden"
          >
            {#each { length: 4 } as _, i (i)}
              <div class="flex flex-col gap-1">
                <Skeleton class="h-4 w-20" />
                <Skeleton class="h-5 w-24" />
              </div>
            {/each}
          </div>
          <Skeleton class="size-8 rounded-md @max-[900px]:ml-auto" />
        </div>
      </div>
      <div
        class="hidden grid-cols-4 gap-px bg-border @max-[900px]:grid @max-[900px]:grid-cols-2 @max-[600px]:grid-cols-1"
      >
        {#each { length: 4 } as _, i (i)}
          <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
            <Skeleton class="h-4 w-20" />
            <Skeleton class="h-5 w-24" />
          </div>
        {/each}
      </div>
      <div class="@max-[900px]:border-t @max-[900px]:border-border">
        <div class="grid grid-cols-2 gap-px bg-border @max-[480px]:grid-cols-1">
          {#each { length: 2 } as _, i (i)}
            <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
              <Skeleton class="h-4 w-16" />
              <Skeleton class="h-5 w-40" />
            </div>
          {/each}
        </div>
      </div>
      <div class="border-t border-border px-4 py-3">
        <Skeleton class="h-4 w-2/3" />
      </div>
    </div>

    <div
      class="mt-4 overflow-hidden rounded border border-border bg-card @container"
    >
      <div
        class="flex min-h-12 items-center border-b border-border px-4 py-2 mb-0.5"
      >
        <Skeleton class="h-7 w-40" />
      </div>
      <div class="grid grid-cols-3 gap-px bg-border @max-[560px]:grid-cols-1">
        {#each { length: 3 } as _, i (i)}
          <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
            <Skeleton class="h-4 w-16" />
            <Skeleton class="h-5 w-40" />
          </div>
        {/each}
      </div>
    </div>

    <div
      class="mt-4 overflow-hidden rounded border border-border bg-card @container"
    >
      <div
        class="flex min-h-12 items-center border-b border-border px-4 py-2 mb-0.5"
      >
        <Skeleton class="h-7 w-44" />
      </div>
      <div
        class="-mx-4 -mb-4 grid grid-cols-3 gap-px bg-border @max-[600px]:grid-cols-1"
      >
        {#each { length: 9 } as _, i (i)}
          <div
            class={cn(
              "flex min-w-0 flex-col gap-1 bg-card px-4 py-3",
              (i === 0 || i === 2) && "col-span-2 @max-[600px]:col-span-1",
            )}
          >
            <Skeleton class="h-4 w-20" />
            <Skeleton class="h-5 w-40" />
          </div>
        {/each}
      </div>
    </div>

    <div
      class="mt-4 overflow-hidden rounded border border-border bg-card @container"
    >
      <div class="flex min-h-12 items-center gap-2 px-4 py-2 mb-0.5">
        <Skeleton class="h-7 w-28" />
        <Skeleton class="ml-auto h-5 w-20" />
      </div>
      <div class="border-t border-border">
        {#each { length: 2 } as _, i (i)}
          <div
            class="flex items-center gap-3 border-b border-border px-4 py-3 last:border-b-0"
          >
            <Skeleton class="size-8 rounded-full" />
            <Skeleton class="h-5 w-40" />
            <Skeleton class="ml-auto h-4 w-20" />
            <Skeleton class="h-4 w-24" />
            <Skeleton class="size-4 rounded-sm" />
          </div>
        {/each}
      </div>
    </div>

    <div class="mt-4 overflow-hidden rounded border border-border bg-card">
      <div class="flex min-h-12 items-center gap-2 px-4 py-2 mb-0.5">
        <Skeleton class="h-7 w-32" />
        <Skeleton class="ml-auto h-5 w-20" />
      </div>
      <div class="border-t border-border">
        <div class="h-10 border-b border-border flex items-center gap-4 px-4">
          <Skeleton class="h-4 w-1/2" />
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-16 ml-auto" />
          <Skeleton class="h-4 w-20" />
        </div>
        {#each { length: 3 } as _, i (i)}
          <div
            class="h-12 border-b border-border last:border-0 flex items-center gap-4 px-4"
          >
            <Skeleton class="h-4 w-1/2" />
            <Skeleton class="h-5 w-16 rounded-full" />
            <Skeleton class="h-4 w-16 ml-auto" />
            <Skeleton class="h-4 w-20" />
          </div>
        {/each}
      </div>
    </div>
  {:else if !clientDetails}
    <NotFound
      message={`Клієнта не знайдено (ID: ${params.clientId})`}
      class="mt-10"
    />
  {:else}
    <div
      class="sticky top-0 z-20 mt-4 overflow-hidden rounded-t border border-border bg-card @container"
    >
      {#if clientDetails?.client?.archivedAt}
        <div
          class="flex items-center justify-between gap-2 border-b border-border bg-muted/40 px-4 py-2 text-sm"
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
        <div class="flex flex-wrap items-center gap-4">
          <ClientAvatar
            name={clientDetails?.client?.name}
            class="size-16 shrink-0"
          />
          <div class="flex min-w-0 flex-col gap-1.5">
            <div class="flex flex-wrap items-center gap-2">
              <p class="text-xl font-semibold">
                {clientDetails?.client?.name}
              </p>
              {#if clientDetails?.client?.clientType !== "individual"}
                <Badge
                  variant="secondary"
                  class="h-6 rounded-2xl bg-muted px-2 text-xs font-medium text-muted-foreground"
                >
                  {clientDetails?.client?.clientType === "company"
                    ? "Організація"
                    : "ФОП"}
                </Badge>
              {/if}
            </div>
            <p class="text-base text-muted-foreground">
              Клієнт з {prettifyIsoDateString(
                clientDetails?.client?.createdAt,
                "long",
              )}
            </p>
          </div>

          <div
            class="ml-auto flex flex-wrap items-start gap-x-6 gap-y-3 @max-[900px]:hidden"
          >
            <div class="flex flex-col gap-0.5">
              <span class="text-sm text-muted-foreground">Замовлень</span>
              <span class="font-semibold tabular-nums">{ordersCount}</span>
            </div>
            <div class="flex flex-col gap-0.5">
              <span class="text-sm text-muted-foreground">Всього витрачено</span
              >
              <CurrencyDisplay
                amount={paidTotal.toString()}
                suffix="грн."
                class="font-semibold"
              />
            </div>
            {#if awaitingPaymentOrders.length > 0}
              <div class="flex flex-col gap-0.5">
                <span class="text-sm text-muted-foreground">Очікує оплати</span>
                <CurrencyDisplay
                  amount={awaitingPaymentTotal.toString()}
                  suffix="грн."
                  class="font-semibold text-primary"
                />
              </div>
            {/if}
            <div class="flex flex-col gap-0.5">
              <span class="text-sm text-muted-foreground">Останній візит</span>
              <span
                class="font-semibold tabular-nums {lastVisit
                  ? ''
                  : 'text-muted-foreground'}"
              >
                {lastVisit ? prettifyIsoDateString(lastVisit, "short") : "—"}
              </span>
            </div>
          </div>

          <div class="flex flex-col items-start @max-[900px]:ml-auto">
            <DropdownMenu
              clientId={clientDetails?.client?.id}
              isArchived={clientDetails?.client?.archivedAt != null}
              phone={clientPhone ?? null}
              email={clientEmail ?? null}
            />
          </div>
        </div>
      </div>

      <div
        class={cn(
          "hidden gap-px bg-border @max-[900px]:grid @max-[900px]:border-t @max-[900px]:border-border @max-[900px]:grid-cols-2 @max-[600px]:grid-cols-1",
          awaitingPaymentOrders.length > 0 ? "grid-cols-4" : "grid-cols-3",
        )}
      >
        <div class="flex flex-col gap-0.5 bg-card px-4 py-3">
          <span class="text-sm text-muted-foreground">Замовлень</span>
          <span class="font-semibold tabular-nums">{ordersCount}</span>
        </div>
        <div class="flex flex-col gap-0.5 bg-card px-4 py-3">
          <span class="text-sm text-muted-foreground">Всього витрачено</span>
          <CurrencyDisplay
            amount={paidTotal.toString()}
            suffix="грн."
            class="font-semibold"
          />
        </div>
        {#if awaitingPaymentOrders.length > 0}
          <div class="flex flex-col gap-0.5 bg-card px-4 py-3">
            <span class="text-sm text-muted-foreground">Очікує оплати</span>
            <CurrencyDisplay
              amount={awaitingPaymentTotal.toString()}
              suffix="грн."
              class="font-semibold text-primary"
            />
          </div>
        {/if}
        <div class="flex flex-col gap-0.5 bg-card px-4 py-3">
          <span class="text-sm text-muted-foreground">Останній візит</span>
          <span
            class="font-semibold tabular-nums {lastVisit
              ? ''
              : 'text-muted-foreground'}"
          >
            {lastVisit ? prettifyIsoDateString(lastVisit, "short") : "—"}
          </span>
        </div>
      </div>
    </div>

    <div
      class="overflow-hidden rounded-b border-x border-b border-border bg-card @container"
    >
      <div class="grid grid-cols-2 gap-px bg-border @max-[480px]:grid-cols-1">
        {@render copyCell("Телефон", clientPhone, "phone")}
        {@render copyCell("Email", clientEmail)}
      </div>

      {#if clientDetails?.client?.notes}
        <div
          class="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-t border-border px-4 py-2"
        >
          <span class="shrink-0 text-sm text-muted-foreground">Нотатки</span>
          <p
            class="min-w-0 text-sm italic text-foreground/90 whitespace-pre-wrap"
          >
            {clientDetails.client.notes}
          </p>
        </div>
      {/if}
    </div>

    {#if clientDetails?.details}
      <div
        class="mt-4 overflow-hidden rounded border border-border bg-card @container"
      >
        <div
          class="flex min-h-12 items-center border-b border-border px-4 py-2 mb-0.5"
        >
          <p class="text-xl font-medium">Контактна особа</p>
        </div>
        <div class="grid grid-cols-3 gap-px bg-border @max-[560px]:grid-cols-1">
          {@render infoCell(
            "ПІБ",
            composeName(
              clientDetails.details.contactSurname,
              clientDetails.details.contactFirstName,
              clientDetails.details.contactPatronymic,
            ),
          )}
          {@render copyCell(
            "Телефон",
            clientDetails.details.contactPhone,
            "phone",
          )}
          {@render copyCell("Email", clientDetails.details.contactEmail)}
        </div>
      </div>

      <div
        class="mt-4 overflow-hidden rounded border border-border bg-card @container"
      >
        <Accordion.Root
          type="single"
          bind:value={orgDetailsOpen}
          class="w-full rounded-none border-none"
        >
          <Accordion.Item
            value="requisites"
            class="border-none data-open:bg-transparent"
          >
            <Accordion.Trigger
              class="min-h-12 items-center border-0 p-0 px-4 py-2 text-xl font-medium aria-expanded:border-b aria-expanded:border-border hover:no-underline mb-0.5"
            >
              {clientDetails?.client?.clientType === "company"
                ? "Дані організації"
                : "Дані ФОП"}
            </Accordion.Trigger>
            <Accordion.Content>
              <div
                class="-mx-4 -mb-4 grid grid-cols-3 gap-px bg-border @max-[600px]:grid-cols-1"
              >
                {#if clientDetails?.client?.clientType === "company"}
                  {@render infoCell(
                    "Юридична назва",
                    clientDetails.details.legalName,
                    true,
                  )}
                  {@render copyCell("ЄДРПОУ", clientDetails.details.edrpou)}
                  {@render infoCell(
                    "Юридична адреса",
                    clientDetails.details.legalAddress,
                    true,
                  )}
                  {@render infoCell(
                    "Керівник",
                    composeName(
                      clientDetails.details.directorSurname,
                      clientDetails.details.directorFirstName,
                      clientDetails.details.directorPatronymic,
                    ),
                  )}
                  {@render copyCell("IBAN", clientDetails.details.iban)}
                  {@render infoCell("Банк", clientDetails.details.bankName)}
                {:else}
                  {@render copyCell("РНОКПП", clientDetails.details.rnokpp)}
                  {@render infoCell(
                    "Юридична адреса",
                    clientDetails.details.legalAddress,
                    true,
                  )}
                  {@render copyCell("IBAN", clientDetails.details.iban)}
                  {@render infoCell("Банк", clientDetails.details.bankName)}
                {/if}
                <div class="flex min-w-0 flex-col gap-1 bg-card px-4 py-3">
                  <span class="text-sm text-muted-foreground">Платник ПДВ</span>
                  <Badge
                    variant="secondary"
                    class="w-fit px-2 py-0.5 text-sm font-normal"
                  >
                    {clientDetails.details.vatPayer ? "Так" : "Ні"}
                  </Badge>
                </div>
              </div>
            </Accordion.Content>
          </Accordion.Item>
        </Accordion.Root>
      </div>
    {/if}

    <div
      class="mt-4 overflow-hidden rounded border border-border bg-card @container"
    >
      <div class="flex min-h-12 flex-wrap items-center gap-2 px-4 py-2 mb-0.5">
        <div class="flex items-baseline gap-2">
          <p class="text-xl font-medium">Автомобілі</p>
          <p class="text-muted-foreground">{clientDetails?.cars?.length}</p>
        </div>
        {#if !clientDetails?.client?.archivedAt}
          <Button
            variant="link"
            class="ml-auto pr-0 text-base"
            onclick={() => (isCarMutationSheetOpen = !isCarMutationSheetOpen)}
            >Додати +</Button
          >
        {/if}
        <CarMutationSheet
          initClientId={clientDetails?.client?.id}
          bind:open={isCarMutationSheetOpen}
        />
      </div>
      {#if clientDetails?.cars?.length}
        <div class="border-t border-border">
          {#each clientDetails.cars as cli (cli.car.id)}
            {@const stats = carOrderStats.get(cli.car.id)}
            <a
              href={`/cars/${cli.car.id}`}
              class={cn(
                "flex w-full items-center gap-3 border-b border-border bg-card px-4 py-3 text-left transition-colors last:border-b-0 hover:bg-muted/50 focus-visible:bg-muted/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset focus-visible:outline-none",
                cli.car?.archivedAt && "opacity-60",
              )}
            >
              <span class="size-8 shrink-0">
                <LogoLoader slug={cli.brand.iconSlug} name={cli.brand.name} />
              </span>
              <span class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
                <span class="truncate font-medium">
                  {isUnknownBrand(cli.brand) ? "" : cli.brand?.name + " "}{cli
                    .car?.model}
                </span>
                <Badge
                  variant="secondary"
                  class="h-5 rounded-2xl bg-muted px-2 text-xs font-medium text-muted-foreground"
                  >{cli.car?.year}</Badge
                >
                {#if cli.car?.licensePlate}
                  <LicensePlate value={cli.car.licensePlate} size="sm" />
                {/if}
                {#if cli.car?.archivedAt}
                  <ArchivedBadge date={cli.car.archivedAt} />
                {/if}
              </span>
              <span
                class="shrink-0 text-sm text-muted-foreground @max-[900px]:hidden"
              >
                {stats?.count ?? 0}
                {pluralizeUk(stats?.count ?? 0, [
                  "замовлення",
                  "замовлення",
                  "замовлень",
                ])}
              </span>
              <CurrencyDisplay
                amount={(stats?.total ?? new Big(0)).toString()}
                suffix="грн."
                class="shrink-0 font-semibold @max-[900px]:hidden"
              />
              <ChevronRightIcon class="size-4 shrink-0 text-muted-foreground" />
            </a>
          {/each}
        </div>
      {:else}
        <div
          class="border-t border-border p-4 text-center text-muted-foreground"
        >
          Автомобілів ще не додано
        </div>
      {/if}
    </div>

    <div class="mt-4 overflow-hidden rounded border border-border bg-card">
      <div class="flex min-h-12 flex-wrap items-center gap-2 px-4 py-2 mb-0.5">
        <div class="flex items-baseline gap-2">
          <p class="text-xl font-medium">Замовлення</p>
          <p class="text-muted-foreground">{workOrderListItems.length}</p>
        </div>
        {#if !clientDetails?.client?.archivedAt}
          <Popover.Root bind:open={isWorkOrderCarPopoverOpen}>
            <Popover.Trigger>
              {#snippet child({ props })}
                <Button
                  variant="link"
                  {...props}
                  class="ml-auto pr-0 text-base"
                  disabled={!clientDetails?.cars?.length}
                >
                  Створити +
                </Button>
              {/snippet}
            </Popover.Trigger>
            <Popover.Content class="w-72 p-2" align="end">
              <p class="px-2 py-1.5 text-sm text-muted-foreground">
                Оберіть автомобіль
              </p>
              {#each clientDetails?.cars ?? [] as cli (cli.car.id)}
                <Button
                  variant="ghost"
                  class="w-full justify-start"
                  onclick={() => {
                    selectedWorkOrderCarId = cli.car.id;
                    isWorkOrderSheetOpen = true;
                    isWorkOrderCarPopoverOpen = false;
                  }}
                >
                  {isUnknownBrand(cli.brand) ? "" : cli.brand?.name + " "}{cli
                    .car?.model}
                  {#if cli.car?.licensePlate}
                    <LicensePlate
                      value={cli.car.licensePlate}
                      size="sm"
                      class="ml-auto"
                    />
                  {/if}
                </Button>
              {/each}
            </Popover.Content>
          </Popover.Root>
        {/if}
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
        <div
          class="border-t border-border p-4 text-center text-muted-foreground"
        >
          Замовлень ще не було
        </div>
      {/if}
      {#if selectedWorkOrderCarId != null}
        <WorkOrderMutationSheet
          bind:open={isWorkOrderSheetOpen}
          mode="create"
          carId={selectedWorkOrderCarId}
        />
      {/if}
    </div>
  {/if}
</PageLayout>
