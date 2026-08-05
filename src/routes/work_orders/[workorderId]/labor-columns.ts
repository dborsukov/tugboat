import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import type { WorkOrderLaborItem } from "$lib/bindings";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";

import DataTableActions from "./data-table-labor-actions.svelte";

export interface LaborTableMeta {
  rate: number;
  onEdit: (id: number) => void;
  onDelete: (id: number) => void;
}

export const columns: ColumnDef<WorkOrderLaborItem>[] = [
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
    header: "Назва",
  },
  {
    accessorKey: "laborHours",
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
            `<div class="text-center font-mono">${row.original.laborHours}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
  },
  {
    id: "sum",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Сума</div>`,
          };
        }),
      );
    },
    cell: ({ row, table }) => {
      let { rate } = table.options.meta as LaborTableMeta;
      return renderComponent(CurrencyDisplay, {
        cents: rate * row.original.laborHours,
        suffix: "грн.",
        class: "font-medium justify-end",
      });
    },
  },
  {
    id: "actions",
    cell: ({ row, table }) => {
      const meta = table.options.meta as LaborTableMeta | undefined;
      return renderComponent(DataTableActions, {
        id: row.original.id,
        onEdit: (id: number) => meta?.onEdit?.(id),
        onDelete: (id: number) => meta?.onDelete?.(id),
      });
    },
  },
];
