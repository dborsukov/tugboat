<script lang="ts">
  import type { PartStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { partStatusStylesRecord } from "./part-status";

  let { status }: { status: PartStatus } = $props();

  const styles = $derived(partStatusStylesRecord[status]);
</script>

<span
  class={cn(
    "inline-flex h-5 max-w-full items-center truncate rounded-full px-2 text-xs font-medium whitespace-nowrap",
    styles.base,
  )}
  title={styles.text}
>
  <span class="truncate">{styles.text}</span>
</span>
