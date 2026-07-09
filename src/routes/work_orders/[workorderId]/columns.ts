import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import type { WorkOrderLaborItem } from "$lib/bindings";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import DataTableActions from "./data-table-actions.svelte";

const currencyFormatter = new Intl.NumberFormat("uk-UA", {
  style: "currency",
  currency: "UAH",
});

export type WorkOrderLaborItemTableData = WorkOrderLaborItem & {
  rate: number;
};

export const columns: ColumnDef<WorkOrderLaborItemTableData>[] = [
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
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () =>
            `<div class="px-2 font-mono">${row.original.code}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
  },
  {
    accessorKey: "name",
    header: "Назва",
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
            `<div class="text-end">${row.original.labor_hours}</div>`,
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
    cell: ({ row }) => {
      const formatted = currencyFormatter.format(
        row.original.rate * row.original.labor_hours,
      );

      const priceCellSnippet = createRawSnippet(() => ({
        render: () => `<div class="text-end font-medium">${formatted}</div>`,
      }));

      return renderSnippet(priceCellSnippet);
    },
  },
  {
    id: "actions",
    cell: ({ row }) => {
      return renderComponent(DataTableActions, { id: row.original.id });
    },
  },
];
