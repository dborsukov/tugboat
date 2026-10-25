<script lang="ts">
  import { tick } from "svelte";

  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import SearchIcon from "@lucide/svelte/icons/search";

  import { Input } from "$lib/components/ui/input";
  import * as Popover from "$lib/components/ui/popover";

  import { PLATE_TYPES, plateTypeMeta, type PlateType } from "$lib/plate";
  import {
    filterPlateCountries,
    plateCountryName,
    type PlateCountry,
  } from "$lib/plate-countries";
  import { cn } from "$lib/utils";

  import "flag-icons/css/flag-icons.min.css";

  let {
    plateType = "ua",
    plateCountry = null,
    previewType = $bindable<PlateType | null>(null),
    open = $bindable(false),
    onChange,
  }: {
    plateType?: PlateType;
    plateCountry?: string | null;
    previewType?: PlateType | null;
    open?: boolean;
    onChange?: (type: PlateType, country: string | null) => void;
  } = $props();

  type View = "type" | "country";

  let view = $state<View>("type");
  let countryQuery = $state("");
  let activeTypeIndex = $state(0);
  let activeCountryIndex = $state(0);
  let typeListEl = $state<HTMLDivElement | null>(null);
  let countryListEl = $state<HTMLDivElement | null>(null);
  let countrySearchEl = $state<HTMLInputElement | null>(null);
  let portalHost = $state<HTMLDivElement | null>(null);

  const displayType = $derived(previewType ?? plateType);
  const displayMeta = $derived(plateTypeMeta(displayType));
  const displayCountry = $derived(
    displayType === "foreign" ? (plateCountry?.toUpperCase() ?? null) : null,
  );

  const countryRows = $derived(filterPlateCountries(countryQuery));

  function resetToTypes(focus = true) {
    view = "type";
    previewType = null;
    countryQuery = "";
    activeTypeIndex = Math.max(
      PLATE_TYPES.findIndex((meta) => meta.value === plateType),
      0,
    );
    if (focus) focusTypeList();
  }

  function focusTypeList(attempt = 0) {
    void tick().then(() => {
      if (typeListEl) {
        typeListEl.focus();
        return;
      }
      if (attempt < 10) setTimeout(() => focusTypeList(attempt + 1), 30);
    });
  }

  function focusCountrySearch(attempt = 0) {
    void tick().then(() => {
      if (countrySearchEl) {
        countrySearchEl.focus();
        return;
      }
      if (attempt < 10) {
        setTimeout(() => focusCountrySearch(attempt + 1), 30);
      }
    });
  }

  function openCountryView() {
    view = "country";
    previewType = "foreign";
    countryQuery = "";
    const index = countryRows.findIndex(
      (country) => country.code === plateCountry,
    );
    activeCountryIndex = index >= 0 ? index : 0;
    focusCountrySearch();
  }

  function chooseType(next: PlateType) {
    if (next === "foreign") {
      openCountryView();
      return;
    }
    previewType = null;
    open = false;
    onChange?.(next, null);
  }

  function chooseCountry(country: PlateCountry) {
    previewType = null;
    open = false;
    onChange?.("foreign", country.code);
  }

  function activeType() {
    return PLATE_TYPES[activeTypeIndex] ?? PLATE_TYPES[0];
  }

  function activeCountry() {
    return countryRows[activeCountryIndex] ?? null;
  }

  function scrollTypeIntoView() {
    if (!typeListEl) return;
    typeListEl
      .querySelector<HTMLElement>(`[data-type-index="${activeTypeIndex}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }

  function scrollCountryIntoView() {
    if (!countryListEl) return;
    countryListEl
      .querySelector<HTMLElement>(
        `[data-country-index="${activeCountryIndex}"]`,
      )
      ?.scrollIntoView({ block: "nearest" });
  }

  function handleTriggerKeydown(event: KeyboardEvent) {
    if (event.key !== "ArrowDown") return;
    event.preventDefault();
    open = true;
  }

  function handleTypeKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      activeTypeIndex =
        (activeTypeIndex + step + PLATE_TYPES.length) % PLATE_TYPES.length;
      return;
    }
    if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      activeTypeIndex = event.key === "Home" ? 0 : PLATE_TYPES.length - 1;
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      chooseType(activeType().value);
    }
  }

  function moveCountry(step: number) {
    if (countryRows.length === 0) return;
    activeCountryIndex =
      (activeCountryIndex + step + countryRows.length) % countryRows.length;
  }

  function handleCountrySearchKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      moveCountry(event.key === "ArrowDown" ? 1 : -1);
      return;
    }
    if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      activeCountryIndex =
        event.key === "Home" ? 0 : Math.max(countryRows.length - 1, 0);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      const country = activeCountry();
      if (country) chooseCountry(country);
    }
  }

  function handleContentKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      if (view === "country") {
        resetToTypes();
      } else {
        open = false;
      }
      return;
    }
    if (
      view === "country" &&
      event.key === "Backspace" &&
      countryQuery === ""
    ) {
      event.preventDefault();
      event.stopPropagation();
      resetToTypes();
    }
  }

  function handleOpenKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.metaKey || event.ctrlKey || event.altKey) return;

    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      if (view === "country") resetToTypes();
      else open = false;
      return;
    }

    if (view === "type") {
      if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
        event.preventDefault();
        event.stopPropagation();
        typeListEl?.focus();
        if (event.key === "ArrowDown") {
          activeTypeIndex = (activeTypeIndex + 1) % PLATE_TYPES.length;
        } else if (event.key === "ArrowUp") {
          activeTypeIndex =
            (activeTypeIndex - 1 + PLATE_TYPES.length) % PLATE_TYPES.length;
        } else if (event.key === "Home") {
          activeTypeIndex = 0;
        } else {
          activeTypeIndex = PLATE_TYPES.length - 1;
        }
        return;
      }
      if (event.key === "Enter") {
        event.preventDefault();
        event.stopPropagation();
        chooseType(activeType().value);
      }
      return;
    }

    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      event.stopPropagation();
      countrySearchEl?.focus();
      moveCountry(
        event.key === "ArrowDown" ? 1 : event.key === "ArrowUp" ? -1 : 0,
      );
      if (event.key === "Home") activeCountryIndex = 0;
      if (event.key === "End") {
        activeCountryIndex = Math.max(countryRows.length - 1, 0);
      }
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      const country = activeCountry();
      if (country) chooseCountry(country);
      return;
    }
    if (event.key === "Backspace" && countryQuery === "") {
      event.preventDefault();
      event.stopPropagation();
      resetToTypes();
    }
  }

  $effect(() => {
    if (!open) return;
    window.addEventListener("keydown", handleOpenKeydown, true);
    return () => window.removeEventListener("keydown", handleOpenKeydown, true);
  });

  $effect(() => {
    if (open) {
      resetToTypes();
    } else {
      view = "type";
      previewType = null;
      countryQuery = "";
    }
  });

  $effect(() => {
    activeTypeIndex;
    scrollTypeIntoView();
  });

  $effect(() => {
    activeCountryIndex;
    countryRows.length;
    scrollCountryIntoView();
  });

  $effect(() => {
    if (view !== "country" || !countryQuery) return;
    activeCountryIndex = 0;
  });
</script>

<Popover.Root bind:open>
  <div bind:this={portalHost} class="contents"></div>
  <Popover.Trigger
    type="button"
    class={cn(
      "flex h-full w-[100px] shrink-0 items-center gap-1 rounded-l-[7px] border-r border-border px-2 text-xs font-medium outline-none transition-colors",
      "hover:bg-muted/60 focus-visible:bg-muted/70",
      open && "bg-muted/70",
    )}
    aria-label="Тип номерного знака"
    aria-haspopup="listbox"
    aria-expanded={open}
    onkeydown={handleTriggerKeydown}
  >
    {#if displayMeta.triggerKind === "ua_flag"}
      <span class="fi fi-ua h-3 w-4 shrink-0 rounded-[2px]"></span>
      <span>UA</span>
    {:else if displayMeta.triggerKind === "country_flag" && displayCountry}
      <span
        class={cn(
          "fi h-3 w-4 shrink-0 rounded-[2px]",
          `fi-${displayCountry.toLowerCase()}`,
        )}
      ></span>
      <span>{displayCountry}</span>
    {:else}
      <span
        class="text-[10.5px] font-bold tracking-[0.06em] whitespace-nowrap text-muted-foreground uppercase"
      >
        {displayMeta.triggerTag ?? "ІНШИЙ"}
      </span>
    {/if}
    <ChevronDownIcon class="ml-auto size-3 shrink-0 opacity-60" />
  </Popover.Trigger>

  <Popover.Content
    align="end"
    sideOffset={6}
    collisionPadding={8}
    class="w-[280px] gap-0 overflow-hidden rounded-[12px] p-0"
    portalProps={{ to: portalHost ?? undefined }}
    onOpenAutoFocus={(event) => event.preventDefault()}
  >
    <div role="presentation" onkeydown={handleContentKeydown}>
      {#if view === "type"}
        <div
          class="flex items-center justify-between px-3 pt-3 pb-1.5 text-[10.5px] font-semibold tracking-[0.06em] text-muted-foreground/70 uppercase"
        >
          Тип номера
        </div>

        <div
          id="plate-type-list"
          bind:this={typeListEl}
          role="listbox"
          aria-label="Тип номерного знака"
          aria-activedescendant={`plate-type-${activeType().value}`}
          tabindex="-1"
          class="flex flex-col gap-0.5 px-1.5 pb-1.5 outline-none"
          onkeydown={handleTypeKeydown}
        >
          {#each PLATE_TYPES as meta, index (meta.value)}
            {@const current = plateType === meta.value}
            <button
              type="button"
              id={`plate-type-${meta.value}`}
              data-type-index={index}
              role="option"
              aria-selected={current}
              tabindex="-1"
              class={cn(
                "flex h-12 w-full items-center gap-2.5 rounded-[8px] px-2 text-left outline-none",
                index === activeTypeIndex && "bg-muted/70",
              )}
              onmouseenter={() => (activeTypeIndex = index)}
              onclick={() => chooseType(meta.value)}
            >
              {#snippet typeTile()}
                <span
                  class="flex h-6 w-8 shrink-0 items-center justify-center overflow-hidden rounded-[3px] border border-border/70 bg-muted/50"
                >
                  {#if meta.previewKind === "ua_flag"}
                    <span class="fi fi-ua h-4 w-6"></span>
                  {:else if meta.value === "foreign" && plateType === "foreign" && plateCountry}
                    <span
                      class={cn(
                        "fi h-4 w-6",
                        `fi-${plateCountry.toLowerCase()}`,
                      )}
                    ></span>
                  {:else if meta.previewKind === "stripes"}
                    <span class="flex h-full w-full">
                      <span class="h-full flex-1 bg-sky-500/80"></span>
                      <span class="h-full flex-1 bg-slate-100/80"></span>
                      <span class="h-full flex-1 bg-rose-500/80"></span>
                    </span>
                  {:else if meta.previewKind === "tag"}
                    <span
                      class="text-[9px] font-bold tracking-[0.06em] text-muted-foreground"
                    >
                      {meta.previewTag}
                    </span>
                  {:else}
                    <span class="text-xs text-muted-foreground">…</span>
                  {/if}
                </span>
              {/snippet}
              {@render typeTile()}

              <span class="min-w-0 flex-1">
                <span
                  class="block truncate text-[13px] leading-tight font-medium"
                >
                  {meta.name}
                </span>
                <span
                  class={cn(
                    "mt-0.5 block truncate text-[11px] leading-tight text-muted-foreground",
                    meta.value !== "foreign" && "font-mono",
                  )}
                >
                  {meta.value === "foreign" ? "обрати країну" : meta.example}
                </span>
              </span>

              {#if current}
                <CheckIcon class="size-3.5 shrink-0 text-brand" />
              {/if}
              {#if meta.value === "foreign"}
                <ChevronRightIcon class="size-3.5 shrink-0 opacity-50" />
              {/if}
            </button>
          {/each}
        </div>

        <div
          class="border-t border-border/70 px-3 py-2 text-[10.5px] text-muted-foreground/70"
        >
          ↑↓ вибір · ↵ обрати · Esc закрити
        </div>
      {:else}
        <div class="flex items-center gap-1 px-2 pt-2 pb-1">
          <button
            type="button"
            class="inline-flex items-center gap-1 rounded-[6px] px-1.5 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
            onclick={() => resetToTypes()}
          >
            <span aria-hidden="true">‹</span>
            Країна
          </button>
        </div>

        <div class="px-2 pb-2">
          <div class="relative">
            <SearchIcon
              class="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground/70"
            />
            <Input
              bind:ref={countrySearchEl}
              bind:value={countryQuery}
              class="h-9.5 rounded-[7px] pr-2.5 pl-8 text-xs"
              placeholder="Пошук країни..."
              role="combobox"
              aria-expanded={true}
              aria-controls="plate-country-list"
              aria-activedescendant={activeCountry()
                ? `plate-country-${activeCountry()!.code}`
                : undefined}
              autocomplete="off"
              spellcheck="false"
              oninput={() => (activeCountryIndex = 0)}
              onkeydown={handleCountrySearchKeydown}
            />
          </div>
        </div>

        <div
          id="plate-country-list"
          bind:this={countryListEl}
          role="listbox"
          aria-label="Країни"
          class="max-h-[288px] overflow-x-hidden overflow-y-auto px-1.5 pb-1.5"
        >
          {#if countryRows.length === 0}
            <p class="px-2 py-6 text-center text-xs text-muted-foreground">
              Країну не знайдено.
            </p>
          {:else}
            {#each countryRows as country, index (country.code)}
              {@render countryRow(country, index)}
            {/each}
          {/if}
        </div>

        <div
          class="border-t border-border/70 px-3 py-2 text-[10.5px] text-muted-foreground/70"
        >
          ↑↓ вибір · ↵ обрати · Esc назад
        </div>
      {/if}
    </div>
  </Popover.Content>
</Popover.Root>

{#snippet countryRow(country: PlateCountry, index: number)}
  <button
    type="button"
    data-country-index={index}
    id={`plate-country-${country.code}`}
    role="option"
    aria-selected={index === activeCountryIndex}
    class={cn(
      "flex h-10 w-full items-center gap-2 rounded-[7px] px-2 text-left outline-none",
      index === activeCountryIndex && "bg-muted/70",
    )}
    onmouseenter={() => (activeCountryIndex = index)}
    onclick={() => chooseCountry(country)}
  >
    <span
      class={cn(
        "fi h-3.5 w-5 shrink-0 rounded-[2px]",
        `fi-${country.code.toLowerCase()}`,
      )}
    ></span>
    <span class="min-w-0 flex-1 truncate text-[12.5px] font-medium">
      {plateCountryName(country.code)}
    </span>
    <span class="shrink-0 text-[10.5px] tabular-nums text-muted-foreground">
      {country.code}
    </span>
  </button>
{/snippet}
