<script lang="ts" generics="T extends string">
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";

  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

  let {
    options,
    value,
    axis,
    heading,
    info,
    onSelect,
    shape,
    class: className,
  }: {
    options: readonly T[];
    value: T;
    axis: "width" | "height";
    heading: string;
    info: Record<T, { label: string; title: string; description: string }>;
    onSelect: (option: T) => void;
    shape: (option: T) => { filled: number; lines: number };
    class?: string;
  } = $props();
</script>

{#snippet glyph(option: T)}
  {@const drawn = shape(option)}
  {#if axis === "height"}
    <span class="flex h-3 w-1.25 flex-col gap-px" aria-hidden="true">
      {#each Array.from({ length: Math.max(1, drawn.lines) }) as _}
        <span class="bg-current flex-1 rounded-[1px]"></span>
      {/each}
    </span>
  {:else}
    <span
      class="flex h-3 w-4 flex-col justify-center gap-0.5"
      aria-hidden="true"
    >
      <span class="bg-current/25 h-1.25 w-full overflow-hidden rounded-[1px]">
        <span
          class="bg-current block h-full rounded-[1px]"
          style="width: {Math.round(drawn.filled * 100)}%"
        ></span>
      </span>
    </span>
  {/if}
{/snippet}

{#if options.length > 1}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="outline"
          size="xs"
          class={className}
          title={info[value].title}
          aria-label="{heading}: {info[value].title}"
        >
          {@render glyph(value)}
          <span class="tabular-nums">{info[value].label}</span>
          <ChevronDownIcon class="text-muted-foreground size-3" />
        </Button>
      {/snippet}
    </DropdownMenu.Trigger>

    <DropdownMenu.Content class="w-max whitespace-nowrap" align="end">
      <DropdownMenu.Label>{heading}</DropdownMenu.Label>
      <DropdownMenu.RadioGroup
        {value}
        onValueChange={(next) => onSelect(next as T)}
      >
        {#each options as option (option)}
          <DropdownMenu.RadioItem
            value={option}
            title={info[option].title}
            class="gap-2"
          >
            {@render glyph(option)}
            <span class="tabular-nums">{info[option].label}</span>
            <span class="text-muted-foreground"
              >— {info[option].description}</span
            >
          </DropdownMenu.RadioItem>
        {/each}
      </DropdownMenu.RadioGroup>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
{/if}
