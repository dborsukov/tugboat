<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";

  import type { PageProps } from "./$types";
  import ClientPage from "./client-page.svelte";
  import EmptyPage from "./empty-page.svelte";

  let { params }: PageProps = $props();

  const client = $derived(
    appStore.clients.find((c) => c.id.toString() === params.clientId),
  );
</script>

{#if client !== undefined}
  <ClientPage {client} />
{:else}
  <EmptyPage />
{/if}
