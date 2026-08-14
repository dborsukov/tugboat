import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CurrencyDisplay from "$lib/components/currency-display.svelte";
import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";

import type { WorkOrderDetailsDto } from "$lib/bindings";
import { prettifyIsoDateString } from "$lib/tools";

import CarBadge from "../cars/car-badge.svelte";
import ClientBadge from "../clients/client-badge.svelte";
import WorkOrderTableStatusCell from "./work-order-table-status-cell.svelte";

export const columns: ColumnDef<WorkOrderDetailsDto>[] = [
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
            `<div class="px-2 font-mono tracking-wider">${row.original.workOrder.id}</div>`,
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
      `${row.carDto.brand.name} ${row.carDto.car.model} ${row.carDto.car.year}`,
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
      return renderComponent(CarBadge, {
        car: row.original.carDto.car,
        carBrand: row.original.carDto.brand,
        class: "h-6",
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
      return renderComponent(WorkOrderTableStatusCell, {
        status: row.original.workOrder.status,
      });
    },
  },
  {
    id: "total",
    accessorFn: (row) => row.totals.grandTotal,
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
        amount: row.original.totals.grandTotal,
        suffix: "грн.",
        class: "px-2 justify-start",
      });
    },
  },
  {
    id: "created_at",
    accessorFn: (row) =>
      prettifyIsoDateString(row.workOrder.createdAt, "datetime"),
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
            `<div class="px-2">${prettifyIsoDateString(row.original.workOrder.createdAt, "datetime")}</div>`,
        };
      });
      return renderSnippet(createdCellSnippet);
    },
  },
];
