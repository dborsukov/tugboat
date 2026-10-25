<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import { sheetInput } from "$lib/components/sheet/field-styles";
  import SegmentedControl from "$lib/components/sheet/segmented-control.svelte";

  import type { LaborCostType } from "$lib/bindings";
  import { cn } from "$lib/utils";

  let {
    type = $bindable<LaborCostType>("Percentage"),
    value = $bindable("0"),
    inputProps = {},
    variant = "default",
  }: {
    type: LaborCostType;
    value: string;
    inputProps?: Record<string, unknown>;
    variant?: "default" | "sheet";
  } = $props();

  const options = [
    { value: "Percentage", label: "Відсоток" },
    { value: "Hourly", label: "По годинам" },
  ] satisfies { value: LaborCostType; label: string }[];
</script>

<div class="flex flex-col gap-2">
  {#if variant === "sheet"}
    <SegmentedControl
      {options}
      value={type}
      clearable={false}
      ariaLabel="Тип собівартості"
      onValueChange={(next) => next && (type = next as LaborCostType)}
    />
  {:else}
    <div class="flex gap-2">
      <Button
        type="button"
        variant={type === "Percentage" ? "default" : "outline"}
        class="flex-1"
        onclick={() => (type = "Percentage")}
      >
        Відсоток
      </Button>
      <Button
        type="button"
        variant={type === "Hourly" ? "default" : "outline"}
        class="flex-1"
        onclick={() => (type = "Hourly")}
      >
        По годинам
      </Button>
    </div>
  {/if}

  <div
    class={cn(variant === "sheet" ? "relative" : "flex gap-2 items-baseline")}
  >
    {#if type === "Hourly"}
      <CurrencyInput
        bind:value
        class={variant === "sheet" ? cn(sheetInput, "pr-20") : undefined}
        onkeydown={(e: KeyboardEvent) => {
          if (e.key === "Enter") {
            e.preventDefault();
            const input = e.currentTarget as HTMLInputElement;
            input.blur();
            input.closest("form")?.requestSubmit();
          }
        }}
      />
      {#if variant === "sheet"}
        <span
          class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs whitespace-nowrap text-muted-foreground"
        >
          грн./год.
        </span>
      {:else}
        <p>грн./год.</p>
      {/if}
    {:else}
      <Input
        bind:value
        class={variant === "sheet" ? cn(sheetInput, "pr-8") : "flex-1"}
        {...inputProps}
      />
      {#if variant === "sheet"}
        <span
          class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs text-muted-foreground"
        >
          %
        </span>
      {:else}
        <p>%</p>
      {/if}
    {/if}
  </div>
</div>
