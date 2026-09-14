<script lang="ts">
  import { cn } from "$lib/utils";

  import { workerStatusStyle } from "./worker-status";

  let { isActive }: { isActive: boolean } = $props();

  const styles = $derived(workerStatusStyle(isActive));
</script>

<div
  class={cn(
    "h-full w-full p-2 flex items-center justify-start gap-2",
    styles.base,
  )}
>
  <styles.icon class="size-5" />
  <p class="font-medium">{styles.text}</p>
</div>
