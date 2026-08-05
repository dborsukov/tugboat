<script lang="ts">
  import type { Car, CarBrand } from "$lib/bindings";

  import Badge from "$lib/components/ui/badge/badge.svelte";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";

  let { car, carBrand }: { car: Car; carBrand: CarBrand } = $props();

  const brandName = $derived(carBrand.id != 0 ? `${carBrand.name} ` : "");
</script>

<div class="flex items-center gap-2 h-8">
  <CarLogoLoader slug={carBrand.iconSlug} />
  <p class="tracking-wide">{brandName}{car.model}</p>
  <Badge variant="outline" class="tracking-wider">{car.year}</Badge>
</div>
