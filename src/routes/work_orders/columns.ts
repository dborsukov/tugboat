import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { prettifyIsoDateString } from "$lib/misc";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";
import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";

import ClientBadge from "../clients/client-badge.svelte";
import CarDisplay from "./car-display.svelte";
import StatusSelector from "./status-selector.svelte";
import type { WorkOrderListItem } from "./types";

export const columns: ColumnDef<WorkOrderListItem>[] = [
  {
    accessorKey: "id",
    header: () => {
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () => `<div class="px-2">Код</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
    cell: ({ row }) => {
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () =>
            `<div class="px-2 font-mono tracking-wider">${row.original.id}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
  },
  {
    id: "client",
    accessorFn: (row) => row.client.name,
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        disabled: !column.getCanSort(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Клієнт</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(ClientBadge, {
        name: row.original.client.name,
      });
    },
  },
  {
    id: "car_id",
    accessorFn: (row) =>
      `${row.carBrand.name} ${row.car.model} ${row.car.year}`,
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        disabled: !column.getCanSort(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Автомобіль</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(CarDisplay, {
        car: row.original.car,
        carBrand: row.original.carBrand,
      });
    },
  },
  {
    accessorKey: "status",
    filterFn: "arrIncludesSome",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        disabled: !column.getCanSort(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Статус</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(StatusSelector, {
        status: row.original.status,
        interactive: false,
      });
    },
  },
  {
    id: "total",
    accessorFn: (row) => {
      let laborTotal = row.laborItems.reduce(
        (sum, item) => sum + Math.round(item.laborHours * row.rateCents),
        0,
      );
      let partsTotal = row.partsItems.reduce(
        (sum, item) =>
          sum +
          Math.round(
            item.quantity * (item.costCents * (1 + item.markup / 100)),
          ),
        0,
      );
      let flatTotal = laborTotal + partsTotal;
      if (row.discountCents != 0 || row.discountPercent != 0) {
        if (row.discountType === "Fixed") {
          return flatTotal - row.discountCents;
        } else {
          return flatTotal * (1 - row.discountPercent / 100);
        }
      }
      return flatTotal;
    },
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        disabled: !column.getCanSort(),
        class: "pl-1",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Сума</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(CurrencyDisplay, {
        cents: row.getValue("total") as number,
        suffix: "грн.",
        class: "px-2 justify-start",
      });
    },
  },
  {
    id: "created_at",
    accessorFn: (row) => prettifyIsoDateString(row.createdAt, "datetime"),
    sortingFn: "datetime",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        disabled: !column.getCanSort(),
        class: "pl-2",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Створено</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      const createdCellSnippet = createRawSnippet(() => {
        return {
          render: () =>
            `<div class="px-2">${prettifyIsoDateString(row.original.createdAt, "datetime")}</div>`,
        };
      });
      return renderSnippet(createdCellSnippet);
    },
  },
];
