<script lang="ts">
  import type { PartStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { partStatusStylesRecord } from "./part-status";

  let { status }: { status: PartStatus } = $props();

  const styles = $derived(partStatusStylesRecord[status]);
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
