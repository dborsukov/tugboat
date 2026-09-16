<script lang="ts">
  import type { Component } from "svelte";

  import { cn } from "$lib/utils";

  let {
    text,
    icon,
    hint,
    class: className,
  }: {
    text: string;
    icon?: Component;
    hint?: string;
    class?: string;
  } = $props();
</script>

<div
  class={cn(
    "flex items-center gap-1.5 border-b border-border/70 pb-1",
    className,
  )}
>
  {#if icon}
    {@const Icon = icon}
    <Icon class="size-3.5 text-muted-foreground shrink-0" />
  {/if}
  <h3
    class="text-xs font-semibold uppercase tracking-wide text-muted-foreground"
  >
    {text}
  </h3>
</div>
{#if hint}
  <p class="text-xs text-muted-foreground">{hint}</p>
{/if}
