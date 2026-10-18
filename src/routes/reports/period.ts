import type { Report, ReportKind } from "$lib/bindings";

/** A closed range of `YYYY-MM-DD` days, both ends included. */
export type DayRange = { from: string; to: string };

export type CalendarMonth = { year: number; month: number };

/**
 * How a period is anchored.
 * The four presets all resolve a span, they differ only in how long it is, and
 * all four are driven by one anchor day, which is why they share a single input.
 */
export type PeriodMode = "day" | "month" | "quarter" | "year" | "custom";

export const PERIOD_MODES: PeriodMode[] = [
  "day",
  "month",
  "quarter",
  "year",
  "custom",
];

export const periodModeLabels: Record<PeriodMode, string> = {
  day: "День",
  month: "Місяць",
  quarter: "Квартал",
  year: "Рік",
  custom: "Довільний",
};

export const REPORT_KINDS: ReportKind[] = ["financial", "vat"];

export const reportKindLabels: Record<ReportKind, string> = {
  financial: "Фінансові",
  vat: "ПДВ",
};

export const reportKindTitles: Record<ReportKind, string> = {
  financial: "Фінансовий звіт",
  vat: "Реєстр ПДВ",
};

export const reportKindChips: Record<ReportKind, string> = {
  financial: "Фінансовий",
  vat: "ПДВ",
};

export const DEFAULT_REPORT_KIND: ReportKind = "financial";

export function isReportKind(value: string | null): value is ReportKind {
  return value != null && (REPORT_KINDS as string[]).includes(value);
}

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

export function dayText(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function dayFromText(text: string): Date {
  const [year, month, day] = text.split("-").map(Number);
  return new Date(year, (month ?? 1) - 1, day ?? 1);
}

/** First and last day of a month. */
export function monthRange(month: CalendarMonth): DayRange {
  // Day zero of the next month is the last day of this one.
  const last = new Date(month.year, month.month, 0).getDate();
  return {
    from: `${month.year}-${pad(month.month)}-01`,
    to: `${month.year}-${pad(month.month)}-${pad(last)}`,
  };
}

/* The span a preset resolves to, from the day it is anchored on */
export function rangeForMode(
  mode: PeriodMode,
  anchor: string,
  custom: DayRange,
): DayRange {
  if (mode === "custom") return custom;
  if (!anchor) return { from: "", to: "" };

  const date = dayFromText(anchor);
  const year = date.getFullYear();

  if (mode === "day") return { from: anchor, to: anchor };

  if (mode === "month") {
    return monthRange({ year, month: date.getMonth() + 1 });
  }

  if (mode === "quarter") {
    const firstMonth = Math.floor(date.getMonth() / 3) * 3 + 1;
    return {
      from: `${year}-${pad(firstMonth)}-01`,
      // Day zero of the month after the quarter's last one.
      to: dayText(new Date(year, firstMonth + 2, 0)),
    };
  }

  return { from: `${year}-01-01`, to: `${year}-12-31` };
}

export function monthKey(month: CalendarMonth): string {
  return `${month.year}-${pad(month.month)}`;
}

export function monthFromKey(key: string): CalendarMonth {
  const [year, month] = key.split("-");
  return { year: Number(year), month: Number(month) };
}

/* Amount of days in a range, both ends counted */
export function rangeDays(range: DayRange): number {
  if (!range.from || !range.to) return 0;
  const from = Date.UTC(...parts(range.from));
  const to = Date.UTC(...parts(range.to));
  return Math.round((to - from) / 86_400_000) + 1;
}

function parts(text: string): [number, number, number] {
  const [year, month, day] = text.split("-").map(Number);
  return [year, (month ?? 1) - 1, day ?? 1];
}

export function dottedDay(text: string): string {
  if (!text) return "";
  const [year, month, day] = text.split("-");
  return `${day}.${month}.${year}`;
}

export function dayFromDotted(text: string): string | null {
  const match = /^(\d{2})\.(\d{2})\.(\d{4})$/.exec(text.trim());
  if (!match) return null;
  const [, day, month, year] = match;
  const date = new Date(Number(year), Number(month) - 1, Number(day));
  // Rejects 31.02. and friends, which `Date` would roll into March.
  if (
    date.getFullYear() !== Number(year) ||
    date.getMonth() !== Number(month) - 1 ||
    date.getDate() !== Number(day)
  ) {
    return null;
  }
  return `${year}-${month}-${day}`;
}

export function shortRange(range: DayRange): string {
  if (!range.from || !range.to) return "Оберіть період";
  return range.from === range.to
    ? dottedDay(range.from)
    : `${dottedDay(range.from)} — ${dottedDay(range.to)}`;
}

const MONTH_FORMATTER = new Intl.DateTimeFormat("uk-UA", {
  month: "long",
  year: "numeric",
});

const SHORT_MONTH_FORMATTER = new Intl.DateTimeFormat("uk-UA", {
  month: "short",
});

function withoutYearMarker(label: string): string {
  return label.replace(/\s+р\.$/, "");
}

export function monthLabel(month: CalendarMonth): string {
  const label = MONTH_FORMATTER.format(
    new Date(month.year, month.month - 1, 1),
  );
  return withoutYearMarker(label.charAt(0).toUpperCase() + label.slice(1));
}

export function shortDay(day: string): string {
  const date = dayFromText(day);
  const month = SHORT_MONTH_FORMATTER.format(date).replace(/\.$/, "");
  return `${date.getDate()} ${month}`;
}

export function shortSpan(range: DayRange): string {
  const from = dayFromText(range.from);
  const to = dayFromText(range.to);
  if (from.getFullYear() === to.getFullYear()) {
    return `${shortDay(range.from)} – ${shortDay(range.to)} ${to.getFullYear()}`;
  }
  return `${shortDay(range.from)} ${from.getFullYear()} – ${shortDay(range.to)} ${to.getFullYear()}`;
}

export const QUARTER_NUMERALS = ["I", "II", "III", "IV"];

export function quarterLabel(year: number, quarter: number): string {
  return `${QUARTER_NUMERALS[quarter - 1]} квартал ${year}`;
}

export function dayLabel(day: string): string {
  if (!day) return "Оберіть день";
  return withoutYearMarker(
    new Intl.DateTimeFormat("uk-UA", {
      day: "numeric",
      month: "long",
      year: "numeric",
    }).format(dayFromText(day)),
  );
}

/* The preset a range happens to match */
export function modeForRange(range: DayRange): PeriodMode {
  if (!range.from || !range.to) return "custom";
  for (const mode of ["day", "month", "quarter", "year"] as PeriodMode[]) {
    const resolved = rangeForMode(mode, range.from, { from: "", to: "" });
    if (resolved.from === range.from && resolved.to === range.to) return mode;
  }
  return "custom";
}

export function rangeTitle(range: DayRange): string {
  if (!range.from || !range.to) return "Оберіть період";
  if (range.from === range.to) return dayLabel(range.from);

  const from = dayFromText(range.from);
  const mode = modeForRange(range);

  if (mode === "month") {
    return monthLabel({ year: from.getFullYear(), month: from.getMonth() + 1 });
  }
  if (mode === "quarter") {
    return quarterLabel(
      from.getFullYear(),
      Math.floor(from.getMonth() / 3) + 1,
    );
  }
  if (mode === "year") return `${from.getFullYear()} рік`;
  return shortSpan(range);
}

/* The name a saved copy is offered under */
export function reportFileName(
  reportType: ReportKind,
  range: DayRange,
  olderBy = 0,
  incomplete = false,
): string {
  const parts = [reportKindTitles[reportType], rangeTitle(range)];
  if (olderBy > 0) parts.push(String(olderBy));
  if (incomplete) parts.push("неповний період");
  return `${parts.map(spaceless).join("_")}.xlsx`;
}

function spaceless(part: string): string {
  return part.replace(/\s+/g, "_");
}

export function fileMissing(report: Report): boolean {
  return report.fileSize == null;
}

export function needsRefresh(report: Report): boolean {
  return report.isStale || fileMissing(report);
}

/* The period as it is written in a query string */
export function periodParam(range: DayRange): string {
  return `${range.from}_${range.to}`;
}

/* Reads back a period from the address bar. */
export function parsePeriodParam(param: string | null): DayRange | null {
  if (!param) return null;

  const month = /^(\d{4})-(\d{2})$/.exec(param);
  if (month) {
    return monthRange({ year: Number(month[1]), month: Number(month[2]) });
  }

  if (/^\d{4}-\d{2}-\d{2}$/.test(param)) return { from: param, to: param };
  const match = /^(\d{4}-\d{2}-\d{2})_(\d{4}-\d{2}-\d{2})$/.exec(param);
  if (!match) return null;
  const from = match[1];
  const to = match[2] < match[1] ? match[1] : match[2];
  return { from, to };
}
