<script lang="ts" generics="T">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";

  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
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
    item?: Snippet<[T]>;
    trigger?: Snippet<[T | undefined]>;
  } = $props();

  let comboOpen = $state(false);
  let triggerRef = $state<HTMLButtonElement>(null!);

  let selected = $derived(items.find((i) => getKey(i) === value));
  let searchValue = $derived(getSearchValue ?? getLabel);

  function closeAndFocus() {
    comboOpen = false;
    tick().then(() => triggerRef?.focus());
  }
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
      <Command.Input placeholder={searchPlaceholder} />
      <Command.List>
        <Command.Empty>{emptyText}</Command.Empty>
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
