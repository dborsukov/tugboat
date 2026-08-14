import type { Component } from "svelte";

import AwaitingPartsIcon from "@lucide/svelte/icons/boxes";
import ReadyForPickupIcon from "@lucide/svelte/icons/flag";
import AwaitingApprovalIcon from "@lucide/svelte/icons/handshake";
import ReceivedIcon from "@lucide/svelte/icons/inbox";
import InDiagnosticsIcon from "@lucide/svelte/icons/stethoscope";
import PaidIcon from "@lucide/svelte/icons/wallet";
import WorkingIcon from "@lucide/svelte/icons/wrench";
import CanceledIcon from "@lucide/svelte/icons/x";

import type { FacetedFilterOptions } from "$lib/components/data-table-faceted-filter/faceted-filter-options";

import type { WorkOrderStatus } from "$lib/bindings";

export const statuses: WorkOrderStatus[] = [
  "Received",
  "InDiagnostics",
  "AwaitingApproval",
  "AwaitingParts",
  "Working",
  "ReadyForPickup",
  "Paid",
  "Canceled",
];

export type StatusColor =
  | "slate"
  | "sky"
  | "amber"
  | "violet"
  | "blue"
  | "teal"
  | "emerald"
  | "rose";

export const statusColorThemes: Record<
  StatusColor,
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
  amber: {
    bgColor: "bg-amber-700",
    textColor: "text-amber-300",
    base: "bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300",
    highlighted:
      "data-highlighted:bg-amber-200 data-highlighted:text-amber-900 dark:data-highlighted:bg-amber-500/25 dark:data-highlighted:text-amber-100",
    border: "border-amber-300 dark:border-amber-500/30",
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
  teal: {
    bgColor: "bg-teal-700",
    textColor: "text-teal-300",
    base: "bg-teal-100 text-teal-700 dark:bg-teal-500/15 dark:text-teal-300",
    highlighted:
      "data-highlighted:bg-teal-200 data-highlighted:text-teal-900 dark:data-highlighted:bg-teal-500/25 dark:data-highlighted:text-teal-100",
    border: "border-teal-300 dark:border-teal-500/30",
  },
  emerald: {
    bgColor: "bg-emerald-700",
    textColor: "text-emerald-300",
    base: "bg-emerald-100 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
    highlighted:
      "data-highlighted:bg-emerald-200 data-highlighted:text-emerald-900 dark:data-highlighted:bg-emerald-500/25 dark:data-highlighted:text-emerald-100",
    border: "border-emerald-300 dark:border-emerald-500/30",
  },
  rose: {
    bgColor: "bg-rose-700",
    textColor: "text-rose-300",
    base: "bg-rose-100 text-rose-700 dark:bg-rose-500/15 dark:text-rose-300",
    highlighted:
      "data-highlighted:bg-rose-200 data-highlighted:text-rose-900 dark:data-highlighted:bg-rose-500/25 dark:data-highlighted:text-rose-100",
    border: "border-rose-300 dark:border-rose-500/30",
  },
};

export const statusConfig: Record<
  WorkOrderStatus,
  {
    text: string;
    icon: Component;
    color: StatusColor;
    rounding: string;
  }
> = {
  Received: {
    text: "Прийнято",
    icon: ReceivedIcon,
    color: "slate",
    rounding: "rounded-t-xl",
  },
  InDiagnostics: {
    text: "На діагностиці",
    icon: InDiagnosticsIcon,
    color: "sky",
    rounding: "",
  },
  AwaitingApproval: {
    text: "Очікує узгодження",
    icon: AwaitingApprovalIcon,
    color: "amber",
    rounding: "",
  },
  AwaitingParts: {
    text: "Очікує запчастини",
    icon: AwaitingPartsIcon,
    color: "amber",
    rounding: "",
  },
  Working: { text: "В роботі", icon: WorkingIcon, color: "blue", rounding: "" },
  ReadyForPickup: {
    text: "Готово до видачі",
    icon: ReadyForPickupIcon,
    color: "emerald",
    rounding: "",
  },
  Paid: { text: "Сплачено", icon: PaidIcon, color: "emerald", rounding: "" },
  Canceled: {
    text: "Скасовано",
    icon: CanceledIcon,
    color: "rose",
    rounding: "rounded-b-xl",
  },
};

export const workOrderStatusStylesRecord = Object.fromEntries(
  Object.entries(statusConfig).map(([status, meta]) => [
    status,
    {
      ...meta,
      ...statusColorThemes[meta.color],
    },
  ]),
) as Record<
  WorkOrderStatus,
  {
    text: string;
    icon: Component;
    color: StatusColor;
    bgColor: string;
    textColor: string;
    base: string;
    highlighted: string;
    rounding: string;
    border: string;
  }
>;

export const workOrderStatusFilterOptions: FacetedFilterOptions[] =
  statuses.map((s) => ({
    label: workOrderStatusStylesRecord[s].text,
    value: s,
    icon: workOrderStatusStylesRecord[s].icon,
  }));
