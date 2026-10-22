<script lang="ts">
  import "flag-icons/css/flag-icons.min.css";

  import { cn } from "$lib/utils";

  let {
    value,
    size = "default",
    plain = false,
    country = "ua",
    class: className,
    title,
  }: {
    value: string;
    /** `sm` matches the 20px year badge, the default is 24px. */
    size?: "default" | "sm";
    /** Flag + text without the filled plate badge */
    plain?: boolean;
    /** ISO country code used by flag-icons */
    country?: string;
    class?: string;
    title?: string;
  } = $props();

  const compact = $derived(size === "sm");
  const flagSize = $derived(compact ? "h-3 w-4" : "h-3.5 w-4.5");
</script>

{#if plain}
  <span
    class={cn(
      "inline-flex w-fit items-center gap-1.5 whitespace-nowrap",
      className,
    )}
    title={title ?? value}
  >
    <span
      class={cn(
        "fi shrink-0 overflow-hidden rounded-[3px]",
        `fi-${country}`,
        flagSize,
      )}
    ></span>
    <span>{value}</span>
  </span>
{:else}
  <span
    class={cn(
      "inline-flex w-fit shrink-0 items-center gap-1.5 whitespace-nowrap rounded-2xl bg-muted px-2 text-xs font-medium text-foreground",
      compact ? "h-5" : "h-6",
      className,
    )}
    title={title ?? value}
  >
    <span
      class={cn(
        "fi shrink-0 overflow-hidden rounded-[3px]",
        `fi-${country}`,
        flagSize,
      )}
    ></span>
    <span>{value}</span>
  </span>
{/if}
