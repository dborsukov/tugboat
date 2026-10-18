<script lang="ts">
  import type { ExpenseCategoryRef } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { expenseCategoryFillClass } from "./categories";

  let { category = null }: { category?: ExpenseCategoryRef | null } = $props();
</script>

<div
  class={cn(
    "flex h-full w-full items-center px-2 font-medium",
    expenseCategoryFillClass(category?.color ?? null),
  )}
>
  {#if category}
    <span class="truncate">{category.name}</span>
  {:else}
    <p class="w-full text-center text-muted-foreground">—</p>
  {/if}
</div>
