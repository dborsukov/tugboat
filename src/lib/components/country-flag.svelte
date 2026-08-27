<script lang="ts">
  import type { CountryCode } from "svelte-tel-input/types";

  import { cn } from "$lib/utils";

  // Flag sprite + css classes shipped by svelte-tel-input
  import "svelte-tel-input/styles/flags.css";

  let {
    country,
    size = "md",
    class: className,
  }: {
    country?: CountryCode | null | undefined;
    size?: "sm" | "md";
    class?: string;
  } = $props();

  const iso = $derived(country?.toLowerCase() ?? "");
</script>

<span
  class={cn(
    "flag inline-block shrink-0 bg-no-repeat",
    iso && `flag-${iso}`,
    size === "sm" ? "h-3.5! w-5!" : "h-4! w-6! md:h-3.5! md:w-5!",
    size === "sm" ? "rounded-xs" : "rounded-[3px]",
    className,
  )}
  aria-hidden="true"
></span>
