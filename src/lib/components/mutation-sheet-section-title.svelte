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

<div class={cn("pt-5 first:pt-0", className)}>
  <div class="flex items-center gap-2 pb-3">
    {#if icon}
      {@const Icon = icon}
      <Icon class="size-3.5 shrink-0 text-muted-foreground/70" />
    {/if}
    <h3
      class="shrink-0 text-[11.5px] font-bold tracking-[0.06em] whitespace-nowrap text-muted-foreground/80 uppercase"
    >
      {text}
    </h3>
    <span class="h-px min-w-4 flex-1 bg-border" aria-hidden="true"></span>
  </div>
  {#if hint}
    <p class="-mt-1 pb-3 text-xs leading-relaxed text-muted-foreground">
      {hint}
    </p>
  {/if}
</div>
