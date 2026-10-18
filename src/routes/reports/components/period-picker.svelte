<script lang="ts">
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import XIcon from "@lucide/svelte/icons/x";

  import { Button } from "$lib/components/ui/button";
  import { ButtonGroup } from "$lib/components/ui/button-group";
  import * as Popover from "$lib/components/ui/popover/index.js";

  import { cn } from "$lib/utils";

  import {
    dayText,
    rangeForMode,
    rangeTitle,
    type DayRange,
    type PeriodMode,
  } from "../period";
  import PeriodPanel from "./period-panel.svelte";

  let {
    mode = $bindable<PeriodMode>("month"),
    anchor = $bindable<string>(dayText(new Date())),
    custom = $bindable<DayRange>({ from: "", to: "" }),
    disabled = false,
    active = false,
    onApply,
    onClear,
  }: {
    mode: PeriodMode;
    anchor: string;
    custom: DayRange;
    disabled?: boolean;
    active?: boolean;
    onApply?: (range: DayRange) => void;
    onClear?: () => void;
  } = $props();

  const APPLIED =
    "border-accent bg-accent text-accent-foreground hover:bg-accent/80 hover:text-accent-foreground";

  let open = $state(false);

  let draftMode = $state<PeriodMode>(mode);
  let draftAnchor = $state(anchor);
  let draftCustom = $state<DayRange>(custom);

  const draftRange = $derived(
    rangeForMode(draftMode, draftAnchor, draftCustom),
  );

  /** What the chip shows: the applied period, never the draft. */
  const label = $derived(rangeTitle(rangeForMode(mode, anchor, custom)));

  $effect(() => {
    if (!open) return;
    draftMode = mode;
    draftAnchor = anchor;
    draftCustom =
      custom.from && custom.to ? custom : rangeForMode(mode, anchor, custom);
  });

  function apply() {
    const range = draftRange;
    if (!range.from || !range.to) return;

    mode = draftMode;
    if (draftMode === "custom") {
      custom = draftCustom;
    } else {
      anchor = draftAnchor;
    }

    onApply?.(range);
    open = false;
  }
</script>

<Popover.Root bind:open>
  <ButtonGroup>
    <Popover.Trigger {disabled}>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="outline"
          class={cn("justify-between gap-2", active && APPLIED)}
        >
          <CalendarIcon class="size-4 shrink-0 opacity-60" />
          <span class="max-w-44 truncate">
            {active ? label : "Усі періоди"}
          </span>
          <ChevronDownIcon class="size-4 shrink-0 opacity-60" />
        </Button>
      {/snippet}
    </Popover.Trigger>

    {#if active && onClear}
      <Button
        variant="outline"
        size="icon"
        class={APPLIED}
        aria-label="Прибрати фільтр за періодом"
        title="Прибрати фільтр за періодом"
        {disabled}
        onclick={() => onClear?.()}
      >
        <XIcon />
      </Button>
    {/if}
  </ButtonGroup>

  <Popover.Content align="start" class="w-auto gap-3 p-3">
    <PeriodPanel
      bind:mode={draftMode}
      bind:anchor={draftAnchor}
      bind:custom={draftCustom}
      showSummary
    />

    <Button
      variant="secondary"
      size="lg"
      class="w-full"
      disabled={!draftRange.from || !draftRange.to}
      onclick={apply}
    >
      Застосувати
    </Button>
  </Popover.Content>
</Popover.Root>
