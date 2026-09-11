<script lang="ts">
  import { openPath } from "@tauri-apps/plugin-opener";
  import { toast } from "svelte-sonner";

  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import type { WorkOrderDocument } from "$lib/bindings";
  import { prettifyIsoDateString } from "$lib/tools";

  import { createWorkOrderDocumentsQuery } from "../queries";

  let { workOrderId }: { workOrderId: string } = $props();

  const documentsQuery = createWorkOrderDocumentsQuery(() => workOrderId);

  const files = $derived(documentsQuery.data?.files ?? []);
  const folderPath = $derived(documentsQuery.data?.folderPath);

  type DocFormat = "pdf" | "xlsx";

  function formatOf(file: WorkOrderDocument): DocFormat | null {
    const extension = file.name.split(".").pop()?.toLowerCase() ?? "";
    if (extension === "pdf" || extension === "xlsx") {
      return extension;
    }
    return null;
  }

  const FORMAT_LABELS: Record<DocFormat, string> = {
    pdf: "PDF",
    xlsx: "XLSX",
  };

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} Б`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} КБ`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} МБ`;
  }

  async function openFile(file: WorkOrderDocument) {
    try {
      await openPath(file.path);
    } catch (error) {
      toast.error("Не вдалося відкрити файл", {
        description: `${file.name} — ${String(error)}`,
        position: "bottom-left",
      });
    }
  }

  async function openFolder() {
    if (!folderPath) return;
    try {
      await openPath(folderPath);
    } catch (error) {
      toast.error("Не вдалося відкрити теку документів", {
        description: `${folderPath} — ${String(error)}`,
        position: "bottom-left",
      });
    }
  }
</script>

<div class="mt-4 border border-border rounded bg-card">
  <div class="p-4">
    <div class="flex flex-row items-center justify-between">
      <div class="flex flex-row items-baseline gap-2">
        <p class="text-xl font-medium">Документи</p>
        {#if documentsQuery.isSuccess}
          <span class="text-base font-normal text-muted-foreground">
            {files.length}
          </span>
        {/if}
      </div>
      <Button
        variant="outline"
        size="sm"
        onclick={openFolder}
        disabled={!folderPath}
      >
        <FolderOpenIcon />
        Відкрити теку
      </Button>
    </div>

    {#if documentsQuery.isPending}
      <div class="mt-4 flex flex-col gap-2">
        <Skeleton class="h-10 w-full" />
        <Skeleton class="h-10 w-full" />
      </div>
    {:else if documentsQuery.isError}
      <p class="mt-4 text-sm text-destructive">
        Не вдалося завантажити документи:
        {documentsQuery.error?.message}
      </p>
    {:else if files.length === 0}
      <p class="mt-4 text-sm text-muted-foreground">
        Документів ще немає. Згенеруйте їх через меню ⋯ → Документи.
      </p>
    {:else}
      <div
        class="mt-2 flex flex-col divide-y divide-border rounded-md border border-border"
      >
        {#each files as file (file.path)}
          {@const fileFormat = formatOf(file)}
          <button
            type="button"
            class="flex items-center gap-3 px-4 py-2 text-left hover:bg-muted/40"
            onclick={() => openFile(file)}
            title="Відкрити файл"
          >
            {#if fileFormat === "xlsx"}
              <FileSpreadsheetIcon
                class="size-5 shrink-0 text-emerald-600 dark:text-emerald-400"
              />
            {:else}
              <FileTextIcon
                class="size-5 shrink-0 text-red-600 dark:text-red-400"
              />
            {/if}
            <span class="min-w-0 flex-1 truncate">{file.name}</span>
            {#if fileFormat}
              <Badge variant="outline">{FORMAT_LABELS[fileFormat]}</Badge>
            {/if}
            <span
              class="w-16 shrink-0 text-right text-sm text-muted-foreground tabular-nums"
            >
              {formatSize(file.size)}
            </span>
            <span
              class="shrink-0 whitespace-nowrap text-right text-sm text-muted-foreground tabular-nums"
            >
              {prettifyIsoDateString(file.modifiedAt, "datetime")}
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>
