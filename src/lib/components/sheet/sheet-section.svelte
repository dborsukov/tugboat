<script lang="ts">
  import type { Snippet } from "svelte";

  import { cn } from "$lib/utils";

  let {
    title,
    class: className,
    children,
  }: {
    title: string;
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<section class={cn("pt-5", className)}>
  <div class="flex items-center gap-2 pb-3">
    <h3
      class="shrink-0 text-[11.5px] font-bold tracking-[0.06em] whitespace-nowrap text-muted-foreground/80 uppercase"
    >
      {title}
    </h3>
    <span class="h-px min-w-4 flex-1 bg-border" aria-hidden="true"></span>
  </div>
  {@render children?.()}
</section>
