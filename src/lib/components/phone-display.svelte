<script lang="ts">
  import { parse } from "svelte-tel-input";

  import { cn } from "$lib/utils";

  let { value, class: className }: { value?: string | null; class?: string } =
    $props();

  // Country-aware international format
  // Fall back to the raw stored value when it can't be parsed
  const detail = $derived(value ? parse(value) : null);
  const formatted = $derived(detail?.formatInternational ?? value ?? "");
</script>

<span
  class={cn("tabular-nums select-none", className)}
  title={value ?? undefined}
>
  {formatted}
</span>
