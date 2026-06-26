<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";

  import type { PageProps } from "./$types";
  import EmptyPage from "./empty-page.svelte";
  import WorkOrderPage from "./work-order-page.svelte";

  let { params }: PageProps = $props();

  const work_order = $derived(
    appStore.work_orders.find((c) => c.id.toString() === params.workorderId),
  );
</script>

{#if work_order !== undefined}
  <WorkOrderPage {work_order} />
{:else}
  <EmptyPage />
{/if}
