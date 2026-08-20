import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";
import DataTableOptionalCell from "$lib/components/data-table-optional-cell.svelte";
import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";

import type { Service } from "$lib/bindings";

import DataTableActions from "./table-actions-cell.svelte";
import TableDecimalDisplayCell from "./table-decimal-display-cell.svelte";

export interface ServiceTableMeta {
  onEdit: (id: number) => void;
  onDelete: (id: number) => void;
}

export const columns: ColumnDef<Service>[] = [
  {
    id: "code",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="px-2">Код</div>`,
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
        class: "pl-1.5",
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
          render: () => `<div class="px-2">${row.original.name}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
  },
  {
    id: "laborHours",
    accessorFn: (row) => row.laborHours,
    sortingFn: "alphanumeric",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-1.5",
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
    accessorFn: (row) => row.fixedPrice,
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        class: "pl-1.5",
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
        onEdit: (id: number) => meta?.onEdit?.(id),
        onDelete: (id: number) => meta?.onDelete?.(id),
      });
    },
  },
];
