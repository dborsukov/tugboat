import type { Component } from "svelte";

import type {
  DashboardModuleHeight,
  DashboardModuleSize,
  DashboardPosition,
} from "$lib/bindings";

import {
  CLIENT_MIX_ID,
  FINANCIAL_RESULT_ID,
  GROSS_INCOME_ID,
  IN_PROGRESS_ID,
  OUTSTANDING_ID,
  REVENUE_ID,
  TOP_SERVICES_ID,
  VAT_DUE_ID,
} from "./module-ids";
import ClientMix from "./modules/client-mix.svelte";
import FinancialResult from "./modules/financial-result.svelte";
import GrossIncome from "./modules/gross-income.svelte";
import InProgress from "./modules/in-progress.svelte";
import Outstanding from "./modules/outstanding.svelte";
import Revenue from "./modules/revenue.svelte";
import TopServices from "./modules/top-services.svelte";
import VatDue from "./modules/vat-due.svelte";

export type DashboardSlot = {
  width: DashboardModuleSize;
  height: DashboardModuleHeight;
};

export type DashboardModuleSetting =
  | {
      kind: "switch";
      id: string;
      label: string;
      description: string;
      default: boolean;
    }
  | {
      kind: "number";
      id: string;
      label: string;
      description: string;
      default: number;
      range: { readonly min: number; readonly max: number };
    };

/** What every module is handed when it is drawn. */
export type DashboardModuleProps = {
  /** The size the module was placed at, which is what it draws its content for */
  slot: DashboardSlot;
};

export type DashboardModule = {
  /** Module's permanent storage key, declared in `module-ids` */
  id: string;
  /** Name shown in the layout editor */
  title: string;
  /** The module itself: `Revenue`, `Outstanding` and the rest */
  component: Component<any>;
  /** Every size the module may be placed at, and the default */
  slots: readonly DashboardSlot[];
  /** Narrowest the module may be drawn, in pixels */
  minWidth: number;
  /** Options this module offers a control for, if any */
  settings?: readonly DashboardModuleSetting[];
  /** Whether the module ships switched off */
  hiddenByDefault?: boolean;
  /**
   * Where the module sits in the arrangement the dashboard ships with.
   * A module with neither a saved position nor one of these takes the first free cell instead.
   */
  defaultPosition?: DashboardPosition;
};

/** Every combination of the widths and heights a module was laid out for. */
function sizeSlots(
  widths: readonly DashboardModuleSize[],
  heights: readonly DashboardModuleHeight[],
): DashboardSlot[] {
  return heights.flatMap((height) =>
    widths.map((width) => ({
      width,
      height,
    })),
  );
}

/** Moves a module's starting slot to the front, where the layout reads it. */
function startingAt(
  slots: DashboardSlot[],
  width: DashboardModuleSize,
  height: DashboardModuleHeight,
): DashboardSlot[] {
  const index = slots.findIndex(
    (slot) => slot.width === width && slot.height === height,
  );
  if (index <= 0) return slots;

  const [slot] = slots.splice(index, 1);
  return [slot, ...slots];
}

/** Dashboard, all available modules, in default arrangement */
export const dashboardModules: DashboardModule[] = [
  {
    id: REVENUE_ID,
    title: "Виручка цього місяця",
    component: Revenue,
    slots: startingAt(
      sizeSlots(
        ["sixth", "third", "half", "twoThirds", "full"],
        ["one", "two"],
      ),
      "half",
      "two",
    ),
    defaultPosition: { col: 0, row: 0 },
    minWidth: 150,
    settings: [
      {
        kind: "switch",
        id: "showTrendLine",
        label: "Лінія прогнозу",
        description: "Увімкніть, щоб бачити на графіку лінію прогнозу(тренд).",
        default: false,
      },
    ],
  },
  {
    id: GROSS_INCOME_ID,
    title: "Валовий дохід",
    component: GrossIncome,
    slots: startingAt(
      sizeSlots(
        ["sixth", "third", "half", "twoThirds", "full"],
        ["one", "two"],
      ),
      "half",
      "two",
    ),
    defaultPosition: { col: 6, row: 0 },
    minWidth: 150,
    settings: [
      {
        kind: "switch",
        id: "includeLaborCost",
        label: "Враховувати собівартість робіт",
        description:
          "Вимкніть, щоб враховувати собівартість лише для запчастин.",
        default: false,
      },
    ],
  },
  {
    id: IN_PROGRESS_ID,
    title: "Гроші в роботі",
    component: InProgress,
    slots: startingAt(
      sizeSlots(["sixth", "half", "third"], ["one"]),
      "sixth",
      "one",
    ),
    defaultPosition: { col: 0, row: 2 },
    minWidth: 120,
  },
  {
    id: OUTSTANDING_ID,
    title: "Несплачені замовлення",
    component: Outstanding,
    slots: startingAt(
      sizeSlots(["sixth", "half", "third"], ["one"]),
      "sixth",
      "one",
    ),
    defaultPosition: { col: 2, row: 2 },
    minWidth: 120,
  },
  {
    id: VAT_DUE_ID,
    title: "ПДВ нараховано",
    component: VatDue,
    // One row and no taller: a figure, what it is, and the line under it. Ships switched
    // off, because a shop that is not a VAT payer has nothing to read here.
    slots: startingAt(
      sizeSlots(["sixth", "third", "half"], ["one"]),
      "sixth",
      "one",
    ),
    minWidth: 120,
    hiddenByDefault: true,
  },
  {
    id: TOP_SERVICES_ID,
    title: "Топ послуг цього місяця",
    component: TopServices,
    slots: startingAt(
      [
        ...sizeSlots(["third"], ["one"]),
        ...sizeSlots(["half", "twoThirds", "full"], ["one", "two", "three"]),
      ],
      "half",
      "two",
    ),
    defaultPosition: { col: 0, row: 3 },
    minWidth: 256,
    settings: [
      {
        kind: "switch",
        id: "splitAcrossRevenue",
        label: "Ділити всю виручку",
        description:
          "Увімкніть, щоб метрика ділила всю виручку місяця, разом із запчастинами",
        default: false,
      },
    ],
  },
  {
    id: CLIENT_MIX_ID,
    title: "Клієнти цього місяця",
    component: ClientMix,
    slots: startingAt(
      [
        ...sizeSlots(["sixth"], ["one"]),
        ...sizeSlots(
          ["third", "half", "twoThirds", "full"],
          ["one", "two", "three"],
        ),
      ],
      "sixth",
      "one",
    ),
    defaultPosition: { col: 4, row: 2 },
    minWidth: 150,
  },
  {
    id: FINANCIAL_RESULT_ID,
    title: "Фінансовий результат цього місяця",
    component: FinancialResult,
    slots: startingAt(
      [
        ...sizeSlots(["sixth", "third"], ["one", "two", "three"]),
        ...sizeSlots(["half"], ["two", "three"]),
        ...sizeSlots(["twoThirds", "full"], ["two", "three"]),
      ],
      "half",
      "three",
    ),
    defaultPosition: { col: 6, row: 2 },
    minWidth: 140,
  },
];

export function defaultSlot(module: DashboardModule): DashboardSlot {
  return module.slots[0];
}

/** The module with this id, if the registry ships one */
export function moduleById(id: string): DashboardModule | undefined {
  return dashboardModules.find((module) => module.id === id);
}

/** The slot for a size, if the module has one. If not, draws default */
export function slotFor(
  module: DashboardModule,
  width: DashboardModuleSize,
  height: DashboardModuleHeight,
): DashboardSlot | undefined {
  return module.slots.find(
    (slot) => slot.width === width && slot.height === height,
  );
}

/** Widths the module can take while it is that many rows tall. */
export function widthsAt(
  module: DashboardModule,
  height: DashboardModuleHeight,
): DashboardModuleSize[] {
  return module.slots
    .filter((slot) => slot.height === height)
    .map((slot) => slot.width);
}

/** Heights the module can take while it is that wide. */
export function heightsAt(
  module: DashboardModule,
  width: DashboardModuleSize,
): DashboardModuleHeight[] {
  return module.slots
    .filter((slot) => slot.width === width)
    .map((slot) => slot.height);
}
