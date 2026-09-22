<script lang="ts" generics="T">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";

  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import XIcon from "@lucide/svelte/icons/x";

  import { Button } from "$lib/components/ui/button";
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";

  import { cn } from "$lib/utils";

  let {
    items,
    value = $bindable(),
    disabled,
    clearable = false,
    getKey,
    getLabel,
    getSearchValue,
    placeholder = "Оберіть...",
    searchPlaceholder = "Пошук...",
    emptyText = "Нічого не знайдено.",
    triggerClass,
    contentClass = "w-full p-0",
    formProps = {},
    onValueChange,
    onCreate,
    createLabel,
    creating = false,
    item,
    trigger,
  }: {
    items: T[];
    value: unknown;
    disabled?: boolean;
    clearable?: boolean;
    getKey: (item: T) => unknown;
    getLabel: (item: T) => string;
    getSearchValue?: (item: T) => string;
    placeholder?: string;
    searchPlaceholder?: string;
    emptyText?: string;
    triggerClass?: string;
    contentClass?: string;
    formProps?: Record<string, unknown>;
    onValueChange?: (value: unknown) => void;
    /** When given, an unmatched search offers to create the typed value. */
    onCreate?: (query: string) => void;
    createLabel?: (query: string) => string;
    creating?: boolean;
    item?: Snippet<[T]>;
    trigger?: Snippet<[T | undefined]>;
  } = $props();

  let comboOpen = $state(false);
  let triggerRef = $state<HTMLButtonElement>(null!);
  let search = $state("");

  let createButton = $state<HTMLButtonElement | null>(null);

  let query = $derived(search.trim());
  let canCreate = $derived(onCreate != null && query.length > 0);

  let selected = $derived(items.find((i) => getKey(i) === value));
  let searchValue = $derived(getSearchValue ?? getLabel);

  function closeAndFocus() {
    comboOpen = false;
    tick().then(() => triggerRef?.focus());
  }

  function submitCreate() {
    if (!onCreate || !canCreate) return;
    const name = query;
    closeAndFocus();
    onCreate(name);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "Enter" || creating || !createButton || !canCreate)
      return;
    event.preventDefault();
    // Keep cmdk's own Enter handling (which would pick the first item) out of it.
    event.stopPropagation();
    submitCreate();
  }

  // cmdk owns its own search state and starts empty every time it mounts, so
  // ours has to be cleared with it or a stale query would resurface.
  $effect(() => {
    if (!comboOpen) search = "";
  });
</script>

<Popover.Root bind:open={comboOpen}>
  <Popover.Trigger bind:ref={triggerRef}>
    {#snippet child({ props: triggerProps })}
      <Button
        {...formProps}
        {...triggerProps}
        variant="outline"
        class={cn(
          "w-full min-w-0 justify-between overflow-hidden",
          triggerClass,
        )}
        role="combobox"
        aria-expanded={comboOpen}
        {disabled}
      >
        {#if trigger}
          {@render trigger(selected)}
        {:else}
          {selected ? getLabel(selected) : placeholder}
        {/if}
        {#if clearable && selected}
          <span
            role="button"
            tabindex="0"
            aria-label="Прибрати вибір"
            class="shrink-0 opacity-70 hover:opacity-100"
            onclick={(e) => {
              e.stopPropagation();
              value = null;
              onValueChange?.(null);
              closeAndFocus();
            }}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                e.stopPropagation();
                value = null;
                onValueChange?.(null);
                closeAndFocus();
              }
            }}
          >
            <XIcon class="size-4" />
          </span>
        {:else}
          <ChevronsUpDownIcon class="opacity-50" />
        {/if}
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class={contentClass} align="start">
    <Command.Root>
      <Command.Input
        placeholder={searchPlaceholder}
        bind:value={search}
        onkeydown={handleKeydown}
      />
      <Command.List>
        <Command.Empty class={canCreate ? "p-1 text-left" : undefined}>
          {#if canCreate}
            <!--
              Deliberately a plain button rather than a `Command.Item`: an item
              would register with cmdk, raise the match count and unmount this
              very block, looping forever.
            -->
            <button
              type="button"
              bind:this={createButton}
              class="hover:bg-accent hover:text-accent-foreground flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-sm disabled:opacity-50"
              disabled={creating}
              onclick={submitCreate}
            >
              <PlusIcon class="size-4 shrink-0 opacity-70" />
              <span class="truncate">
                {creating
                  ? "Створення..."
                  : createLabel
                    ? createLabel(query)
                    : `Створити «${query}»`}
              </span>
            </button>
          {:else}
            {emptyText}
          {/if}
        </Command.Empty>
        <Command.Group>
          {#each items as i (getKey(i))}
            <Command.Item
              value={`${searchValue(i)}-${getKey(i)}`}
              onSelect={() => {
                const next = getKey(i);
                value = next;
                onValueChange?.(next);
                closeAndFocus();
              }}
            >
              <CheckIcon
                class={cn(value !== getKey(i) && "text-transparent")}
              />
              {#if item}
                {@render item(i)}
              {:else}
                {getLabel(i)}
              {/if}
            </Command.Item>
          {/each}
        </Command.Group>
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
