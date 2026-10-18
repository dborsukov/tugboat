import type {
  DashboardLayout,
  DashboardModuleHeight,
  DashboardModuleLayout,
  DashboardModuleSize,
} from "$lib/bindings";
import { settingsState } from "$lib/states/settings.svelte";

import {
  autoPlace,
  DASHBOARD_COLUMNS,
  dashboardHeightInfo,
  dashboardHeightRows,
  dashboardWidthColumns,
  separateOverlaps,
  settle,
  stackThreshold,
  type GridCell,
  type GridItem,
} from "./dashboard-grid";
import {
  dashboardModules,
  defaultSlot,
  heightsAt,
  moduleById,
  slotFor,
  widthsAt,
  type DashboardModule,
  type DashboardModuleSetting,
  type DashboardSlot,
} from "./registry";

export type DashboardSettings = Record<string, unknown>;

export { dashboardHeightRows, dashboardWidthColumns };

/** One module, with the arrangement applied to it */
export type LayoutEntry = {
  id: string;
  module: DashboardModule;
  slot: DashboardSlot;
  hidden: boolean;
  col: number;
  row: number;
  settings: ModuleSettings;
};

/** Resolves a saved layout against the registry */
export function resolveLayout(
  saved: DashboardLayout | null | undefined,
): LayoutEntry[] {
  const modules = saved?.modules ?? {};

  const entries = dashboardModules.map((module) => {
    const stored = modules[module.id];

    return {
      id: module.id,
      module,
      slot: slotOf(module, stored),
      hidden: stored?.hidden ?? module.hiddenByDefault ?? false,
      position: stored?.position ?? module.defaultPosition ?? null,
      settings: resolveSettings(module, storedSettings(saved, module.id)),
    };
  });

  // A module that is switched off takes up no room
  const visible = entries
    .filter((entry) => !entry.hidden)
    .map((entry) => {
      const span = dashboardWidthColumns[entry.slot.width];

      return {
        id: entry.id,
        col: Math.max(
          0,
          Math.min(entry.position?.col ?? 0, DASHBOARD_COLUMNS - span),
        ),
        row: Math.max(0, entry.position?.row ?? 0),
        fixed: entry.position !== null,
        span,
        rows: dashboardHeightRows[entry.slot.height],
      };
    });

  const cells = separateOverlaps(autoPlace(visible), visible);

  return entries.map((entry) => {
    const cell = cells.get(entry.id) ?? entry.position ?? { col: 0, row: 0 };

    return {
      id: entry.id,
      module: entry.module,
      slot: entry.slot,
      hidden: entry.hidden,
      col: cell.col,
      row: cell.row,
      settings: entry.settings,
    };
  });
}

/** The slot a module's saved size resolves to, or its own starting slot. */
function slotOf(
  module: DashboardModule,
  stored: DashboardModuleLayout | undefined,
): DashboardSlot {
  const fallback = defaultSlot(module);
  const width = stored?.width ?? fallback.width;
  const height = stored?.height ?? fallback.height;

  return slotFor(module, width, height) ?? fallback;
}

/** The value to save for an arrangement. */
export function toSavedLayout(
  entries: readonly LayoutEntry[],
  saved: DashboardLayout | null | undefined,
): DashboardLayout {
  const modules: Record<string, DashboardModuleLayout> = {
    ...(saved?.modules ?? {}),
  };

  for (const entry of entries) {
    const stored: DashboardModuleLayout = {
      position: { col: entry.col, row: entry.row },
      width: entry.slot.width,
      height: entry.slot.height,
      hidden: entry.hidden,
    };

    // Only a module that offers options stores any
    if (entry.module.settings?.length) {
      stored.settings = offeredSettings(entry.module, entry.settings);
    }

    modules[entry.id] = stored;
  }

  return { modules };
}

export function storedSettings(
  saved: DashboardLayout | null | undefined,
  id: string,
): DashboardSettings | null {
  const stored: unknown = saved?.modules?.[id]?.settings;

  return isSettings(stored) ? stored : null;
}

/** Whether a stored row's options are the JSON object they are meant to be. */
function isSettings(value: unknown): value is DashboardSettings {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function resolveSettings(
  module: DashboardModule | undefined,
  saved: DashboardSettings | null | undefined,
): ModuleSettings {
  const resolved: ModuleSettings = {};

  for (const setting of module?.settings ?? []) {
    if (setting.kind === "number") {
      resolved[setting.id] = clampToRange(
        storedNumber(saved, setting.id) ?? setting.default,
        setting.range,
      );
    } else {
      const stored = saved?.[setting.id];
      resolved[setting.id] =
        typeof stored === "boolean" ? stored : setting.default;
    }
  }

  return resolved;
}

export function settingValue(moduleId: string, id: string): unknown {
  const module = moduleById(moduleId);
  const saved = storedSettings(
    settingsState.settings?.dashboardLayout,
    moduleId,
  );
  const fallback = declaredSetting(module, id)?.default;

  if (fallback === undefined) return undefined;

  return resolveSettings(module, saved)[id] ?? fallback;
}

export function booleanSetting(moduleId: string, id: string): boolean {
  const value = settingValue(moduleId, id);
  if (typeof value === "boolean") return value;

  const declared = declaredSetting(moduleById(moduleId), id);

  return typeof declared?.default === "boolean" ? declared.default : false;
}

export function numberSetting(moduleId: string, id: string): number {
  const value = settingValue(moduleId, id);

  return typeof value === "number" ? value : 0;
}

function declaredSetting(
  module: DashboardModule | undefined,
  id: string,
): DashboardModuleSetting | undefined {
  return module?.settings?.find((setting) => setting.id === id);
}

function storedNumber(
  saved: DashboardSettings | null | undefined,
  id: string,
): number | null {
  const value = saved?.[id];

  return typeof value === "number" ? value : null;
}

function clampToRange(
  value: number,
  range: { readonly min: number; readonly max: number },
): number {
  if (!Number.isFinite(value)) return range.min;

  return Math.min(Math.max(Math.round(value), range.min), range.max);
}

export type ModuleSettings = Record<string, boolean | number>;

function offeredSettings(
  module: DashboardModule,
  settings: ModuleSettings,
): DashboardSettings {
  const offered: DashboardSettings = {};

  for (const setting of module.settings ?? []) {
    offered[setting.id] = settings[setting.id];
  }

  return offered;
}

/** Whether an arrangement is the one a reset would produce */
export function isDefaultLayout(
  saved: DashboardLayout | null | undefined,
): boolean {
  if (!saved) return true;

  const defaults = new Map(
    resolveLayout(null).map((entry) => [entry.id, entry]),
  );

  return resolveLayout(saved).every((entry) => {
    const fallback = defaults.get(entry.id);
    if (!fallback) return true;

    // The entries whole, rather than a field at a time: a field added to
    // `LayoutEntry` is compared the moment it exists, instead of reading as
    // "arranged" until somebody remembers to list it here.
    return (
      entry.hidden === fallback.hidden &&
      entry.col === fallback.col &&
      entry.row === fallback.row &&
      entry.slot.width === fallback.slot.width &&
      entry.slot.height === fallback.slot.height &&
      sameSettings(entry.settings, fallback.settings)
    );
  });
}

/** Whether two resolved sets of options say the same thing, whichever options exist. */
function sameSettings(left: ModuleSettings, right: ModuleSettings): boolean {
  const keys = new Set([...Object.keys(left), ...Object.keys(right)]);

  return [...keys].every((key) => left[key] === right[key]);
}

/** The modules that are switched on, in the registry's order. */
export function visibleEntries(entries: readonly LayoutEntry[]): LayoutEntry[] {
  return entries.filter((entry) => !entry.hidden);
}

/** Moves a module to a cell, pushing whatever it lands on out of the way */
export function withPosition(
  entries: readonly LayoutEntry[],
  id: string,
  col: number,
  row: number,
): readonly LayoutEntry[] {
  const entry = entries.find((candidate) => candidate.id === id);
  if (!entry) return entries;
  if (entry.col === col && entry.row === row) return entries;

  return withSettled(entries, id, { col, row });
}

/** Applies a settled set of cells, leaving the modules that did not move untouched */
function withSettled(
  entries: readonly LayoutEntry[],
  id: string,
  cell: GridCell,
): readonly LayoutEntry[] {
  const settled = settle(visibleEntries(entries).map(entryRect), id, cell);

  return entries.map((entry) => {
    const next = settled.get(entry.id);
    if (!next || (next.col === entry.col && next.row === entry.row))
      return entry;

    return { ...entry, col: next.col, row: next.row };
  });
}

/** An entry as the grid sees it, the rectangle a module occupies, in grid units */
export function entryRect(entry: LayoutEntry): GridItem {
  return {
    id: entry.id,
    col: entry.col,
    row: entry.row,
    span: dashboardWidthColumns[entry.slot.width],
    rows: dashboardHeightRows[entry.slot.height],
    minWidth: entry.module.minWidth,
  };
}

/** Switches a module off, or back on at the end of the page */
export function withHidden(
  entries: readonly LayoutEntry[],
  id: string,
  hidden: boolean,
): readonly LayoutEntry[] {
  const entry = entries.find((candidate) => candidate.id === id);
  if (!entry || entry.hidden === hidden) return entries;

  if (hidden) {
    return entries.map((candidate) =>
      candidate.id === id ? { ...candidate, hidden } : candidate,
    );
  }

  // Switching a module back on appends it to the end of the page
  const bottom = entries.reduce((lowest, candidate) => {
    if (candidate.hidden || candidate.id === id) return lowest;

    return Math.max(
      lowest,
      candidate.row + dashboardHeightRows[candidate.slot.height],
    );
  }, 0);

  return entries.map((candidate) =>
    candidate.id === id
      ? { ...candidate, hidden: false, col: 0, row: bottom }
      : candidate,
  );
}

export function withSetting(
  entries: readonly LayoutEntry[],
  id: string,
  setting: keyof ModuleSettings,
  value: boolean | number,
): readonly LayoutEntry[] {
  const entry = entries.find((candidate) => candidate.id === id);
  if (!entry || entry.settings[setting] === value) return entries;

  return entries.map((candidate) =>
    candidate.id === id
      ? { ...candidate, settings: { ...candidate.settings, [setting]: value } }
      : candidate,
  );
}

export function withWidth(
  entries: readonly LayoutEntry[],
  id: string,
  width: DashboardModuleSize,
): readonly LayoutEntry[] {
  const entry = entries.find((candidate) => candidate.id === id);
  if (!entry) return entries;

  const span = dashboardWidthColumns[width];
  const col = Math.max(0, Math.min(entry.col, DASHBOARD_COLUMNS - span));

  const slot =
    slotFor(entry.module, width, entry.slot.height) ??
    entry.module.slots.find((candidate) => candidate.width === width);
  if (!slot || slot === entry.slot) return entries;

  const widened = entries.map((candidate) =>
    candidate.id === id ? { ...candidate, slot } : candidate,
  );

  return withSettled(widened, id, { col, row: entry.row });
}

export function withHeight(
  entries: readonly LayoutEntry[],
  id: string,
  height: DashboardModuleHeight,
): readonly LayoutEntry[] {
  const entry = entries.find((candidate) => candidate.id === id);
  if (!entry) return entries;

  const slot =
    slotFor(entry.module, entry.slot.width, height) ??
    entry.module.slots.find((candidate) => candidate.height === height);
  if (!slot || slot === entry.slot) return entries;

  const taller = entries.map((candidate) =>
    candidate.id === id ? { ...candidate, slot } : candidate,
  );

  return withSettled(taller, id, { col: entry.col, row: entry.row });
}

export function allowedWidths(entry: LayoutEntry): DashboardModuleSize[] {
  const byWidth = (left: DashboardModuleSize, right: DashboardModuleSize) =>
    dashboardWidthColumns[left] - dashboardWidthColumns[right];
  const widths = widthsAt(entry.module, entry.slot.height).toSorted(byWidth);

  return widths.includes(entry.slot.width)
    ? widths
    : [entry.slot.width, ...widths].toSorted(byWidth);
}

export function allowedHeights(entry: LayoutEntry): DashboardModuleHeight[] {
  return heightsAt(entry.module, entry.slot.width).toSorted(
    (left, right) =>
      dashboardHeightInfo[left].rows - dashboardHeightInfo[right].rows,
  );
}

export const DASHBOARD_STACK_BELOW = stackThreshold(
  dashboardModules.map((module) => ({
    minWidth: module.minWidth,
    span: dashboardWidthColumns[defaultSlot(module).width],
  })),
);
