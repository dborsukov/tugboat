import type { Component } from "svelte";

import CanceledIcon from "@lucide/svelte/icons/circle-x";
import ReceivedIcon from "@lucide/svelte/icons/clipboard-list";
import AwaitingApprovalIcon from "@lucide/svelte/icons/file-clock";
import ReadyForPickupIcon from "@lucide/svelte/icons/key";
import AwaitingPartsIcon from "@lucide/svelte/icons/package";
import PaidIcon from "@lucide/svelte/icons/receipt";
import InDiagnosticsIcon from "@lucide/svelte/icons/stethoscope";
import WorkingIcon from "@lucide/svelte/icons/wrench";

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

export const statusStyles: Record<
  WorkOrderStatus,
  {
    text: string;
    icon: Component;
    base: string;
    highlighted: string;
    rounding: string;
    border: string;
  }
> = {
  Received: {
    text: "Прийнято",
    icon: ReceivedIcon,
    base: "bg-slate-100 text-slate-700 dark:bg-slate-500/15 dark:text-slate-300",
    highlighted:
      "data-highlighted:bg-slate-200 data-highlighted:text-slate-900 dark:data-highlighted:bg-slate-500/25 dark:data-highlighted:text-slate-100",
    rounding: "rounded-t-xl",
    border: "border-slate-300 dark:border-slate-500/30",
  },
  InDiagnostics: {
    text: "На діагностиці",
    icon: InDiagnosticsIcon,
    base: "bg-sky-100 text-sky-800 dark:bg-sky-500/15 dark:text-sky-300",
    highlighted:
      "data-highlighted:bg-sky-200 data-highlighted:text-sky-900 dark:data-highlighted:bg-sky-500/25 dark:data-highlighted:text-sky-100",
    rounding: "",
    border: "border-sky-300 dark:border-sky-500/30",
  },
  AwaitingApproval: {
    text: "Очікує узгодження",
    icon: AwaitingApprovalIcon,
    base: "bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300",
    highlighted:
      "data-highlighted:bg-amber-200 data-highlighted:text-amber-900 dark:data-highlighted:bg-amber-500/25 dark:data-highlighted:text-amber-100",
    rounding: "",
    border: "border-amber-300 dark:border-amber-500/30",
  },
  AwaitingParts: {
    text: "Очікує запчастини",
    icon: AwaitingPartsIcon,
    base: "bg-violet-100 text-violet-700 dark:bg-violet-500/15 dark:text-violet-300",
    highlighted:
      "data-highlighted:bg-violet-200 data-highlighted:text-violet-900 dark:data-highlighted:bg-violet-500/25 dark:data-highlighted:text-violet-100",
    rounding: "",
    border: "border-violet-300 dark:border-violet-500/30",
  },
  Working: {
    text: "В роботі",
    icon: WorkingIcon,
    base: "bg-blue-100 text-blue-700 dark:bg-blue-500/15 dark:text-blue-300",
    highlighted:
      "data-highlighted:bg-blue-200 data-highlighted:text-blue-900 dark:data-highlighted:bg-blue-500/25 dark:data-highlighted:text-blue-100",
    rounding: "",
    border: "border-blue-300 dark:border-blue-500/30",
  },
  ReadyForPickup: {
    text: "Готово до видачі",
    icon: ReadyForPickupIcon,
    base: "bg-teal-100 text-teal-700 dark:bg-teal-500/15 dark:text-teal-300",
    highlighted:
      "data-highlighted:bg-teal-200 data-highlighted:text-teal-900 dark:data-highlighted:bg-teal-500/25 dark:data-highlighted:text-teal-100",
    rounding: "",
    border: "border-teal-300 dark:border-teal-500/30",
  },
  Paid: {
    text: "Сплачено",
    icon: PaidIcon,
    base: "bg-emerald-100 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
    highlighted:
      "data-highlighted:bg-emerald-200 data-highlighted:text-emerald-900 dark:data-highlighted:bg-emerald-500/25 dark:data-highlighted:text-emerald-100",
    rounding: "",
    border: "border-emerald-300 dark:border-emerald-500/30",
  },
  Canceled: {
    text: "Скасовано",
    icon: CanceledIcon,
    base: "bg-rose-100 text-rose-700 dark:bg-rose-500/15 dark:text-rose-300",
    highlighted:
      "data-highlighted:bg-rose-200 data-highlighted:text-rose-900 dark:data-highlighted:bg-rose-500/25 dark:data-highlighted:text-rose-100",
    rounding: "rounded-b-xl",
    border: "border-rose-300 dark:border-rose-500/30",
  },
};

export const statusFilterOptions: FacetedFilterOptions[] = statuses.map(
  (s) => ({
    label: statusStyles[s].text,
    value: s,
    icon: statusStyles[s].icon,
  }),
);
