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

<div class="mt-4 border border-border rounded bg-card">
  <div class="p-4">
    <p class="text-xl font-medium">Документи</p>

    {#if documentsQuery.isPending}
      <div class="mt-4 flex flex-col gap-2">
        <Skeleton class="h-12 w-full" />
        <Skeleton class="h-12 w-full" />
      </div>
    {:else if documentsQuery.isError}
      <p class="mt-4 text-sm text-destructive">
        Не вдалося завантажити документи:
        {documentsQuery.error?.message}
      </p>
    {:else}
      <div
        class="mt-3 flex flex-col divide-y divide-border rounded-md border border-border"
      >
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
</div>
