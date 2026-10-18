<script lang="ts">
  import { getLocalTimeZone, today } from "@internationalized/date";

  import { Button } from "$lib/components/ui/button";

  import { cn } from "$lib/utils";

  import { QUARTER_NUMERALS } from "../period";
  import YearPanel from "./year-panel.svelte";

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

  const QUARTER_NAMES = QUARTER_NUMERALS.map((numeral) => `${numeral} квартал`);

  let {
    /** `month` picks one month of the year, `quarter` one quarter of it. */
    granularity,
    year = $bindable(0),
    selected,
    onSelect,
  }: {
    granularity: "month" | "quarter";
    year: number;
    /** Selected month (1–12) or quarter (1–4). */
    selected: number;
    onSelect: (value: number) => void;
  } = $props();

  const options = $derived(
    granularity === "month" ? MONTH_NAMES : QUARTER_NAMES,
  );

  const current = today(getLocalTimeZone());

  const lastStarted = $derived(
    granularity === "month"
      ? current.month
      : Math.floor((current.month - 1) / 3) + 1,
  );

  function isFuture(value: number): boolean {
    if (year > current.year) return true;
    if (year < current.year) return false;
    return value > lastStarted;
  }
</script>

<YearPanel bind:year maxYear={current.year}>
  <div
    class={cn(
      "grid gap-1",
      granularity === "month" ? "grid-cols-4" : "grid-cols-2",
    )}
  >
    {#each options as name, index (name)}
      <Button
        variant={selected === index + 1 ? "secondary" : "ghost"}
        size="sm"
        disabled={isFuture(index + 1)}
        onclick={() => onSelect(index + 1)}
      >
        {name}
      </Button>
    {/each}
  </div>
</YearPanel>
