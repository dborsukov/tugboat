import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";
import DataTableOptionalCell from "$lib/components/data-table-optional-cell.svelte";
import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";
import DataTableSelectCheckbox from "$lib/components/data-table/data-table-select-checkbox.svelte";

import type { Service } from "$lib/bindings";

import DataTableActions from "./table-actions-cell.svelte";
import TableDecimalDisplayCell from "./table-decimal-display-cell.svelte";

export interface ServiceTableMeta {
  onEdit: (id: number) => void;
  onArchive: (id: number) => void;
  onRestore: (id: number) => void;
  onDelete: (id: number) => void;
}

export const columns: ColumnDef<Service>[] = [
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
    id: "code",
    accessorFn: (row) => row.code ?? "",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div>Код</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      return renderComponent(DataTableOptionalCell, {
        value: row.original.code,
      });
    },
  },
  {
    accessorKey: "name",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Назва</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () => `<div>${row.original.name}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
  },
  {
    id: "laborHours",
    accessorFn: (row) => Number(row.laborHours),
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Нормо-години</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      if (row.original.priceType === "Hourly") {
        return renderComponent(TableDecimalDisplayCell, {
          value: row.original.laborHours,
        });
      } else {
        return renderSnippet(
          createRawSnippet(() => {
            return {
              render: () => `<p class="text-center">—</p>`,
            };
          }),
        );
      }
    },
  },
  {
    id: "fixedPrice",
    accessorFn: (row) => Number(row.fixedPrice),
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Ціна</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      if (row.original.priceType === "Hourly") {
        return renderSnippet(
          createRawSnippet(() => {
            return {
              render: () => `<p class="text-center">—</p>`,
            };
          }),
        );
      } else {
        return renderComponent(CurrencyDisplay, {
          amount: row.original.fixedPrice,
          suffix: "грн.",
          class: "justify-end",
        });
      }
    },
  },
  {
    id: "actions",
    cell: ({ row, table }) => {
      const meta = table.options.meta as ServiceTableMeta | undefined;
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
