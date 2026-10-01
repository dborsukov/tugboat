import type { AnalyticsPeriod, DateRange } from "$lib/bindings";

const FALLBACK_TIMEZONE = "Europe/Kiev";

export type CalendarMonth = { year: number; month: number };

const formatters = new Map<string, Intl.DateTimeFormat>();

function formatter(timeZone: string): Intl.DateTimeFormat {
  let cached = formatters.get(timeZone);
  if (!cached) {
    cached = new Intl.DateTimeFormat("en-US", {
      timeZone,
      hour12: false,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
    formatters.set(timeZone, cached);
  }
  return cached;
}

/**
 * Keeps an unknown or misspelled timezone from taking the whole dashboard
 * down, if `Intl` throws `FALLBACK_TIMEZONE` is used
 */
export function timeZoneOrDefault(timeZone?: string | null): string {
  if (!timeZone) return FALLBACK_TIMEZONE;
  try {
    formatter(timeZone).format();
    return timeZone;
  } catch {
    return FALLBACK_TIMEZONE;
  }
}

function numberPart(parts: Intl.DateTimeFormatPart[], type: string): number {
  return Number(parts.find((part) => part.type === type)?.value ?? "0");
}

/** Calendar date an instant falls on, as seen in `timeZone`. */
function zonedCalendarDate(date: Date, timeZone: string): Date {
  const parts = formatter(timeZone).formatToParts(date);
  return new Date(
    Date.UTC(
      numberPart(parts, "year"),
      numberPart(parts, "month") - 1,
      numberPart(parts, "day"),
    ),
  );
}

/** Offset of `timeZone` at `date`, in milliseconds east of UTC. */
function timeZoneOffsetMs(date: Date, timeZone: string): number {
  const parts = formatter(timeZone).formatToParts(date);
  const asUtc = Date.UTC(
    numberPart(parts, "year"),
    numberPart(parts, "month") - 1,
    numberPart(parts, "day"),
    // Some locales render midnight as hour 24, which `Date.UTC` would roll into the next day.
    numberPart(parts, "hour") % 24,
    numberPart(parts, "minute"),
    numberPart(parts, "second"),
  );
  return asUtc - Math.trunc(date.getTime() / 1000) * 1000;
}

/**
 * The UTC instant of a wall clock midnight in `timeZone`.
 *
 * `month` and `day` may run out of range (`13`, `0`, `-3`): `Date.UTC`
 * normalizes them, so callers can write "the first of three months ago" or
 * "the Monday of this week" as plain arithmetic.
 */
function zonedMidnight(
  year: number,
  month: number,
  day: number,
  timeZone: string,
): Date {
  const wallClock = Date.UTC(year, month - 1, day);
  // Two passes: the first picks the offset at the guessed instant, the second
  // re-reads it at the corrected one, which settles across a DST change.
  const guess = wallClock - timeZoneOffsetMs(new Date(wallClock), timeZone);
  return new Date(wallClock - timeZoneOffsetMs(new Date(guess), timeZone));
}

function dateRange(from: Date, to: Date): DateRange {
  return { from: from.toISOString(), to: to.toISOString() };
}

/**
 * Builds every window the dashboard module ask for, for the month `month`
 */
export function analyticsPeriod(
  now: Date,
  timeZone?: string | null,
  month?: CalendarMonth | null,
): AnalyticsPeriod {
  const zone = timeZoneOrDefault(timeZone);
  const today = zonedCalendarDate(now, zone);
  const picked = month ?? calendarMonthOf(now, zone);
  const year = picked.year;
  const monthNumber = picked.month;
  const day = today.getUTCDate();

  // `getUTCDay` is Sunday based, the shop's week starts on Monday.
  const daysSinceMonday = (today.getUTCDay() + 6) % 7;

  return {
    month: dateRange(
      zonedMidnight(year, monthNumber, 1, zone),
      zonedMidnight(year, monthNumber + 1, 1, zone),
    ),
    previousMonth: dateRange(
      zonedMidnight(year, monthNumber - 1, 1, zone),
      zonedMidnight(year, monthNumber, 1, zone),
    ),
    week: dateRange(
      zonedMidnight(
        today.getUTCFullYear(),
        today.getUTCMonth() + 1,
        day - daysSinceMonday,
        zone,
      ),
      zonedMidnight(
        today.getUTCFullYear(),
        today.getUTCMonth() + 1,
        day + 1,
        zone,
      ),
    ),
    previousWeek: dateRange(
      zonedMidnight(
        today.getUTCFullYear(),
        today.getUTCMonth() + 1,
        day - daysSinceMonday - 7,
        zone,
      ),
      zonedMidnight(
        today.getUTCFullYear(),
        today.getUTCMonth() + 1,
        day - daysSinceMonday,
        zone,
      ),
    ),
  };
}

/** The calendar month an instant falls in, as seen in `timeZone`. */
export function calendarMonthOf(
  date: Date,
  timeZone?: string | null,
): CalendarMonth {
  const day = zonedCalendarDate(date, timeZoneOrDefault(timeZone));
  return { year: day.getUTCFullYear(), month: day.getUTCMonth() + 1 };
}

/** The day of the month an instant falls on, as seen in `timeZone`. */
export function calendarDayOf(date: Date, timeZone?: string | null): number {
  return zonedCalendarDate(date, timeZoneOrDefault(timeZone)).getUTCDate();
}

/** How many days the month has. */
export function daysInMonth(month: CalendarMonth): number {
  // Day zero of the month after this one is the last day of this one.
  return new Date(Date.UTC(month.year, month.month, 0)).getUTCDate();
}

/** The calendar month before this one. */
export function previousMonth(month: CalendarMonth): CalendarMonth {
  // A month of zero or less normalizes back into the year before.
  const date = new Date(Date.UTC(month.year, month.month - 2, 1));
  return { year: date.getUTCFullYear(), month: date.getUTCMonth() + 1 };
}

export function elapsedDays(
  month: CalendarMonth,
  now: Date,
  timeZone?: string | null,
): number {
  const current = monthKey(calendarMonthOf(now, timeZone));
  const key = monthKey(month);
  if (key < current) return daysInMonth(month);
  if (key > current) return 0;

  return Math.min(calendarDayOf(now, timeZone), daysInMonth(month));
}

/**
 * Stable, sortable key for a month ("2026-09"): fixed width, so comparing two
 * of them as strings compares them in time.
 */
export function monthKey(month: CalendarMonth): string {
  return `${month.year}-${String(month.month).padStart(2, "0")}`;
}

/** Inverse of `monthKey` */
export function monthFromKey(key: string): CalendarMonth {
  const [year, month] = key.split("-");
  return { year: Number(year), month: Number(month) };
}

/** "Вересень 2026" style heading for a month. */
export function monthLabel(
  month: CalendarMonth,
  timeZone?: string | null,
): string {
  const parts = new Intl.DateTimeFormat("uk-UA", {
    month: "long",
    year: "numeric",
    timeZone: timeZoneOrDefault(timeZone),
  }).formatToParts(
    zonedMidnight(month.year, month.month, 1, timeZoneOrDefault(timeZone)),
  );
  // Read the parts instead of the formatted string: Ukrainian CLDR renders
  // "вересень 2026 р." as a whole, and the heading reads better without the
  // "р." the date formats in the rest of the app carry.
  const monthName = parts.find((part) => part.type === "month")?.value ?? "";
  const year = parts.find((part) => part.type === "year")?.value ?? "";
  const label = `${monthName} ${year}`.trim();
  return label.charAt(0).toUpperCase() + label.slice(1);
}

/**
 * Offset of `timeZone` at `at`, in whole minutes east of UTC — the shape
 * SQLite wants for its time modifiers ("+180 minutes").
 */
export function timeZoneOffsetMinutes(
  timeZone?: string | null,
  at: Date = new Date(),
): number {
  return Math.round(timeZoneOffsetMs(at, timeZoneOrDefault(timeZone)) / 60_000);
}
