<script lang="ts" generics="T">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";

  import CheckIcon from "@lucide/svelte/icons/check";
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";

  import { Button } from "$lib/components/ui/button";
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";

  import { cn } from "$lib/utils";

  let {
    items,
    value = $bindable(),
    getKey,
    getLabel,
    getSearchValue,
    placeholder = "Оберіть...",
    searchPlaceholder = "Пошук...",
    emptyText = "Нічого не знайдено.",
    triggerClass,
    contentClass = "w-full p-0",
    formProps = {},
    item,
    trigger,
  }: {
    items: T[];
    value: unknown;
    getKey: (item: T) => unknown;
    getLabel: (item: T) => string;
    getSearchValue?: (item: T) => string;
    placeholder?: string;
    searchPlaceholder?: string;
    emptyText?: string;
    triggerClass?: string;
    contentClass?: string;
    formProps?: Record<string, unknown>;
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
        class={cn("w-full justify-between", triggerClass)}
        role="combobox"
        aria-expanded={comboOpen}
      >
        {#if trigger}
          {@render trigger(selected)}
        {:else}
          {selected ? getLabel(selected) : placeholder}
        {/if}
        <ChevronsUpDownIcon class="opacity-50" />
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
                value = getKey(i);
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
