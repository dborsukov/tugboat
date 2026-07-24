<script lang="ts">
  import "../app.css";

  import { appStore } from "$lib/stores/app.svelte";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import SidebarInset from "$lib/components/ui/sidebar/sidebar-inset.svelte";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { Spinner } from "$lib/components/ui/spinner";

  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import { navHistory } from "$lib/components/nav-history.svelte";

  navHistory.init();

  let { children } = $props();
</script>

{#if appStore.loading}
  <div class="flex justify-center items-center gap-2 w-full h-screen text-2xl">
    <Spinner class="size-6" />
    <p>Завантаження...</p>
  </div>
{:else if appStore.error !== null}
  <div
    class="flex flex-col gap-2 items-center justify-center h-screen p-8 text-center bg-background text-foreground"
  >
    <h1 class="text-3xl font-bold tracking-tight">Виникла критична помилка</h1>
    <p class="mt-2 text-md text-muted-foreground max-w-xs select-text">
      {appStore.error}
    </p>
    <Button
      variant="link"
      class="underline hover:cursor-pointer"
      onclick={() => window.location.reload()}>Перезавантажити додаток</Button
    >
  </div>
{:else}
  <Toaster position="bottom-center" richColors />
  <Sidebar.Provider>
    <AppSidebar />
    <SidebarInset class="h-screen flex flex-col overflow-hidden">
      <main class="flex-1 flex flex-col min-h-0">
        {@render children?.()}
      </main>
    </SidebarInset>
  </Sidebar.Provider>
{/if}
