<script lang="ts">
  import { Bar, Chart, Labels } from "layerchart";

  import InboxIcon from "@lucide/svelte/icons/inbox";

  import { Card, CardContent } from "$lib/components/ui/card";
  import { ChartContainer, type ChartConfig } from "$lib/components/ui/chart";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import { cn } from "$lib/utils";

  import Money from "../components/money.svelte";
  import QueryError from "../components/query-error.svelte";
  import TrendLine from "../components/trend-line.svelte";
  import {
    dashboardHeightRows,
    dashboardWidthColumns,
  } from "../dashboard-grid";
  import { booleanSetting } from "../dashboard-layout";
  import {
    formatCurrencyNumber,
    formatOrders,
    formatPercent,
    percentChange,
    toFixedPlusRound,
    toNumber,
  } from "../format";
  import { GROSS_INCOME_ID } from "../module-ids";
  import {
    createGrossMarginQuery,
    createMarginServicesQuery,
  } from "../queries";
  import type { DashboardModuleProps, DashboardSlot } from "../registry";

  /** How many lines the list has room for. */
  const SERVICE_ROWS = 3;

  /** Gap between the chart bars, in pixels. */
  const BAR_GAP = 12;

  /** Chart colors */
  const REVENUE_FILL =
    "color-mix(in oklab, var(--muted-foreground) 45%, transparent)";
  const COST_FILL =
    "color-mix(in oklab, var(--muted-foreground) 16%, transparent)";
  const INCOME_FILL = "var(--chart-2)";

  /** Module render sizes */
  type ContentSize = "small" | "medium" | "large";

  /** Determines render size based on certain thresholds. */
  function contentSize(slot: DashboardSlot): ContentSize {
    const columns = dashboardWidthColumns[slot.width];
    const rows = dashboardHeightRows[slot.height];

    if (
      columns >= dashboardWidthColumns.half &&
      rows >= dashboardHeightRows.two
    )
      return "large";
    if (
      columns >= dashboardWidthColumns.third &&
      rows >= dashboardHeightRows.two
    )
      return "medium";

    return "small";
  }

  let { slot }: DashboardModuleProps = $props();

  const query = createGrossMarginQuery(GROSS_INCOME_ID);

  const size = $derived(contentSize(slot));

  // The breakdown is shown for large module only
  const servicesQuery = createMarginServicesQuery(
    GROSS_INCOME_ID,
    () => size === "large",
  );

  const summary = $derived(query.data ?? null);
  const services = $derived(servicesQuery.data ?? []);

  /** Whether the month's cost counts what the work cost as well as what the parts did. */
  const withLaborCost = $derived(
    booleanSetting(GROSS_INCOME_ID, "includeLaborCost"),
  );

  const revenue = $derived(toNumber(summary?.revenue));
  const cost = $derived(toNumber(summary?.cost));
  const income = $derived(toNumber(summary?.income));

  /** Nothing has been billed this month yet. */
  const empty = $derived(revenue === 0 && income === 0);

  const incomeChange = $derived(
    empty ? null : percentChange(income, toNumber(summary?.previousIncome)),
  );

  const margin = $derived(summary?.marginPercent ?? 0);
  const previousMargin = $derived(summary?.previousMarginPercent ?? 0);

  const hasBaseline = $derived(toNumber(summary?.previousRevenue) > 0);

  /** The margin's move, in percentage points */
  const marginChange = $derived(margin - previousMargin);

  const orderCount = $derived(summary?.orderCount ?? 0);
  const periodOrderCount = $derived(summary?.periodOrderCount ?? 0);

  /** What the middle bar takes off the revenue, as the module's own switch counts it. */
  const costLabel = $derived(withLaborCost ? "Собівартість" : "Запчастини");

  /** The margin's move, said as the badge says it: "+7 в.п.", "-1.4 в.п.", "0 в.п." */
  const marginChangeLabel = $derived(
    `${marginChange > 0 ? "+" : ""}${toFixedPlusRound(marginChange)} в.п.`,
  );

  /** A row's figures told as the sum they come from, for the row's own tooltip. */
  function contributionTitle(service: (typeof services)[number]): string {
    const revenue = toNumber(service.revenue);
    const cost = toNumber(service.cost);
    const income = toNumber(service.income);

    const told = [
      `Виручка ${formatCurrencyNumber(revenue)} грн. − собівартість ${formatCurrencyNumber(cost)} грн.`,
      `внесок ${formatCurrencyNumber(income)} грн.`,
      `маржа ${formatPercent(toNumber(service.marginPercent))} від виручки`,
    ];

    if (cost > 0) {
      told.push(`націнка ${formatPercent((income / cost) * 100)}`);
    }

    return told.join("; ");
  }

  const marginTone = $derived(
    marginChange === 0
      ? "text-muted-foreground/60"
      : marginChange > 0
        ? "text-emerald-600 dark:text-emerald-500"
        : "text-destructive",
  );

  const bars = $derived([
    {
      key: "revenue",
      label: "Виручка",
      from: 0,
      to: revenue,
      top: revenue,
      caption: formatCurrencyNumber(revenue),
      fill: REVENUE_FILL,
    },
    {
      key: "cost",
      label: costLabel,
      from: income,
      to: revenue,
      top: revenue,
      caption: formatCurrencyNumber(-cost),
      fill: COST_FILL,
    },
    {
      key: "income",
      label: "Валовий дохід",
      from: 0,
      to: income,
      top: income,
      caption: formatCurrencyNumber(income),
      fill: INCOME_FILL,
    },
  ]);

  const barRange = (bar: (typeof bars)[number]) => [bar.from, bar.to];

  const chartConfig = $derived(
    Object.fromEntries(
      bars.map(({ key, label, fill }) => [key, { label, color: fill }]),
    ) satisfies ChartConfig,
  );

  const ceiling = $derived(Math.max(revenue, income, 1) * 1.2);
  const floor = $derived(Math.min(income, 0) * 1.2);
</script>

{#snippet bridge()}
  <div class="flex h-full w-full flex-col">
    <div class="min-h-0 flex-1">
      <ChartContainer
        config={chartConfig}
        class="aspect-auto block h-full w-full"
      >
        <Chart
          data={bars}
          x="label"
          y={barRange}
          yDomain={[floor, ceiling]}
          padding={{ top: 20, right: 0, bottom: 0, left: 0 }}
          axis={false}
          grid={false}
          rule={false}
          class="h-full w-full"
        >
          {#snippet marks()}
            {#each bars as bar (bar.key)}
              <Bar
                data={bar}
                y={barRange}
                fill={bar.fill}
                radius={2}
                insets={{ left: BAR_GAP / 2, right: BAR_GAP / 2 }}
              />
            {/each}
            <Labels
              y="top"
              value="caption"
              placement="outside"
              offset={6}
              fontSize={12}
              class="font-medium"
            />
          {/snippet}
        </Chart>
      </ChartContainer>
    </div>

    <div class="flex pt-1">
      {#each bars as bar (bar.key)}
        <span
          class="text-muted-foreground min-w-0 flex-1 px-1.5 text-center text-[11px] leading-tight"
        >
          {bar.label}
        </span>
      {/each}
    </div>
  </div>
{/snippet}

<Card class="h-full py-4">
  {#if query.isPending}
    {#if size === "large"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
        <div class="flex flex-col gap-1">
          <Skeleton class="h-8 w-40" />
          <Skeleton class="h-5 w-32" />
        </div>
        <div class="flex min-h-0 flex-1 gap-5">
          <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-1.5">
            <Skeleton class="h-4 w-40" />
            <Skeleton class="min-h-0 flex-1" />
            <Skeleton class="h-4 w-52" />
          </div>
          <div class="flex w-[45%] shrink-0 flex-col gap-1.5">
            <div class="grid grid-cols-2 gap-1.5">
              <Skeleton class="h-12" />
              <Skeleton class="h-12" />
            </div>
            <Skeleton class="h-4 w-full" />
            <Skeleton class="h-4 w-full" />
            <Skeleton class="h-4 w-full" />
          </div>
        </div>
      </CardContent>
    {:else if size === "medium"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
        <Skeleton class="h-6 w-32" />
        <Skeleton class="h-5 w-32" />
        <Skeleton class="mt-2 min-h-0 flex-1" />
        <Skeleton class="h-5 w-40" />
      </CardContent>
    {:else}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
        <Skeleton class="h-6 w-28" />
        <Skeleton class="h-5 w-32" />
        <div class="mt-auto">
          <Skeleton class="h-5 w-24" />
        </div>
      </CardContent>
    {/if}
  {:else if query.error}
    <CardContent
      class="flex min-h-0 flex-1 flex-col items-center justify-center"
    >
      <QueryError error={query.error} onRetry={() => query.refetch()} />
    </CardContent>
  {:else if size === "large"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
      <div class="flex items-start justify-between gap-4">
        <div class="flex flex-col gap-1">
          <div class="text-3xl font-bold tracking-tight">
            <Money amount={summary?.income ?? "0"} />
          </div>
          <p class="text-muted-foreground text-sm">
            Валовий дохід цього місяця
          </p>
        </div>
        {#if !empty}
          <div class="text-sm">
            <TrendLine change={incomeChange} suffix="від минулого місяця" />
          </div>
        {/if}
      </div>

      <div class="flex min-h-0 flex-1 gap-5">
        <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-1.5">
          <p class="text-muted-foreground text-xs">Від виручки до доходу</p>

          <div class="relative min-h-0 flex-1">
            {@render bridge()}

            {#if empty}
              <p
                class="text-muted-foreground pointer-events-none absolute inset-0 flex items-center justify-center gap-1.5 text-sm"
              >
                <InboxIcon class="size-4 shrink-0" />
                Ще немає замовлень цього місяця
              </p>
            {/if}
          </div>

          {#if !empty}
            <p class="text-muted-foreground/60 text-xs">
              Розраховано для {orderCount} із {formatOrders(periodOrderCount)}
            </p>
          {/if}
        </div>

        <div class="flex w-[45%] shrink-0 flex-col gap-1.5">
          <div class="grid grid-cols-2 gap-1.5 pb-1.5">
            <div class="flex flex-col rounded-md">
              <p class="text-muted-foreground text-xs leading-tight">Маржа</p>
              <p class="text-base font-semibold whitespace-nowrap">
                {#if empty}
                  <span class="text-muted-foreground" aria-label="Немає даних"
                    >—</span
                  >
                {:else}
                  {formatPercent(margin)}
                {/if}
              </p>
            </div>

            <div class="flex flex-col rounded-md">
              <p class="text-muted-foreground text-xs leading-tight">
                Минулого місяця
              </p>
              <div class="flex items-baseline gap-1.5 whitespace-nowrap">
                {#if hasBaseline}
                  <p class="text-base font-semibold">
                    {formatPercent(previousMargin)}
                  </p>
                  <span class={cn("text-xs", marginTone)}>
                    {marginChangeLabel}
                  </span>
                {:else}
                  <span
                    class="text-muted-foreground text-base font-semibold"
                    aria-label="Немає даних"
                    title="Минулого місяця не було виручки">—</span
                  >
                {/if}
              </div>
            </div>
          </div>

          <div class="flex min-h-0 flex-1 flex-col">
            <div
              class="text-muted-foreground flex items-center gap-2 border-t border-b pb-1 text-xs leading-tight"
            >
              <span class="min-w-0 flex-1">Назва</span>
              <span class="w-18 shrink-0 text-right">Внесок, грн.</span>
              <span class="w-11 shrink-0 text-right">Маржа</span>
            </div>
            {#if servicesQuery.isPending}
              <div class="flex flex-col gap-2.5 pt-2.5">
                {#each Array(SERVICE_ROWS) as _, index (index)}
                  <Skeleton class="h-4 w-full" />
                {/each}
              </div>
            {:else if servicesQuery.error}
              <QueryError
                error={servicesQuery.error}
                onRetry={() => servicesQuery.refetch()}
                class="pt-2"
              />
            {:else if services.length === 0}
              <p class="text-muted-foreground pt-2 text-sm">
                Цього місяця ще немає позицій.
              </p>
            {:else}
              {#each services.slice(0, SERVICE_ROWS) as service (service.name)}
                <div
                  class="flex items-center gap-2 border-b border-border/60 py-1.5 text-sm last:border-b-0"
                  title={contributionTitle(service)}
                >
                  <span class="min-w-0 flex-1 truncate" title={service.name}>
                    {service.name}
                  </span>
                  <span class="w-16 shrink-0 text-right tabular-nums">
                    {formatCurrencyNumber(toNumber(service.income))}
                  </span>
                  <span class="w-11 shrink-0 text-right tabular-nums">
                    {formatPercent(toNumber(service.marginPercent))}
                  </span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    </CardContent>
  {:else if size === "medium"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <div class="text-2xl font-bold tracking-tight">
        <Money amount={summary?.income ?? "0"} />
      </div>
      <p class="text-muted-foreground text-sm">Валовий дохід цього місяця</p>

      <div class="relative mt-2 min-h-0 flex-1">
        {@render bridge()}

        {#if empty}
          <p
            class="text-muted-foreground pointer-events-none absolute inset-0 flex items-center justify-center gap-1.5 text-sm"
          >
            <InboxIcon class="size-4 shrink-0" />
            Ще немає замовлень цього місяця
          </p>
        {/if}
      </div>

      <div class="text-sm">
        {#if empty}
          <p class="text-muted-foreground/60">Ще немає замовлень</p>
        {:else}
          <TrendLine change={incomeChange} suffix="від минулого місяця" />
        {/if}
      </div>
    </CardContent>
  {:else}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <div class="text-2xl font-bold tracking-tight">
        <Money amount={summary?.income ?? "0"} />
      </div>
      <p class="text-muted-foreground text-sm">
        {#if slot.width === "sixth"}
          Дохід цього місяця
        {:else}
          Валовий дохід цього місяця
        {/if}
      </p>
      <div class="mt-auto text-sm">
        {#if empty}
          <p class="text-muted-foreground/60">Ще немає замовлень</p>
        {:else}
          <TrendLine change={incomeChange} digits={0} />
        {/if}
      </div>
    </CardContent>
  {/if}
</Card>
