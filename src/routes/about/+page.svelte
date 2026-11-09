<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  import XIcon from "@lucide/svelte/icons/x";

  import { Button } from "$lib/components/ui/button";

  import PopeyeIcon from "$lib/components/icons/popeye.svelte";

  import { commands } from "$lib/bindings";
  import { isMacOS } from "$lib/platform";
  import { checkForUpdates } from "$lib/updates";

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
  const versionLabel = $derived(commit ? `${version} [${commit}]` : version);

  async function openLogs() {
    const result = await commands.openLogsDir();
    if (result.status === "error") {
      toast.error("Не вдалося відкрити теку журналів", {
        description: result.error.message,
        position: "bottom-left",
      });
    }
  }

  async function copyDiagnostics() {
    // Written to the clipboard in Rust: the webview has no secure context, and
    // the browser fallback loses the user gesture once the report is built.
    const result = await commands.copyDiagnostics();
    if (result.status === "error") {
      toast.error("Не вдалося скопіювати діагностику", {
        description: result.error.message,
        position: "bottom-left",
      });
      return;
    }
    toast.success("Діагностику скопійовано", { position: "bottom-left" });
  }

  onMount(() => {
    document.getElementById("boot-loader")?.remove();

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
      <p class="text-sm text-muted-foreground">Версія {versionLabel}</p>
    </div>

    <div class="flex w-full flex-col gap-2">
      <Button class="w-full" variant="outline" size="sm" onclick={openLogs}>
        Відкрити журнали
      </Button>
      <Button
        class="w-full"
        variant="outline"
        size="sm"
        onclick={copyDiagnostics}
      >
        Скопіювати діагностику
      </Button>
      <Button
        class="w-full"
        variant="outline"
        size="sm"
        onclick={() => void checkForUpdates({ silent: false })}
      >
        Перевірити оновлення
      </Button>
    </div>
  </main>
</div>
