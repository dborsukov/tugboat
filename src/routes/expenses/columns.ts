import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";
import DataTableOptionalCell from "$lib/components/data-table-optional-cell.svelte";
import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";
import DataTableSelectCheckbox from "$lib/components/data-table/data-table-select-checkbox.svelte";

import type { Expense } from "$lib/bindings";

import { paymentMethodLabels } from "../work_orders/payment-method";
import ExpenseCategoryCell from "./expense-category-cell.svelte";
import ExpensePaymentMethodCell from "./expense-payment-method-cell.svelte";
import DataTableActions from "./table-actions-cell.svelte";

export interface ExpenseTableMeta {
  onEdit: (id: number) => void;
  onArchive: (id: number) => void;
  onRestore: (id: number) => void;
  onDelete: (id: number) => void;
}

/**
 * Column that carries the filter for the `category` column above it, and is
 * hidden so it never renders. A field is picked by id while the visible column
 * shows and sorts by the joined name, so the two need separate columns; keep
 * the ids in step if either changes.
 *
 * Its accessor returns an array on purpose: `arrIncludesSome` calls
 * `.includes()` on the cell value, and `getFacetedUniqueValues` counts whatever
 * the accessor returns as the row's set of values. Both need the array form.
 */
export const CATEGORY_FILTER_COLUMN_ID = "categoryId";

const expenseDateSorter = (a: Expense, b: Expense) =>
  a.date.localeCompare(b.date);

function expenseDateDisplay(raw: string): string {
  const [year, month, day] = raw.split("-");
  if (!year || !month || !day) return raw;

  const date = new Date(`${year}-${month}-${day}T00:00:00Z`);
  if (Number.isNaN(date.getTime())) return raw;

  return new Intl.DateTimeFormat("uk-UA", {
    day: "numeric",
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  })
    .format(date)
    .replace(/\s*р\.$/, "");
}

function expenseDateSearchValue(raw: string): string {
  const display = expenseDateDisplay(raw);
  return display === raw ? display : `${display} ${raw}`;
}

/* Sort by the label the user sees, not the enum's storage spelling. */
const paymentMethodSorter = (a: Expense, b: Expense) =>
  (a.paymentMethod ? paymentMethodLabels[a.paymentMethod] : "").localeCompare(
    b.paymentMethod ? paymentMethodLabels[b.paymentMethod] : "",
    "uk",
  );

export const columns: ColumnDef<Expense>[] = [
  {
    id: "select",
    header: ({ table }) => {
      return renderComponent(DataTableSelectCheckbox, {
        title: "Обрати всі на сторінці",
        checked: table.getIsAllPageRowsSelected(),
        indeterminate:
          table.getIsSomePageRowsSelected() &&
          !table.getIsAllPageRowsSelected(),
        onCheckedChange: (checked) => table.toggleAllPageRowsSelected(checked),
      });
    },
    cell: ({ row }) => {
      return renderComponent(DataTableSelectCheckbox, {
        title: "Обрати рядок",
        checked: row.getIsSelected(),
        onCheckedChange: (checked) => row.toggleSelected(checked),
      });
    },
  },
  {
    id: "date",
    accessorFn: (row) => expenseDateSearchValue(row.date),
    sortingFn: (rowA, rowB) => expenseDateSorter(rowA.original, rowB.original),
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Дата</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(DataTableOptionalCell, {
        value: expenseDateDisplay(row.original.date),
        valueClass: "tabular-nums whitespace-nowrap",
      });
    },
  },
  {
    id: "description",
    accessorFn: (row) => row.description,
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Опис</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(DataTableOptionalCell, {
        value: row.original.description,
      });
    },
  },
  {
    id: "category",
    accessorFn: (row) => row.category?.name ?? "",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Категорія</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(ExpenseCategoryCell, {
        category: row.original.category,
      });
    },
  },
  {
    id: CATEGORY_FILTER_COLUMN_ID,
    /*
     * The array is what both the filter and the facet counts expect: a bare
     * number would make `arrIncludesSome` call `.includes` on a number.
     * Uncategorized rows contribute nothing, so they never match a filter.
     *
     * TanStack wraps whatever the accessor returns, so the facet map is keyed by
     * this one-element array; the filter wrapper re-keys it by the value's
     * string form so the counts line up with the options.
     */
    accessorFn: (row) => (row.category ? [String(row.category.id)] : []),
    filterFn: "arrIncludesSome",
    enableSorting: false,
    header: () => null,
    cell: () => null,
  },
  {
    id: "paymentMethod",
    accessorFn: (row) => row.paymentMethod ?? "",
    filterFn: "arrIncludesSome",
    sortingFn: (rowA, rowB) =>
      paymentMethodSorter(rowA.original, rowB.original),
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Оплата</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(ExpensePaymentMethodCell, {
        method: row.original.paymentMethod,
      });
    },
  },
  {
    id: "amount",
    accessorFn: (row) => Number(row.amount),
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Сума</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(CurrencyDisplay, {
        amount: row.original.amount,
        suffix: "грн.",
        class: "justify-end",
      });
    },
  },
  {
    id: "actions",
    cell: ({ row, table }) => {
      const meta = table.options.meta as ExpenseTableMeta | undefined;
      return renderComponent(DataTableActions, {
        id: row.original.id,
        archived: row.original.archivedAt != null,
        onEdit: (id: number) => meta?.onEdit?.(id),
        onArchive: (id: number) => meta?.onArchive?.(id),
        onRestore: (id: number) => meta?.onRestore?.(id),
        onDelete: (id: number) => meta?.onDelete?.(id),
      });
    },
  },
];
