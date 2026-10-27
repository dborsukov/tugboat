import Big from "big.js";

import unitsJson from "../../src-tauri/data/part_units.json";

export type PartUnitPreset = {
  key: string;
  name: string;
  display: string;
  decimals: boolean;
};

export const PART_UNITS = unitsJson as PartUnitPreset[];
export const DEFAULT_UNIT_KEY = "pcs";
export const CUSTOM_UNIT_VALUE = "__custom__";
export const CUSTOM_UNIT_MAX_LENGTH = 12;

const byKey = new Map(PART_UNITS.map((unit) => [unit.key, unit]));

export function partUnitByKey(key: string | null | undefined) {
  return key ? byKey.get(key) : undefined;
}

export function unitAllowsDecimals(key: string | null | undefined) {
  return partUnitByKey(key)?.decimals ?? true;
}

export function unitOptionLabel(unit: PartUnitPreset) {
  return `${unit.display} · ${unit.name}`;
}

const quantityFormat = new Intl.NumberFormat("uk-UA", {
  minimumFractionDigits: 0,
  maximumFractionDigits: 3,
});

/** `2` -> `2`, `1.50` -> `1,5`, `1.5` -> `1,5`. */
export function formatQtyNumber(value: string | number): string {
  const parsed = new Big(String(value).replace(",", "."));
  return quantityFormat.format(Number(parsed.toString()));
}

export function unitDisplay(part: {
  unitKey?: string | null;
  unitType?: string | null;
}): string {
  const preset = partUnitByKey(part.unitKey);
  if (preset) return preset.display;

  const custom = part.unitType?.trim();
  if (custom) return custom;

  return partUnitByKey(DEFAULT_UNIT_KEY)?.display ?? "";
}

export function formatQty(
  quantity: string | number,
  part: { unitKey?: string | null; unitType?: string | null },
): string {
  return `${formatQtyNumber(quantity)} ${unitDisplay(part)}`.trim();
}
