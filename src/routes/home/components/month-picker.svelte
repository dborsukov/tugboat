<script lang="ts">
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import { Button, buttonVariants } from "$lib/components/ui/button";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import { pluralizeUk } from "$lib/tools";
  import { cn } from "$lib/utils";

  import {
    monthFromKey,
    monthKey,
    monthLabel,
    type CalendarMonth,
  } from "../period";
  import { createOrderMonthsQuery } from "../queries";
  import { dashboardPeriod } from "../state.svelte";

  const MONTH_NAMES = [
    "Січ",
    "Лют",
    "Бер",
    "Кві",
    "Тра",
    "Чер",
    "Лип",
    "Сер",
    "Вер",
    "Жов",
    "Лис",
    "Гру",
  ];

  const months = createOrderMonthsQuery();

  const month = $derived(dashboardPeriod.anchor);
  const currentKey = $derived(monthKey(month));
  const label = $derived(monthLabel(month, dashboardPeriod.timeZone));

  /** Orders paid per month, so a month button can say how much was settled in it. */
  const orderCounts = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const orderMonth of months.data ?? []) {
      counts.set(monthKey(orderMonth), orderMonth.orderCount);
    }
    return counts;
  });

  /** Months the dashboard may be pointed at: months with at least one paid order, and current month.*/
  const selectableKeys = $derived.by(() => {
    const keys = new Set(orderCounts.keys());
    keys.add(monthKey(dashboardPeriod.currentAnchor));
    return [...keys].sort();
  });

  const previousKey = $derived.by(() => {
    const earlier = selectableKeys.filter((key) => key < currentKey);
    return earlier.length > 0 ? earlier[earlier.length - 1] : null;
  });

  const nextKey = $derived(
    selectableKeys.find((key) => key > currentKey) ?? null,
  );

  /** Years that hold at least one selectable month, for the year stepper. */
  const years = $derived.by(() =>
    [...new Set(selectableKeys.map((key) => monthFromKey(key).year))].sort(
      (left, right) => left - right,
    ),
  );

  let open = $state(false);

  /** The month being browsed inside the picker */
  let draft = $state<CalendarMonth | null>(null);

  const shown = $derived(draft ?? month);

  const previousYear = $derived.by(() => {
    const earlier = years.filter((year) => year < shown.year);
    return earlier.length > 0 ? earlier[earlier.length - 1] : null;
  });

  const nextYear = $derived(years.find((year) => year > shown.year) ?? null);

  function goToMonth(key: string) {
    dashboardPeriod.goToMonth(monthFromKey(key));
  }

  /**
   * Steps the draft year. When the month being looked at has no orders in the
   * new year, the draft falls back to the closest month that has some, so the
   * picker never sits on a month that cannot be chosen.
   */
  function setDraftYear(year: number) {
    const sameMonth = { year, month: shown.month };
    if (selectableKeys.includes(monthKey(sameMonth))) {
      draft = sameMonth;
      return;
    }

    const monthsOfYear = selectableKeys
      .map(monthFromKey)
      .filter((orderMonth) => orderMonth.year === year);
    if (monthsOfYear.length === 0) return;

    const closest = monthsOfYear.reduce((best, orderMonth) =>
      Math.abs(orderMonth.month - shown.month) <
      Math.abs(best.month - shown.month)
        ? orderMonth
        : best,
    );
    draft = { year, month: closest.month };
  }

  function selectMonth(month: number) {
    const key = monthKey({ year: shown.year, month });
    if (!selectableKeys.includes(key)) return;
    goToMonth(key);
    open = false;
  }

  function goToCurrentMonth() {
    dashboardPeriod.goToMonth(null);
    open = false;
  }

  function formatPaidOrders(count: number): string {
    return `${count} ${pluralizeUk(count, [
      "оплачене замовлення",
      "оплачені замовлення",
      "оплачених замовлень",
    ])}`;
  }

  function monthTitle(key: string): string {
    const count = orderCounts.get(key) ?? 0;

    return count === 0 ? "Немає оплачених замовлень" : formatPaidOrders(count);
  }
</script>

<div class="flex h-full items-center justify-center gap-1">
  <Button
    variant="ghost"
    size="icon"
    aria-label="Попередній місяць"
    disabled={previousKey === null}
    onclick={() => previousKey && goToMonth(previousKey)}
  >
    <ChevronLeftIcon />
  </Button>

  <Popover.Root bind:open>
    <Popover.Trigger
      class={cn(
        buttonVariants({ variant: "secondary", size: "lg" }),
        "gap-2 px-4",
      )}
      aria-label="Обрати місяць"
      onclick={() => (draft = null)}
    >
      <CalendarIcon class="size-4" />
      <span class="text-lg font-medium">{label}</span>
      <ChevronDownIcon class="size-4 opacity-60" />
    </Popover.Trigger>

    <Popover.Content align="center" class="w-64 p-3">
      {#if months.isError}
        <div class="flex flex-col items-start gap-2">
          <p class="text-destructive text-sm">Не вдалося завантажити місяці</p>
          <Button variant="outline" size="xs" onclick={() => months.refetch()}>
            Спробувати ще
          </Button>
        </div>
      {:else if months.isPending}
        <div class="flex flex-col gap-2">
          <Skeleton class="h-8 w-full" />
          <Skeleton class="h-24 w-full" />
        </div>
      {:else}
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between">
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Попередній рік"
              disabled={previousYear === null}
              onclick={() =>
                previousYear !== null && setDraftYear(previousYear)}
            >
              <ChevronLeftIcon />
            </Button>
            <span class="text-sm font-medium tabular-nums">{shown.year}</span>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Наступний рік"
              disabled={nextYear === null}
              onclick={() => nextYear !== null && setDraftYear(nextYear)}
            >
              <ChevronRightIcon />
            </Button>
          </div>

          <div class="grid grid-cols-3 gap-1">
            {#each MONTH_NAMES as name, index (name)}
              {@const key = monthKey({
                year: shown.year,
                month: index + 1,
              })}
              <Button
                variant={index + 1 === shown.month ? "secondary" : "ghost"}
                size="sm"
                disabled={!selectableKeys.includes(key)}
                title={monthTitle(key)}
                onclick={() => selectMonth(index + 1)}
              >
                {name}
              </Button>
            {/each}
          </div>
        </div>
      {/if}

      <div class="border-t pt-2">
        <Button
          variant="secondary"
          size="lg"
          class="w-full"
          onclick={goToCurrentMonth}
        >
          Поточний місяць
        </Button>
      </div>
    </Popover.Content>
  </Popover.Root>

  <Button
    variant="ghost"
    size="icon"
    aria-label="Наступний місяць"
    disabled={nextKey === null}
    onclick={() => nextKey && goToMonth(nextKey)}
  >
    <ChevronRightIcon />
  </Button>
</div>
