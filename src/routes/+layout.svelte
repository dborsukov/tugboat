<script lang="ts">
  import "../app.css";

  import { ModeWatcher, setMode } from "mode-watcher";

  import { appStore } from "$lib/stores/app.svelte";

  import Button from "$lib/components/ui/button/button.svelte";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import SidebarInset from "$lib/components/ui/sidebar/sidebar-inset.svelte";
  import { Toaster } from "$lib/components/ui/sonner/index.js";

  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import { navHistory } from "$lib/components/nav-history.svelte";

  navHistory.init();
  setMode(appStore.settings.theme);

  let { children } = $props();
</script>

{#if appStore.error !== null}
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
  <ModeWatcher />
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
