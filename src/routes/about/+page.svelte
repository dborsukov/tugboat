<script lang="ts">
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import CheckIcon from "@lucide/svelte/icons/check";
  import CircleArrowUpIcon from "@lucide/svelte/icons/circle-arrow-up";
  import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import XIcon from "@lucide/svelte/icons/x";

  import { Button } from "$lib/components/ui/button";

  import { commands } from "$lib/bindings";
  import { brandName } from "$lib/brand";
  import { isMacOS } from "$lib/platform";
  import { aboutUpdate } from "$lib/states/about-update.svelte";
  import { formatCheckTime } from "$lib/update-status";

  import { createCommitQuery, createVersionQuery } from "./queries";

  const appWindow = getCurrentWindow();
  const versionQuery = createVersionQuery();
  const commitQuery = createCommitQuery();

  const version = $derived(
    versionQuery.isError ? "?" : (versionQuery.data ?? "…"),
  );
  const commit = $derived(
    commitQuery.data && commitQuery.data !== "unknown"
      ? commitQuery.data
      : null,
  );
  const versionText = $derived(
    commit ? `Версія ${version} · ${commit}` : `Версія ${version}`,
  );

  const status = $derived(aboutUpdate.status);
  const checkedAt = $derived(
    status.state === "up-to-date" ||
      status.state === "available" ||
      status.state === "error"
      ? formatCheckTime(status.checkedAt)
      : null,
  );
  const buttonLabel = $derived(
    status.state === "available"
      ? "Переглянути оновлення"
      : status.state === "error"
        ? "Спробувати ще раз"
        : "Перевірити оновлення",
  );

  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;
  let contentEl = $state<HTMLDivElement | null>(null);
  let updateButton = $state<HTMLButtonElement | HTMLAnchorElement | null>(null);
  let lastSize = "";

  async function copyVersion() {
    const result = await commands.copyText(versionText);
    if (result.status === "error") {
      toast.error("Не вдалося скопіювати версію", {
        description: result.error.message,
        position: "bottom-left",
      });
      return;
    }

    copied = true;
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => (copied = false), 1500);
  }

  function primaryAction() {
    if (status.state === "available") {
      aboutUpdate.show();
      return;
    }
    aboutUpdate.check();
  }

  function fitWindow(force = false) {
    const element = contentEl;
    if (!element) return;

    const rect = element.getBoundingClientRect();
    if (rect.height === 0) return;

    const width = Math.ceil(rect.width) + 36;
    const height = Math.ceil(rect.height) + 36 + (isMacOS ? 0 : 36);
    const size = `${width}x${height}`;
    if (!force && size === lastSize) return;

    lastSize = size;
    void appWindow.setSize(new LogicalSize(width, height)).catch((error) => {
      console.error("Не вдалося змінити розмір вікна «Про застосунок»", error);
    });
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") void appWindow.close();
  }

  $effect(() => {
    void status;
    void versionText;
    requestAnimationFrame(() => fitWindow());
  });

  $effect(() => {
    const element = contentEl;
    if (!element) return;

    const observer = new ResizeObserver(() =>
      requestAnimationFrame(() => fitWindow()),
    );
    observer.observe(element);
    return () => observer.disconnect();
  });

  onMount(() => {
    document.getElementById("boot-loader")?.remove();
    void aboutUpdate.init();

    requestAnimationFrame(() => {
      appWindow
        .show()
        .then(() => {
          appWindow.setFocus().catch(() => {});
          updateButton?.focus();

          // Resize only while the window is visible. On Windows tao measures
          // the frame of a hidden undecorated window incorrectly, which loses
          // the bottom padding.
          requestAnimationFrame(() => fitWindow(true));
        })
        .catch(() => {});
    });

    return () => {
      aboutUpdate.destroy();
      if (copyTimer) clearTimeout(copyTimer);
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div
  class="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground"
>
  {#if !isMacOS}
    <!-- The other platforms have no native chrome, so this window draws its own. -->
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
  {/if}

  <main class="flex flex-1 items-center justify-center p-[18px]">
    <div
      bind:this={contentEl}
      class="flex w-[240px] flex-col items-center text-center"
    >
      <img src="/tugboat-app.png" alt="" class="size-24" />

      <h1 class="mt-3 text-[22px] leading-tight font-semibold">
        {brandName()}
      </h1>

      <button
        type="button"
        class="text-muted-foreground hover:text-foreground focus-visible:ring-ring/50 mt-1 inline-flex items-center gap-1 rounded-md px-1 py-0.5 text-sm transition-colors focus-visible:ring-[3px] focus-visible:outline-none"
        aria-label="Скопіювати версію"
        onclick={() => void copyVersion()}
      >
        {#if copied}
          <CheckIcon class="size-3.5" />
          Скопійовано
        {:else}
          {versionText}
          <CopyIcon class="size-3.5" />
        {/if}
      </button>

      <span class="sr-only" role="status" aria-live="polite">
        {copied ? "Скопійовано" : ""}
      </span>

      <div class="bg-border mt-4 h-px w-full" aria-hidden="true"></div>

      <div
        class="flex w-full flex-col items-center gap-1"
        class:mt-4={status.state !== "idle"}
        role="status"
        aria-live="polite"
      >
        {#if status.state === "checking"}
          <span class="inline-flex items-center gap-1.5 text-sm">
            <LoaderCircleIcon class="size-4 animate-spin" />
            Перевірка…
          </span>
        {:else if status.state === "up-to-date"}
          <span class="inline-flex items-center gap-1.5 text-sm">
            <CircleCheckIcon class="text-emerald-500 size-4" />
            Встановлено актуальну версію
          </span>
        {:else if status.state === "available"}
          <span class="inline-flex items-center gap-1.5 text-sm">
            <CircleArrowUpIcon class="text-primary size-4" />
            Доступна версія {status.version}
          </span>
        {:else if status.state === "error"}
          <span class="inline-flex items-center gap-1.5 text-sm">
            <TriangleAlertIcon class="text-amber-500 size-4" />
            Не вдалося перевірити оновлення
          </span>
        {/if}

        {#if checkedAt && (status.state === "up-to-date" || status.state === "available")}
          <span class="text-muted-foreground text-xs">
            {status.state === "up-to-date"
              ? `Остання перевірка: ${checkedAt}`
              : `Нову версію знайдено о ${checkedAt}`}
          </span>
        {/if}
      </div>

      <Button
        bind:ref={updateButton}
        class="mt-4 w-full"
        variant={status.state === "available" ? "default" : "secondary"}
        disabled={status.state === "checking"}
        onclick={primaryAction}
      >
        {buttonLabel}
      </Button>
    </div>
  </main>
</div>
