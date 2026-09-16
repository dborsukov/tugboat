<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import {
    getCurrentWebviewWindow,
    WebviewWindow,
  } from "@tauri-apps/api/webviewWindow";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { mode, resetMode, setMode } from "mode-watcher";

  import { goto } from "$app/navigation";

  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import MenuIcon from "@lucide/svelte/icons/menu";
  import MinusIcon from "@lucide/svelte/icons/minus";
  import PanelLeftCloseIcon from "@lucide/svelte/icons/panel-left-close";
  import PanelLeftOpenIcon from "@lucide/svelte/icons/panel-left-open";
  import SquareIcon from "@lucide/svelte/icons/square";
  import XIcon from "@lucide/svelte/icons/x";

  import { type ThemeMode } from "$lib/bindings";
  import { navHistory } from "$lib/states/nav-history.svelte";
  import { privacyState } from "$lib/states/privacy.svelte";
  import { settingsState } from "$lib/states/settings.svelte";
  import { cn } from "$lib/utils.js";

  import {
    createUpdateThemeMutation,
    createUpdateUiScaleMutation,
  } from "../../routes/settings/queries";
  import { Button, buttonVariants } from "./ui/button";
  import * as DropdownMenu from "./ui/dropdown-menu/index.js";
  import { Kbd, KbdGroup } from "./ui/kbd";
  import { useSidebar } from "./ui/sidebar";

  let isMaximized = $state(false);
  const sidebarContext = useSidebar();
  const appWindow = getCurrentWindow();

  const queryClient = useQueryClient();
  const updateThemeMutation = createUpdateThemeMutation(queryClient);
  const updateUiScaleMutation = createUpdateUiScaleMutation(queryClient);

  const uiScale = $derived(settingsState.settings?.uiScale ?? 1);
  const theme = $derived(settingsState.settings?.theme ?? "system");

  const MIN_UI_SCALE = 0.5;
  const MAX_UI_SCALE = 2.0;
  const UI_SCALE_STEP = 0.1;
  const UI_SCALE_PRESETS = [0.5, 0.75, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0];

  $effect(() => {
    appWindow.isMaximized().then((v) => (isMaximized = v));
    const unlistenPromise = appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });

  function goBack() {
    history.back();
  }

  function goForward() {
    history.forward();
  }

  function setTheme(theme: ThemeMode) {
    theme === "system" ? resetMode() : setMode(theme);
    updateThemeMutation.mutate(theme);
  }

  function toggleTheme() {
    const next = mode.current === "dark" ? "light" : "dark";
    setMode(next);
    updateThemeMutation.mutate(next);
  }

  function handleShortcut(e: KeyboardEvent) {
    // Let form fields handle their own keystrokes.
    const target = e.target;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      (target instanceof HTMLElement && target.isContentEditable)
    ) {
      return;
    }

    const hasCtrl = e.ctrlKey || e.metaKey;
    const hasShift = e.shiftKey;
    const hasAlt = e.altKey;

    // App shortcuts are Ctrl chords (optionally with Shift for the privacy
    // shortcuts below); Alt combinations are never shortcuts.
    if (!hasCtrl || hasAlt) return;

    const code = e.code;
    const key = e.key.toLowerCase();

    // NOTE: every condition is precomputed as a single boolean and branches
    // combine them only with plain `&&`. Do NOT "simplify" this back into
    // expressions like `!hasShift && (code === "KeyT" || key === "t")`:
    // a stale Svelte compiler once dropped the parentheses and emitted
    // `!hasShift && a || b`, which parses as `(!hasShift && a) || b` and
    // fired shortcuts even when Shift was held. Splitting the `||` chains
    // into their own variables makes that precedence bug impossible.
    const isComma = code === "Comma" || key === ",";
    const isQ = code === "KeyQ" || key === "q";
    const isT = code === "KeyT" || key === "t";
    const isH = code === "KeyH";
    const isS = code === "KeyS";
    const isZoomIn =
      code === "Equal" || code === "NumpadAdd" || key === "+" || key === "=";
    const isZoomOut =
      code === "Minus" ||
      code === "NumpadSubtract" ||
      key === "-" ||
      key === "_";
    const isZeroKey = code === "Digit0" || code === "Numpad0";

    const isPlain = !hasShift;

    if (isPlain && isQ) {
      e.preventDefault();
      appWindow.close();
    } else if (isPlain && isT) {
      e.preventDefault();
      toggleTheme();
    } else if (isPlain && isComma) {
      e.preventDefault();
      goto("/settings");
    } else if (hasShift && isH) {
      e.preventDefault();
      privacyState.setAll(true);
    } else if (hasShift && isS) {
      e.preventDefault();
      privacyState.setAll(false);
    } else if (isZoomIn) {
      e.preventDefault();
      applyUiScale(uiScale + UI_SCALE_STEP);
    } else if (isZoomOut) {
      e.preventDefault();
      applyUiScale(uiScale - UI_SCALE_STEP);
    } else if (isZeroKey && key === "0") {
      // Require the unshifted "0" so shifted layouts, like QWERTZ,
      // where = lives on the 0 key don't collide with the reset shortcut.
      e.preventDefault();
      applyUiScale(1);
    }
  }

  function applyUiScale(value: number) {
    const clamped = Math.min(MAX_UI_SCALE, Math.max(MIN_UI_SCALE, value));
    getCurrentWebviewWindow()
      .setZoom(clamped)
      .catch(() => {});
    updateUiScaleMutation.mutate(clamped);
  }

  async function openAboutWindow() {
    const existing = await WebviewWindow.getByLabel("about");
    if (existing) {
      existing.setFocus().catch(() => {});
      return;
    }
    // About page created hidden and reveals itself once its first frame is
    // painted, so the window never flashes white while the webview boots.
    const win = new WebviewWindow("about", {
      url: "/about",
      title: "Про застосунок",
      width: 340,
      height: 420,
      center: true,
      resizable: false,
      decorations: false,
      visible: false,
    });
    // If the About page ever fails to boot, don't leave the
    // window stuck hidden forever.
    setTimeout(() => {
      win.show().catch(() => {});
    }, 4000);
  }
</script>

<svelte:window onkeydown={handleShortcut} />

<!-- `relative z-titlebar` keeps every overlay backdrop (dialogs, sheets) painted
     behind the titlebar, and `data-window-controls` tells open overlays that
     interacting with it is not a dismissal. `pointer-events-auto` is required
     because bits-ui sets `pointer-events: none` on <body> while a modal is
     open, without it the controls stay visible but stop receiving clicks. -->
<div
  data-window-controls
  class="pointer-events-auto relative z-titlebar flex h-(--titlebar-height) items-center justify-between border-b border-border bg-sidebar text-sidebar-foreground"
>
  <!-- Left: menu, sidebar, navigation -->
  <div data-tauri-drag-region class="flex items-center gap-0.5 pl-1">
    <DropdownMenu.Root>
      <DropdownMenu.Trigger
        class={cn(
          buttonVariants({ variant: "ghost", size: "icon-sm" }),
          "text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground",
        )}
        aria-label="Menu"
      >
        <MenuIcon class="size-4" />
      </DropdownMenu.Trigger>

      <DropdownMenu.Content
        align="start"
        side="bottom"
        sideOffset={4}
        class="w-max whitespace-nowrap p-1"
      >
        <DropdownMenu.Item onSelect={() => goto("/settings")}>
          Налаштування
          <KbdGroup class="ml-auto">
            <Kbd>Ctrl</Kbd>
            <span class="text-xs text-muted-foreground">+</span>
            <Kbd>,</Kbd>
          </KbdGroup>
        </DropdownMenu.Item>

        <DropdownMenu.Separator />

        <DropdownMenu.Sub>
          <DropdownMenu.SubTrigger>Тема</DropdownMenu.SubTrigger>
          <DropdownMenu.SubContent class="w-max whitespace-nowrap p-1">
            <DropdownMenu.Item onSelect={toggleTheme}>
              Перемкнути тему
              <KbdGroup class="ml-auto">
                <Kbd>Ctrl</Kbd>
                <span class="text-xs text-muted-foreground">+</span>
                <Kbd>T</Kbd>
              </KbdGroup>
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.CheckboxItem
              checked={theme === "system"}
              onCheckedChange={() => setTheme("system")}
            >
              Системна
            </DropdownMenu.CheckboxItem>
            <DropdownMenu.CheckboxItem
              checked={theme === "light"}
              onCheckedChange={() => setTheme("light")}
            >
              Світла
            </DropdownMenu.CheckboxItem>
            <DropdownMenu.CheckboxItem
              checked={theme === "dark"}
              onCheckedChange={() => setTheme("dark")}
            >
              Темна
            </DropdownMenu.CheckboxItem>
          </DropdownMenu.SubContent>
        </DropdownMenu.Sub>

        <DropdownMenu.Sub>
          <DropdownMenu.SubTrigger>Масштаб інтерфейсу</DropdownMenu.SubTrigger>
          <DropdownMenu.SubContent class="w-max whitespace-nowrap p-1">
            <DropdownMenu.Item
              onSelect={() => applyUiScale(uiScale + UI_SCALE_STEP)}
              disabled={uiScale >= MAX_UI_SCALE}
            >
              Збільшити
              <KbdGroup class="ml-auto">
                <Kbd>Ctrl</Kbd>
                <span class="text-xs text-muted-foreground">+</span>
                <Kbd>+</Kbd>
              </KbdGroup>
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onSelect={() => applyUiScale(uiScale - UI_SCALE_STEP)}
              disabled={uiScale <= MIN_UI_SCALE}
            >
              Зменшити
              <KbdGroup class="ml-auto">
                <Kbd>Ctrl</Kbd>
                <span class="text-xs text-muted-foreground">+</span>
                <Kbd>-</Kbd>
              </KbdGroup>
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onSelect={() => applyUiScale(1)}
              disabled={Math.abs(uiScale - 1) < 0.001}
            >
              Скинути
              <KbdGroup class="ml-auto">
                <Kbd>Ctrl</Kbd>
                <span class="text-xs text-muted-foreground">+</span>
                <Kbd>0</Kbd>
              </KbdGroup>
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            {#each UI_SCALE_PRESETS as preset (preset)}
              <DropdownMenu.CheckboxItem
                checked={Math.abs(uiScale - preset) < 0.001}
                onCheckedChange={() => applyUiScale(preset)}
              >
                {Math.round(preset * 100)}%
              </DropdownMenu.CheckboxItem>
            {/each}
          </DropdownMenu.SubContent>
        </DropdownMenu.Sub>

        <DropdownMenu.Sub>
          <DropdownMenu.SubTrigger>Приватність</DropdownMenu.SubTrigger>
          <DropdownMenu.SubContent class="w-max whitespace-nowrap p-1">
            <DropdownMenu.CheckboxItem
              checked={privacyState.allHidden}
              onCheckedChange={privacyState.toggleAll}
            >
              {privacyState.allHidden ? "Показати все" : "Приховати все"}
              <KbdGroup class="ml-auto">
                <Kbd>Ctrl</Kbd>
                <span class="text-xs text-muted-foreground">+</span>
                <Kbd>Shift</Kbd>
                <span class="text-xs text-muted-foreground">+</span>
                <Kbd>{privacyState.allHidden ? "S" : "H"}</Kbd>
              </KbdGroup>
            </DropdownMenu.CheckboxItem>
            <DropdownMenu.Separator />
            <DropdownMenu.CheckboxItem
              checked={privacyState.hidePricing}
              onCheckedChange={() =>
                (privacyState.hidePricing = !privacyState.hidePricing)}
            >
              Приховати ціни запчастин
            </DropdownMenu.CheckboxItem>
            <DropdownMenu.CheckboxItem
              checked={privacyState.hideNotes}
              onCheckedChange={() =>
                (privacyState.hideNotes = !privacyState.hideNotes)}
            >
              Приховати нотатки замовлення
            </DropdownMenu.CheckboxItem>
          </DropdownMenu.SubContent>
        </DropdownMenu.Sub>

        <DropdownMenu.Sub>
          <DropdownMenu.SubTrigger>Довідка</DropdownMenu.SubTrigger>
          <DropdownMenu.SubContent class="w-max whitespace-nowrap p-1">
            <DropdownMenu.Item onSelect={openAboutWindow}>
              Про застосунок
            </DropdownMenu.Item>
          </DropdownMenu.SubContent>
        </DropdownMenu.Sub>

        <DropdownMenu.Separator />

        <DropdownMenu.Item onSelect={() => appWindow.close()}>
          Вийти
          <KbdGroup class="ml-auto">
            <Kbd>Ctrl</Kbd>
            <span class="text-xs text-muted-foreground">+</span>
            <Kbd>Q</Kbd>
          </KbdGroup>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>

    <div class="mx-1 h-4 w-px bg-border"></div>

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

    {#if privacyState.hidePricing || privacyState.hideNotes}
      <span
        title="Приховано частину даних"
        aria-label="Приховано частину даних"
        class="flex size-7 items-center justify-center text-amber-500"
      >
        <EyeOffIcon class="size-4" />
      </span>
    {/if}
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
