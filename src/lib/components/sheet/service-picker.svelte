<script lang="ts">
  import { tick } from "svelte";
  import { NumericText } from "svelte-number-format";

  import SearchIcon from "@lucide/svelte/icons/search";
  import XIcon from "@lucide/svelte/icons/x";

  import { Input } from "$lib/components/ui/input";

  import type { Service } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import HighlightText from "./highlight-text.svelte";

  let {
    services,
    query = $bindable(""),
    active = true,
    onSelect,
    onClose,
  }: {
    services: Service[];
    query?: string;
    active?: boolean;
    onSelect: (service: Service, keepOpen?: boolean) => void;
    onClose?: () => void;
  } = $props();

  let searchEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLDivElement | null>(null);
  let activeIndex = $state(0);

  const trimmed = $derived(query.trim());

  /** "600,00", "2 400", "1,5 год." -> "600.00", "2400", "1.5" */
  function numericQuery(value: string): string {
    return value
      .toLocaleLowerCase("uk")
      .replace(/грн\.?|год\.?/g, "")
      .replace(/\s+/g, "")
      .replace(",", ".");
  }

  const rows = $derived.by(() => {
    const text = trimmed.toLocaleLowerCase("uk");
    const number = numericQuery(trimmed);
    return services.filter((service) => {
      if (service.archivedAt != null) return false;
      if (text === "") return true;

      if (
        service.name.toLocaleLowerCase("uk").includes(text) ||
        (service.code ?? "").toLocaleLowerCase("uk").includes(text)
      ) {
        return true;
      }

      if (number === "") return false;
      const value =
        service.priceType === "Fixed"
          ? service.fixedPrice.replace(",", ".")
          : service.laborHours.replace(",", ".");
      return value.includes(number);
    });
  });

  function focusSearch() {
    void tick().then(() => searchEl?.focus());
  }

  function choose(service: Service) {
    onSelect(service);
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

      if (event.shiftKey) {
        onSelect(row, true);
        query = "";
        activeIndex = 0;
        focusSearch();
        return;
      }

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
      `[data-service-index="${activeIndex}"]`,
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
        placeholder="Назва, код, ціна або години…"
        role="combobox"
        aria-expanded={true}
        aria-controls="service-picker-list"
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
          aria-label="Закрити вибір сервісу"
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
        <span>Сервіси</span>
      {:else}
        <span>Знайдено сервісів: {rows.length}</span>
      {/if}
    </div>

    <div
      id="service-picker-list"
      bind:this={listEl}
      role="listbox"
      aria-label="Сервіси"
      class="flex max-h-[352px] flex-col gap-1 overflow-x-hidden overflow-y-auto px-1.5"
    >
      {#if rows.length === 0}
        <div class="px-3 py-6 text-center text-xs text-muted-foreground">
          {trimmed === ""
            ? "Сервісів ще немає."
            : "Нічого не знайдено. Спробуйте назву або код."}
        </div>
      {:else}
        {#each rows as service, index (service.id)}
          {@const isActive = index === activeIndex}
          <button
            type="button"
            data-service-index={index}
            role="option"
            aria-selected={isActive}
            tabindex="-1"
            class={cn(
              "flex h-[58px] w-full items-center gap-3 rounded-[7px] py-1 px-2 text-left outline-none",
              isActive && "bg-muted/70",
            )}
            onmousedown={(event) => event.preventDefault()}
            onmouseenter={() => (activeIndex = index)}
            onclick={() => choose(service)}
          >
            <span class="min-w-0 flex-1">
              <span
                class="block truncate text-[13px] leading-tight font-medium"
              >
                <HighlightText text={service.name} {query} />
              </span>
              <span
                class="mt-0.5 flex min-w-0 items-center gap-1.5 truncate text-[11.5px] leading-tight text-muted-foreground"
              >
                {#if service.code}
                  <span class="font-mono">
                    <HighlightText text={service.code} {query} muted />
                  </span>
                  <span aria-hidden="true">·</span>
                {/if}
                {#if service.priceType === "Fixed"}
                  <NumericText
                    value={service.fixedPrice}
                    formatStyle="currency"
                    currency="UAH"
                    locale="uk-UA"
                    options={{ precision: 2 }}
                  />
                  <span>грн.</span>
                {:else}
                  <span>{service.laborHours} год.</span>
                {/if}
              </span>
            </span>
          </button>
        {/each}
      {/if}
    </div>

    <div class="h-1" aria-hidden="true"></div>

    <div
      class="border-t border-border/70 px-4 py-2 text-[10.5px] text-muted-foreground/70"
    >
      ↑↓ вибір · Enter додати · Shift+Enter додати і продовжити · Esc закрити
    </div>
  </div>
</div>
