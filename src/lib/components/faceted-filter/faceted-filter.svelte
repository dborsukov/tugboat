<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";

  import CheckIcon from "@lucide/svelte/icons/check";
  import FunnelIcon from "@lucide/svelte/icons/funnel";

  import { Badge } from "$lib/components/ui/badge/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Command from "$lib/components/ui/command/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";

  import { cn } from "$lib/utils.js";

  import type { FacetedFilterOptions } from "../data-table-faceted-filter/faceted-filter-options";

  let {
    title,
    options,
    selected,
    facets,
    disabled = false,
    onValueChange,
  }: {
    title: string;
    options: FacetedFilterOptions[];
    selected: SvelteSet<string>;
    facets?: Map<string, number>;
    disabled?: boolean;
    onValueChange?: (values: string[]) => void;
  } = $props();
</script>

<Popover.Root>
  <Popover.Trigger {disabled}>
    {#snippet child({ props })}
      <Button {...props} variant="outline" size="sm" class="h-8 border-dashed">
        <FunnelIcon />
        {title}
        {#if selected.size > 0}
          <Separator orientation="vertical" class="mx-2 h-4" />
          <Badge
            variant="secondary"
            class="rounded-sm px-1 font-normal lg:hidden"
          >
            {selected.size}
          </Badge>
          <div class="hidden space-x-1 lg:flex lg:gap-2">
            {#if selected.size > 2}
              <Badge variant="secondary" class="rounded-sm px-1 font-normal">
                {selected.size} обрано
              </Badge>
            {:else}
              {#each options.filter( (opt) => selected.has(opt.value), ) as option (option)}
                <Badge variant="secondary" class="rounded-sm px-1 font-normal">
                  {option.label}
                </Badge>
              {/each}
            {/if}
          </div>
        {/if}
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-65 p-0" align="start">
    <Command.Root>
      <Command.Input placeholder={title} />
      <Command.List class="min-h-fit">
        <Command.Empty>Результатів не знайдено...</Command.Empty>
        <Command.Group>
          {#each options as option (option)}
            {@const isSelected = selected.has(option.value)}
            <Command.Item
              onSelect={() => {
                if (isSelected) {
                  selected.delete(option.value);
                } else {
                  selected.add(option.value);
                }
                onValueChange?.(Array.from(selected));
              }}
            >
              <div
                class={cn(
                  "border-primary rounded me-2 flex size-4 items-center justify-center border",
                  isSelected
                    ? "bg-primary text-primary-foreground"
                    : "opacity-50 [&_svg]:invisible",
                )}
              >
                <CheckIcon class="size-4" />
              </div>
              {#if option.dotClass}
                <span
                  class={cn("size-2 shrink-0 rounded-full", option.dotClass)}
                ></span>
              {:else if option.icon}
                {@const Icon = option.icon}
                <Icon class="text-muted-foreground" />
              {/if}
              <span class="truncate">{option.label}</span>
              {#if facets?.get(option.value)}
                <span class="font-mono text-sm text-muted-foreground">
                  {facets.get(option.value)}
                </span>
              {/if}
            </Command.Item>
          {/each}
        </Command.Group>
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
