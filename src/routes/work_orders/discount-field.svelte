<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import CurrencyInput from "$lib/components/currency-input.svelte";
  import { sheetInput } from "$lib/components/sheet/field-styles";
  import SegmentedControl from "$lib/components/sheet/segmented-control.svelte";

  import type { DiscountType } from "$lib/bindings";
  import { cn } from "$lib/utils";

  let {
    type = $bindable<DiscountType>("Fixed"),
    value = $bindable("0"),
    inputProps = {},
    variant = "default",
  }: {
    type: DiscountType;
    value: string;
    inputProps?: Record<string, unknown>;
    variant?: "default" | "sheet";
  } = $props();

  const options = [
    { value: "None", label: "Немає" },
    { value: "Fixed", label: "Фіксована" },
    { value: "Percentage", label: "Відсоток" },
  ] satisfies { value: DiscountType; label: string }[];

  function selectType(next: DiscountType) {
    if (next === type) return;
    type = next;
    value = next === "None" ? "0" : "";
  }
</script>

<div class="flex flex-col gap-2">
  {#if variant === "sheet"}
    <SegmentedControl
      {options}
      value={type}
      clearable={false}
      ariaLabel="Тип знижки"
      onValueChange={(next) => next && selectType(next as DiscountType)}
    />
  {:else}
    <div class="flex gap-2">
      <Button
        type="button"
        variant={type === "None" ? "default" : "outline"}
        class="flex-1"
        onclick={() => selectType("None")}
      >
        Немає
      </Button>
      <Button
        type="button"
        variant={type === "Fixed" ? "default" : "outline"}
        class="flex-1"
        onclick={() => selectType("Fixed")}
      >
        Фіксована
      </Button>
      <Button
        type="button"
        variant={type === "Percentage" ? "default" : "outline"}
        class="flex-1"
        onclick={() => selectType("Percentage")}
      >
        Відсоток
      </Button>
    </div>
  {/if}

  {#if type != "None"}
    <div
      class={cn(variant === "sheet" ? "relative" : "flex gap-2 items-baseline")}
    >
      {#if type === "Fixed"}
        <CurrencyInput
          bind:value
          class={variant === "sheet" ? cn(sheetInput, "pr-14") : undefined}
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
            грн.
          </span>
        {:else}
          <p>грн.</p>
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
  {/if}
</div>
