<script lang="ts">
  import InfoIcon from "@lucide/svelte/icons/info";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";

  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Spinner } from "$lib/components/ui/spinner";

  import {
    sheetLabel,
    sheetPrimaryButton,
    sheetSelectTrigger,
  } from "$lib/components/sheet/field-styles";

  import type { Report, ReportKind } from "$lib/bindings";
  import { settingsState } from "$lib/states/settings.svelte";
  import { cn } from "$lib/utils";

  import {
    dayLabel,
    dayText,
    DEFAULT_REPORT_KIND,
    modeForRange,
    needsRefresh,
    rangeForMode,
    rangeTitle,
    REPORT_KINDS,
    reportKindTitles,
    shortDay,
    shortSpan,
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

  const summaryRange = $derived({
    from: range.from,
    to: range.to && range.to > todayText ? todayText : range.to,
  });

  const summaryLine = $derived.by(() => {
    if (!range.from || !range.to) return "";
    if (mode === "custom") return shortSpan(summaryRange);
    if (mode === "day") return dayLabel(range.from);
    return `${rangeTitle(range)} · ${shortSpan(summaryRange)}`;
  });

  /** What the journal already holds for the chosen period. */
  const versionsNote = $derived.by(() => {
    if (versions.length === 0) return null;
    return stale
      ? "Звіт за цей період уже є, але дані змінилися."
      : "Звіт за цей період уже є і він актуальний.";
  });

  const canSubmit = $derived(Boolean(range.from && range.to) && !generating);

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
    if (!canSubmit) return;

    generating = true;
    const generated = await onGenerate(reportType, range).catch(() => false);
    generating = false;

    if (generated) open = false;
  }

  $effect(() => {
    if (!open) return;
    const onKeydown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
        event.preventDefault();
        void submit();
      }
    };
    window.addEventListener("keydown", onKeydown, true);
    return () => window.removeEventListener("keydown", onKeydown, true);
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content
    class="flex max-h-[calc(100dvh-var(--overlay-inset,0px)-2rem)] w-full flex-col gap-0 overflow-hidden rounded-[16px] p-0 sm:max-w-[520px]"
  >
    <Dialog.Header class="shrink-0 border-b border-border px-6 pt-5 pb-4 pr-14">
      <Dialog.Title
        class="text-[17px] leading-tight font-semibold tracking-[-0.01em]"
      >
        Новий звіт
      </Dialog.Title>
      <Dialog.Description class="text-[13px] leading-snug">
        Оберіть тип і період — звіт збережеться в журнал.
      </Dialog.Description>
    </Dialog.Header>

    <div class="min-h-0 flex-1 overflow-y-auto px-6 py-6">
      <div class="flex flex-col gap-6">
        <div class="flex flex-col gap-1.5">
          <Label class={sheetLabel}>Тип</Label>
          <Select.Root type="single" bind:value={reportType}>
            <Select.Trigger class={cn(sheetSelectTrigger, "w-full")}>
              {reportKindTitles[reportType]}
            </Select.Trigger>
            <Select.Content>
              {#each availableKinds as kind (kind)}
                <Select.Item value={kind}>{reportKindTitles[kind]}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>

        <section class="space-y-3">
          <div class="flex items-center gap-2">
            <h3
              class="shrink-0 text-[11.5px] font-bold tracking-[0.06em] whitespace-nowrap text-muted-foreground/80 uppercase"
            >
              Період
            </h3>
            <span class="h-px min-w-4 flex-1 bg-border" aria-hidden="true"
            ></span>
          </div>

          <PeriodPanel bind:mode bind:anchor bind:custom rangeMonths={2} />
        </section>

        {#if range.from && range.to}
          <div
            class={cn(
              "flex gap-2.5 rounded-[8px] border px-3 py-3",
              stale
                ? "border-amber-500/25 bg-amber-500/10"
                : "border-border bg-muted/30",
            )}
          >
            {#if stale}
              <TriangleAlertIcon
                class="mt-0.5 size-4 shrink-0 text-amber-600 dark:text-amber-500"
              />
            {:else}
              <InfoIcon class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
            {/if}

            <div class="flex min-w-0 flex-col gap-1">
              <p class="text-[12.5px] font-semibold">{summaryLine}</p>
              {#if ongoing}
                <p class="text-xs leading-relaxed text-muted-foreground">
                  Період ще триває, тому звіт охопить дані станом на {shortDay(
                    todayText,
                  )}.
                </p>
              {/if}
              {#if versionsNote}
                <p
                  class={cn(
                    "text-xs leading-relaxed",
                    stale
                      ? "text-amber-700 dark:text-amber-400"
                      : "text-muted-foreground",
                  )}
                >
                  {versionsNote}
                </p>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div
      class="flex shrink-0 items-center gap-2 border-t border-border px-6 py-4"
    >
      <span class="mr-auto"></span>
      <Button
        variant="ghost"
        class="h-9.5 shrink-0 rounded-[8px] px-2.5 text-xs font-medium text-muted-foreground hover:text-foreground"
        disabled={generating}
        onclick={() => (open = false)}
      >
        Скасувати
      </Button>
      <Button
        variant="brand"
        class={sheetPrimaryButton}
        disabled={!canSubmit}
        aria-busy={generating}
        onclick={submit}
      >
        {#if generating}<Spinner class="size-3.5" />{/if}
        Створити звіт
      </Button>
    </div>
  </Dialog.Content>
</Dialog.Root>
