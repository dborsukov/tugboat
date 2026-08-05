<script lang="ts">
  import "../app.css";

  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import { SvelteQueryDevtools } from "@tanstack/svelte-query-devtools";
  import { ModeWatcher } from "mode-watcher";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import SidebarInset from "$lib/components/ui/sidebar/sidebar-inset.svelte";
  import { Toaster } from "$lib/components/ui/sonner/index.js";

  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import { navHistory } from "$lib/components/nav-history.svelte";

  let { children } = $props();
  const queryClient = new QueryClient();

  navHistory.init();
</script>

<QueryClientProvider client={queryClient}>
  <ModeWatcher />
  <SvelteQueryDevtools />
  <Toaster position="bottom-center" richColors />

  <Sidebar.Provider>
    <AppSidebar />
    <SidebarInset class="h-screen flex flex-col overflow-hidden">
      <main class="flex-1 flex flex-col min-h-0">
        {@render children?.()}
      </main>
    </SidebarInset>
  </Sidebar.Provider>
</QueryClientProvider>
