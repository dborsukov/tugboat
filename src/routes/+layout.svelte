<script lang="ts">
  import "../app.css";

  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools";
  import { ModeWatcher } from "mode-watcher";

  import { dev } from "$app/environment";
  import { page } from "$app/state";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import SidebarInset from "$lib/components/ui/sidebar/sidebar-inset.svelte";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import TitleBar from "$lib/components/title-bar.svelte";
  import UiScaleApplier from "$lib/components/ui-scale-applier.svelte";

  import { navHistory } from "$lib/states/nav-history.svelte";

  let { children } = $props();
  const queryClient = new QueryClient();

  // Windows like the About dialog are standalone webviews: skip the app shell
  // (titlebar + sidebar) and render just the route content.
  const isStandaloneWindow = $derived(page.url.pathname.startsWith("/about"));

  navHistory.init();
</script>

<QueryClientProvider client={queryClient}>
  {#if dev}
    <SvelteQueryDevtools />
  {/if}

  <ModeWatcher />
  <UiScaleApplier />
  <Toaster position="bottom-center" richColors />

  {#if isStandaloneWindow}
    <main class="h-screen w-screen overflow-hidden">
      {@render children?.()}
    </main>
  {:else}
    <Tooltip.Provider>
      <Sidebar.Provider>
        <div class="flex h-screen w-screen flex-col overflow-hidden">
          <TitleBar />
          <div class="flex flex-1 overflow-hidden transform-gpu">
            <AppSidebar />
            <SidebarInset class="h-full flex flex-col overflow-hidden">
              <main class="flex-1 flex flex-col min-h-0">
                {@render children?.()}
              </main>
            </SidebarInset>
          </div>
        </div>
      </Sidebar.Provider>
    </Tooltip.Provider>
  {/if}
</QueryClientProvider>
