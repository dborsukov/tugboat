import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import type { Service } from "$lib/bindings";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import DataTableActions from "./data-table-actions.svelte";
import DataTableCodeCell from "./data-table-code-cell.svelte";
import DataTableNameButton from "./data-table-name-button.svelte";

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
      return renderComponent(DataTableCodeCell, { code: row.original.code });
    },
  },
  {
    accessorKey: "name",
    header: ({ column }) => {
      return renderComponent(DataTableNameButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
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
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Нормо-години</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () =>
            `<div class="text-center font-mono">${row.original.labor_hours.toFixed(1)}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
  },
  {
    id: "actions",
    cell: ({ row }) => {
      return renderComponent(DataTableActions, { id: row.original.id });
    },
  },
];
