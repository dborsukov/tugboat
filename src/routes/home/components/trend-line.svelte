<script lang="ts">
  import ArrowDownRightIcon from "@lucide/svelte/icons/arrow-down-right";
  import ArrowUpRightIcon from "@lucide/svelte/icons/arrow-up-right";
  import MinusIcon from "@lucide/svelte/icons/minus";

  import { cn } from "$lib/utils";

  import { formatPercent } from "../format";

  let {
    change,
    unit = "",
    suffix = "",
    digits = 1,
    positiveIsGood = true,
    format,
    class: className,
  }: {
    /** `null` when there is no baseline yet */
    change: number | null;
    unit?: string;
    suffix?: string;
    /**
     * Decimal places in the figure.
     * Represent max amount of digits, a whole figure showing will collapse into integer
     */
    digits?: number;
    positiveIsGood?: boolean;
    /** Custom figure format */
    format?: (change: number) => string;
    class?: string;
  } = $props();

  const tone = $derived(
    change == null || change === 0
      ? "text-muted-foreground/60"
      : change > 0 === positiveIsGood
        ? "text-emerald-600 dark:text-emerald-500"
        : "text-destructive",
  );

  function figure(value: number): string {
    if (format) return format(Math.abs(value));
    return formatPercent(Math.abs(value), digits, true);
  }
</script>

<span class={cn("flex items-center gap-1", tone, className)}>
  {#if change == null}
    <span>Немає з чим порівняти</span>
  {:else}
    {#if change === 0}
      <MinusIcon class="size-3.5 shrink-0" />
    {:else if change > 0}
      <ArrowUpRightIcon class="size-3.5 shrink-0" />
    {:else}
      <ArrowDownRightIcon class="size-3.5 shrink-0" />
    {/if}
    <span class="truncate">
      {figure(change)}{unit}
      {suffix}
    </span>
  {/if}
</span>
