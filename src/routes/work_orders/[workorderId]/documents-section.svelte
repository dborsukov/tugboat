<script lang="ts">
  import { useQueryClient } from "@tanstack/svelte-query";
  import { toast } from "svelte-sonner";

  import { Skeleton } from "$lib/components/ui/skeleton";

  import type {
    DocumentFormat,
    DocumentKind,
    GenerateDocumentOptions,
    WorkOrderDocumentVersion,
  } from "$lib/bindings";
  import { fileNameOf } from "$lib/tools";

  import {
    createDownloadWorkOrderDocumentMutation,
    createWorkOrderDocumentsQuery,
  } from "../queries";
  import DocumentRow from "./document-row.svelte";

  let {
    workOrderId,
    canGenerate = true,
    onGenerate,
  }: {
    workOrderId: string;
    /** False for an archived order, which may not be written to. */
    canGenerate?: boolean;
    onGenerate: (
      kind: DocumentKind,
      seed: GenerateDocumentOptions | null,
    ) => void;
  } = $props();

  const queryClient = useQueryClient();

  const documentsQuery = createWorkOrderDocumentsQuery(() => workOrderId);
  const downloadMutation = createDownloadWorkOrderDocumentMutation(queryClient);

  const documents = $derived(documentsQuery.data?.documents ?? []);

  async function download(
    version: WorkOrderDocumentVersion,
    format: DocumentFormat,
    file_name: string,
  ) {
    const saved = await downloadMutation
      .mutateAsync({ id: version.id, format, fileName: file_name })
      .catch(() => undefined);
    // `null` means the save dialog was dismissed, which is not a failure.
    if (!saved) return;
    toast.success("Документ збережено", {
      description: fileNameOf(saved),
      position: "bottom-left",
    });
  }
</script>

<div class="mt-4 border border-border rounded bg-card overflow-hidden">
  <div class="flex min-h-12 items-center border-b border-border px-4 py-2">
    <p class="text-xl font-medium mb-0.5">Документи</p>
  </div>

  {#if documentsQuery.isPending}
    <div class="flex flex-col gap-2 px-4 py-3">
      <Skeleton class="h-12 w-full" />
      <Skeleton class="h-12 w-full" />
    </div>
  {:else if documentsQuery.isError}
    <p class="px-4 py-3 text-sm text-destructive">
      Не вдалося завантажити документи:
      {documentsQuery.error?.message}
    </p>
  {:else}
    <div class="flex flex-col divide-y divide-border">
      {#each documents as document (document.kind)}
        <DocumentRow
          {document}
          {workOrderId}
          {canGenerate}
          downloading={downloadMutation.isPending}
          {onGenerate}
          onDownload={download}
        />
      {/each}
    </div>
  {/if}
</div>
