<script lang="ts">
  import type { ComponentProps } from "svelte";
  import { NumberFormatStyle, NumericFormat } from "svelte-number-format";

  import { cn } from "$lib/utils";

  interface Props extends ComponentProps<typeof NumericFormat> {
    class?: string;
  }

  let {
    value = $bindable(),
    onkeydown,
    locale = "ua-UK",
    options = {
      formatStyle: NumberFormatStyle.Decimal,
      currency: "UAH",
      precision: 2,
    },
    class: className,
    ...props
  }: Props = $props();
</script>

<NumericFormat
  {...props}
  bind:value
  {onkeydown}
  {locale}
  {options}
  class={cn(
    // default shadcn Input classes
    "flex-1 bg-input/50 focus-visible:border-ring focus-visible:ring-ring/30 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 h-8 rounded-2xl border border-transparent px-2.5 py-1 text-base transition-[color,box-shadow] duration-200 file:h-6 file:text-sm file:font-medium focus-visible:ring-3 aria-invalid:ring-3 md:text-sm file:text-foreground placeholder:text-muted-foreground w-full min-w-0 outline-none file:inline-flex file:border-0 file:bg-transparent disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
/>
