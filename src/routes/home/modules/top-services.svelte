<script lang="ts">
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import { pluralizeUk } from "$lib/tools";

  import Money from "../components/money.svelte";
  import QueryError from "../components/query-error.svelte";
  import {
    dashboardHeightRows,
    dashboardWidthColumns,
  } from "../dashboard-grid";
  import { booleanSetting } from "../dashboard-layout";
  import { formatPercent, toNumber } from "../format";
  import { TOP_SERVICES_ID } from "../module-ids";
  import { createTopServicesQuery } from "../queries";
  import type { DashboardModuleProps, DashboardSlot } from "../registry";

  /** Module render sizes */
  type ContentSize = "small" | "large";

  /** Determines render size based on certain thresholds. */
  function contentSize(slot: DashboardSlot): ContentSize {
    const columns = dashboardWidthColumns[slot.width];
    const rows = dashboardHeightRows[slot.height];

    if (
      columns >= dashboardWidthColumns.half &&
      rows >= dashboardHeightRows.two
    )
      return "large";

    return "small";
  }

  /** How many services the list has room for. */
  function listRows(slot: DashboardSlot): number {
    return dashboardHeightRows[slot.height] >= dashboardHeightRows.three
      ? 9
      : 5;
  }

  let { slot }: DashboardModuleProps = $props();

  const query = createTopServicesQuery();

  const size = $derived(contentSize(slot));
  const rows = $derived(listRows(slot));

  const report = $derived(query.data ?? null);
  const services = $derived(report?.services ?? []);
  const totalRevenue = $derived(toNumber(report?.totalRevenue));
  const servicesRevenue = $derived(toNumber(report?.servicesRevenue));

  /** Whether the module splits full revenue or only what the services earned. */
  const acrossEverything = $derived(
    booleanSetting(TOP_SERVICES_ID, "splitAcrossRevenue"),
  );
  const whole = $derived(acrossEverything ? totalRevenue : servicesRevenue);
  const smallShareLabel = $derived(
    acrossEverything ? "від виручки" : "від виручки серед робіт",
  );
  const shareLabel = $derived(
    acrossEverything
      ? "Частка виручки"
      : "Частка виручки серед виконаних робіт",
  );

  /** Month's best earner. */
  const leader = $derived(services[0] ?? null);

  /** Nothing has been billed this month yet. */
  const empty = $derived(services.length === 0);

  /** The shades the services are drawn in, strongest first, one per row.*/
  const SHADES = [100, 78, 60, 46, 35, 26, 19, 14];

  /** What the row at `index` is drawn in, and its segment of the bar with it. */
  function shade(index: number): string {
    const step = SHADES[index] ?? SHADES[SHADES.length - 1];

    return `color-mix(in oklab, var(--chart-2) ${step}%, transparent)`;
  }

  /** What part of the whole a service is answerable for. */
  function shareOf(revenue: string | number): number {
    if (whole <= 0) return 0;

    return (toNumber(revenue) / whole) * 100;
  }

  function formatTimes(count: number): string {
    return `${count} ${pluralizeUk(count, ["раз", "рази", "разів"])}`;
  }

  const listed = $derived(
    services
      .slice(0, rows)
      .map((service, index) => ({ ...service, shade: shade(index) })),
  );
</script>

<Card class="h-full py-4">
  {#if query.isPending}
    {#if size === "small"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
        <Skeleton class="h-5 w-44" />
        <div class="mt-auto flex flex-col gap-2">
          <Skeleton class="h-6 w-40" />
          <Skeleton class="h-5 w-48" />
          <Skeleton class="h-1.5 w-full" />
        </div>
      </CardContent>
    {:else}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-2">
        <div class="flex flex-col gap-1">
          <Skeleton class="h-5 w-44" />
          <Skeleton class="h-4 w-32" />
          <Skeleton class="h-2 w-full" />
        </div>
        <div class="flex flex-col gap-1">
          <Skeleton class="h-4 w-full" />
          {#each Array(rows) as _, index (index)}
            <Skeleton class="h-7 w-full" />
          {/each}
        </div>
      </CardContent>
    {/if}
  {:else if query.error}
    <CardContent
      class="flex min-h-0 flex-1 flex-col items-center justify-center"
    >
      <QueryError error={query.error} onRetry={() => query.refetch()} />
    </CardContent>
  {:else if size === "small"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <p class="text-muted-foreground text-sm font-medium">
        Топова послуга цього місяця
      </p>

      {#if empty || !leader}
        <div class="mt-auto flex flex-col gap-2">
          <p class="text-muted-foreground/60 text-sm">
            Ще немає оплачених робіт
          </p>
          <div
            class="border-border h-1.5 w-full rounded-full border border-dashed"
          ></div>
        </div>
      {:else}
        <div class="mt-auto flex flex-col gap-2">
          <p class="truncate text-lg font-semibold" title={leader.name}>
            {leader.name}
          </p>
          <p class="text-muted-foreground flex items-center gap-1.5 text-sm">
            <Money
              amount={leader.revenue}
              precision={0}
              class="inline-flex gap-1"
            />
            <span aria-hidden="true">·</span>
            <span>{formatTimes(leader.count)}</span>
            <span aria-hidden="true">·</span>
            <span
              >{formatPercent(shareOf(leader.revenue))} {smallShareLabel}</span
            >
          </p>
          <div class="bg-muted h-1.5 w-full overflow-hidden rounded-full">
            <div
              class="bg-chart-2 h-full rounded-full"
              style="width: {shareOf(leader.revenue)}%"
            ></div>
          </div>
        </div>
      {/if}
    </CardContent>
  {:else}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-2">
      <div class="flex flex-col gap-1">
        <p class="text-muted-foreground text-sm font-medium">
          Топ послуг цього місяця
        </p>

        <p class="text-muted-foreground text-xs">{shareLabel}</p>
        {#if empty}
          <div
            class="border-border h-2 w-full rounded-full border border-dashed"
          ></div>
        {:else}
          <div
            class="bg-muted flex h-2 w-full gap-1 overflow-hidden rounded-full"
          >
            {#each listed as service (service.name)}
              <span
                class="h-full rounded-full"
                style="flex: 0 1 {shareOf(
                  service.revenue,
                )}%; background: {service.shade}"
                title="{service.name}: {formatPercent(
                  shareOf(service.revenue),
                )}"
              ></span>
            {/each}
          </div>
        {/if}
      </div>

      {#if empty}
        <p
          class="text-muted-foreground flex min-h-0 flex-1 items-center justify-center text-sm"
        >
          Ще немає оплачених робіт
        </p>
      {:else}
        <div class="flex min-h-0 flex-1 flex-col">
          <div
            class="text-muted-foreground flex items-end gap-2 border-b pb-1 text-xs leading-tight"
          >
            <span class="min-w-0 flex-1">Послуга</span>
            <span class="w-10 shrink-0 text-right">Разів</span>
            <span class="w-20 shrink-0 text-right">Сер. ціна</span>
            <span class="w-24 shrink-0 text-right">Виручка</span>
            <span class="w-20 shrink-0 text-right">До мин. міс.</span>
          </div>

          {#each listed as service (service.name)}
            <div
              class="flex items-center gap-2 border-b border-border/60 py-1.5 text-sm last:border-b-0"
            >
              <span class="flex min-w-0 flex-1 items-center gap-2">
                <span
                  class="size-2 shrink-0 rounded-xs"
                  style="background: {service.shade}"
                  aria-hidden="true"
                ></span>
                <span class="truncate" title={service.name}>{service.name}</span
                >
              </span>
              <span class="w-10 shrink-0 text-right tabular-nums">
                {service.count}
              </span>
              <span class="w-20 shrink-0 text-right">
                <Money
                  amount={service.averagePrice}
                  precision={0}
                  class="justify-end gap-1"
                />
              </span>
              <span class="w-24 shrink-0 text-right">
                <Money
                  amount={service.revenue}
                  precision={0}
                  class="justify-end gap-1"
                />
              </span>
              <span class="flex w-20 shrink-0 justify-end text-right">
                {#if service.change === null}
                  <span class="text-muted-foreground text-xs">нова</span>
                {:else if service.change === 0}
                  <span class="text-muted-foreground/60 tabular-nums">0</span>
                {:else if service.change > 0}
                  <span
                    class="text-emerald-600 tabular-nums dark:text-emerald-500"
                    >+{service.change}</span
                  >
                {:else}
                  <span class="text-destructive tabular-nums">
                    {service.change}
                  </span>
                {/if}
              </span>
            </div>
          {/each}
        </div>
      {/if}
    </CardContent>
  {/if}
</Card>
