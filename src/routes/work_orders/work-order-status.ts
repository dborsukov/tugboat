import type { Component } from "svelte";

import CanceledIcon from "@lucide/svelte/icons/circle-x";
import ReceivedIcon from "@lucide/svelte/icons/clipboard-list";
import AwaitingApprovalIcon from "@lucide/svelte/icons/file-clock";
import ReadyForPickupIcon from "@lucide/svelte/icons/key";
import AwaitingPartsIcon from "@lucide/svelte/icons/package";
import PaidIcon from "@lucide/svelte/icons/receipt";
import InDiagnosticsIcon from "@lucide/svelte/icons/stethoscope";
import WorkingIcon from "@lucide/svelte/icons/wrench";

import type { WorkOrderStatus } from "$lib/bindings";

import type { FacetedFilterOptions } from "$lib/components/data-table-faceted-filter/faceted-filter-options";

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
    base: "bg-slate-100 text-slate-700",
    highlighted:
      "data-highlighted:bg-slate-200 data-highlighted:text-slate-900",
    rounding: "rounded-b-none",
    border: "border-slate-300",
  },
  InDiagnostics: {
    text: "На діагностиці",
    icon: InDiagnosticsIcon,
    base: "bg-sky-100 text-sky-800",
    highlighted: "data-highlighted:bg-sky-200 data-highlighted:text-sky-900",
    rounding: "",
    border: "border-sky-300",
  },
  AwaitingApproval: {
    text: "Очікує узгодження",
    icon: AwaitingApprovalIcon,
    base: "bg-amber-100 text-amber-700",
    highlighted:
      " data-highlighted:bg-amber-200 data-highlighted:text-amber-900",
    rounding: "",
    border: "border-amber-300",
  },
  AwaitingParts: {
    text: "Очікує запчастини",
    icon: AwaitingPartsIcon,
    base: "bg-violet-100 text-violet-700",
    highlighted:
      " data-highlighted:bg-violet-200 data-highlighted:text-violet-900",
    rounding: "",
    border: "border-violet-300",
  },
  Working: {
    text: "В роботі",
    icon: WorkingIcon,
    base: "bg-blue-100 text-blue-700",
    highlighted: " data-highlighted:bg-blue-200 data-highlighted:text-blue-900",
    rounding: "",
    border: "border-blue-300",
  },
  ReadyForPickup: {
    text: "Готово до видачі",
    icon: ReadyForPickupIcon,
    base: "bg-teal-100 text-teal-700",
    highlighted: " data-highlighted:bg-teal-200 data-highlighted:text-teal-900",
    rounding: "",
    border: "border-teal-300",
  },
  Paid: {
    text: "Сплачено",
    icon: PaidIcon,
    base: "bg-emerald-100 text-emerald-700",
    highlighted:
      " data-highlighted:bg-emerald-200 data-highlighted:text-emerald-900",
    rounding: "",
    border: "border-emerald-300",
  },
  Canceled: {
    text: "Скасовано",
    icon: CanceledIcon,
    base: "bg-rose-100 text-rose-700",
    highlighted: " data-highlighted:bg-rose-200 data-highlighted:text-rose-900",
    rounding: "rounded-t-none",
    border: "border-rose-300",
  },
};

export const statusFilterOptions: FacetedFilterOptions[] = statuses.map(
  (s) => ({
    label: statusStyles[s].text,
    value: s,
    icon: statusStyles[s].icon,
  }),
);
