<script lang="ts">
  import type { Component } from "svelte";

  import { cn } from "$lib/utils";

  let {
    icon: Icon,
    class: className = "",
  }: {
    icon: Component<any>;
    class?: string;
  } = $props();
</script>

<Icon class={cn("size-4 text-muted-foreground/70", className)} />
