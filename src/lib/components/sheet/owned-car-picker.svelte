<script lang="ts">
  import { tick } from "svelte";

  import SearchIcon from "@lucide/svelte/icons/search";
  import XIcon from "@lucide/svelte/icons/x";

  import { Input } from "$lib/components/ui/input";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import LicensePlate from "$lib/components/license-plate.svelte";

  import type { CarDto } from "$lib/bindings";
  import { isUnknownBrand } from "$lib/brands";
  import { plateSearchKey } from "$lib/plate";
  import { cn } from "$lib/utils";

  import HighlightText from "./highlight-text.svelte";

  let {
    cars,
    query = $bindable(""),
    active = true,
    onSelect,
    onClose,
  }: {
    cars: CarDto[];
    query?: string;
    active?: boolean;
    onSelect: (carId: number) => void;
    onClose?: () => void;
  } = $props();

  let searchEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLDivElement | null>(null);
  let activeIndex = $state(0);

  const trimmed = $derived(query.trim());
  const available = $derived(cars.filter((car) => car.car.archivedAt == null));

  const rows = $derived.by(() => {
    const text = trimmed.toLocaleLowerCase("uk");
    const plate = plateSearchKey(trimmed);
    const vin = trimmed.toUpperCase().replace(/[\s-]/g, "");

    return available.filter(({ car, brand }) => {
      if (text === "") return true;
      const label = `${brand.name} ${car.model}`.toLocaleLowerCase("uk");
      if (label.includes(text)) return true;
      if (plate && (car.plateSearchKey ?? "").includes(plate)) return true;
      if (vin && (car.vin ?? "").toUpperCase().includes(vin)) return true;
      return false;
    });
  });

  const carName = (car: CarDto) =>
    isUnknownBrand(car.brand)
      ? car.car.model
      : `${car.brand.name} ${car.car.model}`;

  function focusSearch() {
    void tick().then(() => searchEl?.focus());
  }

  function choose(car: CarDto) {
    onSelect(car.car.id);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onClose?.();
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (rows.length === 0) return;
      const step = event.key === "ArrowDown" ? 1 : -1;
      activeIndex = (activeIndex + step + rows.length) % rows.length;
      return;
    }

    if (event.key === "Enter") {
      const row = rows[activeIndex];
      if (!row) return;
      event.preventDefault();
      event.stopPropagation();
      choose(row);
    }
  }

  $effect(() => {
    if (!active) return;
    focusSearch();
    const timer = setTimeout(() => {
      if (active) searchEl?.focus();
    }, 120);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    trimmed;
    activeIndex = 0;
  });

  $effect(() => {
    activeIndex;
    rows.length;
    if (!listEl) return;
    const item = listEl.querySelector<HTMLElement>(
      `[data-owned-car-index="${activeIndex}"]`,
    );
    if (!item) return;
    const listRect = listEl.getBoundingClientRect();
    const itemRect = item.getBoundingClientRect();
    if (itemRect.top < listRect.top) {
      listEl.scrollTop += itemRect.top - listRect.top;
    } else if (itemRect.bottom > listRect.bottom) {
      listEl.scrollTop += itemRect.bottom - listRect.bottom;
    }
  });
</script>

<div class="flex flex-col">
  <div class="p-2 pb-1.5">
    <div class="relative">
      <Input
        bind:ref={searchEl}
        bind:value={query}
        class="h-9.5 rounded-[8px] border border-border bg-muted/40 pr-10 pl-9 text-sm shadow-none placeholder:text-muted-foreground/70 focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20"
        placeholder="Номер, VIN або модель…"
        role="combobox"
        aria-expanded={true}
        aria-controls="owned-car-picker-list"
        aria-autocomplete="list"
        autocomplete="off"
        spellcheck="false"
        oninput={() => (activeIndex = 0)}
        onkeydown={handleKeydown}
      />
      <SearchIcon
        class="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground/70"
      />
      {#if onClose}
        <button
          type="button"
          class="absolute top-1/2 right-1.5 flex size-7 -translate-y-1/2 items-center justify-center rounded-[6px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          aria-label="Закрити вибір автомобіля"
          title="Закрити"
          onclick={onClose}
        >
          <XIcon class="size-4" />
        </button>
      {/if}
    </div>
  </div>

  <div class="overflow-hidden">
    <div
      class="flex items-center justify-between px-4 pt-2 pb-1 text-[10.5px] font-semibold tracking-[0.06em] text-muted-foreground/70 uppercase"
    >
      {#if trimmed === ""}
        <span>Авто клієнта</span>
      {:else}
        <span>Знайдено автомобілів: {rows.length}</span>
      {/if}
    </div>

    <div
      id="owned-car-picker-list"
      bind:this={listEl}
      role="listbox"
      aria-label="Автомобілі клієнта"
      class="flex max-h-[352px] flex-col gap-1 overflow-x-hidden overflow-y-auto px-1.5 pb-1"
    >
      {#if rows.length === 0}
        <div class="px-3 py-6 text-center text-xs text-muted-foreground">
          {trimmed === ""
            ? "У клієнта ще немає авто."
            : "Нічого не знайдено. Спробуйте номер, VIN або модель."}
        </div>
      {:else}
        {#each rows as car, index (car.car.id)}
          {@const isActive = index === activeIndex}
          <button
            type="button"
            data-owned-car-index={index}
            role="option"
            aria-selected={isActive}
            tabindex="-1"
            class={cn(
              "flex h-[54px] w-full items-center gap-3 rounded-[7px] px-2 text-left outline-none",
              isActive && "bg-muted/70",
            )}
            onmousedown={(event) => event.preventDefault()}
            onmouseenter={() => (activeIndex = index)}
            onclick={() => choose(car)}
          >
            <span class="size-7 shrink-0">
              <CarLogoLoader slug={car.brand.iconSlug} name={car.brand.name} />
            </span>
            <span class="min-w-0 flex-1">
              <span
                class="block truncate text-[13px] leading-tight font-medium"
              >
                <HighlightText text={carName(car)} {query} />
              </span>
              <span
                class="mt-0.5 block truncate text-[11.5px] leading-tight text-muted-foreground"
              >
                {car.car.year}
              </span>
            </span>
            {#if car.car.licensePlate}
              <LicensePlate
                value={car.car.licensePlate}
                plateType={car.car.plateType}
                country={car.car.plateCountry}
                size="sm"
                class="shrink-0"
              />
            {/if}
          </button>
        {/each}
      {/if}
    </div>

    <div
      class="border-t border-border/70 px-4 py-2 text-[10.5px] text-muted-foreground/70"
    >
      ↑↓ вибір · Enter обрати · Esc закрити
    </div>
  </div>
</div>
