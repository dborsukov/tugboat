import Big from "big.js";

import type { DiscountType } from "$lib/bindings";

export const PURCHASE_COLUMN_KEY = "popeye.work-order.purchase-column";

const numberFormat = new Intl.NumberFormat("uk-UA", {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

/** `1234.5` -> `1 234,50` */
export function moneyText(value: string | number): string {
  return numberFormat.format(Number(new Big(value).toString()));
}

/** `1.5` -> `1,5`, `2.00` -> `2` */
export function decimalText(value: string | number): string {
  return new Big(value).toString().replace(".", ",");
}

export function discountText(
  type: DiscountType,
  value: string | number,
): string {
  if (type === "Percentage") return `−${decimalText(value)}%`;
  if (type === "Fixed") return `−${moneyText(value)}`;
  return "";
}

export function hasDiscount(type: DiscountType, value: string | number) {
  return type !== "None" && new Big(value).gt(0);
}

export function gridStyle(columns: string[]) {
  return `grid-template-columns: ${columns.join(" ")}; column-gap: 10px;`;
}

export function laborGridStyle(readOnly = false) {
  return gridStyle([
    ...(readOnly ? [] : ["44px"]),
    "minmax(0,1fr)",
    "108px",
    "104px",
    "104px",
    ...(readOnly ? [] : ["24px"]),
  ]);
}

export function partsGridStyle(showPurchase: boolean, readOnly = false) {
  return gridStyle([
    ...(readOnly ? [] : ["44px"]),
    "minmax(0,1fr)",
    "108px",
    "66px",
    ...(showPurchase ? ["98px"] : []),
    "104px",
    "104px",
    ...(readOnly ? [] : ["24px"]),
  ]);
}

/** Keeps the last digits visible when a long monetary value is clipped. */
export const leftTruncateClass =
  "block max-w-full overflow-hidden text-ellipsis whitespace-nowrap text-right [direction:rtl] [unicode-bidi:plaintext]";
