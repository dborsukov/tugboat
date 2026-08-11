import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";

import type { Service } from "$lib/bindings";

import DataTableActions from "./table-actions-cell.svelte";

export interface ServiceTableMeta {
  onEdit: (id: number) => void;
  onDelete: (id: number) => void;
}

export const columns: ColumnDef<Service>[] = [
  {
    accessorKey: "code",
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
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="font-mono">${row.original.code}</div>`,
          };
        }),
      );
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
    accessorKey: "labor_hours",
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
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () =>
            `<div class="text-center font-mono">${row.original.laborHours}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
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
