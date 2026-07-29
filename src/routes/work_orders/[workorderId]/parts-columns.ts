import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import type { WorkOrderPartItem } from "$lib/bindings";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";

import DataTableActions from "./data-table-part-actions.svelte";
import PartStatusSelector from "./part-status-selector.svelte";

export const columns: ColumnDef<WorkOrderPartItem>[] = [
  {
    accessorKey: "status",
    header: "Статус",
    cell: ({ row }) => {
      return renderComponent(PartStatusSelector, {
        status: row.original.status,
        interactive: false,
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
    accessorKey: "costCents",
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
        cents: row.original.costCents,
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
              `<div class="text-center">${row.original.markup} %</div>`,
          };
        }),
      );
    },
  },
  {
    id: "price_cents",
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
      return renderComponent(CurrencyDisplay, {
        cents: Math.round(
          row.original.costCents * (1 + row.original.markup / 100),
        ),
        suffix: "грн.",
        class: "justify-end",
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
        cents:
          row.original.quantity *
          Math.round(row.original.costCents * (1 + row.original.markup / 100)),
        suffix: "грн.",
        class: "font-medium justify-end",
      });
    },
  },
  {
    id: "actions",
    cell: ({ row }) => {
      return renderComponent(DataTableActions, { id: row.original.id });
    },
  },
];
