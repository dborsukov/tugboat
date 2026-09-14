import type { Component } from "svelte";

import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
import CircleOffIcon from "@lucide/svelte/icons/circle-off";

export type WorkerStatusStyle = {
  text: string;
  icon: Component;
  base: string;
};

export const workerStatusStyles: Record<
  "active" | "inactive",
  WorkerStatusStyle
> = {
  active: {
    text: "Активний",
    icon: CircleCheckIcon,
    base: "bg-emerald-100 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
  },
  inactive: {
    text: "Неактивний",
    icon: CircleOffIcon,
    base: "bg-gray-100 text-gray-700 dark:bg-gray-500/15 dark:text-gray-300",
  },
};

export function workerStatusStyle(isActive: boolean): WorkerStatusStyle {
  return isActive ? workerStatusStyles.active : workerStatusStyles.inactive;
}
