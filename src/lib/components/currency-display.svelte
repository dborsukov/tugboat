<script lang="ts">
  import { NumericText } from "svelte-number-format";

  import { cn } from "$lib/utils";

  interface Props {
    cents: number;
    suffix?: string;
    suffix_muted?: boolean;
    class?: string;
  }

  let {
    cents,
    suffix,
    suffix_muted = false,
    class: className,
  }: Props = $props();
</script>

<div class={cn("flex gap-2", className)}>
  <NumericText
    value={cents / 100}
    formatStyle="currency"
    currency="UAH"
    locale="uk-UA"
    options={{ precision: 2 }}
  />
  {#if suffix}
    <p class={cn(suffix_muted && "text-muted-foreground")}>{suffix}</p>
  {/if}
</div>
