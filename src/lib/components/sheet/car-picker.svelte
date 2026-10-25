<script lang="ts">
  import { tick } from "svelte";

  import PlusIcon from "@lucide/svelte/icons/plus";
  import SearchIcon from "@lucide/svelte/icons/search";
  import XIcon from "@lucide/svelte/icons/x";

  import { Input } from "$lib/components/ui/input";

  import CarLogoLoader from "$lib/components/car-logo-loader.svelte";
  import LicensePlate from "$lib/components/license-plate.svelte";

  import type { CarDetailsDto, ClientListItem } from "$lib/bindings";
  import { isUnknownBrand } from "$lib/brands";
  import { cn } from "$lib/utils";

  import ClientAvatar from "../../../routes/clients/client-avatar.svelte";
  import {
    normalizePlateKey,
    recentCars,
    searchCars,
    singleClientOfResults,
    type PickerResult,
  } from "./car-picker-search";
  import HighlightText from "./highlight-text.svelte";

  let {
    cars,
    clients,
    recentCarIds = [],
    scopedClientId = null,
    query = $bindable(""),
    onSelect,
    onClose,
    onNewCarForClient,
    onNewClientAndCar,
    allowCreateActions = true,
    active = true,
  }: {
    cars: CarDetailsDto[];
    clients: ClientListItem[];
    recentCarIds?: number[];
    scopedClientId?: number | null;
    query?: string;
    onSelect: (carId: number) => void;
    onClose?: () => void;
    onNewCarForClient?: (client: ClientListItem) => void;
    onNewClientAndCar?: () => void;
    allowCreateActions?: boolean;
    active?: boolean;
  } = $props();

  let searchEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLDivElement | null>(null);
  let dropdownOpen = $state(true);
  let activeIndex = $state(0);

  const trimmedQuery = $derived(query.trim());
  const results = $derived(
    searchCars(cars, clients, trimmedQuery, scopedClientId),
  );
  const recent = $derived(
    recentCars(cars, clients, recentCarIds, scopedClientId),
  );
  const rows = $derived<PickerResult[]>(
    trimmedQuery === ""
      ? recent.map((item, index) => ({
          kind: "car",
          id: item.car.carDto.car.id,
          rank: index,
          car: item.car,
          owner: item.owner,
        }))
      : results,
  );
  const carCount = $derived(rows.filter((row) => row.kind === "car").length);
  const singleClient = $derived(
    singleClientOfResults(rows, clients, trimmedQuery, scopedClientId),
  );
  const canAddCarForClient = $derived(
    allowCreateActions && singleClient != null,
  );

  function displayOwner(row: Extract<PickerResult, { kind: "car" }>) {
    return row.owner ?? row.car.client;
  }

  function rowKey(row: PickerResult) {
    return `${row.kind}:${row.id}`;
  }

  function focusSearch() {
    void tick().then(() => searchEl?.focus());
  }

  function choose(row: PickerResult) {
    dropdownOpen = false;
    if (row.kind === "car") {
      onSelect(row.id);
      return;
    }
    onNewCarForClient?.(row.client);
  }

  function openAndReset() {
    dropdownOpen = true;
    activeIndex = 0;
  }

  function scrollActiveIntoView() {
    if (!listEl || !dropdownOpen) return;
    const item = listEl.querySelector<HTMLElement>(
      `[data-picker-index="${activeIndex}"]`,
    );
    if (!item) return;
    const listRect = listEl.getBoundingClientRect();
    const itemRect = item.getBoundingClientRect();
    if (itemRect.top < listRect.top) {
      listEl.scrollTop += itemRect.top - listRect.top;
    } else if (itemRect.bottom > listRect.bottom) {
      listEl.scrollTop += itemRect.bottom - listRect.bottom;
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

  // A new set of rows arrives on every keystroke.
  $effect(() => {
    trimmedQuery;
    activeIndex = 0;
  });

  $effect(() => {
    activeIndex;
    rows.length;
    scrollActiveIntoView();
  });

  function closeDropdown() {
    dropdownOpen = false;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (!active || !dropdownOpen || event.key !== "Escape") return;
    event.preventDefault();
    closeDropdown();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      if (!dropdownOpen) return;
      event.preventDefault();
      event.stopPropagation();
      closeDropdown();
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (!dropdownOpen) {
        dropdownOpen = true;
        return;
      }
      if (rows.length === 0) return;
      const step = event.key === "ArrowDown" ? 1 : -1;
      activeIndex = (activeIndex + step + rows.length) % rows.length;
      return;
    }

    if (event.key === "Enter") {
      if (!dropdownOpen) return;
      const row = rows[activeIndex];
      if (!row) return;
      event.preventDefault();
      event.stopPropagation();
      choose(row);
    }
  }

  const plateHighlighted = $derived.by(() => {
    const normalized = normalizePlateKey(query);
    return normalized.length > 0;
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="relative" data-sheet-escape-owner={dropdownOpen}>
  <Input
    bind:ref={searchEl}
    bind:value={query}
    data-sheet-autofocus
    class="h-9.5 rounded-[8px] border border-border bg-muted/40 pr-10 pl-9 text-sm shadow-none placeholder:text-muted-foreground/70 focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20"
    placeholder="Номер, VIN, модель або власник…"
    role="combobox"
    aria-expanded={dropdownOpen}
    aria-controls="car-picker-list"
    aria-autocomplete="list"
    aria-activedescendant={dropdownOpen && rows[activeIndex]
      ? `car-picker-row-${rowKey(rows[activeIndex])}`
      : undefined}
    autocomplete="off"
    spellcheck="false"
    oninput={openAndReset}
    onfocus={() => (dropdownOpen = true)}
    onkeydown={handleKeydown}
  />
  <SearchIcon
    class="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground/70"
  />
  {#if onClose}
    <button
      type="button"
      class="absolute top-1/2 right-1.5 flex size-7 -translate-y-1/2 items-center justify-center rounded-[6px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
      aria-label="Закрити пошук автомобіля"
      title="Закрити"
      onclick={onClose}
    >
      <XIcon class="size-4" />
    </button>
  {/if}
</div>

{#if dropdownOpen}
  <div
    class="mt-1.5 overflow-hidden rounded-[10px] border border-border bg-popover shadow-xl"
  >
    <div
      class="flex items-center justify-between px-3 pt-2 pb-1 text-[10.5px] font-semibold tracking-[0.06em] text-muted-foreground/70 uppercase"
    >
      {#if trimmedQuery === ""}
        <span>Нещодавні авто</span>
      {:else if carCount > 0}
        <span>Знайдено автомобілів: {carCount}</span>
      {:else}
        <span>Знайдено клієнтів: {rows.length}</span>
      {/if}
    </div>

    <div
      id="car-picker-list"
      bind:this={listEl}
      role="listbox"
      aria-label="Результати пошуку автомобілів"
      class="flex max-h-[352px] flex-col gap-1 overflow-x-hidden overflow-y-auto px-1.5"
    >
      {#if rows.length === 0}
        <div class="px-3 py-6 text-center text-xs text-muted-foreground">
          {trimmedQuery === ""
            ? "Авто ще немає. Додайте авто або клієнта нижче."
            : "Нічого не знайдено. Спробуйте номер, VIN, модель або власника."}
        </div>
      {:else}
        {#each rows as row, index (rowKey(row))}
          {@const isActive = index === activeIndex}
          {#if row.kind === "car"}
            {@const owner = displayOwner(row)}
            {@const name = isUnknownBrand(row.car.carDto.brand)
              ? row.car.carDto.car.model
              : `${row.car.carDto.brand.name} ${row.car.carDto.car.model}`}
            {@const plateMatch =
              plateHighlighted &&
              (row.car.carDto.car.plateSearchKey ?? "").includes(
                normalizePlateKey(query),
              )}
            <button
              type="button"
              id={`car-picker-row-${rowKey(row)}`}
              data-picker-index={index}
              role="option"
              aria-selected={isActive}
              tabindex="-1"
              class={cn(
                "flex h-[54px] w-full items-center gap-3 rounded-[7px] px-2 text-left outline-none",
                isActive && "bg-muted/70",
              )}
              onmousedown={(event) => event.preventDefault()}
              onmouseenter={() => (activeIndex = index)}
              onclick={() => choose(row)}
            >
              <span class="size-7 shrink-0">
                <CarLogoLoader
                  slug={row.car.carDto.brand.iconSlug}
                  name={row.car.carDto.brand.name}
                />
              </span>
              <span class="min-w-0 flex-1">
                <span
                  class="block truncate text-[13px] leading-tight font-medium"
                >
                  <HighlightText text={name} {query} />
                </span>
                <span
                  class="mt-0.5 block truncate text-[11.5px] leading-tight text-muted-foreground"
                >
                  <HighlightText text={owner.name} {query} muted />
                </span>
              </span>
              {#if row.car.carDto.car.licensePlate}
                <LicensePlate
                  value={row.car.carDto.car.licensePlate}
                  plateType={row.car.carDto.car.plateType}
                  country={row.car.carDto.car.plateCountry}
                  size="sm"
                  class={cn(
                    "shrink-0",
                    plateMatch &&
                      "ring-2 ring-brand/50 ring-offset-1 ring-offset-popover",
                  )}
                />
              {/if}
            </button>
          {:else}
            <button
              type="button"
              id={`car-picker-row-${rowKey(row)}`}
              data-picker-index={index}
              role="option"
              aria-selected={isActive}
              tabindex="-1"
              class={cn(
                "flex h-[54px] w-full items-center gap-3 rounded-[7px] px-2 text-left outline-none",
                isActive && "bg-muted/70",
              )}
              onmousedown={(event) => event.preventDefault()}
              onmouseenter={() => (activeIndex = index)}
              onclick={() => choose(row)}
            >
              <ClientAvatar name={row.client.name} class="size-7 shrink-0" />
              <span class="min-w-0 flex-1">
                <span
                  class="block truncate text-[13px] leading-tight font-medium"
                >
                  <HighlightText text={row.client.name} {query} />
                </span>
                <span
                  class="mt-0.5 block truncate text-[11.5px] leading-tight text-muted-foreground"
                >
                  (немає авто)
                  {#if row.client.phone}
                    · {row.client.phone}
                  {/if}
                </span>
              </span>
              <span
                class="shrink-0 text-right text-[10.5px] leading-tight font-medium text-brand"
              >
                Нове авто для цього клієнта
              </span>
            </button>
          {/if}
        {/each}
      {/if}
    </div>

    <div class="h-1" aria-hidden="true"></div>

    {#if canAddCarForClient}
      {@const client = singleClient}
      <div class="border-t border-border/70 p-1">
        <button
          type="button"
          class="flex h-8 w-full items-center gap-2 rounded-[6px] px-4 text-left text-xs font-medium text-brand transition-colors hover:bg-brand/10"
          onclick={() => client && onNewCarForClient?.(client)}
        >
          <PlusIcon class="size-3.5 shrink-0" />
          <span class="truncate">Нове авто для «{client?.name}»</span>
        </button>
      </div>
    {/if}

    {#if allowCreateActions && scopedClientId == null && onNewClientAndCar}
      <div class="border-t border-border/70 p-1">
        <button
          type="button"
          class="flex h-8 w-full items-center gap-2 rounded-[6px] px-4 text-left text-xs font-medium text-brand transition-colors hover:bg-brand/10"
          onclick={onNewClientAndCar}
        >
          <PlusIcon class="size-3.5 shrink-0" />
          Новий клієнт і авто
        </button>
      </div>
    {/if}

    <div
      class="border-t border-border/70 px-5 py-2 text-[10.5px] text-muted-foreground/70"
    >
      ↑↓ вибір · Enter обрати · Esc закрити
    </div>
  </div>
{/if}
