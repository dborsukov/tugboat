<script lang="ts">
  import type { Snippet } from "svelte";

  import ArrowDownIcon from "@lucide/svelte/icons/chevron-down";
  import ArrowUpIcon from "@lucide/svelte/icons/chevron-up";
  import ArrowUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";

  import { cn } from "$lib/utils";

  import { Button, type ButtonProps } from "$lib/components/ui/button/index.js";

  interface Props extends ButtonProps {
    sortStatus: boolean | string;
    onclick: ((event: unknown) => void) | undefined;
  }

  let { sortStatus, onclick, children, class: className }: Props = $props();
</script>

<Button variant="ghost" class={cn("hover:cursor-pointer", className)} {onclick}>
  {@render children?.()}
  {#if sortStatus === "asc"}
    <ArrowUpIcon />
  {:else if sortStatus === "desc"}
    <ArrowDownIcon />
  {:else}
    <ArrowUpDownIcon />
  {/if}
</Button>
