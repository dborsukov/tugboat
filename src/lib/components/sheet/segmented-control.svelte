<script lang="ts" generics="T extends string">
  import { cn } from "$lib/utils";

  import { sheetControlHeight } from "./field-styles";

  type Option = { value: T; label: string; title?: string };

  let {
    options,
    value,
    onValueChange,
    clearable = true,
    disabled = false,
    ariaLabel,
    class: className,
    selectedClass,
  }: {
    options: Option[];
    value: T | null;
    /** Clicking the selected option again clears the value when clearable. */
    onValueChange?: (value: T | null) => void;
    clearable?: boolean;
    disabled?: boolean;
    ariaLabel?: string;
    class?: string;
    /** Override the filled look of the selected segment. */
    selectedClass?: string;
  } = $props();

  function select(next: T) {
    if (disabled) return;
    const value2 = clearable && value === next ? null : next;
    onValueChange?.(value2);
  }
</script>

<div
  role="radiogroup"
  aria-label={ariaLabel}
  class={cn(
    sheetControlHeight,
    "grid w-full gap-1 rounded-[10px] border border-border/70 bg-muted/25 p-1",
    disabled && "pointer-events-none opacity-50",
    className,
  )}
  style:grid-template-columns={`repeat(${options.length}, minmax(0, 1fr))`}
>
  {#each options as option, index (option.value)}
    {@const selected = value === option.value}
    {@const focusable = selected || (value == null && index === 0)}
    <button
      type="button"
      role="radio"
      aria-checked={selected}
      aria-label={option.title ?? option.label}
      title={option.title ?? option.label}
      tabindex={focusable ? 0 : -1}
      class={cn(
        "flex h-7 min-w-0 items-center justify-center rounded-[7px] px-2.5 text-[12.5px] font-medium whitespace-nowrap transition-colors outline-none",
        "focus-visible:ring-3 focus-visible:ring-brand/25",
        selected
          ? cn(
              "bg-secondary font-semibold text-secondary-foreground",
              selectedClass,
            )
          : "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
      )}
      onclick={() => select(option.value)}
      onkeydown={(event) => {
        if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
        event.preventDefault();
        const index = options.findIndex((item) => item.value === value);
        const step = event.key === "ArrowRight" ? 1 : -1;
        const next =
          options[
            (Math.max(index, 0) + step + options.length) % options.length
          ];
        if (next) {
          onValueChange?.(next.value);
          const buttons =
            event.currentTarget.parentElement?.querySelectorAll("button");
          const nextIndex = options.findIndex(
            (item) => item.value === next.value,
          );
          (buttons?.[nextIndex] as HTMLButtonElement | undefined)?.focus();
        }
      }}
    >
      {option.label}
    </button>
  {/each}
</div>
