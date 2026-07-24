<script lang="ts">
  import type { Snippet } from "svelte";

  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import { Button } from "$lib/components/ui/button/index.js";

  import { navHistory } from "./nav-history.svelte.js";

  let { children }: { children?: Snippet } = $props();

  function goBack() {
    history.back();
  }

  function goForward() {
    history.forward();
  }
</script>

<div
  class="flex h-16 w-full items-center border-b sticky top-0 bg-background/70 backdrop-blur-md"
>
  <div
    class="flex h-full w-full max-w-7xl items-center gap-2 mx-auto px-4 overflow-hidden"
  >
    <div class="flex shrink-0 items-center gap-1">
      <Button
        variant="secondary"
        size="icon"
        onclick={goBack}
        disabled={!navHistory.canGoBack}
        aria-label="Назад"
        class="hover:cursor-pointer"
      >
        <ChevronLeftIcon class="h-5 w-5" />
      </Button>
      <Button
        variant="secondary"
        size="icon"
        onclick={goForward}
        disabled={!navHistory.canGoForward}
        aria-label="Вперед"
        class="hover:cursor-pointer"
      >
        <ChevronRightIcon class="h-5 w-5" />
      </Button>
    </div>
    {#if children}
      <div class="h-full min-w-0 flex-1 overflow-hidden">
        {@render children()}
      </div>
    {/if}
  </div>
</div>
