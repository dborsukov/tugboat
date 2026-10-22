<script lang="ts">
  import type { WorkOrderStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { workOrderStatusStylesRecord } from "./work-order-status";

  let { status }: { status: WorkOrderStatus } = $props();

  const styles = $derived(workOrderStatusStylesRecord[status]);
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
