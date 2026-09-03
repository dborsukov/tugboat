<script lang="ts">
  import ArchivedBadge from "$lib/components/archived-badge.svelte";

  import type { WorkOrderType } from "$lib/bindings";

  import WorkOrderTypeBadge from "./work-order-type-badge.svelte";

  let {
    id,
    type,
    archivedAt,
  }: { id: string; type?: WorkOrderType; archivedAt: string | null } = $props();
</script>

<div class="px-2 flex items-center gap-2 whitespace-nowrap">
  <span class="font-mono">{id}</span>
  {#if type === "Warranty"}
    <WorkOrderTypeBadge />
  {/if}
  {#if archivedAt}
    <ArchivedBadge date={archivedAt} />
  {/if}
</div>
