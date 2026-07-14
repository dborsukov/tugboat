<script lang="ts">
  import type { Snippet } from "svelte";

  import { cn } from "$lib/utils";

  let {
    children,
    class: className = "",
  }: {
    children: Snippet;
    class?: string;
  } = $props();
</script>

<div
  class={cn(
    "flex items-center justify-between border-b border-muted pb-2 mt-2 first:mt-0 last:border-0 last:pb-0",
    className,
  )}
>
  {@render children()}
</div>
