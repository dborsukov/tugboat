<script lang="ts">
  import Badge from "$lib/components/ui/badge/badge.svelte";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";

  import type { Car, CarBrand } from "$lib/bindings";
  import { isUnknownBrand } from "$lib/brands";
  import { cn } from "$lib/utils";

  let {
    car,
    carBrand,
    class: className,
  }: { car: Car; carBrand: CarBrand; class?: string } = $props();

  const brandName = $derived(
    isUnknownBrand(carBrand) ? "" : `${carBrand.name} `,
  );
</script>

<div class={cn("flex items-center gap-2 h-8", className)}>
  <CarLogoLoader slug={carBrand.iconSlug} name={carBrand.name} />
  <p class="tracking-wide">{brandName}{car.model}</p>
  <Badge variant="outline" class="tracking-wider">{car.year}</Badge>
</div>
