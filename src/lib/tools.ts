import type { CountryCode } from "svelte-tel-input/types";

import { settingsState } from "$lib/states/settings.svelte";

type SpectaResult<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

export async function unwrap<T, E>(
  promise: Promise<SpectaResult<T, E>>,
): Promise<T> {
  const result = await promise;
  if (result.status === "error") {
    throw result.error;
  }
  return result.data;
}

export function fileNameOf(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

let date_locale = "uk-UA";

/**
 * The calendar day an instant falls on, in `YYYY-MM-DD`, read in `timeZone`.
 *
 * Built from parts rather than from `toISOString`, so the day is the one the
 * user's clock shows and not the one UTC happens to be on.
 */
function dayKeyOf(date: Date, timeZone?: string): string {
  const parts = new Intl.DateTimeFormat("en-US", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    timeZone,
  }).formatToParts(date);
  const get = (type: "year" | "month" | "day") =>
    parts.find((part) => part.type === type)?.value ?? "";
  return `${get("year")}-${get("month")}-${get("day")}`;
}

function timeOf(date: Date, timeZone?: string): string {
  return new Intl.DateTimeFormat(date_locale, {
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
    timeZone,
  }).format(date);
}

/**
 * The calendar day an instant falls on, as `YYYY-MM-DD`, in the settings'
 * timezone — the day the user's own clock shows it on.
 */
export function isoDayOf(iso_string: string): string {
  return dayKeyOf(
    new Date(iso_string),
    settingsState.settings?.timezone ?? undefined,
  );
}

export function prettifyIsoDateString(
  iso_string: string,
  type: "short" | "long" | "datetime" | "datetime-short",
) {
  const timeZone = settingsState.settings?.timezone ?? undefined;
  if (type === "short") {
    return new Date(iso_string).toLocaleDateString(date_locale, {
      month: "numeric",
      day: "numeric",
      year: "numeric",
      timeZone,
    });
  }
  if (type === "long") {
    return new Date(iso_string).toLocaleDateString(date_locale, {
      month: "long",
      day: "numeric",
      year: "numeric",
      timeZone,
    });
  }
  if (type === "datetime") {
    return new Date(iso_string).toLocaleString(date_locale, {
      month: "long",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "numeric",
      timeZone,
    });
  }

  /*
   * The compact form, for a timestamp that is context rather than content:
   * "сьогодні, 09:21" while the day is still today, "9 жовт, 17:48" otherwise.
   * The year is left out for now because it's only used for reports atm.
   */
  const date = new Date(iso_string);
  const day = dayKeyOf(date, timeZone);
  const time = timeOf(date, timeZone);
  if (day === dayKeyOf(new Date(), timeZone)) return `сьогодні, ${time}`;

  const yesterday = new Date();
  yesterday.setDate(yesterday.getDate() - 1);
  if (day === dayKeyOf(yesterday, timeZone)) return `вчора, ${time}`;

  const month = new Intl.DateTimeFormat(date_locale, {
    month: "short",
    timeZone,
  })
    .format(date)
    .replace(/\.$/, "");

  return `${new Intl.DateTimeFormat(date_locale, { day: "numeric", timeZone }).format(date)} ${month}, ${time}`;
}

export function pluralizeUk(count: number, forms: [string, string, string]) {
  const mod10 = count % 10;
  const mod100 = count % 100;
  if (mod10 === 1 && mod100 !== 11) return forms[0];
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
    return forms[1];
  }
  return forms[2];
}

const displayNames =
  typeof Intl !== "undefined" && "DisplayNames" in Intl
    ? new Intl.DisplayNames(["uk"], { type: "region" })
    : null;

const countryNameCache = new Map<CountryCode, string>();

/**
 * Ukrainian country name for an ISO 3166-1 alpha-2 code (e.g. "UA" → "Україна",
 * "DE" → "Німеччина"), built with the built-in `Intl.DisplayNames` CLDR data.
 * Falls back to `fallback` when the name is unavailable.
 */
export function getCountryNameUk(iso2: CountryCode, fallback: string): string {
  const cached = countryNameCache.get(iso2);
  if (cached !== undefined) return cached;

  let name = fallback;
  try {
    const resolved = displayNames?.of(iso2);
    if (resolved && resolved !== iso2) name = resolved;
  } catch {
    // keep fallback
  }
  countryNameCache.set(iso2, name);
  return name;
}
