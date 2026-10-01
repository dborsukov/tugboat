<script lang="ts">
  import { NumericText } from "svelte-number-format";

  import { cn } from "$lib/utils";

  let {
    amount,
    precision = 2,
    class: className,
  }: {
    amount: string | number;
    precision?: number;
    class?: string;
  } = $props();
</script>

<span class={cn("tabular-nums whitespace-nowrap flex gap-2", className)}>
  <NumericText
    value={amount}
    formatStyle="currency"
    currency="UAH"
    locale="uk-UA"
    options={{ precision }}
  />
  <p>грн.</p>
</span>
