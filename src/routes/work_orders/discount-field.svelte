<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import CurrencyInput from "$lib/components/currency-input.svelte";

  import type { DiscountType } from "$lib/bindings";

  let {
    type = $bindable<DiscountType>("Fixed"),
    value = $bindable("0"),
    inputProps = {},
  }: {
    type: DiscountType;
    value: string;
    inputProps?: Record<string, unknown>;
  } = $props();
</script>

<div class="flex flex-col gap-2">
  <div class="flex gap-2">
    <Button
      type="button"
      variant={type === "None" ? "default" : "outline"}
      class="flex-1"
      onclick={() => (type = "None")}
    >
      Немає
    </Button>
    <Button
      type="button"
      variant={type === "Fixed" ? "default" : "outline"}
      class="flex-1"
      onclick={() => (type = "Fixed")}
    >
      Фіксована
    </Button>
    <Button
      type="button"
      variant={type === "Percentage" ? "default" : "outline"}
      class="flex-1"
      onclick={() => (type = "Percentage")}
    >
      Відсоток
    </Button>
  </div>
  {#if type != "None"}
    <div class="flex gap-2 items-baseline">
      {#if type === "Fixed"}
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
        <p>грн.</p>
      {:else}
        <Input bind:value class="flex-1" {...inputProps} />
        <p>%</p>
      {/if}
    </div>
  {/if}
</div>
