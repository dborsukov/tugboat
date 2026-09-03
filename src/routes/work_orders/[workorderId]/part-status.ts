import type { Component } from "svelte";

import InstalledIcon from "@lucide/svelte/icons/circle-check";
import AwaitingReturnIcon from "@lucide/svelte/icons/history";
import InStockIcon from "@lucide/svelte/icons/package-2";
import ReturnedIcon from "@lucide/svelte/icons/rotate-ccw";
import QuotedIcon from "@lucide/svelte/icons/scan-search";
import OrderedIcon from "@lucide/svelte/icons/truck";
import InstallingIcon from "@lucide/svelte/icons/wrench";

import type { PartStatus } from "$lib/bindings";

export const statuses: PartStatus[] = [
  "Quoted",
  "Ordered",
  "InStock",
  "Installing",
  "Installed",
  "AwaitingReturn",
  "Returned",
];

export type PartStatusColor =
  | "slate"
  | "sky"
  | "violet"
  | "blue"
  | "emerald"
  | "amber"
  | "gray";

export const partStatusColorThemes: Record<
  PartStatusColor,
  {
    bgColor: string;
    textColor: string;
    base: string;
    highlighted: string;
    border: string;
  }
> = {
  slate: {
    bgColor: "bg-slate-700",
    textColor: "text-slate-300",
    base: "bg-slate-100 text-slate-700 dark:bg-slate-500/15 dark:text-slate-300",
    highlighted:
      "data-highlighted:bg-slate-200 data-highlighted:text-slate-900 dark:data-highlighted:bg-slate-500/25 dark:data-highlighted:text-slate-100",
    border: "border-slate-300 dark:border-slate-500/30",
  },
  sky: {
    bgColor: "bg-sky-700",
    textColor: "text-sky-300",
    base: "bg-sky-100 text-sky-800 dark:bg-sky-500/15 dark:text-sky-300",
    highlighted:
      "data-highlighted:bg-sky-200 data-highlighted:text-sky-900 dark:data-highlighted:bg-sky-500/25 dark:data-highlighted:text-sky-100",
    border: "border-sky-300 dark:border-sky-500/30",
  },
  violet: {
    bgColor: "bg-violet-700",
    textColor: "text-violet-300",
    base: "bg-violet-100 text-violet-700 dark:bg-violet-500/15 dark:text-violet-300",
    highlighted:
      "data-highlighted:bg-violet-200 data-highlighted:text-violet-900 dark:data-highlighted:bg-violet-500/25 dark:data-highlighted:text-violet-100",
    border: "border-violet-300 dark:border-violet-500/30",
  },
  blue: {
    bgColor: "bg-blue-700",
    textColor: "text-blue-300",
    base: "bg-blue-100 text-blue-700 dark:bg-blue-500/15 dark:text-blue-300",
    highlighted:
      "data-highlighted:bg-blue-200 data-highlighted:text-blue-900 dark:data-highlighted:bg-blue-500/25 dark:data-highlighted:text-blue-100",
    border: "border-blue-300 dark:border-blue-500/30",
  },
  emerald: {
    bgColor: "bg-emerald-700",
    textColor: "text-emerald-300",
    base: "bg-emerald-100 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
    highlighted:
      "data-highlighted:bg-emerald-200 data-highlighted:text-emerald-900 dark:data-highlighted:bg-emerald-500/25 dark:data-highlighted:text-emerald-100",
    border: "border-emerald-300 dark:border-emerald-500/30",
  },
  amber: {
    bgColor: "bg-amber-700",
    textColor: "text-amber-300",
    base: "bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300",
    highlighted:
      "data-highlighted:bg-amber-200 data-highlighted:text-amber-900 dark:data-highlighted:bg-amber-500/25 dark:data-highlighted:text-amber-100",
    border: "border-amber-300 dark:border-amber-500/30",
  },
  gray: {
    bgColor: "bg-gray-700",
    textColor: "text-gray-300",
    base: "bg-gray-100 text-gray-700 dark:bg-gray-500/15 dark:text-gray-300",
    highlighted:
      "data-highlighted:bg-gray-200 data-highlighted:text-gray-900 dark:data-highlighted:bg-gray-500/25 dark:data-highlighted:text-gray-100",
    border: "border-gray-300 dark:border-gray-500/30",
  },
};

export const partStatusConfig: Record<
  PartStatus,
  {
    text: string;
    icon: Component;
    color: PartStatusColor;
    rounding: string;
  }
> = {
  Quoted: {
    text: "Підбір",
    icon: QuotedIcon,
    color: "slate",
    rounding: "rounded-t-xl",
  },
  Ordered: { text: "Замовлено", icon: OrderedIcon, color: "sky", rounding: "" },
  InStock: {
    text: "В наявності",
    icon: InStockIcon,
    color: "violet",
    rounding: "",
  },
  Installing: {
    text: "Встановлення",
    icon: InstallingIcon,
    color: "blue",
    rounding: "",
  },
  Installed: {
    text: "Встановлено",
    icon: InstalledIcon,
    color: "emerald",
    rounding: "",
  },
  AwaitingReturn: {
    text: "Очікує на повернення",
    icon: AwaitingReturnIcon,
    color: "amber",
    rounding: "",
  },
  Returned: {
    text: "Повернено",
    icon: ReturnedIcon,
    color: "gray",
    rounding: "rounded-b-xl",
  },
};

export const partStatusStylesRecord = Object.fromEntries(
  Object.entries(partStatusConfig).map(([status, meta]) => [
    status,
    {
      ...meta,
      ...partStatusColorThemes[meta.color as PartStatusColor],
    },
  ]),
) as Record<
  PartStatus,
  {
    text: string;
    icon: Component;
    color: PartStatusColor;
    bgColor: string;
    textColor: string;
    base: string;
    highlighted: string;
    rounding: string;
    border: string;
  }
>;
