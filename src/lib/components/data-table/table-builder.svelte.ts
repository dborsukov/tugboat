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
  type Row,
  type RowSelectionState,
  type SortingState,
  type TableMeta,
  type VisibilityState,
} from "@tanstack/table-core";

import { createSvelteTable } from "../ui/data-table/data-table.svelte";

declare module "@tanstack/table-core" {
  interface FilterFns {
    fuzzy: FilterFn<unknown>;
  }
}

export type TableBuilderOptions<TData, TValue, TMeta> = {
  data: TData[];
  columns: ColumnDef<TData, TValue>[];
  meta?: TMeta;
  searchTerm?: string;
  disableSorting?: boolean;
  disableFiltering?: boolean;
  disablePagination?: boolean;
  disableRowSelection?: boolean;
  initialSorting?: SortingState;
  hiddenColumns?: string[];
  getRowId?: (originalRow: TData, index: number, parent?: Row<TData>) => string;
};

const fuzzyFilter: FilterFn<any> = (row, columnId, value, addMeta) => {
  const itemRank = rankItem(row.getValue(columnId), value);
  addMeta({ itemRank });
  return itemRank.passed;
};

export function buildTable<TData, TValue, TMeta>(
  options: TableBuilderOptions<TData, TValue, TMeta>,
) {
  let sorting = $state<SortingState>(options?.initialSorting || []);
  let pagination = $state<PaginationState>({ pageIndex: 0, pageSize: 10 });
  let columnFilters = $state<ColumnFiltersState>([]);
  let rowSelection = $state<RowSelectionState>({});
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
    get meta() {
      return options.meta as TableMeta<TData> | undefined;
    },
    getRowId: options?.getRowId,

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
      get rowSelection() {
        return rowSelection;
      },
      get columnVisibility() {
        return columnVisibility;
      },
    },
    filterFns: {
      fuzzy: fuzzyFilter,
    },
    globalFilterFn: "fuzzy",
    enableSorting: !options?.disableSorting,
    enableFilters: !options?.disableFiltering,
    enableRowSelection: !options?.disableRowSelection,

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
    onRowSelectionChange: (updater) => {
      rowSelection =
        typeof updater === "function" ? updater(rowSelection) : updater;
    },
    onColumnVisibilityChange: (updater) => {
      columnVisibility =
        typeof updater === "function" ? updater(columnVisibility) : updater;
    },
  });
}
