<script lang="ts">
  import InfoIcon from "@lucide/svelte/icons/info";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Spinner } from "$lib/components/ui/spinner";

  import type { Report, ReportKind } from "$lib/bindings";
  import { settingsState } from "$lib/states/settings.svelte";
  import { cn } from "$lib/utils";

  import {
    dayText,
    DEFAULT_REPORT_KIND,
    modeForRange,
    needsRefresh,
    rangeForMode,
    REPORT_KINDS,
    reportKindTitles,
    shortDay,
    type DayRange,
    type PeriodMode,
  } from "../period";
  import PeriodPanel from "./period-panel.svelte";

  let {
    open = $bindable(false),
    reports = [],
    /* Period to open on */
    defaultPeriod = null,
    defaultType = DEFAULT_REPORT_KIND,
    onGenerate,
  }: {
    open: boolean;
    reports: Report[];
    defaultPeriod: DayRange | null;
    defaultType: ReportKind;
    onGenerate: (reportType: ReportKind, period: DayRange) => Promise<boolean>;
  } = $props();

  const todayText = dayText(new Date());

  const availableKinds = $derived(
    REPORT_KINDS.filter((kind) => kind !== "vat" || settingsState.isVatPayer),
  );

  let reportType = $state<ReportKind>(DEFAULT_REPORT_KIND);
  let mode = $state<PeriodMode>("month");
  let anchor = $state<string>(todayText);
  let custom = $state<DayRange>({ from: "", to: "" });
  let generating = $state(false);

  const range = $derived(rangeForMode(mode, anchor, custom));

  /** Every version already written for this type and period, newest first. */
  const versions = $derived(
    reports.filter(
      (report) =>
        report.reportType === reportType &&
        report.periodStart === range.from &&
        report.periodEnd === range.to,
    ),
  );

  const latest = $derived(versions[0]);
  const stale = $derived(latest ? needsRefresh(latest) : false);
  const ongoing = $derived(Boolean(range.to) && range.to >= todayText);

  const submitLabel = $derived(
    versions.length > 0
      ? `Створити версію ${versions.length + 1}`
      : "Створити звіт",
  );

  /** What the journal already holds for the chosen period. */
  const versionsNote = $derived.by(() => {
    if (versions.length === 0) return null;
    return stale
      ? "Звіт за цей період уже є, але дані змінилися."
      : "Звіт за цей період уже є і він актуальний.";
  });

  const ongoingNote = $derived.by(() => {
    if (!ongoing) return null;
    const asOf = shortDay(todayText);
    return versions.length > 0
      ? `Період триває: дані станом на ${asOf}.`
      : `Період ще триває — звіт охопить дані станом на ${asOf}.`;
  });

  // Each opening starts from the period the list is showing, not from whatever
  // was picked the last time.
  $effect(() => {
    if (!open) return;

    reportType = availableKinds.includes(defaultType)
      ? defaultType
      : DEFAULT_REPORT_KIND;
    const period =
      defaultPeriod ?? rangeForMode("month", todayText, { from: "", to: "" });
    mode = modeForRange(period);
    anchor = period.from;
    custom = period;
  });

  async function submit() {
    if (!range.from || !range.to) return;

    generating = true;
    const generated = await onGenerate(reportType, range).catch(() => false);
    generating = false;

    if (generated) open = false;
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="gap-4 sm:max-w-sm">
    <Dialog.Header>
      <Dialog.Title>Новий звіт</Dialog.Title>
      <Dialog.Description>
        Оберіть тип і період — звіт збережеться в журнал.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1.5">
        <Label>Тип</Label>
        <Select.Root type="single" bind:value={reportType}>
          <Select.Trigger class="w-full">
            {reportKindTitles[reportType]}
          </Select.Trigger>
          <Select.Content>
            {#each availableKinds as kind (kind)}
              <Select.Item value={kind}>{reportKindTitles[kind]}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <div class="flex flex-col gap-2">
        <Label>Період</Label>
        <PeriodPanel
          bind:mode
          bind:anchor
          bind:custom
          rangeMonths={1}
          calendarClass="[--cell-size:--spacing(10)]"
        />
      </div>

      {#if versionsNote || ongoingNote}
        <div
          class={cn(
            "flex gap-2.5 rounded-lg border px-3 py-2.5",
            stale
              ? "border-amber-500/25 bg-amber-500/10"
              : "border-border bg-muted/40",
          )}
        >
          {#if stale}
            <TriangleAlertIcon
              class="mt-0.5 size-4 shrink-0 text-amber-600 dark:text-amber-500"
            />
          {:else}
            <InfoIcon class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
          {/if}

          <div class="flex flex-col gap-1 text-sm">
            {#if versionsNote}
              <p
                class={stale
                  ? "text-amber-700 dark:text-amber-400"
                  : "text-foreground"}
              >
                {versionsNote}
              </p>
            {/if}

            {#if ongoingNote}
              <p class="text-muted-foreground">{ongoingNote}</p>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <Dialog.Footer>
      <Button
        variant="outline"
        disabled={generating}
        onclick={() => (open = false)}
      >
        Скасувати
      </Button>
      <Button
        disabled={!range.from || !range.to || generating}
        onclick={submit}
      >
        {#if generating}<Spinner />{/if}
        {submitLabel}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
