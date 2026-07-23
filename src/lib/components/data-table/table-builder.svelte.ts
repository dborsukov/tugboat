import { rankItem } from "@tanstack/match-sorter-utils";
import {
  getCoreRowModel,
  getFacetedRowModel,
  getFacetedUniqueValues,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  type ColumnDef,
  type ColumnFiltersState,
  type FilterFn,
  type PaginationState,
  type SortingState,
  type VisibilityState,
} from "@tanstack/table-core";

import { createSvelteTable } from "../ui/data-table/data-table.svelte";

declare module "@tanstack/table-core" {
  interface FilterFns {
    fuzzy: FilterFn<unknown>;
  }
}

export type TableBuilderOptions<TData, TValue> = {
  data: TData[];
  columns: ColumnDef<TData, TValue>[];
  searchTerm?: string;
  disableSorting?: boolean;
  disableFiltering?: boolean;
  disablePagination?: boolean;
  initialSorting?: SortingState;
  hiddenColumns?: string[];
};

const fuzzyFilter: FilterFn<any> = (row, columnId, value, addMeta) => {
  const itemRank = rankItem(row.getValue(columnId), value);
  addMeta({ itemRank });
  return itemRank.passed;
};

export function buildTable<TData, TValue>(
  options: TableBuilderOptions<TData, TValue>,
) {
  let sorting = $state<SortingState>(options?.initialSorting || []);
  let pagination = $state<PaginationState>({ pageIndex: 0, pageSize: 10 });
  let columnFilters = $state<ColumnFiltersState>([]);
  let columnVisibility = $state<VisibilityState>(
    options?.hiddenColumns?.reduce(
      (acc, colId) => ({ ...acc, [colId]: false }),
      {},
    ) || {},
  );

  return createSvelteTable({
    get data() {
      return options.data;
    },
    get columns() {
      return options.columns;
    },
    state: {
      get sorting() {
        return sorting;
      },
      get columnFilters() {
        return columnFilters;
      },
      get pagination() {
        return pagination;
      },
      get globalFilter() {
        return options.searchTerm ?? "";
      },
      get columnVisibility() {
        return columnVisibility;
      },
    },
    filterFns: {
      fuzzy: fuzzyFilter,
    },
    globalFilterFn: "fuzzy",
    getCoreRowModel: getCoreRowModel(),
    getFacetedRowModel: getFacetedRowModel(),
    getFacetedUniqueValues: getFacetedUniqueValues(),
    getSortedRowModel: options?.disableSorting
      ? undefined
      : getSortedRowModel(),
    getFilteredRowModel: options?.disableFiltering
      ? undefined
      : getFilteredRowModel(),
    getPaginationRowModel: options?.disablePagination
      ? undefined
      : getPaginationRowModel(),
    onSortingChange: (updater) => {
      sorting = typeof updater === "function" ? updater(sorting) : updater;
    },
    onColumnFiltersChange: (updater) => {
      columnFilters =
        typeof updater === "function" ? updater(columnFilters) : updater;
    },
    onPaginationChange: (updater) => {
      pagination =
        typeof updater === "function" ? updater(pagination) : updater;
    },
    onColumnVisibilityChange: (updater) => {
      columnVisibility =
        typeof updater === "function" ? updater(columnVisibility) : updater;
    },
  });
}
