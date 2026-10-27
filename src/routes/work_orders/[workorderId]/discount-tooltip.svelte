<script lang="ts">
  import type { Snippet } from "svelte";

  import AppTooltip from "$lib/components/app-tooltip.svelte";

  import { cn } from "$lib/utils";

  let {
    first,
    second,
    triggerClass = "",
    children,
  }: {
    first: string;
    second: string;
    triggerClass?: string;
    children: Snippet;
  } = $props();
</script>

{#snippet tooltipContent()}
  <span class="block font-medium">{first}</span>
  <span class="block opacity-80">{second}</span>
{/snippet}

<AppTooltip content={tooltipContent}>
  {#snippet trigger({ props })}
    <button
      {...props}
      type="button"
      class={cn("cursor-help border-0 bg-transparent p-0", triggerClass)}
      aria-label={`${first}. ${second}`}
    >
      {@render children()}
    </button>
  {/snippet}
</AppTooltip>
