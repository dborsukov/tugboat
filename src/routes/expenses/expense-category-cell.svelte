<script lang="ts">
  import type { ExpenseCategoryRef } from "$lib/bindings";

  import { expenseCategoryIndicatorClass } from "./categories";

  let { category = null }: { category?: ExpenseCategoryRef | null } = $props();
</script>

<div class="flex items-center gap-2">
  {#if category}
    <span class={expenseCategoryIndicatorClass(category.color)}></span>
    <span class="truncate">{category.name}</span>
  {:else}
    <p class="text-muted-foreground">—</p>
  {/if}
</div>
