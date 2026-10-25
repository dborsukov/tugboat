<script lang="ts">
  import { tick } from "svelte";

  import SearchIcon from "@lucide/svelte/icons/search";
  import XIcon from "@lucide/svelte/icons/x";

  import { Input } from "$lib/components/ui/input";

  import PhoneDisplay from "$lib/components/phone-display.svelte";

  import type { ClientListItem } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import ClientAvatar from "../../../routes/clients/client-avatar.svelte";
  import { matchesWords, phoneDigits } from "./car-picker-search";
  import HighlightText from "./highlight-text.svelte";

  let {
    clients,
    recentClientIds = [],
    query = $bindable(""),
    autofocus = false,
    active = true,
    onSelect,
    onClose,
  }: {
    clients: ClientListItem[];
    recentClientIds?: number[];
    query?: string;
    autofocus?: boolean;
    active?: boolean;
    onSelect: (client: ClientListItem) => void;
    onClose?: () => void;
  } = $props();

  const TYPE_LABELS: Record<ClientListItem["clientType"], string> = {
    individual: "Фізична особа",
    fop: "ФОП",
    company: "Організація",
  };

  let searchEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLDivElement | null>(null);
  let activeIndex = $state(0);

  const activeClients = $derived(
    clients.filter((client) => client.archivedAt == null),
  );

  const recentClients = $derived.by(() => {
    const byId = new Map(activeClients.map((client) => [client.id, client]));
    return recentClientIds
      .map((id) => byId.get(id))
      .filter((client): client is ClientListItem => client != null)
      .slice(0, 8);
  });

  const trimmed = $derived(query.trim());

  const rows = $derived.by(() => {
    if (trimmed === "") {
      if (recentClients.length > 0) return recentClients;
      return [...activeClients]
        .sort((a, b) => a.name.localeCompare(b.name, "uk"))
        .slice(0, 8);
    }

    const lower = trimmed.toLowerCase();
    const digits = phoneDigits(trimmed);

    return activeClients
      .filter((client) => {
        if (matchesWords(client.name, trimmed)) return true;
        if ((client.email ?? "").toLowerCase().includes(lower)) return true;
        if ((client.contactEmail ?? "").toLowerCase().includes(lower))
          return true;
        if ((client.phone ?? "").toLowerCase().includes(lower)) return true;
        if ((client.contactPhone ?? "").toLowerCase().includes(lower))
          return true;
        if (digits.length >= 3) {
          return phoneDigits(
            `${client.phone ?? ""}${client.contactPhone ?? ""}`,
          ).includes(digits);
        }
        return false;
      })
      .sort((a, b) => a.name.localeCompare(b.name, "uk"))
      .slice(0, 8);
  });

  const phoneFor = (client: ClientListItem) =>
    client.phone ?? client.contactPhone ?? "";

  function focusSearch() {
    void tick().then(() => searchEl?.focus());
  }

  function select(client: ClientListItem) {
    onSelect(client);
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
      const client = rows[activeIndex];
      if (!client) return;
      event.preventDefault();
      event.stopPropagation();
      select(client);
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (!active || event.key !== "Escape" || !onClose) return;
    event.preventDefault();
    onClose();
  }

  $effect(() => {
    if (!autofocus) return;
    focusSearch();
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
      `[data-client-index="${activeIndex}"]`,
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

<svelte:window onkeydown={handleWindowKeydown} />

<div class="flex flex-col gap-1.5" data-sheet-escape-owner="true">
  <div class="relative">
    <Input
      bind:ref={searchEl}
      bind:value={query}
      class="h-9.5 rounded-[8px] border border-border bg-muted/40 pr-10 pl-9 text-sm shadow-none placeholder:text-muted-foreground/70 focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20"
      placeholder="Ім'я, телефон або email…"
      role="combobox"
      aria-expanded={true}
      aria-controls="client-picker-list"
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
        aria-label="Закрити вибір власника"
        title="Закрити"
        onclick={onClose}
      >
        <XIcon class="size-4" />
      </button>
    {/if}
  </div>

  <div
    class="overflow-hidden rounded-[10px] border border-border bg-popover shadow-xl"
  >
    <div
      class="flex items-center justify-between px-5 pt-2 pb-1 text-[10.5px] font-semibold tracking-[0.06em] text-muted-foreground/70 uppercase"
    >
      {#if trimmed === ""}
        <span>Клієнти</span>
      {:else}
        <span>Знайдено клієнтів: {rows.length}</span>
      {/if}
    </div>

    <div
      id="client-picker-list"
      bind:this={listEl}
      role="listbox"
      aria-label="Клієнти"
      class="flex max-h-[352px] flex-col gap-1 overflow-x-hidden overflow-y-auto px-1.5 pb-1"
    >
      {#if rows.length === 0}
        <div class="px-3 py-6 text-center text-xs text-muted-foreground">
          {trimmed === ""
            ? "Клієнтів ще немає."
            : "Нічого не знайдено. Спробуйте ім'я або телефон."}
        </div>
      {:else}
        {#each rows as client, index (client.id)}
          {@const isActive = index === activeIndex}
          {@const phone = phoneFor(client)}
          <button
            type="button"
            data-client-index={index}
            role="option"
            aria-selected={isActive}
            tabindex="-1"
            class={cn(
              "flex h-[54px] w-full items-center gap-3 rounded-[7px] px-2 text-left outline-none",
              isActive && "bg-muted/70",
            )}
            onmousedown={(event) => event.preventDefault()}
            onmouseenter={() => (activeIndex = index)}
            onclick={() => select(client)}
          >
            <ClientAvatar name={client.name} class="size-7 shrink-0" />
            <span class="min-w-0 flex-1">
              <span
                class="block truncate text-[13px] leading-tight font-medium"
              >
                <HighlightText text={client.name} {query} />
              </span>
              <span
                class="mt-0.5 block truncate text-[11.5px] leading-tight text-muted-foreground"
              >
                {TYPE_LABELS[client.clientType]}
              </span>
            </span>
            {#if phone}
              <span
                class="shrink-0 rounded-[6px] bg-muted/60 px-2 py-1 text-[11px] tabular-nums text-muted-foreground"
              >
                <PhoneDisplay value={phone} />
              </span>
            {/if}
          </button>
        {/each}
      {/if}
    </div>

    <div class="h-1" aria-hidden="true"></div>

    <div
      class="border-t border-border/70 px-5 py-2 text-[10.5px] text-muted-foreground/70"
    >
      ↑↓ вибір · Enter обрати · Esc закрити
    </div>
  </div>
</div>
