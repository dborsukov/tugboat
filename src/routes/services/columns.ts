import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import type { Service } from "$lib/bindings";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import DataTableActions from "./data-table-actions.svelte";
import DataTableNameButton from "./data-table-name-button.svelte";
import DataTablePriceButton from "./data-table-price-button.svelte";

export const columns: ColumnDef<Service>[] = [
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
    accessorKey: "unit_price",
    header: ({ column }) => {
      return renderComponent(DataTablePriceButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
      });
    },
    cell: ({ row }) => {
      const formatter = new Intl.NumberFormat("uk-UA", {
        style: "currency",
        currency: "UAH",
      });

      const formatted = formatter.format(row.original.unit_price);

      const priceCellSnippet = createRawSnippet(() => ({
        render: () =>
          `<div class="text-end font-medium px-4">${formatted}</div>`,
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
