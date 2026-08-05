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

<span class={cn("flex items-center gap-2 text-muted-foreground", className)}>
  {@render children()}
</span>
