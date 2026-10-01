import type { DashboardModuleHeight, DashboardModuleSize } from "$lib/bindings";

/** Module that does grid arithmetic */

/** The gap between modules, in pixels, mirrors tailwind `gap-4`. */
export const DASHBOARD_GAP = 16;

/** Row height */
export const DASHBOARD_ROW_HEIGHT = 135;

/** Amount of columns */
export const DASHBOARD_COLUMNS = 12;

/** Width presets, should divide `DASHBOARD_COLUMNS` cleanly */
const dashboardWidths = {
  sixth: {
    span: 2,
    label: "⅙",
    title: "Шоста частина рядка",
    description: "шоста частина",
  },
  third: {
    span: 4,
    label: "⅓",
    title: "Третина рядка",
    description: "третина",
  },
  half: {
    span: 6,
    label: "½",
    title: "Половина рядка",
    description: "половина",
  },
  twoThirds: {
    span: 8,
    label: "⅔",
    title: "Дві третини рядка",
    description: "дві третини",
  },
  full: {
    span: 12,
    label: "1",
    title: "Уся ширина рядка",
    description: "повна",
  },
} as const satisfies Record<
  DashboardModuleSize,
  {
    span: number;
    label: string;
    title: string;
    description: string;
  }
>;

/** Height presets */
const dashboardHeights = {
  one: {
    rows: 1,
    label: "1",
    title: "Один рядок заввишки",
    description: "один рядок",
  },
  two: {
    rows: 2,
    label: "2",
    title: "Два рядки заввишки",
    description: "два рядки",
  },
  three: {
    rows: 3,
    label: "3",
    title: "Три рядки заввишки",
    description: "три рядки",
  },
} as const satisfies Record<
  DashboardModuleHeight,
  { rows: number; label: string; title: string; description: string }
>;

export type DashboardSizeInfo = {
  label: string;
  title: string;
  description: string;
};

export type DashboardHeightInfo = DashboardSizeInfo & { rows: number };

export const dashboardSizeInfo: Record<DashboardModuleSize, DashboardSizeInfo> =
  dashboardWidths;

export const dashboardHeightInfo: Record<
  DashboardModuleHeight,
  DashboardHeightInfo
> = dashboardHeights;

export const dashboardWidthColumns: Record<DashboardModuleSize, number> = {
  sixth: dashboardWidths.sixth.span,
  third: dashboardWidths.third.span,
  half: dashboardWidths.half.span,
  twoThirds: dashboardWidths.twoThirds.span,
  full: dashboardWidths.full.span,
};

export const dashboardHeightRows: Record<DashboardModuleHeight, number> = {
  one: dashboardHeights.one.rows,
  two: dashboardHeights.two.rows,
  three: dashboardHeights.three.rows,
};

export const DASHBOARD_MAX_ROWS = 1000;

/** The narrowest flow that still gives every module the width its content needs */
export function stackThreshold(
  needs: readonly { minWidth: number; span: number }[],
  gap = DASHBOARD_GAP,
): number {
  const required = needs.reduce((widest, need) => {
    const span = Math.max(1, Math.min(need.span, DASHBOARD_COLUMNS));
    const track = (need.minWidth - (span - 1) * gap) / span;

    return Math.max(
      widest,
      track * DASHBOARD_COLUMNS + gap * (DASHBOARD_COLUMNS - 1),
    );
  }, 0);

  return Math.ceil(required);
}

export type GridItem = {
  id: string;
  col: number;
  row: number;
  span: number;
  rows: number;
  minWidth: number;
};

export type GridMetrics = {
  width: number;
  gap?: number;
  rowHeight?: number;
  /** Width below which the modules are stacked in one column */
  stackBelow?: number;
};

export type GridPlacement = {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type GridPlan = {
  placements: GridPlacement[];
  /** Height the grid needs, so the container can be sized. */
  height: number;
};

export type GridCell = { col: number; row: number };

export type GridRect = {
  col: number;
  row: number;
  span: number;
  rows: number;
};

export type GridModule = GridRect & {
  id: string;
  fixed?: boolean;
};

/** Plans every module's rectangle */
export function planGrid(
  items: readonly GridItem[],
  metrics: GridMetrics,
): GridPlan {
  const gap = metrics.gap ?? DASHBOARD_GAP;
  const rowHeight = metrics.rowHeight ?? DASHBOARD_ROW_HEIGHT;
  const width = Math.max(0, metrics.width);

  if (width < (metrics.stackBelow ?? stackThreshold(items))) {
    return stackGrid(items, width, { gap, rowHeight });
  }

  const track = trackWidth(width, gap);
  const pitch = track + gap;

  const placements = items.map((item) => {
    const span = unit(item.span, 1, DASHBOARD_COLUMNS);
    const rows = unit(item.rows, 1, DASHBOARD_MAX_ROWS);
    const col = unit(item.col, 0, DASHBOARD_COLUMNS - span);
    const row = unit(item.row, 0, DASHBOARD_MAX_ROWS);

    return {
      id: item.id,
      x: Math.round(cellLeft(col, pitch)),
      y: Math.round(cellTop(row, rowHeight, gap)),
      width: Math.round(spanWidth(span, track, gap)),
      height: Math.round(spanHeight(rows, rowHeight, gap)),
    };
  });

  const bottom = placements.reduce(
    (lowest, placement) =>
      Math.max(lowest, placement.y + placement.height + gap),
    0,
  );

  return { placements, height: Math.round(Math.max(0, bottom - gap)) };
}

/** The width one column is worth */
export function trackWidth(width: number, gap = DASHBOARD_GAP): number {
  return Math.max(
    0,
    (width - gap * (DASHBOARD_COLUMNS - 1)) / DASHBOARD_COLUMNS,
  );
}

function cellLeft(col: number, pitch: number): number {
  return col * pitch;
}

function cellTop(row: number, rowHeight: number, gap: number): number {
  return row * (rowHeight + gap);
}

function spanWidth(span: number, track: number, gap: number): number {
  return span * track + (span - 1) * gap;
}

function spanHeight(rows: number, rowHeight: number, gap: number): number {
  return rows * rowHeight + (rows - 1) * gap;
}

/** Turns a point into the cell a module of that width would sit in */
export function cellFromPoint(
  left: number,
  top: number,
  span: number,
  metrics: GridMetrics,
): GridCell {
  const gap = metrics.gap ?? DASHBOARD_GAP;
  const rowHeight = metrics.rowHeight ?? DASHBOARD_ROW_HEIGHT;
  const pitch = trackWidth(Math.max(0, metrics.width), gap) + gap;
  const columns = Math.max(1, Math.min(span, DASHBOARD_COLUMNS));

  return {
    col: clamp(Math.round(left / pitch), 0, DASHBOARD_COLUMNS - columns),
    row: Math.max(0, Math.round(top / (rowHeight + gap))),
  };
}

export function cellsOverlap(left: GridRect, right: GridRect): boolean {
  return (
    left.col < right.col + right.span &&
    right.col < left.col + left.span &&
    left.row < right.row + right.rows &&
    right.row < left.row + left.rows
  );
}

/** Moves one module to a cell and pushes whatever it lands on out of the way */
export function settle(
  modules: readonly GridModule[],
  movedId: string,
  cell: GridCell,
): Map<string, GridCell> {
  const positions = new Map<string, GridCell>(
    modules.map((module) => [module.id, { col: module.col, row: module.row }]),
  );

  const moved = modules.find((module) => module.id === movedId);
  if (!moved) return positions;

  const settled: (GridRect & { id: string })[] = [
    {
      id: movedId,
      col: cell.col,
      row: cell.row,
      span: moved.span,
      rows: moved.rows,
    },
  ];
  positions.set(movedId, { col: cell.col, row: cell.row });

  const rest = modules
    .filter((module) => module.id !== movedId)
    .sort((left, right) => left.row - right.row || left.col - right.col);

  for (const module of rest) {
    const rect: GridRect & { id: string } = { ...module };

    for (let pass = 0; pass <= modules.length; pass++) {
      const under = settled.find((other) => cellsOverlap(rect, other));
      if (!under) break;
      rect.row = under.row + under.rows;
    }

    settled.push(rect);
    positions.set(module.id, { col: rect.col, row: rect.row });
  }

  return closeEmptyRows(positions, modules);
}

/** Removes rows nothing sits in, pulling everything below them up */
function closeEmptyRows(
  positions: Map<string, GridCell>,
  modules: readonly GridModule[],
): Map<string, GridCell> {
  const heights = new Map(
    modules.map((module) => [module.id, Math.max(1, module.rows)]),
  );

  const occupied = new Set<number>();
  for (const [id, cell] of positions) {
    const rows = heights.get(id) ?? 1;
    for (let row = cell.row; row < cell.row + rows; row++) occupied.add(row);
  }

  const shifted = new Map<number, number>();
  let seen = 0;
  let removed = 0;
  for (const row of [...occupied].sort((left, right) => left - right)) {
    removed += Math.max(0, row - seen);
    shifted.set(row, row - removed);
    seen = row + 1;
  }

  if (removed === 0) return positions;

  return new Map(
    [...positions].map(([id, cell]) => [
      id,
      { col: cell.col, row: shifted.get(cell.row) ?? cell.row },
    ]),
  );
}

/** Pushes apart modules a stored layout put in the same cell */
export function separateOverlaps(
  positions: Map<string, GridCell>,
  modules: readonly GridModule[],
): Map<string, GridCell> {
  const byId = new Map(modules.map((module) => [module.id, module]));
  const order = [...positions.keys()].sort((left, right) => {
    const a = positions.get(left);
    const b = positions.get(right);
    if (!a || !b) return left.localeCompare(right);
    return a.row - b.row || a.col - b.col || left.localeCompare(right);
  });

  const separated = new Map<string, GridCell>();
  const placed: (GridRect & { id: string })[] = [];
  let moved = false;

  for (const id of order) {
    const cell = positions.get(id);
    if (!cell) continue;

    const module = byId.get(id);
    if (!module) {
      separated.set(id, cell);
      continue;
    }

    const rect: GridRect & { id: string } = {
      id,
      col: cell.col,
      row: cell.row,
      span: module.span,
      rows: module.rows,
    };

    for (let pass = 0; pass <= modules.length; pass++) {
      const under = placed.find((other) => cellsOverlap(rect, other));
      if (!under) break;
      rect.row = under.row + under.rows;
    }

    placed.push(rect);
    if (rect.row === cell.row) {
      separated.set(id, cell);
    } else {
      moved = true;
      separated.set(id, { col: rect.col, row: rect.row });
    }
  }

  return closeEmptyRows(moved ? separated : positions, modules);
}

/** The first free cell for every module that has no position of its own */
export function autoPlace(
  modules: readonly GridModule[],
): Map<string, GridCell> {
  const positions = new Map<string, GridCell>();
  const taken: GridRect[] = [];

  for (const module of modules) {
    if (!module.fixed) continue;
    positions.set(module.id, { col: module.col, row: module.row });
    taken.push({ ...module });
  }

  for (const module of modules) {
    if (module.fixed) continue;

    const span = Math.max(1, Math.min(module.span, DASHBOARD_COLUMNS));
    const rect: GridRect = { col: 0, row: 0, span, rows: module.rows };
    let found: GridCell | null = null;

    // There is always a free cell below everything, so this always ends.
    for (let row = 0; found === null; row++) {
      for (let col = 0; col + span <= DASHBOARD_COLUMNS; col++) {
        rect.col = col;
        rect.row = row;
        if (taken.some((other) => cellsOverlap(rect, other))) continue;

        found = { col, row };
        break;
      }
    }

    taken.push({ ...rect });
    positions.set(module.id, found);
  }

  return closeEmptyRows(positions, modules);
}

/** One column, every module at its own height, in reading order. */
function stackGrid(
  items: readonly GridItem[],
  width: number,
  metrics: { gap: number; rowHeight: number },
): GridPlan {
  const { gap, rowHeight } = metrics;
  const order = [...items].sort(
    (left, right) => left.row - right.row || left.col - right.col,
  );

  let top = 0;
  const placements = order.map((item) => {
    const rows = unit(item.rows, 1, DASHBOARD_MAX_ROWS);
    const height = spanHeight(rows, rowHeight, gap);
    const placement = {
      id: item.id,
      x: 0,
      y: Math.round(top),
      width: Math.round(width),
      height: Math.round(height),
    };
    top += height + gap;
    return placement;
  });

  return { placements, height: Math.round(Math.max(0, top - gap)) };
}

function clamp(value: number, low: number, high: number): number {
  return Math.max(low, Math.min(high, value));
}

function unit(value: number, low: number, high: number): number {
  if (!Number.isFinite(value)) return low;

  return Math.max(low, Math.min(high, Math.round(value)));
}
