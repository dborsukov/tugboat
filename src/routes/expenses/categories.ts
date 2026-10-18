import type { ExpenseCategoryColor } from "$lib/bindings";
import { cn } from "$lib/utils";

export const EXPENSE_CATEGORY_COLORS: ExpenseCategoryColor[] = [
  "slate",
  "red",
  "orange",
  "amber",
  "yellow",
  "lime",
  "green",
  "emerald",
  "teal",
  "cyan",
  "sky",
  "blue",
  "indigo",
  "violet",
  "purple",
  "fuchsia",
  "pink",
  "rose",
  "brown",
  "gray",
];

export const expenseCategoryColorLabels: Record<ExpenseCategoryColor, string> =
  {
    slate: "Сірий",
    red: "Червоний",
    orange: "Оранжевий",
    amber: "Бурштиновий",
    yellow: "Жовтий",
    lime: "Лаймовий",
    green: "Зелений",
    emerald: "Смарагдовий",
    teal: "Бірюзовий",
    cyan: "Блакитний",
    sky: "Небесний",
    blue: "Синій",
    indigo: "Індиго",
    violet: "Фіолетовий",
    purple: "Пурпуровий",
    fuchsia: "Фуксія",
    pink: "Рожевий",
    rose: "Малиновий",
    brown: "Коричневий",
    gray: "Темно-сірий",
  };

const EXPENSE_CATEGORY_COLOR_CLASSES: Record<
  ExpenseCategoryColor,
  { dot: string; fill: string; ring: string }
> = {
  slate: {
    dot: "bg-slate-400",
    fill: "bg-slate-100 text-slate-800 dark:bg-slate-500/15 dark:text-slate-300",
    ring: "ring-slate-400",
  },
  red: {
    dot: "bg-red-400",
    fill: "bg-red-100 text-red-800 dark:bg-red-500/15 dark:text-red-300",
    ring: "ring-red-400",
  },
  orange: {
    dot: "bg-orange-400",
    fill: "bg-orange-100 text-orange-800 dark:bg-orange-500/15 dark:text-orange-300",
    ring: "ring-orange-400",
  },
  amber: {
    dot: "bg-amber-400",
    fill: "bg-amber-100 text-amber-800 dark:bg-amber-500/15 dark:text-amber-300",
    ring: "ring-amber-400",
  },
  yellow: {
    dot: "bg-yellow-400",
    fill: "bg-yellow-100 text-yellow-800 dark:bg-yellow-500/15 dark:text-yellow-300",
    ring: "ring-yellow-400",
  },
  lime: {
    dot: "bg-lime-400",
    fill: "bg-lime-100 text-lime-800 dark:bg-lime-500/15 dark:text-lime-300",
    ring: "ring-lime-400",
  },
  green: {
    dot: "bg-green-400",
    fill: "bg-green-100 text-green-800 dark:bg-green-500/15 dark:text-green-300",
    ring: "ring-green-400",
  },
  emerald: {
    dot: "bg-emerald-400",
    fill: "bg-emerald-100 text-emerald-800 dark:bg-emerald-500/15 dark:text-emerald-300",
    ring: "ring-emerald-400",
  },
  teal: {
    dot: "bg-teal-400",
    fill: "bg-teal-100 text-teal-800 dark:bg-teal-500/15 dark:text-teal-300",
    ring: "ring-teal-400",
  },
  cyan: {
    dot: "bg-cyan-400",
    fill: "bg-cyan-100 text-cyan-800 dark:bg-cyan-500/15 dark:text-cyan-300",
    ring: "ring-cyan-400",
  },
  sky: {
    dot: "bg-sky-400",
    fill: "bg-sky-100 text-sky-800 dark:bg-sky-500/15 dark:text-sky-300",
    ring: "ring-sky-400",
  },
  blue: {
    dot: "bg-blue-400",
    fill: "bg-blue-100 text-blue-800 dark:bg-blue-500/15 dark:text-blue-300",
    ring: "ring-blue-400",
  },
  indigo: {
    dot: "bg-indigo-400",
    fill: "bg-indigo-100 text-indigo-800 dark:bg-indigo-500/15 dark:text-indigo-300",
    ring: "ring-indigo-400",
  },
  violet: {
    dot: "bg-violet-400",
    fill: "bg-violet-100 text-violet-800 dark:bg-violet-500/15 dark:text-violet-300",
    ring: "ring-violet-400",
  },
  purple: {
    dot: "bg-purple-400",
    fill: "bg-purple-100 text-purple-800 dark:bg-purple-500/15 dark:text-purple-300",
    ring: "ring-purple-400",
  },
  fuchsia: {
    dot: "bg-fuchsia-400",
    fill: "bg-fuchsia-100 text-fuchsia-800 dark:bg-fuchsia-500/15 dark:text-fuchsia-300",
    ring: "ring-fuchsia-400",
  },
  pink: {
    dot: "bg-pink-400",
    fill: "bg-pink-100 text-pink-800 dark:bg-pink-500/15 dark:text-pink-300",
    ring: "ring-pink-400",
  },
  rose: {
    dot: "bg-rose-400",
    fill: "bg-rose-100 text-rose-800 dark:bg-rose-500/15 dark:text-rose-300",
    ring: "ring-rose-400",
  },
  brown: {
    dot: "bg-stone-400",
    fill: "bg-stone-100 text-stone-800 dark:bg-stone-500/15 dark:text-stone-300",
    ring: "ring-stone-400",
  },
  gray: {
    dot: "bg-gray-400",
    fill: "bg-gray-100 text-gray-800 dark:bg-gray-500/15 dark:text-gray-300",
    ring: "ring-gray-400",
  },
};

export function expenseCategoryColorClasses(color: string | null | undefined) {
  const known = EXPENSE_CATEGORY_COLORS.includes(color as ExpenseCategoryColor);
  return EXPENSE_CATEGORY_COLOR_CLASSES[
    (known ? color : "slate") as ExpenseCategoryColor
  ];
}

export function expenseCategoryDotClass(color: string | null | undefined) {
  return expenseCategoryColorClasses(color).dot;
}

export function expenseCategoryFillClass(color: string | null | undefined) {
  if (!color) return null;
  return expenseCategoryColorClasses(color).fill;
}

export function expenseCategoryIndicatorClass(
  color: string | null | undefined,
) {
  return cn(
    "inline-block size-3 shrink-0 rounded-full",
    expenseCategoryDotClass(color),
  );
}
