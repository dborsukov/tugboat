<script lang="ts">
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import FileCheckIcon from "@lucide/svelte/icons/file-check-2";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import FileTextIcon from "@lucide/svelte/icons/file-text";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  import type {
    DocumentFormat,
    GenerateDocumentOptions,
    WorkOrderDocument,
    WorkOrderDocumentVersion,
  } from "$lib/bindings";
  import { pluralizeUk, prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  let {
    document: doc,
    workOrderId,
    canGenerate = true,
    downloading = false,
    onGenerate,
    onDownload,
  }: {
    document: WorkOrderDocument;
    workOrderId: string;
    /** False for an archived order, which may not be written to. */
    canGenerate?: boolean;
    downloading?: boolean;
    onGenerate: (
      kind: WorkOrderDocument["kind"],
      seed: GenerateDocumentOptions | null,
    ) => void;
    onDownload: (
      version: WorkOrderDocumentVersion,
      format: DocumentFormat,
      fileName: string,
    ) => void;
  } = $props();

  /** How many earlier versions the disclosure shows before trailing off. */
  const VISIBLE_VERSIONS = 3;

  let expanded = $state(false);
  let showAll = $state(false);

  const current = $derived(doc.versions[0] ?? null);
  const previous = $derived(doc.versions.slice(1));
  const total = $derived(doc.versions.length);

  const shownVersions = $derived(
    showAll ? previous : previous.slice(0, VISIBLE_VERSIONS),
  );
  const hiddenCount = $derived(previous.length - shownVersions.length);

  const versionsNote = $derived(
    `Версія ${total} · ${previous.length} ${pluralizeUk(previous.length, ["попередня", "попередні", "попередніх"])}`,
  );

  const currentIsMissingFiles = $derived(hasMissingFiles(current));
  const currentNeedsRefresh = $derived(
    current != null && (current.isStale || hasMissingFiles(current)),
  );

  function hasMissingFiles(version: WorkOrderDocumentVersion | null): boolean {
    return (
      version != null && version.files.some((file) => file.fileSize === null)
    );
  }

  function versionNumber(index: number): number {
    return total - index;
  }

  function stamp(version: WorkOrderDocumentVersion): string {
    return prettifyIsoDateString(version.generatedAt, "datetime-short");
  }

  function fullStamp(version: WorkOrderDocumentVersion): string {
    return prettifyIsoDateString(version.generatedAt, "datetime");
  }

  function fileLabel(format: DocumentFormat): string {
    return format === "pdf" ? "PDF" : "XLSX";
  }

  function saveLabel(format: DocumentFormat, number: number | null): string {
    const label = `Зберегти ${fileLabel(format)}`;
    return number === null ? label : `${label} версії ${number}`;
  }

  /** `Наряд-замовлення_202608-002.pdf`, with the version when it is not current. */
  function suggestedFileName(
    format: DocumentFormat,
    number: number | null,
  ): string {
    const suffix = number === null ? "" : `_v${number}`;
    return `${doc.title}_${workOrderId}${suffix}.${format}`;
  }

  function download(
    version: WorkOrderDocumentVersion,
    format: DocumentFormat,
    number: number | null,
  ) {
    onDownload(version, format, suggestedFileName(format, number));
  }
</script>

{#snippet downloadSplit(
  version: WorkOrderDocumentVersion,
  number: number | null,
)}
  {@const pdf = version.files.find((file) => file.format === "pdf")}
  <div class="flex items-center">
    <Button
      variant="outline"
      size="icon-sm"
      class="rounded-e-none"
      aria-label={saveLabel("pdf", number)}
      title={saveLabel("pdf", number)}
      disabled={downloading || pdf == null || pdf.fileSize === null}
      onclick={() => download(version, "pdf", number)}
    >
      <DownloadIcon />
    </Button>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="outline"
            size="icon-sm"
            class="-ms-px rounded-s-none"
            aria-label="Зберегти в іншому форматі"
            title="Зберегти в іншому форматі"
          >
            <ChevronDownIcon />
          </Button>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content class="w-max whitespace-nowrap" align="end">
        <DropdownMenu.Label>Зберегти як</DropdownMenu.Label>
        {#each version.files as file (file.format)}
          <DropdownMenu.Item
            disabled={file.fileSize === null}
            onclick={() => download(version, file.format, number)}
          >
            {#if file.format === "pdf"}
              <FileTextIcon />
            {:else}
              <FileSpreadsheetIcon />
            {/if}
            {fileLabel(file.format)}
          </DropdownMenu.Item>
        {/each}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </div>
{/snippet}

<div class="flex flex-col">
  <div
    class="flex min-h-18 items-center gap-3 px-4 py-3 transition-colors hover:bg-muted/50"
  >
    <span
      class={cn(
        "flex h-6 w-5 shrink-0 items-center",
        current == null ? "text-muted-foreground/60" : "text-muted-foreground",
      )}
    >
      {#if doc.kind === "naryad_zamovlennya"}
        <FileTextIcon class="size-5" />
      {:else}
        <FileCheckIcon class="size-5" />
      {/if}
    </span>

    <div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-2 gap-y-1">
      <span
        class={cn(
          "truncate font-semibold",
          current == null && "text-muted-foreground",
        )}>{doc.title}</span
      >

      {#if current == null}
        <span class="text-sm text-muted-foreground">ще не створено</span>
      {:else}
        {#if current.isStale}
          <Badge
            variant="warning"
            title="Дані змінилися після створення документа"
          >
            <TriangleAlertIcon />
            дані змінились
          </Badge>
        {/if}
        {#if currentIsMissingFiles}
          <Badge variant="destructive">файл видалено</Badge>
        {/if}
      {/if}

      {#if previous.length > 0}
        <Button
          variant="link"
          size="sm"
          class="h-6 shrink-0 gap-1 px-1 text-xs font-normal text-blue-800/70 dark:text-blue-300/70"
          aria-expanded={expanded}
          onclick={() => (expanded = !expanded)}
        >
          {versionsNote}
          <ChevronDownIcon
            class={cn(
              "size-3.5 transition-transform",
              expanded && "rotate-180",
            )}
          />
        </Button>
      {/if}
    </div>

    {#if current}
      <span
        class="flex h-6 w-32 shrink-0 items-center text-sm text-muted-foreground tabular-nums"
        title={fullStamp(current)}
      >
        {stamp(current)}
      </span>

      <span class="flex h-6 w-48 shrink-0 items-center justify-end gap-1">
        {#if currentNeedsRefresh && canGenerate}
          <Button
            variant="outline"
            size="sm"
            title="Створити нову версію з поточних даних"
            onclick={() => onGenerate(doc.kind, current.options)}
          >
            <RotateCwIcon />
            Оновити
          </Button>
        {/if}

        {@render downloadSplit(current, null)}
      </span>
    {:else if canGenerate}
      <Button
        variant="outline"
        size="sm"
        onclick={() => onGenerate(doc.kind, null)}
      >
        <PlusIcon />
        Створити
      </Button>
    {/if}
  </div>

  {#if expanded}
    <div class="relative pb-2">
      <span
        aria-hidden="true"
        class="absolute top-0 bottom-2 left-6.5 w-px bg-border"
      ></span>

      {#each shownVersions as version, index (version.id)}
        <div
          class="flex items-center gap-3 px-4 py-1.5 transition-colors hover:bg-muted/50"
        >
          <span class="w-5 shrink-0"></span>

          <span class="flex min-w-0 flex-1 items-center gap-2">
            <span class="truncate text-xs text-muted-foreground">
              Версія {versionNumber(index + 1)}
            </span>
            {#if hasMissingFiles(version)}
              <Badge variant="destructive" title="Файл цієї версії видалено">
                файл видалено
              </Badge>
            {/if}
          </span>

          <span
            class="w-32 shrink-0 text-xs text-muted-foreground tabular-nums"
            title={fullStamp(version)}
          >
            {stamp(version)}
          </span>

          <span class="flex w-48 shrink-0 items-center justify-end gap-1">
            {@render downloadSplit(version, versionNumber(index + 1))}
          </span>
        </div>
      {/each}

      {#if hiddenCount > 0}
        <div class="flex items-center gap-3 px-4 py-1.5">
          <span class="w-5 shrink-0"></span>
          <Button
            variant="link"
            size="sm"
            class="h-6 px-0 text-xs font-normal text-blue-800/70 dark:text-blue-300/70"
            onclick={() => (showAll = true)}
          >
            ще {hiddenCount}
            {pluralizeUk(hiddenCount, ["версія", "версії", "версій"])}…
          </Button>
        </div>
      {/if}
    </div>
  {/if}
</div>
