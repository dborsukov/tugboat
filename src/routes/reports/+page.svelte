<script lang="ts">
  import { rankItem } from "@tanstack/match-sorter-utils";
  import { useQueryClient } from "@tanstack/svelte-query";
  import { tick } from "svelte";
  import { toast } from "svelte-sonner";
  import { SvelteSet } from "svelte/reactivity";

  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  import ArrowDownIcon from "@lucide/svelte/icons/arrow-down";
  import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
  import SortIcon from "@lucide/svelte/icons/arrow-up-down";
  import FileSpreadsheetIcon from "@lucide/svelte/icons/file-spreadsheet";
  import FunnelXIcon from "@lucide/svelte/icons/funnel-x";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RotateCwIcon from "@lucide/svelte/icons/rotate-cw";
  import SearchIcon from "@lucide/svelte/icons/search";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";

  import { Button } from "$lib/components/ui/button";
  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Spinner } from "$lib/components/ui/spinner";

  import type { FacetedFilterOptions } from "$lib/components/data-table-faceted-filter/faceted-filter-options";
  import FacetedFilter from "$lib/components/faceted-filter/faceted-filter.svelte";
  import FetchErrorDisplay from "$lib/components/fetch-error-display.svelte";
  import PageLayout from "$lib/components/page-layout.svelte";

  import type { Report, ReportKind } from "$lib/bindings";
  import { settingsState } from "$lib/states/settings.svelte";
  import { fileNameOf, pluralizeUk } from "$lib/tools";

  import GenerateReportDialog from "./components/generate-report-dialog.svelte";
  import PeriodPicker from "./components/period-picker.svelte";
  import RefreshAllDialog from "./components/refresh-all-dialog.svelte";
  import ReportRow from "./components/report-row.svelte";
  import VatPeriodsDialog from "./components/vat-periods-dialog.svelte";
  import {
    dayText,
    DEFAULT_REPORT_KIND,
    dottedDay,
    isReportKind,
    modeForRange,
    needsRefresh,
    parsePeriodParam,
    periodParam,
    rangeForMode,
    rangeTitle,
    REPORT_KINDS,
    reportKindLabels,
    reportKindTitles,
    shortRange,
    type DayRange,
    type PeriodMode,
  } from "./period";
  import {
    createDownloadReportMutation,
    createGenerateReportMutation,
    createOpenReportCopyMutation,
    createReportsQuery,
  } from "./queries";

  type SortOption =
    | "created-desc"
    | "created-asc"
    | "period-desc"
    | "period-asc";

  const SORT_LABELS: Record<SortOption, string> = {
    "created-desc": "Спочатку нові",
    "created-asc": "Спочатку старі",
    "period-desc": "Період: нові",
    "period-asc": "Період: старі",
  };

  const queryClient = useQueryClient();

  const reportsQuery = createReportsQuery();
  const generateMutation = createGenerateReportMutation(queryClient);
  const downloadMutation = createDownloadReportMutation(queryClient);
  const openMutation = createOpenReportCopyMutation(queryClient);

  const now = new Date();
  const todayText = dayText(now);
  const thisMonth = rangeForMode("month", todayText, { from: "", to: "" });

  /* Filters, seeded from the address bar */

  const linkedPeriod = parsePeriodParam(page.url.searchParams.get("period"));
  const linkedTypes = (page.url.searchParams.get("type") ?? "")
    .split(",")
    .filter(isReportKind);

  let listPeriod = $state<DayRange | null>(linkedPeriod);
  let listPeriodMode = $state<PeriodMode>(
    linkedPeriod ? modeForRange(linkedPeriod) : "month",
  );
  let listPeriodAnchor = $state<string>(linkedPeriod?.from ?? todayText);
  let listPeriodDraft = $state<DayRange>(linkedPeriod ?? thisMonth);

  let selectedTypes = $state<ReportKind[]>(linkedTypes);
  let searchQuery = $state("");
  let sortBy = $state<SortOption>("created-desc");

  const businessIsVatPayer = $derived(settingsState.isVatPayer);
  const availableKinds = $derived(
    REPORT_KINDS.filter((kind) => kind !== "vat" || businessIsVatPayer),
  );

  $effect(() => {
    if (!businessIsVatPayer && selectedTypes.includes("vat")) {
      selectedTypes = selectedTypes.filter((kind) => kind !== "vat");
    }
  });

  const searching = $derived(searchQuery.trim().length > 0);

  function syncUrl() {
    const params = new URLSearchParams();
    if (selectedTypes.length > 0) params.set("type", selectedTypes.join(","));
    if (listPeriod) params.set("period", periodParam(listPeriod));

    const search = params.toString();
    const query = search ? `?${search}` : "";
    if (page.url.search === query) return;

    goto(`/reports${query}`, {
      replaceState: true,
      noScroll: true,
      keepFocus: true,
    });
  }

  function applyListPeriod(next: DayRange | null) {
    listPeriod = next;
    syncUrl();
  }

  /* List */

  const reports = $derived(reportsQuery.data ?? []);

  function midSentence(label: string): string {
    return label.replace(/^([А-ЯЇІЄҐ])/, (letter) => letter.toLowerCase());
  }

  function reportKey(report: Report): string {
    return `${report.reportType}|${report.periodStart}|${report.periodEnd}`;
  }

  /**
   * Every version of every period, newest first.
   * The query already returns them newest first, so each group lands in order without another sort.
   */
  const versionsByKey = $derived.by(() => {
    const byKey = new Map<string, Report[]>();
    for (const report of reports) {
      const key = reportKey(report);
      const versions = byKey.get(key);
      if (versions) {
        versions.push(report);
      } else {
        byKey.set(key, [report]);
      }
    }
    return byKey;
  });

  const latest = $derived(
    [...versionsByKey.values()].map((versions) => versions[0]),
  );

  function previousVersions(report: Report): Report[] {
    return versionsByKey.get(reportKey(report))?.slice(1) ?? [];
  }

  /** Everything a search can match against, in one string. */
  function searchText(report: Report): string {
    const range = { from: report.periodStart, to: report.periodEnd };
    return [
      reportKindTitles[report.reportType],
      rangeTitle(range),
      shortRange(range),
      report.periodStart,
      report.periodEnd,
      dottedDay(report.periodStart),
      dottedDay(report.periodEnd),
    ].join(" ");
  }

  const filtered = $derived.by(() => {
    const query = searchQuery.trim();

    return latest.filter((report) => {
      if (
        selectedTypes.length > 0 &&
        !selectedTypes.includes(report.reportType)
      ) {
        return false;
      }
      // A report is in the period when its own span overlaps it.
      if (
        listPeriod &&
        (report.periodEnd < listPeriod.from ||
          report.periodStart > listPeriod.to)
      ) {
        return false;
      }
      if (query && !rankItem(searchText(report), query).passed) return false;
      return true;
    });
  });

  /* Sorting */

  const sortByPeriod = $derived(sortBy.startsWith("period"));
  const sortAscending = $derived(sortBy.endsWith("asc"));

  const sorted = $derived.by(() => {
    const list = [...filtered];
    const newer = (a: string, b: string) => b.localeCompare(a);

    return list.sort((a, b) => {
      switch (sortBy) {
        case "created-asc":
          return a.generatedAt.localeCompare(b.generatedAt);
        case "period-desc":
          return (
            newer(a.periodStart, b.periodStart) ||
            newer(a.periodEnd, b.periodEnd) ||
            newer(a.generatedAt, b.generatedAt)
          );
        case "period-asc":
          return (
            a.periodStart.localeCompare(b.periodStart) ||
            a.periodEnd.localeCompare(b.periodEnd) ||
            newer(a.generatedAt, b.generatedAt)
          );
        default:
          return newer(a.generatedAt, b.generatedAt);
      }
    });
  });

  /* Type filter */

  const typeFilterOptions = $derived<FacetedFilterOptions[]>(
    availableKinds.map((kind) => ({
      label: reportKindLabels[kind],
      value: kind,
      icon: FileSpreadsheetIcon,
    })),
  );

  const typeFacets = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const report of latest) {
      counts.set(report.reportType, (counts.get(report.reportType) ?? 0) + 1);
    }
    return counts;
  });

  const selectedTypeValues = $derived(new SvelteSet<string>(selectedTypes));

  const isFiltered = $derived(
    searching || selectedTypes.length > 0 || listPeriod !== null,
  );

  function clearFilters() {
    searchQuery = "";
    selectedTypes = [];
    listPeriod = null;
    syncUrl();
  }

  /* Reports in view that need to be regenerated */
  const toRefresh = $derived(filtered.filter(needsRefresh));

  /* Actions */

  /* What is generating right now */
  let generatingId = $state<number | "new" | "all" | null>(null);

  const isGenerating = $derived(generatingId !== null);

  function announce(
    kind: ReportKind,
    period: DayRange,
    created: boolean,
    createdTitle: string,
  ) {
    const description = `${reportKindTitles[kind]} · ${shortRange(period)}`;
    if (created) {
      toast.success(createdTitle, { description, position: "bottom-left" });
      return;
    }
    toast.info("Дані не змінились, звіт актуальний", {
      description,
      position: "bottom-left",
    });
  }

  let highlightedId = $state<number | null>(null);
  let highlightTimer: ReturnType<typeof setTimeout> | undefined;

  function highlight(id: number) {
    highlightedId = id;
    clearTimeout(highlightTimer);
    // Outlasts the `report-row-flash` animation, three flashes of 0.6s
    highlightTimer = setTimeout(() => (highlightedId = null), 1900);
  }

  /* Brings a row into view and flashes it. */
  function focusRow(id: number) {
    highlight(id);
    document
      .getElementById(`report-row-${id}`)
      ?.scrollIntoView({ block: "center", behavior: "smooth" });
  }

  let answeredLink = false;
  $effect(() => {
    if (answeredLink || !linkedPeriod || !reportsQuery.isSuccess) return;
    answeredLink = true;

    const linked = sorted.find(
      (report) =>
        report.periodStart === linkedPeriod.from &&
        report.periodEnd === linkedPeriod.to,
    );
    if (linked) focusRow(linked.id);
  });

  async function generate(
    reportType: ReportKind,
    period: DayRange,
    reveal = false,
  ): Promise<boolean> {
    generatingId = "new";
    try {
      const outcome = await generateMutation.mutateAsync({
        reportType,
        periodStart: period.from,
        periodEnd: period.to,
      });

      announce(reportType, period, outcome.created, "Звіт створено");

      if (reveal) clearFilters();

      // Wait for the refreshed list, so the row is there to scroll to.
      await reportsQuery.refetch();
      await tick();
      focusRow(outcome.report.id);

      return true;
    } catch {
      return false;
    } finally {
      generatingId = null;
    }
  }

  function generateForDialog(reportType: ReportKind, period: DayRange) {
    return generate(reportType, period, true);
  }

  async function refresh(report: Report) {
    const period = { from: report.periodStart, to: report.periodEnd };
    generatingId = report.id;
    await generateMutation
      .mutateAsync({
        reportType: report.reportType,
        periodStart: period.from,
        periodEnd: period.to,
      })
      .then((outcome) => {
        announce(report.reportType, period, outcome.created, "Звіт оновлено");
        highlight(outcome.report.id);
      })
      .catch(() => {})
      .finally(() => {
        generatingId = null;
      });
  }

  let refreshAllOpen = $state(false);

  /* Rewrites every stale report, one after another */
  async function refreshAll() {
    const targets = toRefresh;
    generatingId = "all";
    let created = 0;
    let unchanged = 0;

    for (const report of targets) {
      const outcome = await generateMutation
        .mutateAsync({
          reportType: report.reportType,
          periodStart: report.periodStart,
          periodEnd: report.periodEnd,
        })
        .catch(() => undefined);
      if (!outcome) continue;
      if (outcome.created) {
        created += 1;
      } else {
        unchanged += 1;
      }
    }

    generatingId = null;

    if (created === 0) {
      toast.info("Дані не змінились, звіти актуальні", {
        position: "bottom-left",
      });
      return;
    }

    toast.success(
      `Оновлено ${created} ${pluralizeUk(created, ["звіт", "звіти", "звітів"])}`,
      {
        description: unchanged > 0 ? `${unchanged} без змін` : undefined,
        position: "bottom-left",
      },
    );
  }

  async function download(report: Report, fileName: string) {
    const saved = await downloadMutation
      .mutateAsync({ id: report.id, fileName })
      .catch(() => undefined);
    if (!saved) return;
    toast.success("Звіт збережено", {
      description: fileNameOf(saved),
      position: "bottom-left",
      action: {
        label: "Відкрити",
        onClick: () => openMutation.mutate(saved),
      },
    });
  }

  /* The creation dialog */

  let createOpen = $state(false);
  let vatPeriodsOpen = $state(false);

  const createType = $derived<ReportKind>(
    selectedTypes.length === 1 ? selectedTypes[0] : DEFAULT_REPORT_KIND,
  );

  async function quickGenerate() {
    if (!listPeriod) return;
    await generate(createType, listPeriod);
  }
</script>

<PageLayout>
  {#snippet header()}
    <div class="flex h-full min-w-0 items-center gap-2">
      <InputGroup.Root class="max-w-sm flex-1">
        <InputGroup.Addon>
          <SearchIcon class="size-4 shrink-0 opacity-50" />
        </InputGroup.Addon>
        <InputGroup.Input
          bind:value={searchQuery}
          placeholder="Пошук звітів..."
          aria-label="Пошук звітів"
          disabled={!reportsQuery.isSuccess}
        />
      </InputGroup.Root>

      <PeriodPicker
        bind:mode={listPeriodMode}
        bind:anchor={listPeriodAnchor}
        bind:custom={listPeriodDraft}
        active={listPeriod !== null}
        disabled={!reportsQuery.isSuccess}
        onApply={applyListPeriod}
        onClear={() => applyListPeriod(null)}
      />

      <FacetedFilter
        title="Тип звіту"
        options={typeFilterOptions}
        selected={selectedTypeValues}
        facets={typeFacets}
        disabled={!reportsQuery.isSuccess}
        onValueChange={(values) => {
          selectedTypes = values as ReportKind[];
          syncUrl();
        }}
      />

      <Select.Root
        type="single"
        value={sortBy}
        onValueChange={(value) => (sortBy = value as SortOption)}
      >
        <Select.Trigger class="w-44 shrink-0">
          <SortIcon />
          {SORT_LABELS[sortBy]}
        </Select.Trigger>
        <Select.Content>
          {#each Object.entries(SORT_LABELS) as [value, label] (value)}
            <Select.Item {value}>{label}</Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>

      {#if isFiltered}
        <Button
          variant="ghost"
          class="shrink-0"
          disabled={!reportsQuery.isSuccess}
          onclick={clearFilters}
        >
          <FunnelXIcon />
          Прибрати фільтри
        </Button>
      {/if}

      {#if businessIsVatPayer}
        <Button
          variant="outline"
          class="ms-auto shrink-0"
          onclick={() => (vatPeriodsOpen = true)}
        >
          Періоди ПДВ
        </Button>
      {/if}

      <Button
        class="shrink-0 {businessIsVatPayer ? '' : 'ms-auto'}"
        onclick={() => (createOpen = true)}
      >
        <PlusIcon />
        Створити звіт
      </Button>
    </div>
  {/snippet}

  {#if reportsQuery.isError}
    <FetchErrorDisplay
      message={reportsQuery.error?.message}
      onRetry={reportsQuery.refetch}
      class="mt-10"
    />
  {:else if reportsQuery.isLoading}
    <Skeleton class="mt-4 h-50 w-full rounded" />
  {:else}
    <div class="flex flex-col gap-3 pt-4">
      {#if toRefresh.length > 0}
        <div
          class="flex items-center gap-2 rounded border border-amber-500/25 bg-amber-500/10 px-3 py-2"
        >
          <TriangleAlertIcon
            class="size-4 shrink-0 text-amber-600 dark:text-amber-500"
          />
          <p class="min-w-0 flex-1 text-sm text-amber-700 dark:text-amber-400">
            {toRefresh.length}
            {pluralizeUk(toRefresh.length, [
              "звіт потребує",
              "звіти потребують",
              "звітів потребують",
            ])}
            оновлення.
          </p>
          <Button
            variant="outline"
            size="sm"
            disabled={isGenerating}
            onclick={() => (refreshAllOpen = true)}
          >
            {#if generatingId === "all"}
              <Spinner />
            {:else}
              <RotateCwIcon />
            {/if}
            Оновити всі
          </Button>
        </div>
      {/if}

      <!--
        `overflow-clip` rather than `overflow-hidden`: hidden would make this card
        the scroll container, and the column headings below could then never stick
        to anything. Clip still rounds the corners off without trapping them.
      -->
      <div class="overflow-clip rounded border border-border bg-card">
        {#if sorted.length === 0}
          <div class="flex flex-col items-center gap-3 px-4 py-10">
            <p class="max-w-md text-center text-sm text-muted-foreground">
              {#if searching}
                За запитом «{searchQuery.trim()}» звітів не знайдено.
              {:else if listPeriod}
                За {midSentence(rangeTitle(listPeriod))} звітів ще немає.
              {:else if selectedTypes.length > 0}
                Звітів обраного типу ще немає.
              {:else}
                Звітів ще немає. Створіть перший кнопкою «Створити звіт».
              {/if}
            </p>

            <div class="flex flex-wrap items-center justify-center gap-2">
              {#if listPeriod && !searching}
                <Button
                  variant="outline"
                  disabled={isGenerating}
                  onclick={quickGenerate}
                >
                  {#if generatingId === "new"}<Spinner />{/if}
                  Створити звіт
                </Button>
              {/if}
              {#if isFiltered}
                <Button variant="ghost" onclick={clearFilters}>
                  Показати всі звіти
                </Button>
              {/if}
            </div>
          </div>
        {:else}
          <!--
            `-mt-px` lays the heading's top border exactly over the card's own, so
            the card keeps its rounded corners while the heading keeps a top edge
            of its own to show once it has stuck under the page header and the
            card's border has scrolled away behind it.
          -->
          <div
            class="sticky top-0 z-10 -mt-px flex items-center gap-3 border-y border-border bg-card px-4 py-2 text-sm text-muted-foreground"
          >
            <span class="w-5 shrink-0"></span>
            <span class="flex min-w-0 flex-1 items-center gap-1">
              Звіт
              {#if sortByPeriod}
                {#if sortAscending}
                  <ArrowUpIcon class="size-3" />
                {:else}
                  <ArrowDownIcon class="size-3" />
                {/if}
              {/if}
            </span>
            <span class="flex w-32 shrink-0 items-center gap-1">
              Створено
              {#if !sortByPeriod}
                {#if sortAscending}
                  <ArrowUpIcon class="size-3" />
                {:else}
                  <ArrowDownIcon class="size-3" />
                {/if}
              {/if}
            </span>
            <span class="w-36 shrink-0"></span>
          </div>

          {#each sorted as report (report.id)}
            <ReportRow
              {report}
              previous={previousVersions(report)}
              refreshing={generatingId === report.id}
              busy={isGenerating}
              downloading={downloadMutation.isPending}
              highlighted={highlightedId === report.id}
              onDownload={download}
              onRefresh={refresh}
            />
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</PageLayout>

<RefreshAllDialog
  bind:open={refreshAllOpen}
  count={toRefresh.length}
  onConfirm={refreshAll}
/>

{#if businessIsVatPayer}
  <VatPeriodsDialog bind:open={vatPeriodsOpen} />
{/if}

<GenerateReportDialog
  bind:open={createOpen}
  {reports}
  defaultPeriod={listPeriod}
  defaultType={createType}
  onGenerate={generateForDialog}
/>
