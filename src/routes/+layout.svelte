<script lang="ts">
  import "../app.css";

  import { onMount } from "svelte";

  import { appStore } from "$lib/stores/app.svelte";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import SidebarInset from "$lib/components/ui/sidebar/sidebar-inset.svelte";

  import AppSidebar from "$lib/components/app-sidebar.svelte";

  let { children } = $props();

  onMount(() => appStore.loadAll());
</script>

{#if appStore.loading}
  Loading...
{:else if appStore.error !== null}
  Error: {appStore.error}
{:else}
  <Sidebar.Provider>
    <AppSidebar />
    <SidebarInset class="h-screen flex flex-col overflow-hidden">
      <main class="flex-1 flex flex-col min-h-0">
        {@render children?.()}
      </main>
    </SidebarInset>
  </Sidebar.Provider>
{/if}
