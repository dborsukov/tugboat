<script lang="ts">
  import type { WorkOrderStatus } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import { workOrderStatusStylesRecord } from "./work-order-status";

  let { status }: { status: WorkOrderStatus } = $props();

  const styles = $derived(workOrderStatusStylesRecord[status]);
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
