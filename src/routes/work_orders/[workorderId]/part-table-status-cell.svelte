<script lang="ts">
  import type { PartStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { partStatusStylesRecord } from "./part-status";

  let { status }: { status: PartStatus } = $props();

  const styles = $derived(partStatusStylesRecord[status]);
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
