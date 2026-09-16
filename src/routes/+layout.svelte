<script lang="ts">
  import "../app.css";

  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools";
  import {
    getCurrentWebviewWindow,
    WebviewWindow,
  } from "@tauri-apps/api/webviewWindow";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { ModeWatcher } from "mode-watcher";
  import { onMount } from "svelte";

  import { dev } from "$app/environment";
  import { page } from "$app/state";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import SidebarInset from "$lib/components/ui/sidebar/sidebar-inset.svelte";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import SettingsSync from "$lib/components/settings-sync.svelte";
  import TitleBar from "$lib/components/title-bar.svelte";
  import UiScaleApplier from "$lib/components/ui-scale-applier.svelte";

  import { navHistory } from "$lib/states/nav-history.svelte";

  let { children } = $props();
  const queryClient = new QueryClient();

  // Windows like the About dialog are standalone webviews: skip the app shell
  // (titlebar + sidebar) and render just the route content.
  const isStandaloneWindow = $derived(page.url.pathname.startsWith("/about"));

  navHistory.init();

  const splashShownAt = Date.now();
  const MIN_SPLASH_MS = 600;

  onMount(() => {
    if (getCurrentWebviewWindow().label !== "main") return;
    // Wait two frames, the first paint of the mounted app is guaranteed to be
    // on screen before we swap windows.
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        const remaining = Math.max(
          0,
          MIN_SPLASH_MS - (Date.now() - splashShownAt),
        );
        setTimeout(async () => {
          try {
            const appWindow = getCurrentWindow();
            await appWindow.show();
            await appWindow.setFocus();
            await (await WebviewWindow.getByLabel("splashscreen"))?.close();
          } catch {
            // Skip. App still works if the hand-off fails.
          }
        }, remaining);
      }),
    );
  });
</script>

<QueryClientProvider client={queryClient}>
  {#if dev}
    <SvelteQueryDevtools />
  {/if}

  <ModeWatcher />
  <SettingsSync />
  <UiScaleApplier />
  <Toaster position="bottom-center" richColors />

  {#if isStandaloneWindow}
    <!-- No titlebar in standalone windows, set overlay inset to zero -->
    <main class="h-screen w-screen overflow-hidden [--overlay-inset:0px]">
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
