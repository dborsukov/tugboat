import { cn } from "$lib/utils";

export const sheetControlHeight = "h-9.5";

const controlBase = cn(
  sheetControlHeight,
  "rounded-[8px] border border-border bg-muted/40 text-sm shadow-none",
  "placeholder:text-muted-foreground/70",
  "focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20",
  "disabled:cursor-not-allowed disabled:opacity-50",
);

export const sheetInput = cn(controlBase, "px-3");

export const sheetTextarea = cn(
  "min-h-16 resize-none rounded-[8px] border border-border bg-muted/40 px-3 py-2.5 text-sm shadow-none",
  "placeholder:text-muted-foreground/70",
  "focus-visible:border-brand focus-visible:ring-3 focus-visible:ring-brand/20",
  "disabled:cursor-not-allowed disabled:opacity-50",
);

export const sheetSelectTrigger = cn(
  controlBase,
  "w-full justify-between px-3",
  "data-[size=default]:h-9.5 data-[size=default]:rounded-[8px]",
);

export const sheetLabel =
  "text-[12.5px] leading-none font-normal text-muted-foreground";

export const sheetHint = "text-xs leading-relaxed text-muted-foreground";

/** Compact outline presets used under rate and unit fields. */
export const sheetPresetButton = "h-7 rounded-[6px] px-2 text-[11px]";

export const sheetPrimaryButton = cn(
  sheetControlHeight,
  "shrink-0 rounded-[8px] px-4 text-sm font-semibold disabled:opacity-40",
);

export const sheetSecondaryButton = cn(
  sheetControlHeight,
  "shrink-0 rounded-[8px] px-3 text-xs font-medium",
  "whitespace-normal leading-tight",
);

export const sheetIconButton =
  "size-8 shrink-0 rounded-[8px] text-muted-foreground hover:text-foreground";
