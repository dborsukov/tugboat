<script lang="ts">
  import { NumericText } from "svelte-number-format";

  import { cn } from "$lib/utils";

  let {
    amount,
    suffix,
    suffix_muted = false,
    class: className,
  }: {
    amount: string | number;
    suffix?: string;
    suffix_muted?: boolean;
    class?: string;
  } = $props();
</script>

<div class={cn("flex gap-2", className)}>
  <NumericText
    value={amount}
    formatStyle="currency"
    currency="UAH"
    locale="uk-UA"
    options={{ precision: 2 }}
  />
  {#if suffix}
    <p class={cn(suffix_muted && "text-muted-foreground")}>{suffix}</p>
  {/if}
</div>
