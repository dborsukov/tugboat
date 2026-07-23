import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import type { Car, CarBrand, Client, WorkOrder } from "$lib/bindings";

import { prettifyIsoDateString } from "$lib/misc";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";

import ClientBadge from "../clients/client-badge.svelte";
import CarDisplay from "./car-display.svelte";
import StatusSelector from "./status-selector.svelte";

export type WorkOrderTableData = WorkOrder & {
  car: Car;
  carBrand: CarBrand;
  client: Client;
};

export const columns: ColumnDef<WorkOrderTableData>[] = [
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
        client: row.original.client,
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
    id: "created_at",
    accessorFn: (row) => prettifyIsoDateString(row.created_at, "datetime"),
    sortingFn: "datetime",
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
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
            `<div class="px-2">${prettifyIsoDateString(row.original.created_at, "datetime")}</div>`,
        };
      });
      return renderSnippet(createdCellSnippet);
    },
  },
];
