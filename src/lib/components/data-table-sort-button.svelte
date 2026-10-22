<script lang="ts">
  import type { Snippet } from "svelte";

  import ArrowUpDownIcon from "@lucide/svelte/icons/arrow-up-down";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronUpIcon from "@lucide/svelte/icons/chevron-up";

  import { cn } from "$lib/utils";

  let {
    disabled = false,
    sortStatus,
    onclick,
    children,
    class: className,
  }: {
    disabled?: boolean;
    sortStatus: boolean | string;
    onclick?: (event: unknown) => void;
    children?: Snippet;
    class?: string;
  } = $props();
</script>

<button
  type="button"
  class={cn(
    "inline-flex items-center gap-1 rounded-sm text-sm font-medium whitespace-nowrap text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:cursor-default",
    !disabled && "cursor-pointer",
    className,
  )}
  {disabled}
  {onclick}
>
  {@render children?.()}
  {#if !disabled}
    {#if sortStatus === "asc"}
      <ChevronUpIcon class="size-3.5" />
    {:else if sortStatus === "desc"}
      <ChevronDownIcon class="size-3.5" />
    {:else}
      <ArrowUpDownIcon class="size-3.5 opacity-60" />
    {/if}
  {/if}
</button>
