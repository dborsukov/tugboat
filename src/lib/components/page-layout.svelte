<script lang="ts">
  import type { Snippet } from "svelte";

  import PageHeader from "./page-header.svelte"; // Adjust path as needed

  interface Props {
    header?: Snippet;
    children?: Snippet;
    footer?: Snippet;
  }

  let { header, children, footer }: Props = $props();
</script>

<div class="flex flex-col h-full min-h-0">
  {#if header}
    <PageHeader>
      {@render header()}
    </PageHeader>
  {/if}

  <div class="flex-1 w-full overflow-auto scrollbar-gutter-both">
    <div class="max-w-7xl mx-auto">
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
