<script lang="ts">
  import { TelInput } from "svelte-tel-input";
  import { type Props as TelInputProps } from "svelte-tel-input/types";

  import { cn } from "$lib/utils";

  let {
    value = $bindable(),
    country = $bindable(),
    placeholder = "+380 XX XXX XX XX",
    options,
    class: className,
    ...props
  }: TelInputProps = $props();
</script>

<TelInput
  {...props}
  bind:value
  bind:country
  {placeholder}
  {options}
  class={cn(
    // default shadcn Input classes
    "bg-input/50 focus-visible:border-ring focus-visible:ring-ring/30 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 h-8 rounded-2xl border border-transparent px-2.5 py-1 text-base transition-[color,box-shadow] duration-200 file:h-6 file:text-sm file:font-medium focus-visible:ring-3 aria-invalid:ring-3 md:text-sm file:text-foreground placeholder:text-muted-foreground w-full min-w-0 outline-none file:inline-flex file:border-0 file:bg-transparent disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
/>
