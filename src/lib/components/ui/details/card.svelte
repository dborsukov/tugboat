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

<div class={cn("flex flex-col border border-border p-4 rounded", className)}>
  {@render children()}
</div>
