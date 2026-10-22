<script lang="ts">
  import { cn } from "$lib/utils";

  import { workerStatusStyle } from "./worker-status";

  let { isActive }: { isActive: boolean } = $props();

  const styles = $derived(workerStatusStyle(isActive));
</script>

<div class="flex items-center">
  <span
    class={cn(
      "inline-flex h-7 items-center gap-1.5 rounded-full px-2.5 text-sm font-medium whitespace-nowrap",
      styles.base,
    )}
  >
    <styles.icon class="size-3.5 shrink-0" />
    {styles.text}
  </span>
</div>
