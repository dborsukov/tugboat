import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent } from "$lib/components/ui/data-table";

import DataTableOptionalCell from "$lib/components/data-table-optional-cell.svelte";
import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";
import DataTableSelectCheckbox from "$lib/components/data-table/data-table-select-checkbox.svelte";

import type { Worker } from "$lib/bindings";

import { workerFullName } from "./labels";
import DataTableActions from "./table-actions-cell.svelte";
import WorkerNameCell from "./worker-name-cell.svelte";
import WorkerStatusCell from "./worker-status-cell.svelte";

export interface WorkerTableMeta {
  onEdit: (id: number) => void;
  onDelete: (id: number) => void;
}

export const columns: ColumnDef<Worker>[] = [
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
    id: "name",
    accessorFn: (row) => workerFullName(row),
    sortingFn: "alphanumeric",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Ім'я</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(WorkerNameCell, {
        worker: row.original,
      });
    },
  },
  {
    id: "position",
    accessorFn: (row) => row.position ?? "",
    sortingFn: "alphanumeric",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Посада</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(DataTableOptionalCell, {
        value: row.original.position,
      });
    },
  },
  {
    id: "status",
    accessorFn: (row) => (row.isActive ? "Активний" : "Неактивний"),
    sortingFn: "alphanumeric",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Статус</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(WorkerStatusCell, {
        isActive: row.original.isActive,
      });
    },
  },
  {
    id: "actions",
    cell: ({ row, table }) => {
      const meta = table.options.meta as WorkerTableMeta | undefined;
      return renderComponent(DataTableActions, {
        id: row.original.id,
        onEdit: (id: number) => meta?.onEdit?.(id),
        onDelete: (id: number) => meta?.onDelete?.(id),
      });
    },
  },
];
