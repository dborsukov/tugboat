<script lang="ts">
  import type { Snippet } from "svelte";

  import { Button } from "$lib/components/ui/button";

  let {
    expanded = $bindable(false),
    buttonLabel = "+ Додати нотатку",
    children,
  }: {
    expanded?: boolean;
    buttonLabel?: string;
    children: Snippet;
  } = $props();
</script>

{#if expanded}
  <div class="pt-1">
    {@render children()}
  </div>
{:else}
  <Button
    type="button"
    variant="link"
    class="h-auto w-fit p-0 text-[13px] font-medium text-brand hover:no-underline"
    onclick={() => (expanded = true)}
  >
    {buttonLabel}
  </Button>
{/if}
