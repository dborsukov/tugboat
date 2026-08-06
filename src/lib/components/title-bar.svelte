<script lang="ts">
  import { Menu as NativeMenu, PredefinedMenuItem } from "@tauri-apps/api/menu";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import MenuIcon from "@lucide/svelte/icons/menu";
  import MinusIcon from "@lucide/svelte/icons/minus";
  import PanelLeftCloseIcon from "@lucide/svelte/icons/panel-left-close";
  import PanelLeftOpenIcon from "@lucide/svelte/icons/panel-left-open";
  import SquareIcon from "@lucide/svelte/icons/square";
  import XIcon from "@lucide/svelte/icons/x";

  import { navHistory } from "$lib/states/nav-history.svelte";

  import { Button } from "./ui/button";
  import { useSidebar } from "./ui/sidebar";

  let isMaximized = $state(false);
  const sidebarContext = useSidebar();
  const appWindow = getCurrentWindow();

  $effect(() => {
    appWindow.isMaximized().then((v) => (isMaximized = v));
    const unlistenPromise = appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });

  async function showMenu() {
    const menu = await NativeMenu.new({
      items: [
        { id: "new-window", text: "New Window", action: () => {} },
        { id: "settings", text: "Settings", action: () => {} },
        await PredefinedMenuItem.new({ item: "Separator" }),
        { id: "quit", text: "Quit", action: () => appWindow.close() },
      ],
    });
    await menu.popup();
  }

  function goBack() {
    history.back();
  }

  function goForward() {
    history.forward();
  }
</script>

<div
  class="flex h-9 items-center justify-between border-b border-border bg-sidebar text-sidebar-foreground"
>
  <!-- Left: menu, sidebar, nav -->
  <div data-tauri-drag-region class="flex items-center gap-0.5 pl-1">
    <Button
      size="icon-sm"
      variant="ghost"
      onclick={showMenu}
      aria-label="Menu"
      class="text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
    >
      <MenuIcon class="size-4" />
    </Button>

    <Button
      size="icon-sm"
      variant="ghost"
      onclick={sidebarContext.toggle}
      aria-label="Toggle sidebar"
      class="text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
    >
      {#if sidebarContext.state === "expanded"}
        <PanelLeftCloseIcon class="size-4" />
      {:else}
        <PanelLeftOpenIcon class="size-4" />
      {/if}
    </Button>

    <div class="mx-1 h-4 w-px bg-border"></div>

    <Button
      size="icon-sm"
      variant="ghost"
      onclick={goBack}
      disabled={!navHistory.canGoBack}
      aria-label="Back"
      class="text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
    >
      <ChevronLeftIcon class="size-4" />
    </Button>

    <Button
      size="icon-sm"
      variant="ghost"
      onclick={goForward}
      disabled={!navHistory.canGoForward}
      aria-label="Forward"
      class="text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
    >
      <ChevronRightIcon class="size-4" />
    </Button>
  </div>

  <!-- Middle: drag space -->
  <div data-tauri-drag-region class="h-full flex-1"></div>

  <!-- Right: window controls -->
  <div class="flex h-full items-stretch">
    <button
      onclick={() => appWindow.minimize()}
      aria-label="Minimize"
      class="flex h-full w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-[#aaaaaa]/30 hover:text-foreground"
    >
      <MinusIcon class="size-4" />
    </button>

    <button
      onclick={() => appWindow.toggleMaximize()}
      aria-label="Maximize"
      class="flex h-full w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-[#aaaaaa]/30 hover:text-foreground"
    >
      {#if isMaximized}
        <CopyIcon class="size-3.5 rotate-90" />
      {:else}
        <SquareIcon class="size-3.5" />
      {/if}
    </button>

    <button
      onclick={() => appWindow.close()}
      aria-label="Close"
      class="flex h-full w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-[#e81123] hover:text-white"
    >
      <XIcon class="h-4 w-4" />
    </button>
  </div>
</div>
