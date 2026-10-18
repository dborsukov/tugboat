<script lang="ts">
  import InboxIcon from "@lucide/svelte/icons/inbox";

  import { Card, CardContent } from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import type { ExpenseCategorySummary } from "$lib/bindings";
  import { pluralizeUk } from "$lib/tools";
  import { cn } from "$lib/utils";

  import { expenseCategoryDotClass } from "../../expenses/categories";
  import Meter from "../components/meter.svelte";
  import Money from "../components/money.svelte";
  import QueryError from "../components/query-error.svelte";
  import TrendLine from "../components/trend-line.svelte";
  import {
    dashboardHeightRows,
    dashboardWidthColumns,
  } from "../dashboard-grid";
  import {
    formatCurrencyNumber,
    formatDayMonth,
    formatPercent,
    toNumber,
  } from "../format";
  import { GROSS_INCOME_ID } from "../module-ids";
  import {
    createExpenseSummaryQuery,
    createGrossMarginQuery,
  } from "../queries";
  import type { DashboardModuleProps, DashboardSlot } from "../registry";

  /** How many categories the tile names before folding the rest into one "Інші" row. */
  const CATEGORY_ROWS = 4;

  /**
   * What a half-width large tile has room for, its chart shares the row with the
   * figure, so six bars would be drawn narrower than the figures they carry.
   */
  const HALF_WIDTH_ROWS = 2;

  /** Module render sizes */
  type ContentSize = "small" | "medium" | "large";

  /** Determines render size based on certain thresholds. */
  function contentSize(slot: DashboardSlot): ContentSize {
    const columns = dashboardWidthColumns[slot.width];
    const rows = dashboardHeightRows[slot.height];

    if (
      columns >= dashboardWidthColumns.half &&
      rows >= dashboardHeightRows.three
    )
      return "large";
    if (
      columns >= dashboardWidthColumns.twoThirds &&
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

  /** One category as the tile draws it, with the tail of the list folded into one. */
  type ExpenseRow = {
    key: string;
    name: string;
    color: string;
    amount: number;
    count: number;
    share: number;
    latest: ExpenseCategorySummary["latestExpense"] | null;
    /** The categories an "Інші" row holds, for the tooltip. */
    names?: string[];
  };

  /** Waterfall bar. */
  type Bar = {
    key: string;
    label: string;
    caption: string;
    fill: string;
    from: number;
    to: number;
  };

  let { slot }: DashboardModuleProps = $props();

  const incomeQuery = createGrossMarginQuery(GROSS_INCOME_ID);
  const expenseQuery = createExpenseSummaryQuery();

  const size = $derived(contentSize(slot));
  const columns = $derived(dashboardWidthColumns[slot.width]);
  const rows = $derived(dashboardHeightRows[slot.height]);

  /** How many categories this tile has room to name. */
  const categoryLimit = $derived(
    size === "large" && columns < dashboardWidthColumns.twoThirds
      ? HALF_WIDTH_ROWS
      : CATEGORY_ROWS,
  );

  const figureClass = $derived(
    columns >= dashboardWidthColumns.half ? "text-3xl" : "text-2xl",
  );

  const income = $derived(incomeQuery.data ?? null);
  const expenses = $derived(expenseQuery.data ?? null);

  const grossIncome = $derived(toNumber(income?.income));
  const previousIncome = $derived(toNumber(income?.previousIncome));
  const marginPercent = $derived(income?.marginPercent ?? 0);

  const expenseTotal = $derived(toNumber(expenses?.total));
  const previousExpenseTotal = $derived(toNumber(expenses?.previousTotal));

  const spent = $derived(expenseTotal === 0 ? 0 : -expenseTotal);

  const result = $derived(grossIncome - expenseTotal);

  const figureTone = $derived(result < 0 ? "text-destructive" : "");
  const previousResult = $derived(previousIncome - previousExpenseTotal);

  const hasBaseline = $derived(
    previousIncome !== 0 || previousExpenseTotal !== 0,
  );

  const resultChange = $derived(hasBaseline ? result - previousResult : null);

  /** Nothing has been billed or spent this month yet. */
  const empty = $derived(grossIncome === 0 && expenseTotal === 0);

  const emptyExpenses = $derived(expenseTotal === 0);

  /** Share of the month's expenses the gross income already covers. */
  const coverage = $derived(
    emptyExpenses ? 0 : (grossIncome / expenseTotal) * 100,
  );

  const covered = $derived(Math.min(100, Math.max(0, coverage)));

  /** How much more gross income the month needs to break even. */
  const gap = $derived(Math.max(0, -result));

  /** The revenue that gap is worth at the margin the month was earned at. */
  const gapRevenue = $derived(
    gap > 0 && marginPercent > 0 ? (gap / marginPercent) * 100 : 0,
  );

  const coverageLabel = $derived(
    emptyExpenses
      ? "Витрат цього місяця немає"
      : covered >= 100
        ? "Валовий дохід покриває всі витрати"
        : `Валовий дохід покриває ${formatPercent(coverage)} витрат`,
  );

  /** The same, said short enough for a sixth of a row. */
  const coveredLabel = $derived(
    emptyExpenses ? "Без витрат" : `Покрито ${formatPercent(coverage)}`,
  );

  const resultNote = $derived.by(() => {
    if (empty) return "Ще немає даних цього місяця";
    if (gap === 0)
      return result > 0
        ? `Прибуток: ${formatCurrencyNumber(result)} грн.`
        : "Витрати покриті повністю";

    const revenue =
      gapRevenue > 0
        ? ` ≈ ${formatCurrencyNumber(gapRevenue)} грн. виручки`
        : "";

    return `До нуля: ще ${formatCurrencyNumber(gap)} грн. валового доходу${revenue}`;
  });

  const coverageSegments = $derived([
    { share: covered, class: "bg-chart-2" },
    { share: 100 - covered, class: "bg-muted-foreground/15" },
  ]);

  function toRow(category: ExpenseCategorySummary): ExpenseRow {
    return {
      key: `category-${category.name}`,
      name: category.name,
      color: expenseCategoryDotClass(category.color),
      amount: toNumber(category.amount),
      count: category.count,
      share: category.share,
      latest: category.latestExpense,
    };
  }

  function rowTitle(row: ExpenseRow): string {
    if (row.names?.length) return row.names.join(", ");

    return `${row.count} ${pluralizeUk(row.count, ["витрата", "витрати", "витрат"])}`;
  }

  const expenseRows = $derived.by<ExpenseRow[]>(() => {
    const categories = expenses?.categories ?? [];
    if (categories.length <= categoryLimit + 1) return categories.map(toRow);

    const kept = categories.slice(0, categoryLimit).map(toRow);
    const rest = categories.slice(categoryLimit);
    const sum = (pick: (category: ExpenseCategorySummary) => number) =>
      rest.reduce((total, category) => total + pick(category), 0);

    return [
      ...kept,
      {
        key: "other-categories",
        name: "Інші",
        color: "bg-muted-foreground/40",
        amount: sum((category) => toNumber(category.amount)),
        count: sum((category) => category.count),
        share: sum((category) => category.share),
        latest: null,
        names: rest.map((category) => category.name),
      },
    ];
  });

  const shares = $derived(
    expenseRows.map((row) => ({
      share: row.share,
      class: row.color,
      label: `${row.name}: ${formatPercent(row.share)}`,
    })),
  );

  const bars = $derived.by<Bar[]>(() => {
    const list: Bar[] = [];

    if (grossIncome !== 0) {
      list.push({
        key: "income",
        label: "Дохід",
        caption: formatCurrencyNumber(grossIncome),
        fill: "bg-chart-2",
        from: 0,
        to: grossIncome,
      });
    }

    let running = grossIncome;
    for (const row of expenseRows) {
      const to = running - row.amount;
      list.push({
        key: row.key,
        label: row.name,
        caption: formatCurrencyNumber(-row.amount),
        fill: row.color,
        from: running,
        to,
      });
      running = to;
    }

    list.push({
      key: "result",
      label: "Результат",
      caption: formatCurrencyNumber(result),
      fill: result < 0 ? "bg-destructive" : "bg-chart-4",
      from: 0,
      to: result,
    });

    return list;
  });

  const plot = $derived.by(() => {
    const low = Math.min(0, ...bars.map((bar) => Math.min(bar.from, bar.to)));
    const high = Math.max(0, ...bars.map((bar) => Math.max(bar.from, bar.to)));

    const top = high + (high - low) * 0.14;
    const span = top - low || 1;

    return {
      zero: (top / span) * 100,
      bars: bars.map((bar) => {
        const start = Math.max(bar.from, bar.to);
        const end = Math.min(bar.from, bar.to);

        return {
          ...bar,
          top: ((top - start) / span) * 100,
          height: Math.max((Math.abs(end - start) / span) * 100, 0.6),
        };
      }),
    };
  });
</script>

{#snippet headline(figure: string)}
  <div class={cn("font-bold tracking-tight", figure, figureTone)}>
    <Money amount={result} />
  </div>
  <p class="text-muted-foreground text-sm">Фінансовий результат цього місяця</p>
  <div class="text-sm">
    <TrendLine
      change={resultChange}
      format={formatCurrencyNumber}
      unit=" грн."
      suffix="від минулого місяця"
    />
  </div>
{/snippet}

{#snippet coverageBar()}
  <div class="flex flex-col gap-1">
    <p class="text-muted-foreground text-xs">{coverageLabel}</p>
    {#if emptyExpenses}
      <div
        class="border-border h-1.5 w-full rounded-full border border-dashed"
      ></div>
    {:else}
      <Meter class="h-1.5" segments={coverageSegments} />
    {/if}
  </div>
{/snippet}

{#snippet splits()}
  <div class="flex flex-col gap-2">
    <div class="flex items-baseline justify-between gap-3 text-sm">
      <span class="text-muted-foreground">Валовий дохід</span>
      <span class="font-semibold"><Money amount={grossIncome} /></span>
    </div>
    <div class="flex items-baseline justify-between gap-3 text-sm">
      <span class="text-muted-foreground">Витрати</span>
      <span class="font-semibold"><Money amount={spent} /></span>
    </div>
  </div>
{/snippet}

{#snippet stackedSplits()}
  <div class="flex flex-col gap-2">
    <div class="flex flex-col">
      <p class="text-muted-foreground text-xs">Валовий дохід</p>
      <p class="text-base font-semibold">
        <Money amount={grossIncome} precision={0} />
      </p>
    </div>
    <div class="flex flex-col">
      <p class="text-muted-foreground text-xs">Витрати</p>
      <p class="text-base font-semibold">
        <Money amount={spent} precision={0} />
      </p>
    </div>
  </div>
{/snippet}

{#snippet legend()}
  <div class="flex min-h-0 flex-col gap-1.5">
    <p class="text-muted-foreground text-xs">Витрати за категоріями</p>
    <Meter class="h-2" segments={shares} />
    <div class="flex min-h-0 flex-1 flex-col gap-1">
      {#each expenseRows as row (row.key)}
        <div
          class="flex min-w-0 items-center gap-2 text-xs"
          title={rowTitle(row)}
        >
          <span class={cn("size-2 shrink-0 rounded-full", row.color)}></span>
          <span class="min-w-0 flex-1 truncate">{row.name}</span>
          <span class="w-14 shrink-0 text-right tabular-nums">
            {formatCurrencyNumber(row.amount)}
          </span>
          <span class="w-9 shrink-0 text-right tabular-nums">
            {formatPercent(row.share)}
          </span>
        </div>
      {/each}
    </div>
    <div
      class="border-border/60 flex items-baseline justify-between gap-2 border-t pt-1.5 text-xs"
    >
      <span class="text-muted-foreground">Усього витрат</span>
      <span class="font-semibold"><Money amount={expenseTotal} /></span>
    </div>
  </div>
{/snippet}

{#snippet sideTable()}
  <div class="flex min-h-0 flex-col gap-1.5">
    <p class="text-muted-foreground text-xs">Витрати за категоріями</p>
    <Meter class="h-1.5" segments={shares} />
    <div class="flex min-h-0 flex-1 flex-col">
      <div
        class="text-muted-foreground flex items-center gap-2 border-b pb-1 text-[11px] leading-tight"
      >
        <span class="min-w-0 flex-1">Витрати</span>
        <span class="w-14 shrink-0 text-right">Сума</span>
        <span class="w-9 shrink-0 text-right">Частка</span>
      </div>
      {#each expenseRows as row (row.key)}
        <div
          class="border-border/60 flex items-center gap-2 border-b py-1 text-xs last:border-b-0"
          title={rowTitle(row)}
        >
          <span class="flex min-w-0 flex-1 items-center gap-1.5">
            <span class={cn("size-2 shrink-0 rounded-full", row.color)}></span>
            <span class="truncate">{row.name}</span>
          </span>
          <span class="w-14 shrink-0 text-right tabular-nums">
            {formatCurrencyNumber(row.amount)}
          </span>
          <span class="w-9 shrink-0 text-right tabular-nums">
            {formatPercent(row.share)}
          </span>
        </div>
      {/each}
      <div
        class="flex items-baseline justify-between gap-2 border-t pt-1.5 text-xs"
      >
        <span class="text-muted-foreground">Усього витрат</span>
        <span class="font-semibold"><Money amount={expenseTotal} /></span>
      </div>
    </div>
  </div>
{/snippet}

{#snippet wideTable()}
  <div class="border-border flex min-h-0 shrink-0 flex-col border-t">
    <div
      class="text-muted-foreground flex h-7 items-stretch gap-2 border-b text-[11px] leading-tight"
    >
      <span class="flex min-w-0 max-w-56 flex-1 items-center">Витрати</span>
      <span class="flex w-20 shrink-0 items-center justify-end">Сума</span>
      <span class="flex w-12 shrink-0 items-center justify-end">Частка</span>

      <span
        class="border-border flex w-12 shrink-0 items-center justify-end border-l pl-2"
        >Дата</span
      >
      <span class="flex min-w-0 flex-1 items-center">Останні витрати</span>
      <span class="flex w-20 shrink-0 items-center justify-end">Сума</span>
    </div>
    {#each expenseRows as row (row.key)}
      <div
        class="border-border/60 flex h-6 items-stretch gap-2 border-b text-xs last:border-b-0"
      >
        <span class="flex min-w-0 max-w-56 flex-1 items-center gap-1.5">
          <span class={cn("size-2 shrink-0 rounded-full", row.color)}></span>
          <span class="truncate" title={rowTitle(row)}>
            {row.name}
          </span>
        </span>
        <span class="flex w-20 shrink-0 items-center justify-end tabular-nums">
          {formatCurrencyNumber(row.amount)}
        </span>
        <span class="flex w-12 shrink-0 items-center justify-end tabular-nums">
          {formatPercent(row.share)}
        </span>
        <span
          class="border-border flex w-12 shrink-0 items-center justify-end border-l pl-2 tabular-nums"
        >
          {row.latest ? formatDayMonth(row.latest.date) : "—"}
        </span>
        <span class="flex min-w-0 flex-1 items-center">
          <span class="min-w-0 truncate" title={row.latest?.description ?? ""}>
            {row.latest?.description ?? "—"}
          </span>
        </span>
        <span class="flex w-20 shrink-0 items-center justify-end tabular-nums">
          {row.latest ? formatCurrencyNumber(toNumber(row.latest.amount)) : "—"}
        </span>
      </div>
    {/each}
    <div class="flex h-7 items-stretch gap-2 border-t text-xs font-medium">
      <span class="flex min-w-0 max-w-56 flex-1 items-center"
        >Усього витрат</span
      >
      <span class="flex w-20 shrink-0 items-center justify-end tabular-nums">
        {formatCurrencyNumber(expenseTotal)}
      </span>
      <span class="w-12 shrink-0"></span>
      <span class="border-border w-12 shrink-0 border-l"></span>
      <span class="min-w-0 flex-1"></span>
      <span class="w-20 shrink-0"></span>
    </div>
  </div>
{/snippet}

{#snippet chart()}
  <div class="flex min-h-0 flex-1 flex-col gap-1.5">
    <p class="text-muted-foreground text-xs">
      Від валового доходу до результату
    </p>

    <div class="relative min-h-0 flex-1">
      {#if plot.zero > 1 && plot.zero < 99}
        <div
          class="border-border/70 absolute inset-x-0 border-t border-dashed"
          style="top: {plot.zero}%"
        ></div>
      {/if}

      <div class="flex h-full items-stretch gap-2">
        {#each plot.bars as bar (bar.key)}
          <div class="relative min-w-0 flex-1">
            <span
              class="text-muted-foreground absolute inset-x-0 text-center text-[10px] leading-none tabular-nums"
              style="top: calc({bar.top}% - 0.85rem)">{bar.caption}</span
            >
            <div
              class={cn(
                "absolute left-1/2 w-full max-w-16 -translate-x-1/2 rounded-xs",
                bar.fill,
              )}
              style="top: {bar.top}%; height: {bar.height}%"
            ></div>
          </div>
        {/each}
      </div>
    </div>

    <div class="flex gap-2">
      {#each plot.bars as bar (bar.key)}
        <span
          class="text-muted-foreground line-clamp-2 min-w-0 flex-1 text-center text-[10px] leading-tight"
          title={bar.label}>{bar.label}</span
        >
      {/each}
    </div>
  </div>
{/snippet}

{#snippet nothingSpent()}
  <p
    class="text-muted-foreground flex min-h-0 flex-1 items-center justify-center gap-1.5 text-center text-sm"
  >
    <InboxIcon class="size-4 shrink-0" />
    {empty ? "Ще немає даних цього місяця" : "Витрат цього місяця ще немає"}
  </p>
{/snippet}

<Card class="h-full py-4">
  {#if incomeQuery.isPending || expenseQuery.isPending}
    {#if size === "small"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
        <Skeleton class="h-7 w-28" />
        <Skeleton class="h-5 w-40" />
        <div class="mt-auto">
          <Skeleton class="h-4 w-32" />
        </div>
      </CardContent>
    {:else if size === "medium"}
      <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
        <div class="flex flex-col gap-1">
          <Skeleton class="h-8 w-36" />
          <Skeleton class="h-5 w-48" />
          <Skeleton class="h-4 w-44" />
        </div>
        <div class="flex min-h-0 flex-1 gap-5">
          <div class="flex flex-1 flex-col gap-2">
            <Skeleton class="h-5 w-full" />
            <Skeleton class="h-5 w-full" />
            <div class="mt-auto flex flex-col gap-1">
              <Skeleton class="h-4 w-40" />
              <Skeleton class="h-1.5 w-full" />
            </div>
          </div>
          {#if columns >= dashboardWidthColumns.half}
            <div class="flex w-[42%] shrink-0 flex-col gap-1.5">
              <Skeleton class="h-4 w-32" />
              <Skeleton class="h-2 w-full" />
              {#each Array(3) as _, index (index)}
                <Skeleton class="h-4 w-full" />
              {/each}
              <Skeleton class="mt-auto h-4 w-full" />
            </div>
          {/if}
        </div>
      </CardContent>
    {:else}
      <CardContent class="flex min-h-0 flex-1 gap-5">
        <div class="flex w-[34%] shrink-0 flex-col gap-1.5">
          <Skeleton class="h-8 w-40" />
          <Skeleton class="h-5 w-full" />
          <Skeleton class="h-4 w-44" />
          <div class="mt-auto flex flex-col gap-1">
            <Skeleton class="h-4 w-full" />
            <Skeleton class="h-1.5 w-full" />
            <Skeleton class="h-8 w-full" />
          </div>
        </div>
        <Skeleton class="min-h-0 flex-1" />
      </CardContent>
    {/if}
  {:else if incomeQuery.error || expenseQuery.error}
    <CardContent
      class="flex min-h-0 flex-1 flex-col items-center justify-center"
    >
      <QueryError
        error={incomeQuery.error ?? expenseQuery.error}
        onRetry={() => {
          incomeQuery.refetch();
          expenseQuery.refetch();
        }}
      />
    </CardContent>
  {:else if size === "small"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-1">
      <div class={cn("font-bold tracking-tight", figureClass, figureTone)}>
        <Money amount={result} />
      </div>

      <p class="text-muted-foreground text-sm">
        {#if columns <= dashboardWidthColumns.sixth}
          Фінансовий результат
        {:else}
          Фінансовий результат цього місяця
        {/if}
      </p>

      {#if columns <= dashboardWidthColumns.sixth}
        {#if rows >= dashboardHeightRows.two}
          <div
            class={cn(
              "mt-2",
              rows >= dashboardHeightRows.three &&
                "mt-0 flex min-h-0 flex-1 flex-col justify-center",
            )}
          >
            {@render stackedSplits()}
          </div>
          <div class="mt-auto flex flex-col gap-1">
            <p class="text-muted-foreground text-xs">{coveredLabel}</p>
            {#if emptyExpenses}
              <div
                class="border-border h-1.5 w-full rounded-full border border-dashed"
              ></div>
            {:else}
              <Meter class="h-1.5" segments={coverageSegments} />
            {/if}
          </div>
        {:else}
          <p class="text-muted-foreground mt-auto text-sm">
            Витрати {formatCurrencyNumber(expenseTotal)}
          </p>
        {/if}
      {:else}
        <div class="mt-auto flex items-end justify-between gap-3 text-sm">
          <p class="text-muted-foreground min-w-0">
            Дохід {formatCurrencyNumber(grossIncome)} − Витрати {formatCurrencyNumber(
              expenseTotal,
            )}
          </p>
          <div class="shrink-0">
            <TrendLine change={resultChange} format={formatCurrencyNumber} />
          </div>
        </div>
      {/if}
    </CardContent>
  {:else if size === "medium"}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
      <div class="flex flex-col gap-1">
        {@render headline(figureClass)}
      </div>

      <div class="flex min-h-0 flex-1 gap-5">
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          {@render splits()}
          {#if rows >= dashboardHeightRows.three && columns < dashboardWidthColumns.half && !emptyExpenses}
            <div class="border-border/60 border-t pt-2">
              {@render legend()}
            </div>
          {/if}
          <div class="mt-auto">
            {@render coverageBar()}
          </div>
        </div>

        {#if columns >= dashboardWidthColumns.half}
          <div class="flex w-[42%] min-w-42.5 shrink-0 flex-col">
            {#if emptyExpenses}
              {@render nothingSpent()}
            {:else}
              {@render legend()}
            {/if}
          </div>
        {/if}
      </div>
    </CardContent>
  {:else}
    <CardContent class="flex min-h-0 flex-1 flex-col gap-3">
      <div class="flex min-h-0 flex-1 gap-5">
        <div
          class={cn(
            "flex shrink-0 flex-col gap-1",
            columns >= dashboardWidthColumns.full
              ? "w-[26%] min-w-60"
              : columns >= dashboardWidthColumns.twoThirds
                ? "w-[34%] min-w-60"
                : "w-[42%] min-w-60",
          )}
        >
          {@render headline(figureClass)}
          <div class="mt-auto flex flex-col gap-1">
            {@render coverageBar()}
            <p class="text-muted-foreground/70 text-xs leading-snug">
              {resultNote}
            </p>
          </div>
        </div>

        <div class="flex min-h-0 min-w-0 flex-1 flex-col">
          {#if emptyExpenses}
            {@render nothingSpent()}
          {:else}
            {@render chart()}
          {/if}
        </div>

        {#if columns >= dashboardWidthColumns.full && rows < dashboardHeightRows.three && !emptyExpenses}
          <div class="flex w-[30%] min-w-50 shrink-0 flex-col">
            {@render sideTable()}
          </div>
        {/if}
      </div>

      {#if rows >= dashboardHeightRows.three && expenseRows.length > 0}
        {@render wideTable()}
      {/if}
    </CardContent>
  {/if}
</Card>
