import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";

import type { LaborItemDto } from "$lib/bindings";

import TableDecimalDisplayCell from "../../services/table-decimal-display-cell.svelte";
import DataTableActions from "./labor-table-actions-cell.svelte";
import TableDiscountCell from "./table-discount-cell.svelte";
import TableDiscountedWarningCell from "./table-discounted-warning-cell.svelte";

export interface LaborTableMeta {
  onEdit: (id: number) => void;
  onDelete: (id: number) => void;
}

export const columns: ColumnDef<LaborItemDto>[] = [
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
    id: "laborHours",
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
    id: "price",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Ціна</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      return renderComponent(CurrencyDisplay, {
        amount: row.original.totals.subtotal,
        suffix: "грн.",
        class: "font-medium justify-end",
      });
    },
  },
  {
    id: "discount",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Знижка</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      return renderComponent(TableDiscountCell, {
        type: row.original.discountType,
        value: row.original.discountValue,
        class: "justify-center",
      });
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
      return renderComponent(TableDiscountedWarningCell, {
        cost: row.original.totals.cost,
        price: row.original.totals.total,
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
