<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import CurrencyInput from "$lib/components/currency-input.svelte";

  import type { LaborCostType } from "$lib/bindings";

  let {
    type = $bindable<LaborCostType>("Percentage"),
    value = $bindable("0"),
    inputProps = {},
  }: {
    type: LaborCostType;
    value: string;
    inputProps?: Record<string, unknown>;
  } = $props();
</script>

<div class="flex flex-col gap-2">
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
  <div class="flex gap-2 items-baseline">
    {#if type === "Hourly"}
      <CurrencyInput
        bind:value
        onkeydown={(e: KeyboardEvent) => {
          if (e.key === "Enter") {
            e.preventDefault();
            const input = e.currentTarget as HTMLInputElement;
            input.blur();
            input.closest("form")?.requestSubmit();
          }
        }}
      />
      <p>грн./год.</p>
    {:else}
      <Input bind:value class="flex-1" {...inputProps} />
      <p>%</p>
    {/if}
  </div>
</div>
