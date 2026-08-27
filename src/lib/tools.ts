import type { CountryCode } from "svelte-tel-input/types";

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

let date_locale = "uk-UA";

export function prettifyIsoDateString(
  iso_string: string,
  type: "short" | "long" | "datetime",
) {
  if (type === "short") {
    return new Date(iso_string).toLocaleDateString(date_locale, {
      month: "numeric",
      day: "numeric",
      year: "numeric",
    });
  }
  if (type === "long") {
    return new Date(iso_string).toLocaleDateString(date_locale, {
      month: "long",
      day: "numeric",
      year: "numeric",
    });
  }
  if (type === "datetime") {
    return new Date(iso_string).toLocaleString(date_locale, {
      month: "long",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "numeric",
    });
  }
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
