<script lang="ts">
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  import { prettifyIsoDateString } from "$lib/tools";

  let { createdAt }: { createdAt: string } = $props();

  const display = $derived(prettifyIsoDateString(createdAt, "table-datetime"));
  const full = $derived(
    prettifyIsoDateString(createdAt, "table-datetime-full"),
  );
</script>

<Tooltip.Root>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      <button
        {...props}
        type="button"
        class="cursor-default appearance-none rounded-sm border-0 bg-transparent p-0 text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:outline-none"
      >
        {display}
      </button>
    {/snippet}
  </Tooltip.Trigger>
  <Tooltip.Content>{full}</Tooltip.Content>
</Tooltip.Root>
