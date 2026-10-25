<script lang="ts">
  import { Button } from "$lib/components/ui/button";

  import LogoLoader from "$lib/components/car-logo-loader.svelte";
  import LicensePlate from "$lib/components/license-plate.svelte";

  import type { CarDto } from "$lib/bindings";
  import { isUnknownBrand } from "$lib/brands";
  import { cn } from "$lib/utils";

  let {
    car,
    ownerName,
    onEdit,
    editLabel = "Змінити",
    class: className,
  }: {
    car: CarDto;
    ownerName?: string | null;
    onEdit?: () => void;
    editLabel?: string;
    class?: string;
  } = $props();

  const brandName = $derived(
    isUnknownBrand(car.brand) ? "" : `${car.brand.name} `,
  );
</script>

<div
  class={cn(
    "flex min-h-14 items-center gap-3 rounded-[10px] border border-border bg-card px-3 py-2",
    className,
  )}
>
  <span class="size-9 shrink-0">
    <LogoLoader slug={car.brand.iconSlug} name={car.brand.name} />
  </span>

  <div class="min-w-0 flex-1">
    <div class="flex min-w-0 items-center gap-2">
      <p class="truncate text-sm font-semibold">{brandName}{car.car.model}</p>
      <span
        class="shrink-0 rounded-full bg-muted px-1.5 py-0.5 text-[10.5px] leading-none font-medium text-muted-foreground"
      >
        {car.car.year}
      </span>
    </div>
    <div class="mt-1 flex min-w-0 items-center gap-2">
      {#if car.car.licensePlate}
        <LicensePlate
          value={car.car.licensePlate}
          plateType={car.car.plateType}
          country={car.car.plateCountry}
          size="sm"
        />
      {/if}
      {#if ownerName}
        <span class="truncate text-xs text-muted-foreground">{ownerName}</span>
      {/if}
    </div>
  </div>

  {#if onEdit}
    <Button
      type="button"
      variant="ghost"
      size="xs"
      class="h-7 shrink-0 rounded-[6px] px-2 text-xs font-medium text-brand hover:bg-brand/10 hover:text-brand"
      onclick={onEdit}
    >
      {editLabel}
    </Button>
  {/if}
</div>
