<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import XIcon from "@lucide/svelte/icons/x";

  import PopeyeIcon from "$lib/components/icons/popeye.svelte";

  import { commands } from "$lib/bindings";

  const appWindow = getCurrentWindow();

  let version = $state("…");
  $effect(() => {
    commands
      .getAppVersion()
      .then((v) => (version = v))
      .catch(() => (version = "?"));
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
      class="flex size-24 items-center justify-center overflow-hidden rounded-2xl bg-muted"
    >
      <PopeyeIcon class="size-24!" />
    </div>
    <div class="flex flex-col gap-0.5">
      <h1 class="text-lg font-semibold">Папай CRM</h1>
      <p class="text-sm text-muted-foreground">Версія {version}</p>
    </div>
  </main>
</div>
