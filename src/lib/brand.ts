/** Product names by UI language; English is the fallback. */
export const BRAND_EN = "Tugboat";
export const BRAND_UK = "Буксир";

/**
 * The brand to show for a locale such as `navigator.language`.
 */
export function brandName(locale?: string | null): string {
  const value =
    locale ?? (typeof navigator === "undefined" ? "" : navigator.language);
  return value.toLowerCase().startsWith("uk") ? BRAND_UK : BRAND_EN;
}
