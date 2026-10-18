<script lang="ts">
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";

  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  import type { Report } from "$lib/bindings";
  import { isoDayOf, pluralizeUk, prettifyIsoDateString } from "$lib/tools";
  import { cn } from "$lib/utils";

  import {
    dayLabel,
    DEFAULT_REPORT_KIND,
    fileMissing,
    needsRefresh,
    rangeTitle,
    reportFileName,
    reportKindChips,
    shortRange,
  } from "../period";

  let {
    report,
    /** Older versions of this period, newest first. */
    previous = [],
    refreshing = false,
    busy = false,
    downloading = false,
    highlighted = false,
    onDownload,
    onRefresh,
  }: {
    report: Report;
    previous?: Report[];
    refreshing?: boolean;
    /** A generation is running somewhere, wait. */
    busy?: boolean;
    downloading?: boolean;
    highlighted?: boolean;
    onDownload: (report: Report, fileName: string) => void;
    onRefresh: (report: Report) => void;
  } = $props();

  /** How many earlier versions the disclosure shows before trailing off. */
  const VISIBLE_VERSIONS = 3;

  let expanded = $state(false);
  let showAll = $state(false);

  const range = $derived({ from: report.periodStart, to: report.periodEnd });

  const title = $derived(rangeTitle(range));
  const preciseRange = $derived(shortRange(range));

  const madeOn = $derived(isoDayOf(report.generatedAt));

  /* Whether the period was still running when a given version was written */
  function writtenBeforeEnd(value: Report): boolean {
    return isoDayOf(value.generatedAt) < report.periodEnd;
  }

  const unfinished = $derived(writtenBeforeEnd(report));

  const shownVersions = $derived(
    showAll ? previous : previous.slice(0, VISIBLE_VERSIONS),
  );
  const hiddenCount = $derived(previous.length - shownVersions.length);

  const versionsNote = $derived(
    `Версія ${previous.length + 1} · ${previous.length} ${pluralizeUk(previous.length, ["попередня", "попередні", "попередніх"])}`,
  );

  function versionNumber(index: number): number {
    return previous.length - index;
  }

  function stamp(report: Report): string {
    return prettifyIsoDateString(report.generatedAt, "datetime-short");
  }

  function fullStamp(report: Report): string {
    return prettifyIsoDateString(report.generatedAt, "datetime");
  }
</script>

<!-- The id is how the page finds a row to scroll to -->
<div
  id="report-row-{report.id}"
  class={cn("border-b border-border", highlighted && "report-row-flash")}
>
  <div
    class={cn(
      "flex min-h-16 gap-3 px-4 py-2 transition-colors hover:bg-muted/50",
      previous.length > 0 ? "items-start" : "items-center",
    )}
  >
    <span class="flex h-6 w-5 shrink-0 items-center">
      <FileSpreadsheetIcon
        class="size-5 text-emerald-600 dark:text-emerald-400"
      />
    </span>

    <div class="flex min-w-0 flex-1 flex-col gap-1">
      <div class="flex min-h-6 min-w-0 flex-wrap items-center gap-2">
        <Tooltip.Root>
          <Tooltip.Trigger
            class="min-w-0 cursor-default truncate text-left font-medium"
          >
            {title}
          </Tooltip.Trigger>
          <Tooltip.Content>{preciseRange}</Tooltip.Content>
        </Tooltip.Root>

        {#if unfinished}
          <Tooltip.Root>
            <Tooltip.Trigger
              class="cursor-default text-left text-sm text-muted-foreground"
            >
              (період триває)
            </Tooltip.Trigger>
            <Tooltip.Content>Дані станом на {dayLabel(madeOn)}</Tooltip.Content>
          </Tooltip.Root>
        {/if}

        {#if report.reportType !== DEFAULT_REPORT_KIND}
          <Badge variant="secondary">
            {reportKindChips[report.reportType]}
          </Badge>
        {/if}

        {#if report.isStale}
          <Badge variant="warning" title="Дані змінилися після створення звіту">
            <TriangleAlertIcon />
            дані змінились
          </Badge>
        {/if}

        {#if fileMissing(report)}
          <Badge variant="destructive">файл видалено</Badge>
        {/if}
      </div>

      {#if previous.length > 0}
        <Button
          variant="link"
          size="sm"
          class="-ms-2 h-6 self-start gap-1 px-2 text-xs font-normal text-blue-800/70 dark:text-blue-300/70"
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

    <span
      class="flex h-6 w-32 shrink-0 items-center text-sm text-muted-foreground tabular-nums"
      title={fullStamp(report)}
    >
      {stamp(report)}
    </span>

    <span class="flex h-6 w-36 shrink-0 items-center justify-end gap-1">
      {#if needsRefresh(report)}
        <Button
          variant="outline"
          size="sm"
          disabled={busy}
          title="Створити нову версію за цей період"
          onclick={() => onRefresh(report)}
        >
          <RotateCwIcon class={cn(refreshing && "animate-spin")} />
          Оновити
        </Button>
      {/if}

      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Зберегти звіт"
        title="Зберегти звіт"
        disabled={downloading || fileMissing(report)}
        onclick={() =>
          onDownload(
            report,
            reportFileName(report.reportType, range, 0, unfinished),
          )}
      >
        <DownloadIcon />
      </Button>
    </span>
  </div>

  {#if expanded}
    <div class="relative">
      <span
        aria-hidden="true"
        class="absolute inset-y-0 left-6.5 w-px bg-border"
      ></span>

      {#each shownVersions as version, index (version.id)}
        <div
          class="flex items-center gap-3 px-4 py-1.5 transition-colors hover:bg-muted/50"
        >
          <span class="w-5 shrink-0"></span>

          <span class="flex min-w-0 flex-1 items-center gap-2">
            <span class="truncate text-xs text-muted-foreground">
              Версія {versionNumber(index)}
            </span>
            {#if fileMissing(version)}
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

          <span class="flex w-36 shrink-0 items-center justify-end">
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Зберегти версію {versionNumber(index)}"
              title="Зберегти цю версію"
              disabled={downloading || fileMissing(version)}
              onclick={() =>
                onDownload(
                  version,
                  reportFileName(
                    report.reportType,
                    range,
                    index + 1,
                    writtenBeforeEnd(version),
                  ),
                )}
            >
              <DownloadIcon />
            </Button>
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
