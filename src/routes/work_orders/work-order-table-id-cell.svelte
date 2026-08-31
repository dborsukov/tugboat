<script lang="ts">
  import ArchivedBadge from "$lib/components/archived-badge.svelte";

  let { id, archivedAt }: { id: string; archivedAt: string | null } = $props();
</script>

<div class="px-2 flex items-center gap-2 whitespace-nowrap">
  <span class="font-mono">{id}</span>
  {#if archivedAt}
    <ArchivedBadge date={archivedAt} />
  {/if}
</div>
