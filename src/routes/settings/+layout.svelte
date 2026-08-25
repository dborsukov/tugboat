<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { fly } from "svelte/transition";

  import { page } from "$app/state";

  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import { type AppSettings } from "$lib/bindings";

  import { createSettingsQuery } from "./queries";
  import {
    setSettingsContext,
    SETTINGS_CONTENT_WIDTH_CLASS,
  } from "./settings-context";
  import SettingsSidebar from "./settings-sidebar.svelte";

  let { children } = $props();

  const settingsQuery = createSettingsQuery();

  let seeded = false;
  let draft = $state<AppSettings | null>(null);
  $effect(() => {
    if (!seeded && settingsQuery.data) {
      seeded = true;
      draft = $state.snapshot(settingsQuery.data);
    }
  });

  setSettingsContext({
    get draft() {
      return draft;
    },
  });
</script>

<Sidebar.Provider>
  <SettingsSidebar />
  <Sidebar.Inset class="h-full flex flex-col overflow-hidden">
    <main class="flex-1 flex flex-col min-h-0">
      {#if draft === null}
        {#if settingsQuery.isError}
          <FetchErrorDisplay
            message={settingsQuery.error?.message}
            onRetry={settingsQuery.refetch}
            class="mt-20"
          />
        {:else}
          <PageLayout contentWidthClass={SETTINGS_CONTENT_WIDTH_CLASS}>
            <Skeleton class="h-30 w-full rounded-lg" />
            <Skeleton class="h-30 w-full rounded-lg" />
            <Skeleton class="h-30 w-full rounded-lg" />
          </PageLayout>
        {/if}
      {:else}
        {#key page.url.pathname}
          <div
            class="flex-1 flex flex-col min-h-0 w-full max-w-5xl mx-auto"
            in:fly={{ y: 12, duration: 220, easing: cubicOut }}
          >
            {@render children?.()}
          </div>
        {/key}
      {/if}
    </main>
  </Sidebar.Inset>
</Sidebar.Provider>
