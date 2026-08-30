<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  import XIcon from "@lucide/svelte/icons/x";

  import PopeyeIcon from "$lib/components/icons/popeye.svelte";

  import { createVersionQuery } from "./queries";

  const appWindow = getCurrentWindow();

  const versionQuery = createVersionQuery();

  onMount(() => {
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        appWindow
          .show()
          .then(() => appWindow.setFocus())
          .catch(() => {});
      }),
    );
  });
</script>

<div
  class="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground"
>
  <!-- Minimal titlebar: drag region + close -->
  <header
    data-tauri-drag-region
    class="flex h-9 shrink-0 items-center justify-between border-b border-border bg-sidebar text-sidebar-foreground"
  >
    <div
      data-tauri-drag-region
      class="flex h-full items-center px-3 text-sm font-medium"
    >
      Про застосунок
    </div>
    <button
      onclick={() => appWindow.close()}
      aria-label="Close"
      class="flex h-full w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-[#e81123] hover:text-white"
    >
      <XIcon class="h-4 w-4" />
    </button>
  </header>

  <main
    class="flex flex-1 flex-col items-center justify-center gap-4 p-6 text-center"
  >
    <div
      class="flex size-24 items-center justify-center rounded-2xl"
      style="background: linear-gradient(145deg, #4a4380, #332d5e); box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.14), 0 6px 18px rgba(64, 57, 110, 0.28);"
    >
      <PopeyeIcon class="size-16" />
    </div>
    <div class="flex flex-col gap-0.5">
      <h1 class="text-lg font-semibold">Папай CRM</h1>
      <p class="text-sm text-muted-foreground">
        Версія {versionQuery.isError ? "?" : (versionQuery.data ?? "…")}
      </p>
    </div>
  </main>
</div>
