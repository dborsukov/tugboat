import type { CarBrand } from "$lib/bindings";

export const UNKNOWN_BRAND_SLUG = "unknown-brand";

export function isUnknownBrand(
  brand: Pick<CarBrand, "slug"> | null | undefined,
): boolean {
  return brand?.slug === UNKNOWN_BRAND_SLUG;
}

// Text the brand picker matches against (name + aliases)
export function brandSearchValue(
  brand: Pick<CarBrand, "name" | "aliases">,
): string {
  return [brand.name, ...(brand.aliases ?? [])].join(" ");
}
