<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  import type { DiscountType } from "$lib/bindings";

  let {
    type = $bindable<DiscountType>("Fixed"),
    value = $bindable(0),
    inputProps = {},
  }: {
    type: DiscountType;
    value: number;
    inputProps?: Record<string, unknown>;
  } = $props();
</script>

<div class="flex flex-col gap-2">
  <div class="flex gap-2">
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
  <div class="flex gap-2 items-baseline">
    {#if type === "Fixed"}
      <Input
        type="number"
        min="0"
        step="0.01"
        {...inputProps}
        bind:value
        class="flex-1"
      />
      <p>грн.</p>
    {:else}
      <Input
        type="number"
        min="0"
        max="100"
        step="0.01"
        {...inputProps}
        bind:value
        class="flex-1"
      />
      <p>%</p>
    {/if}
  </div>
</div>
