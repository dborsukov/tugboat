<script lang="ts">
  import { NumericText } from "svelte-number-format";

  import { cn } from "$lib/utils";

  let { value, class: className }: { value: string | number; class?: string } =
    $props();
</script>

<div class={cn("text-center", className)}>
  <NumericText
    {value}
    locale="uk-UA"
    options={{
      precision: 1,
    }}
  />
</div>
