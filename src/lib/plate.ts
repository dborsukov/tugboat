export type PlateType =
  | "ua"
  | "ua_old"
  | "military"
  | "temporary"
  | "foreign"
  | "other";

export type PlatePreviewKind =
  | "ua_flag"
  | "country_flag"
  | "tag"
  | "stripes"
  | "ellipsis";

export interface PlateTypeMeta {
  value: PlateType;
  name: string;
  example: string;
  placeholder: string;
  /** What the compact trigger segment shows. */
  triggerKind: "ua_flag" | "country_flag" | "tag";
  triggerTag?: string;
  /** What the small filled chip prefix shows. */
  chipKind: "ua_flag" | "country_flag" | "tag" | "none";
  chipTag?: string;
  previewKind: PlatePreviewKind;
  previewTag?: string;
}

export const PLATE_TYPES: PlateTypeMeta[] = [
  {
    value: "ua",
    name: "Український",
    example: "AA 1234 BB",
    placeholder: "AA 1234 BB",
    triggerKind: "ua_flag",
    chipKind: "ua_flag",
    previewKind: "ua_flag",
  },
  {
    value: "ua_old",
    name: "Старий зразок",
    example: "12-34 КВА",
    placeholder: "12-34 КВА",
    triggerKind: "ua_flag",
    chipKind: "ua_flag",
    previewKind: "ua_flag",
  },
  {
    value: "military",
    name: "Військовий",
    example: "1234 ВА",
    placeholder: "1234 ВА",
    triggerKind: "tag",
    triggerTag: "ВІЙСЬК.",
    chipKind: "tag",
    chipTag: "ВІЙСЬК.",
    previewKind: "tag",
    previewTag: "ВС",
  },
  {
    value: "temporary",
    name: "Транзитний / тимчасовий",
    example: "TR 0000",
    placeholder: "TR 0000",
    triggerKind: "tag",
    triggerTag: "ТРАНЗ.",
    chipKind: "tag",
    chipTag: "ТИМЧ.",
    previewKind: "tag",
    previewTag: "ТР",
  },
  {
    value: "foreign",
    name: "Іноземний",
    example: "Номер",
    placeholder: "Номер",
    triggerKind: "country_flag",
    triggerTag: "ІНОЗ.",
    chipKind: "country_flag",
    previewKind: "stripes",
  },
  {
    value: "other",
    name: "Інший",
    example: "Номер",
    placeholder: "Номер",
    triggerKind: "tag",
    triggerTag: "ІНШИЙ",
    chipKind: "none",
    previewKind: "ellipsis",
  },
];

const PLATE_TYPE_BY_VALUE = new Map(
  PLATE_TYPES.map((meta) => [meta.value, meta]),
);

export function plateTypeMeta(
  value: PlateType | null | undefined,
): PlateTypeMeta {
  return PLATE_TYPE_BY_VALUE.get(value ?? "ua") ?? PLATE_TYPES[0];
}

/** Trim, uppercase and collapse repeated whitespace. Hyphens are kept. */
export function normalizePlateText(
  value: string | null | undefined,
): string | null {
  const normalized = (value ?? "").trim().replace(/\s+/g, " ").toUpperCase();
  return normalized === "" ? null : normalized;
}

const CYRILLIC_TO_LATIN: Record<string, string> = {
  А: "A",
  В: "B",
  Е: "E",
  К: "K",
  М: "M",
  Н: "H",
  О: "O",
  Р: "P",
  С: "C",
  Т: "T",
  У: "Y",
  Х: "X",
  І: "I",
};

/** Uppercase, strip whitespace/hyphens and map Cyrillic lookalikes. */
export function plateSearchKey(value: string | null | undefined): string {
  return (normalizePlateText(value) ?? "")
    .replace(/[\s-]/g, "")
    .replace(/[АВЕКМНОРСТУХІ]/g, (char) => CYRILLIC_TO_LATIN[char] ?? char);
}
