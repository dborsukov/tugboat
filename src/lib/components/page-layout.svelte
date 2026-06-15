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

  <div class="flex-1 overflow-auto min-h-0">
    {#if children}
      {@render children()}
    {/if}
  </div>

  {#if footer}
    <div class="border-t bg-background">
      {@render footer()}
    </div>
  {/if}
</div>
