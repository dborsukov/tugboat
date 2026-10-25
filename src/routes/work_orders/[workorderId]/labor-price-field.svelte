<script lang="ts">
  import { Input } from "$lib/components/ui/input";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import { sheetInput } from "$lib/components/sheet/field-styles";
  import SegmentedControl from "$lib/components/sheet/segmented-control.svelte";

  import type { LaborPriceType } from "$lib/bindings";
  import { cn } from "$lib/utils";

  let {
    type = $bindable<LaborPriceType>("Hourly"),
    value = $bindable("0"),
    inputProps = {},
  }: {
    type: LaborPriceType;
    value: string;
    inputProps?: Record<string, unknown>;
  } = $props();

  const options = [
    { value: "Hourly", label: "По годинам" },
    { value: "Fixed", label: "Фіксована" },
  ] satisfies { value: LaborPriceType; label: string }[];
</script>

<div class="flex flex-col gap-2">
  <SegmentedControl
    {options}
    value={type}
    clearable={false}
    ariaLabel="Тип вартості"
    onValueChange={(next) => next && (type = next as LaborPriceType)}
  />
  <div class="relative">
    {#if type === "Hourly"}
      <Input bind:value class={cn(sheetInput, "pr-12")} {...inputProps} />
      <span
        class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground"
      >
        год.
      </span>
    {:else}
      <CurrencyInput
        bind:value
        class={cn(sheetInput, "pr-14")}
        onkeydown={(event: KeyboardEvent) => {
          if (event.key === "Enter") {
            event.preventDefault();
            const input = event.currentTarget as HTMLInputElement;
            input.blur();
            input.closest("form")?.requestSubmit();
          }
        }}
      />
      <span
        class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground"
      >
        грн.
      </span>
    {/if}
  </div>
</div>
