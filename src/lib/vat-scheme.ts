import type { VatScheme } from "$lib/bindings";

export const VAT_SCHEMES: VatScheme[] = [
  "standard",
  "zero",
  "exempt",
  "not_object",
  "export",
];

export const vatSchemeLabels: Record<VatScheme, string> = {
  standard: "Оподатковувана",
  zero: "Нульова",
  exempt: "Звільнена",
  not_object: "Не є об'єктом",
  export: "Експорт",
};

/** A non-standard treatment is only valid at a zero rate. */
export function schemeNeedsZeroRate(scheme: VatScheme): boolean {
  return scheme !== "standard";
}
