import { countries } from "svelte-tel-input";
import type { CountryCode } from "svelte-tel-input/types";

const PLATE_RE = /^[A-ZА-ЯІЇЄ]{2}\d{4}[A-ZА-ЯІЇЄ]{2}$/;

export type InterpretedQuery =
  | { kind: "empty" }
  | { kind: "plate"; value: string }
  | { kind: "phone"; value: string }
  | { kind: "text"; value: string; words: string[] };

/** Decide what the picker search should seed the next sheet with. */
export function interpretQuery(query: string): InterpretedQuery {
  const text = query.trim();
  if (text === "") return { kind: "empty" };

  const compactPlate = text.replace(/[\s-]/g, "").toUpperCase();
  if (PLATE_RE.test(compactPlate)) {
    return { kind: "plate", value: compactPlate };
  }

  const digits = text.replace(/\D/g, "");
  const letters = text.replace(/[^\p{L}]/gu, "");
  const mostlyDigits =
    digits.length >= 5 &&
    digits.length >= letters.length &&
    !text.includes("@");

  if (mostlyDigits) return { kind: "phone", value: digits };

  return {
    kind: "text",
    value: text,
    words: text.split(/\s+/).filter(Boolean),
  };
}

export function toInternationalPhone(
  digits: string,
  countryCode: CountryCode | null,
) {
  const country = countries.find((item) => item.iso2 === countryCode);
  const dialCode = country?.dialCode ?? "380";
  let national = digits.replace(/\D/g, "");

  if (national.startsWith(`+${dialCode}`)) {
    national = national.slice(dialCode.length + 1);
  }
  if (national.startsWith(dialCode) && national.length > dialCode.length + 4) {
    national = national.slice(dialCode.length);
  }
  if (national.startsWith("0")) national = national.slice(1);

  return `+${dialCode}${national}`;
}
