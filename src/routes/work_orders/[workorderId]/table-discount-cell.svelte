<script lang="ts">
  import { NumericText } from "svelte-number-format";

  import type { DiscountType } from "$lib/bindings";
  import { cn } from "$lib/utils";

  let {
    amount,
    type,
    class: className,
  }: {
    amount: number;
    type: DiscountType;
    class?: string;
  } = $props();
</script>

<div class={cn("flex gap-2", className)}>
  {#if type === "Fixed"}
    <NumericText
      value={amount / 100}
      formatStyle="currency"
      currency="UAH"
      locale="uk-UA"
      options={{ precision: 2 }}
    />
    <p>грн.</p>
  {:else}
    <p>{amount} %</p>
  {/if}
</div>
