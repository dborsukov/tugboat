<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    value = null,
    valueClass,
    containerClass,
  }: {
    value: string | null;
    valueClass?: string;
    containerClass?: string;
  } = $props();
</script>

<div class={cn(containerClass)}>
  {#if value}
    <p class={cn(valueClass)}>{value}</p>
  {:else}
    <p class="text-muted-foreground text-center">—</p>
  {/if}
</div>
