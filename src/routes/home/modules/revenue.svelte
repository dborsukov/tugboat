<script lang="ts">
  import { curveLinear, curveStepAfter } from "d3-shape";
  import { Chart, Points, Spline } from "layerchart";

  import ChartLineIcon from "@lucide/svelte/icons/chart-line";
  import FinishIcon from "@lucide/svelte/icons/flag";
  import InboxIcon from "@lucide/svelte/icons/inbox";

  import { Card, CardContent } from "$lib/components/ui/card";
  import {
    ChartContainer,
    ChartTooltip,
    type ChartConfig,
  } from "$lib/components/ui/chart";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import { cn } from "$lib/utils";

  import Meter from "../components/meter.svelte";
  import Money from "../components/money.svelte";
  import QueryError from "../components/query-error.svelte";
  import TrendLine from "../components/trend-line.svelte";
  import {
    dashboardHeightRows,
    dashboardWidthColumns,
  } from "../dashboard-grid";
  import { booleanSetting } from "../dashboard-layout";
  import { formatOrders, percentChange, toNumber } from "../format";
  import { REVENUE_ID } from "../module-ids";
  import { daysInMonth, elapsedDays, previousMonth } from "../period";
  import { createRevenueSummaryQuery } from "../queries";
  import type { DashboardModuleProps, DashboardSlot } from "../registry";
  import { dashboardPeriod } from "../state.svelte";

  /** Days left in the month below which a forecast has nothing left to project onto. */
  const FORECAST_MIN_DAYS_LEFT = 3;

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

  const query = createRevenueSummaryQuery();

  /** The chart's projection line, which the module's own settings switch asks for. */
  const showTrendLine = $derived(booleanSetting(REVENUE_ID, "showTrendLine"));

  const summary = $derived(query.data ?? null);
  const size = $derived(contentSize(slot));

  const month = $derived(dashboardPeriod.anchor);
  const days = $derived(daysInMonth(month));
  const elapsed = $derived(
    elapsedDays(month, dashboardPeriod.now, dashboardPeriod.timeZone),
  );

  const total = $derived(toNumber(summary?.month.total));
  const previous = $derived(toNumber(summary?.previousMonth.total));
  const change = $derived(percentChange(total, previous));

  const orders = $derived(summary?.month.orderCount ?? 0);
  const averageCheck = $derived(orders === 0 ? 0 : total / orders);

  /** Nothing has been billed this month yet. */
  const empty = $derived(orders === 0);

  /** Simple trend based on alrady paid orders*/
  const forecast = $derived(elapsed === 0 ? null : (total / elapsed) * days);

  /** Whether there is too little month left for a forecast to say anything. */
  const monthClosed = $derived(days - elapsed < FORECAST_MIN_DAYS_LEFT);

  const labor = $derived(toNumber(summary?.monthSplit.labor));
  const parts = $derived(toNumber(summary?.monthSplit.parts));
  const earned = $derived(labor + parts);

  const shares = $derived({
    labor: earned === 0 ? 0 : (labor / earned) * 100,
    parts: earned === 0 ? 0 : (parts / earned) * 100,
  });

  const rows = $derived.by(() => {
    const byDay = summary?.monthByDay ?? [];
    const lastPaid = byDay.at(-1);
    const previousByDay = summary?.previousMonthByDay ?? [];
    const lastPrevious = previousByDay.at(-1);
    const previousLength = Math.min(daysInMonth(previousMonth(month)), days);

    return Array.from({ length: days }, (_, index) => {
      const day = index + 1;

      return {
        day,
        month: day <= elapsed ? toNumber(byDay[index] ?? lastPaid) : undefined,
        previous:
          baseline && day <= previousLength
            ? toNumber(previousByDay[index] ?? lastPrevious)
            : undefined,
        pace: paceDrawn ? (day / elapsed) * total : undefined,
      };
    });
  });

  const head = $derived({ day: elapsed, month: total });

  /** Nothing came in last month, so there is no second line to draw. */
  const baseline = $derived(previous > 0);

  /** Whether to draw forecast/pace line or not*/
  const paceDrawn = $derived(
    showTrendLine && !empty && elapsed > 0 && !monthClosed,
  );

  /** Top of the chart: the highest of the lines drawn on it, plus a little headroom. */
  const ceiling = $derived(
    Math.max(total, previous, paceDrawn ? (forecast ?? 0) : 0, 1) * 1.06,
  );

  const currentLine = {
    key: "month",
    label: "Цей місяць",
    color: "var(--chart-2)",
  };
  const previousLine = {
    key: "previous",
    label: "Минулий місяць",
    color: "var(--muted-foreground)",
  };
  const paceLine = {
    key: "pace",
    label: "Прогноз",
    color: "color-mix(in oklab, var(--chart-2) 45%, transparent)",
  };
  const series = [currentLine, previousLine, paceLine];

  const chartConfig = Object.fromEntries(
    series.map(({ key, label, color }) => [key, { color, label }]),
  ) satisfies ChartConfig;

  const legend = $derived.by(() => {
    const entries = [currentLine];

    if (baseline) entries.push(previousLine);
    if (paceDrawn) entries.push(paceLine);

    return entries;
  });

  const chartSeries = $derived(
    legend.map(({ key, label, color }) => ({ key, label, color, value: key })),
  );

  /** The hovered day tooltip header. */
  const dayHeading = (day: number) => `День ${day}`;
</script>

{#snippet moneyTooltip({
  value,
  name,
  item,
}: {
  value: unknown;
  name: string;
  item: { color?: string; config?: { color?: string } };
})}
  <div class="flex w-full items-center justify-between gap-4">
    <span class="flex items-center gap-2">
      {#if item.config?.color ?? item.color}
        <span
          class="size-2.5 shrink-0 rounded-xs"
          style="background: {item.config?.color ?? item.color}"
        ></span>
      {/if}
      <span class="text-muted-foreground">{name}</span>
    </span>
    <Money amount={Number(value)} precision={0} />
  </div>
{/snippet}

<Card class="h-full py-4">
  {#if query.isPending}
    {#if size === "large"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
        <div class="flex flex-col gap-1">
          <Skeleton class="h-8 w-40" />
          <Skeleton class="h-5 w-36" />
        </div>
        <div class="flex min-h-0 flex-1 gap-5">
          <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-2">
            <Skeleton class="h-4 w-48" />
            <Skeleton class="min-h-0 flex-1" />
          </div>
          <div class="flex w-60 shrink-0 flex-col gap-1.5">
            <div class="grid grid-cols-2 gap-1.5">
              <Skeleton class="h-12" />
              <Skeleton class="h-12" />
            </div>
            <Skeleton class="h-16" />
            <Skeleton class="h-16" />
          </div>
        </div>
      </CardContent>
    {:else if size === "medium"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
        <Skeleton class="h-6 w-32" />
        <Skeleton class="h-5 w-36" />
        <div class="mt-auto flex flex-col gap-2">
          <Skeleton class="h-24 w-full" />
          <Skeleton class="h-5 w-40" />
        </div>
      </CardContent>
    {:else}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
        <Skeleton class="h-6 w-28" />
        <Skeleton class="h-5 w-36" />
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
            <Money amount={summary?.month.total ?? "0"} />
          </div>
          <p class="text-muted-foreground text-sm">Виручка цього місяця</p>
        </div>
        {#if !empty}
          <div class="text-sm">
            <TrendLine {change} suffix="від минулого місяця" />
          </div>
        {/if}
      </div>

      <div class="flex min-h-0 flex-1 gap-5">
        <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-2">
          <div
            class="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 text-xs"
          >
            {#each legend as entry (entry.key)}
              <span class="flex items-center gap-1.5 whitespace-nowrap">
                <span class="size-2 shrink-0" style="background: {entry.color}"
                ></span>
                {entry.label}
              </span>
            {/each}
          </div>

          <div class="relative min-h-0 flex-1">
            <ChartContainer
              config={chartConfig}
              class="aspect-auto block h-full w-full"
            >
              <Chart
                data={rows}
                x="day"
                series={chartSeries}
                xDomain={[1, days]}
                yDomain={[0, ceiling]}
                padding={{ top: 8, right: 8, bottom: 22, left: 8 }}
                axis="x"
                grid={false}
                rule={false}
                tooltipContext={{ mode: "quadtree-x" }}
                class="h-full w-full"
                props={{
                  xAxis: {
                    ticks: [1, 10, 20, days],
                    rule: true,
                    tickMarks: false,
                  },
                }}
              >
                {#snippet marks()}
                  {#if baseline}
                    <Spline
                      seriesKey="previous"
                      y="previous"
                      curve={curveStepAfter}
                      stroke={previousLine.color}
                      class="stroke-2"
                    />
                  {/if}
                  {#if paceDrawn}
                    <Spline
                      seriesKey="pace"
                      y="pace"
                      curve={curveLinear}
                      stroke={paceLine.color}
                      class="stroke-2 [stroke-dasharray:4_4]"
                    />
                  {/if}
                  {#if empty}
                    <!-- Dashed placeholder for empty month -->
                    <Spline
                      seriesKey="month"
                      y="month"
                      class="stroke-muted-foreground/40 [stroke-dasharray:4_4]"
                    />
                  {:else}
                    <Spline
                      seriesKey="month"
                      y="month"
                      curve={curveStepAfter}
                      stroke={currentLine.color}
                      class="stroke-2"
                    />
                  {/if}
                {/snippet}

                {#snippet aboveMarks()}
                  {#if !empty}
                    <Points
                      seriesKey="month"
                      data={[head]}
                      x="day"
                      y="month"
                      r={4}
                      fill={currentLine.color}
                    />
                  {/if}
                {/snippet}

                {#snippet tooltip()}
                  <ChartTooltip
                    labelFormatter={dayHeading}
                    formatter={moneyTooltip}
                  />
                {/snippet}
              </Chart>
            </ChartContainer>

            {#if empty}
              <p
                class="text-muted-foreground pointer-events-none absolute inset-0 flex items-center justify-center gap-1.5 text-sm"
              >
                <InboxIcon class="size-4 shrink-0" />
                Ще немає замовлень цього місяця
              </p>
            {/if}
          </div>
        </div>

        <div class="flex w-60 shrink-0 flex-col gap-1.5">
          <div class="grid grid-cols-2 gap-1.5">
            <div class="flex flex-col rounded-md px-2.5 py-1.5">
              <p class="text-muted-foreground text-xs">Виконано</p>
              <p class="text-base font-semibold" title={formatOrders(orders)}>
                {orders} зам.
              </p>
            </div>
            <div class="flex flex-col rounded-md px-2.5 py-1.5">
              <p class="text-muted-foreground text-xs">Середній чек</p>
              <div class="text-base font-semibold">
                {#if empty}
                  <span aria-label="Немає даних">—</span>
                {:else}
                  <Money amount={averageCheck} precision={0} />
                {/if}
              </div>
            </div>
          </div>

          <div class="flex flex-col gap-1.5 rounded-md px-2.5 py-1.5">
            <div class="flex items-end justify-between gap-2">
              <div class="flex flex-col">
                <p class="text-muted-foreground text-xs">Робота</p>
                <div class="text-base font-semibold">
                  <Money amount={labor} precision={0} />
                </div>
              </div>
              <div class="flex flex-col items-end">
                <p class="text-muted-foreground text-xs">Запчастини</p>
                <div class="text-base font-semibold">
                  <Money amount={parts} precision={0} />
                </div>
              </div>
            </div>
            <Meter
              class={cn(
                "h-1.5",
                empty && "border-border border border-dashed bg-transparent",
              )}
              segments={[
                { share: shares.labor, class: "bg-primary" },
                { share: shares.parts, class: "bg-muted-foreground/30" },
              ]}
            />
          </div>

          <div class="flex flex-col rounded-md px-2.5 py-1.5">
            <p class="text-muted-foreground text-xs pb-1">
              Прогноз на кінець місяця
            </p>
            {#if monthClosed}
              <div
                class="text-muted-foreground flex items-center gap-1.5 text-sm"
              >
                <FinishIcon class="size-3.5 shrink-0" />
                Місяць закрито
              </div>
            {:else if empty || forecast === null}
              <div
                class="text-muted-foreground flex items-center gap-1.5 text-sm"
              >
                <ChartLineIcon class="size-3.5 shrink-0" />
                Недостатньо даних
              </div>
            {:else}
              <div class="flex items-baseline gap-1 text-base font-semibold">
                <span>≈</span>
                <Money amount={forecast} precision={0} class="gap-1" />
              </div>
            {/if}
          </div>
        </div>
      </div>
    </CardContent>
  {:else if size === "medium"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <div class="text-2xl font-bold tracking-tight">
        <Money amount={summary?.month.total ?? "0"} />
      </div>
      <p class="text-muted-foreground text-sm">Виручка цього місяця</p>

      <div class="mt-auto flex flex-col gap-2">
        <div class="relative h-24">
          <ChartContainer
            config={chartConfig}
            class="aspect-auto block h-full w-full"
          >
            <Chart
              data={rows}
              x="day"
              series={chartSeries}
              xDomain={[1, days]}
              yDomain={[0, ceiling]}
              padding={{ top: 6, right: 4, bottom: 4, left: 4 }}
              axis={false}
              grid={false}
              rule={false}
              tooltipContext={{ mode: "quadtree-x" }}
              class="h-full w-full"
            >
              {#snippet marks()}
                {#if baseline}
                  <Spline
                    seriesKey="previous"
                    y="previous"
                    curve={curveStepAfter}
                    stroke={previousLine.color}
                    class="stroke-2"
                  />
                  <Spline
                    y="pace"
                    curve={curveLinear}
                    class="stroke-muted-foreground/25 [stroke-dasharray:4_4]"
                  />
                {/if}
                {#if empty}
                  <Spline
                    seriesKey="month"
                    y="month"
                    class="stroke-muted-foreground/40 [stroke-dasharray:4_4]"
                  />
                {:else}
                  <Spline
                    seriesKey="month"
                    y="month"
                    curve={curveStepAfter}
                    stroke={currentLine.color}
                    class="stroke-2"
                  />
                {/if}
              {/snippet}

              {#snippet tooltip()}
                <ChartTooltip
                  labelFormatter={dayHeading}
                  formatter={moneyTooltip}
                />
              {/snippet}
            </Chart>
          </ChartContainer>

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
          <div class="text-sm">
            <TrendLine {change} suffix="від минулого місяця" />
          </div>
        {/if}
      </div>
    </CardContent>
  {:else}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <div class="text-2xl font-bold tracking-tight">
        <Money amount={summary?.month.total ?? "0"} />
      </div>
      <p class="text-muted-foreground text-sm">Виручка цього місяця</p>
      <div class="mt-auto text-sm">
        {#if empty}
          <p class="text-muted-foreground/60">Ще немає замовлень</p>
        {:else}
          <TrendLine {change} digits={0} />
        {/if}
      </div>
    </CardContent>
  {/if}
</Card>
