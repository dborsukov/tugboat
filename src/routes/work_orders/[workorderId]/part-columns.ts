import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";

import type { PartItemDto } from "$lib/bindings";

import DataTableActions from "./part-table-actions-cell.svelte";
import PartTableStatusCell from "./part-table-status-cell.svelte";
import TableDiscountCell from "./table-discount-cell.svelte";
import TableDiscountedWarningCell from "./table-discounted-warning-cell.svelte";

export interface PartsTableMeta {
  onEdit: (id: number) => void;
  onDelete: (id: number) => void;
}

export const columns: ColumnDef<PartItemDto>[] = [
  {
    accessorKey: "status",
    header: "Статус",
    cell: ({ row }) => {
      return renderComponent(PartTableStatusCell, {
        status: row.original.status,
      });
    },
  },
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
    accessorKey: "quantity",
    header: "Кількість",
    cell: ({ row }) => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () =>
              `<div class="text-center">${row.original.quantity}</div>`,
          };
        }),
      );
    },
  },
  {
    accessorKey: "unitType",
    header: "Од. вим.",
    cell: ({ row }) => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () =>
              `<div class="text-center">${row.original.unitType}</div>`,
          };
        }),
      );
    },
  },
  {
    accessorKey: "baseCost",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Ціна закупівлі</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      return renderComponent(CurrencyDisplay, {
        amount: row.original.baseCost,
        suffix: "грн.",
        class: "justify-end",
      });
    },
  },
  {
    id: "markup",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Націнка</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () =>
              `<div class="text-center">${row.original.markup != "0" ? row.original.markup + "%" : "—"}</div>`,
          };
        }),
      );
    },
  },
  {
    id: "retailPrice",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Ціна продажу</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      return renderComponent(CurrencyDisplay, {
        amount: row.original.totals.retailPrice,
        suffix: "грн.",
        class: "justify-end",
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
        amount: row.original.discountAmount,
        percent: row.original.discountPercent,
        class: "justify-center",
      });
    },
  },
  {
    id: "discountedPrice",
    header: () => {
      return renderSnippet(
        createRawSnippet(() => {
          return {
            render: () => `<div class="text-end">Відпускна ціна</div>`,
          };
        }),
      );
    },
    cell: ({ row }) => {
      return renderComponent(TableDiscountedWarningCell, {
        cost: row.original.baseCost,
        price: row.original.totals.discountedPrice,
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
      return renderComponent(CurrencyDisplay, {
        amount: row.original.totals.total,
        suffix: "грн.",
        class: "font-medium justify-end",
      });
    },
  },
  {
    id: "actions",
    cell: ({ row, table }) => {
      const meta = table.options.meta as PartsTableMeta | undefined;
      return renderComponent(DataTableActions, {
        id: row.original.id,
        onEdit: (id: number) => meta?.onEdit?.(id),
        onDelete: (id: number) => meta?.onDelete?.(id),
      });
    },
  },
];
