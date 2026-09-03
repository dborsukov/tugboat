import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";
import DataTableOptionalCell from "$lib/components/data-table-optional-cell.svelte";
import DataTableSelectCheckbox from "$lib/components/data-table/data-table-select-checkbox.svelte";

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
    id: "select",
    header: ({ table }) => {
      return renderComponent(DataTableSelectCheckbox, {
        title: "Обрати всі",
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
      if (row.original.isFree) {
        return renderSnippet(
          createRawSnippet(() => {
            return {
              render: () =>
                `<div class="w-full h-full flex items-center justify-end px-2 bg-amber-100 text-amber-800 dark:bg-amber-500/15 dark:text-amber-300 font-medium whitespace-nowrap">Безкоштовно</div>`,
            };
          }),
        );
      }
      return renderComponent(TableDiscountedWarningCell, {
        price: row.original.totals.total,
        belowCost: row.original.totals.belowCost,
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
