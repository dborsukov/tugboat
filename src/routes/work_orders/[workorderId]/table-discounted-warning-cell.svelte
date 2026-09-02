<script lang="ts">
  import CurrencyDisplay from "$lib/components/currency-display.svelte";

  import { cn } from "$lib/utils";

  let {
    price: priceString,
    belowCost = false,
    class: className,
  }: {
    price: string;
    belowCost?: boolean;
    class?: string;
  } = $props();
</script>

<div
  class={cn(
    "w-full h-full flex items-center justify-end px-2",
    belowCost &&
      "bg-rose-100 text-rose-800 dark:bg-rose-500/15 dark:text-rose-300 font-medium",
    className,
  )}
>
  <CurrencyDisplay amount={priceString} suffix="грн." class="justify-end" />
</div>
