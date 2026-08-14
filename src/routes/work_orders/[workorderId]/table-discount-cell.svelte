<script lang="ts">
  import { NumericText } from "svelte-number-format";

  import type { DiscountType } from "$lib/bindings";
  import { cn } from "$lib/utils";

  let {
    type,
    amount,
    percent,
    class: className,
  }: {
    type: DiscountType;
    amount: string;
    percent: string;
    class?: string;
  } = $props();
</script>

<div class={cn("flex gap-2", className)}>
  {#if type === "None"}
    <p class="text-muted-foreground">—</p>
  {:else if type === "Fixed"}
    <NumericText
      value={amount}
      formatStyle="currency"
      currency="UAH"
      locale="uk-UA"
      options={{ precision: 2 }}
    />
    <p>грн.</p>
  {:else}
    <p>{percent} %</p>
  {/if}
</div>
