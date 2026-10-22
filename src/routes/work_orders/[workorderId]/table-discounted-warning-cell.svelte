<script lang="ts">
  import CurrencyDisplay from "$lib/components/currency-display.svelte";

  import { cn } from "$lib/utils";

  let {
    price: priceString,
    belowCost = false,
    suffix = "грн.",
    class: className,
  }: {
    price: string;
    belowCost?: boolean;
    suffix?: string;
    class?: string;
  } = $props();
</script>

<div
  class={cn(
    "flex w-full items-center justify-end gap-1.5 tabular-nums",
    belowCost && "text-rose-600 dark:text-rose-400 font-medium",
    className,
  )}
  title={belowCost ? "Нижче собівартості" : undefined}
>
  <CurrencyDisplay amount={priceString} {suffix} />
</div>
