<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import DownloadIcon from "@lucide/svelte/icons/download";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import TrashIcon from "@lucide/svelte/icons/trash";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";

  import DeleteDialog from "$lib/components/delete-dialog.svelte";

  import type { WorkOrderDocument } from "$lib/bindings";
  import { prettifyIsoDateString } from "$lib/tools";

  import {
    createDeleteWorkOrderDocumentMutation,
    createDownloadWorkOrderDocumentMutation,
    createWorkOrderDocumentsQuery,
  } from "../queries";

  let { workOrderId }: { workOrderId: string } = $props();

  const queryClient = useQueryClient();

  const documentsQuery = createWorkOrderDocumentsQuery(() => workOrderId);
  const downloadMutation = createDownloadWorkOrderDocumentMutation(queryClient);
  const deleteMutation = createDeleteWorkOrderDocumentMutation(queryClient);

  const files = $derived(documentsQuery.data?.files ?? []);

  /** The document awaiting delete confirmation, if any. */
  let pendingDelete = $state<WorkOrderDocument | null>(null);

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

  async function download(file: WorkOrderDocument) {
    const saved = await downloadMutation
      .mutateAsync({ workOrderId, fileName: file.name })
      .catch(() => undefined);
    // `null` means the save dialog was dismissed, which is not a failure.
    if (!saved) return;
    toast.success("Документ збережено", {
      description: saved,
      position: "bottom-left",
    });
  }

  async function confirmDelete() {
    const file = pendingDelete;
    if (!file) return;
    pendingDelete = null;
    await deleteMutation
      .mutateAsync({ workOrderId, fileName: file.name })
      .catch(() => {});
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
          <div class="flex items-center gap-3 px-4 py-2">
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
            <div class="flex shrink-0 items-center gap-1">
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Зберегти документ"
                title="Зберегти документ"
                disabled={downloadMutation.isPending}
                onclick={() => download(file)}
              >
                <DownloadIcon />
              </Button>
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Видалити документ"
                title="Видалити документ"
                disabled={deleteMutation.isPending}
                onclick={() => (pendingDelete = file)}
              >
                <TrashIcon class="text-muted-foreground" />
              </Button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<DeleteDialog
  open={pendingDelete !== null}
  onOpenChange={(value) => {
    if (!value) pendingDelete = null;
  }}
  onDelete={confirmDelete}
  title="Видалити документ"
  description={pendingDelete
    ? `Файл «${pendingDelete.name}» буде видалено безповоротно.`
    : null}
/>
