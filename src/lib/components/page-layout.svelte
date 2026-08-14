<script lang="ts">
  import type { Snippet } from "svelte";

  import { cn } from "$lib/utils";

  let {
    header,
    children,
    footer,
    contentWidthClass,
    headerWidthClass,
    class: className,
  }: {
    header?: Snippet;
    children?: Snippet;
    footer?: Snippet;
    headerWidthClass?: string;
    contentWidthClass?: string;
    class?: string;
  } = $props();
</script>

<div class={cn("flex flex-col h-full min-h-0", className)}>
  {#if header}
    <div
      class="flex h-16 w-full px-4 items-center border-b sticky top-0 bg-background/20 backdrop-blur-md"
    >
      <div
        class={cn(
          "flex h-full w-full items-center gap-2 mx-auto max-w-7xl",
          headerWidthClass,
        )}
      >
        <div class="h-full min-w-0 flex-1">
          {@render header()}
        </div>
      </div>
    </div>
  {/if}

  <div class="flex-1 w-full overflow-auto scrollbar-gutter-both">
    <div class={cn("max-w-7xl mx-auto", contentWidthClass)}>
      {#if children}
        {@render children()}
      {/if}
    </div>
  </div>

  {#if footer}
    <div class="border-t bg-background">
      {@render footer()}
    </div>
  {/if}
</div>
