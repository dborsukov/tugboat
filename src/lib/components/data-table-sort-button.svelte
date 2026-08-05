<script lang="ts">
  import ArrowDownIcon from "@lucide/svelte/icons/chevron-down";
  import ArrowUpIcon from "@lucide/svelte/icons/chevron-up";
  import ArrowUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";

  import { Button, type ButtonProps } from "$lib/components/ui/button/index.js";

  import { cn } from "$lib/utils";

  interface Props extends ButtonProps {
    disabled?: boolean;
    sortStatus: boolean | string;
    onclick: ((event: unknown) => void) | undefined;
  }

  let {
    disabled = false,
    sortStatus,
    onclick,
    children,
    class: className,
    ...restProps
  }: Props = $props();
</script>

<Button
  variant="ghost"
  class={cn(
    "disabled:opacity-100 disabled:text-foreground",
    disabled ? "cursor-default" : "hover:cursor-pointer",
    className,
  )}
  {disabled}
  {onclick}
  {...restProps}
>
  {@render children?.()}
  {#if !disabled}
    {#if sortStatus === "asc"}
      <ArrowUpIcon />
    {:else if sortStatus === "desc"}
      <ArrowDownIcon />
    {:else}
      <ArrowUpDownIcon />
    {/if}
  {/if}
</Button>
