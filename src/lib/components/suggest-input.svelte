<script lang="ts">
  import {
    compareItems,
    rankItem,
    type RankingInfo,
  } from "@tanstack/match-sorter-utils";
  import { tick } from "svelte";

  import { Input } from "$lib/components/ui/input";
  import * as Popover from "$lib/components/ui/popover";

  import { cn } from "$lib/utils";

  let {
    items,
    value = $bindable(""),
    placeholder = "",
    disabled = false,
    maxItems = 50,
    formProps = {},
    class: className,
  }: {
    /** Suggestions for the current context; the typed text is never restricted to them. */
    items: string[];
    value: string;
    placeholder?: string;
    disabled?: boolean;
    /** Guards against a pathological list; the list is scrollable, not infinite. */
    maxItems?: number;
    formProps?: Record<string, unknown>;
    class?: string;
  } = $props();

  const listID = $props.id();

  let open = $state(false);
  let activeIndex = $state(-1);
  let anchorEl = $state<HTMLDivElement | null>(null);
  let inputEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLDivElement | null>(null);

  let query = $derived(value.trim());

  // One word is enough to match an entry, and the best matching word decides
  // how high it ranks, so "octavia a7" and "golf 7" still find their model:
  // words the catalog does not carry refine the list instead of emptying it.
  function matchRank(item: string, words: string[]): RankingInfo | null {
    let best: RankingInfo | null = null;

    for (const word of words) {
      const rank = rankItem(item, word);
      if (!rank.passed) continue;
      if (best === null || rank.rank > best.rank) best = rank;
    }

    return best;
  }

  // Number of typos tolerated in a word: one in a short word, two in a longer
  // one, which also covers a swapped pair ("sueprb" -> Superb).
  function typoBudget(word: string) {
    return word.length <= 5 ? 1 : 2;
  }

  function editDistance(a: string, b: string) {
    const row = Array.from({ length: b.length + 1 }, (_, index) => index);

    for (let i = 1; i <= a.length; i++) {
      let diagonal = row[0];
      row[0] = i;

      for (let j = 1; j <= b.length; j++) {
        const previous = row[j];
        row[j] = Math.min(
          previous + 1,
          row[j - 1] + 1,
          diagonal + (a[i - 1] === b[j - 1] ? 0 : 1),
        );
        diagonal = previous;
      }
    }

    return row[b.length];
  }

  // Closest of the whole entry and each of its words, so "cruiser" is compared
  // against "Cruiser" rather than against "Land Cruiser".
  function closestDistance(item: string, word: string) {
    const target = word.toLowerCase();
    let best = editDistance(item.toLowerCase(), target);

    for (const token of item.toLowerCase().split(/\s+/)) {
      best = Math.min(best, editDistance(token, target));
    }

    return best;
  }

  // List missplelling as well as match-sorter matches
  function typoDistance(item: string, words: string[]): number | null {
    let best: number | null = null;

    for (const word of words) {
      const distance = closestDistance(item, word);
      if (distance <= typoBudget(word) && (best === null || distance < best)) {
        best = distance;
      }
    }

    return best;
  }

  let matches = $derived.by(() => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    // An entry identical to what is already typed is not a suggestion: leaving
    // "Octavia" in the list under "Octavia" is noise. A different spelling of it
    // is still offered, which is what pulls the field towards catalog casing.
    const candidates = items.filter((item) => item.trim() !== query);

    if (words.length === 0) return candidates.slice(0, maxItems);

    const ranked: { item: string; rank: RankingInfo }[] = [];
    const approximate: { item: string; distance: number }[] = [];

    for (const item of candidates) {
      const rank = matchRank(item, words);
      if (rank !== null) {
        ranked.push({ item, rank });
        continue;
      }

      const distance = typoDistance(item, words);
      if (distance !== null) approximate.push({ item, distance });
    }

    ranked.sort((a, b) => compareItems(a.rank, b.rank));
    approximate.sort(
      (a, b) => a.distance - b.distance || a.item.localeCompare(b.item),
    );

    return [...ranked, ...approximate]
      .slice(0, maxItems)
      .map(({ item }) => item);
  });

  let activeID = $derived(
    activeIndex >= 0 && matches[activeIndex] != null
      ? `${listID}-option-${activeIndex}`
      : undefined,
  );

  function close() {
    open = false;
    activeIndex = -1;
  }

  function show() {
    if (matches.length === 0) {
      close();
      return;
    }
    open = true;
  }

  function handleInput() {
    activeIndex = -1;
    show();
  }

  // The list follows the context, so it can also become empty while it is open
  // (the brand changed underneath it), which would leave an empty box behind.
  $effect(() => {
    if (open && matches.length === 0) close();
  });

  // Arrow keys and hovering move the highlight through a list that is taller
  // than its own box, so the highlighted entry has to be brought into view.
  // Only the list itself is scrolled, never the sheet behind it.
  $effect(() => {
    const list = listEl;
    if (!list || !open || activeIndex < 0) return;

    const option = list.querySelector<HTMLElement>(
      `[data-index="${activeIndex}"]`,
    );
    if (!option) return;

    const listRect = list.getBoundingClientRect();
    const optionRect = option.getBoundingClientRect();

    if (optionRect.top < listRect.top) {
      list.scrollTop += optionRect.top - listRect.top;
    } else if (optionRect.bottom > listRect.bottom) {
      list.scrollTop += optionRect.bottom - listRect.bottom;
    }
  });

  async function select(name: string) {
    value = name;
    close();
    // `bind:value` writes the DOM value without firing an event, and the form
    // validates on input, so the field has to report the change itself or a
    // stale error message would stay on screen.
    if (inputEl) {
      inputEl.value = name;
      inputEl.dispatchEvent(new Event("input", { bubbles: true }));
    }
    // Choosing an entry leaves the caret in the field, however it was chosen:
    // the option sits in a portaled layer that can take focus with it on close.
    await tick();
    inputEl?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      if (matches.length === 0) return;
      // Moves the highlight instead of the caret.
      event.preventDefault();
      open = true;
      const step = event.key === "ArrowDown" ? 1 : -1;
      activeIndex =
        activeIndex < 0
          ? step === 1
            ? 0
            : matches.length - 1
          : (activeIndex + step + matches.length) % matches.length;
      return;
    }

    if (event.key === "Enter") {
      const match = activeIndex >= 0 ? matches[activeIndex] : undefined;
      if (!open || match == null) return;
      // Taking a suggestion must not submit the sheet, which a text input in a
      // form does on Enter.
      event.preventDefault();
      event.stopPropagation();
      select(match);
      return;
    }

    if (event.key === "Escape" && open) {
      event.preventDefault();
      close();
    }
  }
</script>

<div class="relative w-full min-w-0" bind:this={anchorEl}>
  <Input
    {...formProps}
    bind:ref={inputEl}
    bind:value
    {disabled}
    {placeholder}
    class={className}
    role="combobox"
    aria-expanded={open}
    aria-controls={open ? listID : undefined}
    aria-activedescendant={activeID}
    aria-autocomplete="list"
    autocomplete="off"
    spellcheck="false"
    oninput={handleInput}
    onfocus={show}
    onkeydown={handleKeydown}
    onblur={close}
  />
</div>

<Popover.Root bind:open>
  <Popover.Content
    customAnchor={anchorEl}
    align="start"
    sideOffset={4}
    trapFocus={false}
    class="w-(--bits-floating-anchor-width) min-w-40 p-0"
    onOpenAutoFocus={(event) => event.preventDefault()}
    onCloseAutoFocus={(event) => event.preventDefault()}
  >
    <div
      id={listID}
      role="listbox"
      bind:this={listEl}
      class="suggestions max-h-72 overflow-x-hidden overflow-y-auto rounded-3xl p-1"
    >
      {#each matches as match, index (match)}
        <!--
          Keyboard access lives on the input
        -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          id={`${listID}-option-${index}`}
          data-index={index}
          role="option"
          aria-selected={index === activeIndex}
          tabindex="-1"
          class={cn(
            "cursor-default rounded-sm px-2 py-1.5 text-sm select-none",
            index === activeIndex && "bg-muted text-foreground",
          )}
          onmousedown={(event) => event.preventDefault()}
          onclick={() => select(match)}
          onmouseenter={() => (activeIndex = index)}
        >
          {match}
        </div>
      {/each}
    </div>
  </Popover.Content>
</Popover.Root>

<style>
  .suggestions {
    scrollbar-width: thin;
    scrollbar-color: color-mix(
        in oklab,
        var(--muted-foreground) 45%,
        transparent
      )
      transparent;
  }
</style>
