import type { CarBrand } from "$lib/bindings";

export const UNKNOWN_BRAND_SLUG = "unknown-brand";

export function isUnknownBrand(
  brand: Pick<CarBrand, "slug"> | null | undefined,
): boolean {
  return brand?.slug === UNKNOWN_BRAND_SLUG;
}
