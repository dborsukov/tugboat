<script lang="ts">
  import type { Snippet } from "svelte";

  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";

  import { Button } from "$lib/components/ui/button";

  let {
    year = $bindable(0),
    onYearChange,
    maxYear = Number.POSITIVE_INFINITY,
    children,
  }: {
    year: number;
    onYearChange?: (year: number) => void;
    /* Latest year the stepper reaches */
    maxYear?: number;
    /* Optional content rendered inside the panel, under the year stepper. */
    children?: Snippet;
  } = $props();
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label="Попередній рік"
      onclick={() => {
        year -= 1;
        onYearChange?.(year);
      }}
    >
      <ChevronLeftIcon />
    </Button>
    <span class="text-sm font-medium tabular-nums">{year}</span>
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label="Наступний рік"
      disabled={year >= maxYear}
      onclick={() => {
        year += 1;
        onYearChange?.(year);
      }}
    >
      <ChevronRightIcon />
    </Button>
  </div>

  {#if children}
    {@render children()}
  {/if}
</div>
