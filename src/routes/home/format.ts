import { pluralizeUk } from "$lib/tools";

export function toNumber(value: string | number | null | undefined): number {
  if (value == null) return 0;
  const parsed = typeof value === "number" ? value : Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

export function percentChange(
  current: number,
  previous: number,
): number | null {
  if (!previous) return null;
  return ((current - previous) / Math.abs(previous)) * 100;
}

export function formatPercent(
  value: number,
  digits = 0,
  roundWhole = false,
): string {
  const figure = roundWhole
    ? toFixedPlusRound(value, digits)
    : value.toFixed(digits);

  // Show a string(e.g. <0.1%) instead of 0 for values that rounded to 0
  if (value !== 0 && Number(figure) === 0) return `<${smallestStep(digits)}%`;

  return `${figure}%`;
}

function smallestStep(digits: number): string {
  return (10 ** -digits).toFixed(digits);
}

/**
 * `value` at the precision it needs and no more
 * (100 reads as "100" where 23.34 reads as "23.3" and 23.999 reads as "24")
 */
export function toFixedPlusRound(value: number, digits = 1): string {
  const rounded = Number(value.toFixed(digits));
  return rounded % 1 === 0 ? rounded.toFixed(0) : rounded.toFixed(digits);
}

export function formatCurrencyNumber(
  value: string | number | null | undefined,
  digits = 0,
): string {
  const amount = toNumber(value);
  return new Intl.NumberFormat("uk-UA", {
    maximumFractionDigits: digits,
  }).format(amount === 0 ? 0 : amount);
}

export function formatOrders(count: number): string {
  return `${count} ${pluralizeUk(count, ["замовлення", "замовлення", "замовлень"])}`;
}
