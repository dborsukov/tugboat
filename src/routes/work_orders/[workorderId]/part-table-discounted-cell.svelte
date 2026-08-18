<script lang="ts">
  import Big from "big.js";

  import CurrencyDisplay from "$lib/components/currency-display.svelte";

  import { cn } from "$lib/utils";

  let {
    cost: costString,
    price: priceString,
    class: className,
  }: { cost: string; price: string; class?: string } = $props();

  let cost = $derived(Big(costString ?? "0"));
  let price = $derived(Big(priceString ?? "0"));
</script>

<div
  class={cn(
    "w-full h-full flex items-center justify-center",
    cost.gt(price) && "bg-rose-100 text-rose-800 font-medium",
    className,
  )}
>
  <CurrencyDisplay amount={priceString} suffix="грн." class="justify-end" />
</div>
