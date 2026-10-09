<script lang="ts">
  import "../app.css";

  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
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

  import { commands } from "$lib/bindings";
  import { navHistory } from "$lib/states/nav-history.svelte";
  import { settingsState } from "$lib/states/settings.svelte";

  let { children } = $props();
  const queryClient = new QueryClient();

  // Windows like the About dialog are standalone webviews: skip the app shell
  // (titlebar + sidebar) and render just the route content.
  const isStandaloneWindow = $derived(page.url.pathname.startsWith("/about"));

  navHistory.init();

  const bootShownAt = Date.now();
  const MIN_LOADING_MS = 600;
  const BOOT_LOADER_TIMEOUT_MS = 5000;

  /** Resolves after `count` animation frames, but never waits longer than 250ms. */
  function nextFrames(count: number): Promise<void> {
    return new Promise((resolve) => {
      let settled = false;
      const finish = () => {
        if (settled) return;
        settled = true;
        resolve();
      };
      const tick = (left: number) =>
        left <= 0 ? finish() : requestAnimationFrame(() => tick(left - 1));
      tick(count);
      setTimeout(finish, 250);
    });
  }

  /** Fades out the loading state that `src/app.html` puts in the window. */
  function removeBootLoader() {
    const loader = document.getElementById("boot-loader");
    if (!loader) return;
    loader.dataset.hidden = "true";
    setTimeout(() => loader.remove(), 200);
  }

  onMount(() => {
    const appWindow = getCurrentWebviewWindow();

    if (appWindow.label !== "main") {
      removeBootLoader();
      return;
    }

    const deadline = Date.now() + BOOT_LOADER_TIMEOUT_MS;
    const waitForSettings = () => {
      if (settingsState.settings !== null || Date.now() > deadline) {
        void startHandOff();
        return;
      }
      setTimeout(waitForSettings, 50);
    };
    waitForSettings();
  });

  async function startHandOff() {
    const remaining = Math.max(0, MIN_LOADING_MS - (Date.now() - bootShownAt));
    if (remaining > 0) await new Promise((r) => setTimeout(r, remaining));

    removeBootLoader();

    await nextFrames(2);

    try {
      // Rust shows the window, puts it back where the user left it and closes the
      // splash screen in one go. The restore has to happen after the window is
      // visible, so it cannot be done from here directly.
      await commands.revealWindow();
    } catch {
      // Skip. App still works if the hand-off fails.
    }
  }
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
