<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children?: Snippet } = $props();
</script>

<div
  class="flex h-16 w-full items-center border-b sticky top-0 bg-background/70 backdrop-blur-md"
>
  <div class="flex h-full w-full max-w-7xl items-center gap-2 mx-auto px-4">
    {#if children}
      <div class="h-full min-w-0 flex-1">
        {@render children()}
      </div>
    {/if}
  </div>
</div>
