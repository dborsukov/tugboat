import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import type { Car, CarBrand, Client, WorkOrder } from "$lib/bindings";

import { prettifyIsoDateString } from "$lib/utils";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import CarDisplay from "./car-display.svelte";
import DataTableClientButton from "./data-table-client-button.svelte";
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
      return renderComponent(DataTableClientButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
      });
    },
    cell: ({ row }) => {
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () => `<div>${row.original.client.name}</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
  },
  {
    id: "car_id",
    accessorFn: (row) =>
      `${row.carBrand.name} ${row.car.model} ${row.car.year}`,
    header: "Автомобіль",
    cell: ({ row }) => {
      return renderComponent(CarDisplay, {
        car: row.original.car,
        carBrand: row.original.carBrand,
      });
    },
  },
  {
    accessorKey: "status",
    header: "Статус",
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
    header: () => {
      const createdHeaderSnippet = createRawSnippet(() => {
        return {
          render: () => `<div class="text-end px-2">Створено</div>`,
        };
      });
      return renderSnippet(createdHeaderSnippet);
    },
    cell: ({ row }) => {
      const createdCellSnippet = createRawSnippet(() => {
        return {
          render: () =>
            `<div class="font-mono tracking-tighter text-end px-2 text-muted-foreground">${prettifyIsoDateString(row.original.created_at, "datetime")}</div>`,
        };
      });
      return renderSnippet(createdCellSnippet);
    },
  },
];
