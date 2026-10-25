<script lang="ts">
  import {
    getLocalTimeZone,
    parseDate,
    today,
    type CalendarDate,
    type DateValue,
  } from "@internationalized/date";
  import type { RangeCalendar as RangeCalendarPrimitive } from "bits-ui";

  import { Calendar } from "$lib/components/ui/calendar/index.js";
  import { RangeCalendar } from "$lib/components/ui/range-calendar/index.js";

  import SegmentedControl from "$lib/components/sheet/segmented-control.svelte";

  import { pluralizeUk } from "$lib/tools";
  import { cn } from "$lib/utils";

  import {
    dayText,
    dottedDay,
    PERIOD_MODES,
    periodModeLabels,
    rangeDays,
    rangeForMode,
    type DayRange,
    type PeriodMode,
  } from "../period";
  import MonthPanel from "./month-panel.svelte";
  import YearPanel from "./year-panel.svelte";

  let {
    mode = $bindable<PeriodMode>("month"),
    anchor = $bindable<string>(dayText(new Date())),
    custom = $bindable<DayRange>({ from: "", to: "" }),
    /** How many months the custom range calendar shows at once. */
    rangeMonths = 2,
    /** Prints the span in days. */
    showSummary = false,
  }: {
    mode: PeriodMode;
    anchor: string;
    custom: DayRange;
    rangeMonths?: 1 | 2;
    showSummary?: boolean;
  } = $props();

  const maxDay = today(getLocalTimeZone());

  const range = $derived(rangeForMode(mode, anchor, custom));
  const days = $derived(rangeDays(range));
  const anchorDate = $derived(toCalendarDate(anchor));

  let browsedYear = $state(new Date().getFullYear());

  $effect(() => {
    browsedYear = anchorDate.year;
  });

  function textOf(day: DateValue): string {
    return `${day.year}-${String(day.month).padStart(2, "0")}-${String(day.day).padStart(2, "0")}`;
  }

  function toCalendarDate(text: string): CalendarDate {
    try {
      return parseDate(text);
    } catch {
      return today(getLocalTimeZone());
    }
  }

  function setAnchor(day: DateValue | undefined) {
    if (day) anchor = textOf(day);
  }

  function pickMonth(month: number) {
    anchor = textOf(anchorDate.set({ year: browsedYear, month }));
  }

  function pickQuarter(quarter: number) {
    anchor = textOf(
      anchorDate.set({ year: browsedYear, month: (quarter - 1) * 3 + 1 }),
    );
  }

  const customValue = $derived.by(() => {
    if (!custom.from || !custom.to) return undefined;
    try {
      return {
        start: parseDate(custom.from),
        end: parseDate(custom.to),
      };
    } catch {
      return undefined;
    }
  });

  function setCustomRange(value: RangeCalendarPrimitive.RootProps["value"]) {
    if (!value?.start || !value.end) return;
    custom = { from: textOf(value.start), to: textOf(value.end) };
  }
</script>

<div class="flex w-full flex-col gap-3">
  <SegmentedControl
    options={PERIOD_MODES.map((option) => ({
      value: option,
      label: periodModeLabels[option],
    }))}
    value={mode}
    clearable={false}
    ariaLabel="Період"
    class="w-full"
    selectedClass="bg-slate-600 text-white hover:bg-slate-500 dark:bg-slate-700 dark:hover:bg-slate-600"
    onValueChange={(next) => next && (mode = next)}
  />

  <div class="flex w-full justify-center">
    {#if mode === "day"}
      <Calendar
        type="single"
        value={anchorDate}
        onValueChange={setAnchor}
        maxValue={maxDay}
        locale="uk-UA"
        class={cn(
          "w-full bg-transparent p-0",
          "[&_[data-slot=calendar-head-cell]]:w-auto [&_[data-slot=calendar-head-cell]]:flex-1",
          "[&_[data-slot=calendar-grid-row]]:w-full",
          "[&_[data-slot=calendar-cell]]:w-auto [&_[data-slot=calendar-cell]]:flex-1 [&_[data-slot=calendar-cell]]:min-w-0",
          "[&_[data-slot=calendar-day]]:w-full [&_[data-slot=calendar-day]]:min-w-0",
        )}
      />
    {:else if mode === "month"}
      <div class="w-full">
        <MonthPanel
          granularity="month"
          bind:year={browsedYear}
          selected={anchorDate.month}
          onSelect={pickMonth}
        />
      </div>
    {:else if mode === "quarter"}
      <div class="w-full">
        <MonthPanel
          granularity="quarter"
          bind:year={browsedYear}
          selected={Math.floor((anchorDate.month - 1) / 3) + 1}
          onSelect={pickQuarter}
        />
      </div>
    {:else if mode === "year"}
      <div class="w-full">
        <YearPanel
          bind:year={browsedYear}
          maxYear={maxDay.year}
          onYearChange={(year) => (anchor = textOf(anchorDate.set({ year })))}
        />
      </div>
    {:else}
      <RangeCalendar
        value={customValue}
        onValueChange={setCustomRange}
        maxValue={maxDay}
        locale="uk-UA"
        numberOfMonths={rangeMonths}
        class="bg-transparent p-0"
      />
    {/if}
  </div>

  {#if showSummary}
    <p class="text-center text-xs text-muted-foreground">
      {#if days > 0}
        {dottedDay(range.from)} — {dottedDay(range.to)}
        · {days}
        {pluralizeUk(days, ["день", "дні", "днів"])}
      {:else}
        Оберіть період
      {/if}
    </p>
  {/if}
</div>
