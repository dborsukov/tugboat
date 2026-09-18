import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";

import { renderComponent, renderSnippet } from "$lib/components/ui/data-table";

import DataTableSortButton from "$lib/components/data-table-sort-button.svelte";

import type { WorkOrderDetailsDto } from "$lib/bindings";
import { prettifyIsoDateString } from "$lib/tools";

import CarBadge from "../cars/car-badge.svelte";
import type { WorkOrderClientColumn } from "./work-order-client-column";
import { statusWeights } from "./work-order-status";
import WorkOrderTableClientCell from "./work-order-table-client-cell.svelte";
import WorkOrderTableIdCell from "./work-order-table-id-cell.svelte";
import WorkOrderTableStatusCell from "./work-order-table-status-cell.svelte";
import WorkOrderTableTotalCell from "./work-order-table-total-cell.svelte";

export type WorkOrderRow = WorkOrderDetailsDto & {
  clientColumn?: WorkOrderClientColumn;
};

export const columns: ColumnDef<WorkOrderRow>[] = [
  {
    id: "id",
    accessorFn: (row) => row.workOrder.id,
    header: () => {
      const nameCellSnippet = createRawSnippet(() => {
        return {
          render: () => `<div class="px-2">Код</div>`,
        };
      });
      return renderSnippet(nameCellSnippet);
    },
    cell: ({ row }) => {
      return renderComponent(WorkOrderTableIdCell, {
        id: row.original.workOrder.id,
        type: row.original.workOrder.type,
        archivedAt: row.original.workOrder.archivedAt,
      });
    },
  },
  {
    id: "client",
    accessorFn: (row) =>
      [
        row.clientColumn?.mainName ?? row.client.name,
        ...(row.clientColumn?.others ?? []).map((line) => line.name),
      ]
        .filter((name): name is string => Boolean(name))
        .join(" "),
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        disabled: !column.getCanSort(),
        class: "pl-0",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Замовник</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      const { client, clientColumn } = row.original;
      return renderComponent(WorkOrderTableClientCell, {
        mainName: clientColumn?.mainName ?? client.name,
        others: clientColumn?.others ?? [],
        loading: clientColumn?.loading ?? false,
      });
    },
  },
  {
    id: "carId",
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
    id: "status",
    accessorFn: (row) => row.workOrder.status,
    filterFn: "arrIncludesSome",
    sortingFn: (rowA, rowB) => {
      const statusA = rowA.original.workOrder?.status;
      const statusB = rowB.original.workOrder?.status;
      const weightA = statusWeights[statusA] ?? 999;
      const weightB = statusWeights[statusB] ?? 999;
      return weightB - weightA;
    },
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
    accessorFn: (row) => {
      const rawTotal =
        row.totals?.totalWithVat ?? row.totals?.grandTotal ?? "0";
      return parseFloat(rawTotal);
    },
    header: ({ column }) => {
      return renderComponent(DataTableSortButton, {
        onclick: column.getToggleSortingHandler(),
        sortStatus: column.getIsSorted(),
        disabled: !column.getCanSort(),
        class: "px-2",
        children: createRawSnippet(() => {
          return {
            render: () => "<span>Сума</span>",
          };
        }),
      });
    },
    cell: ({ row }) => {
      return renderComponent(WorkOrderTableTotalCell, {
        workOrderDetails: row.original,
        class: "justify-end px-2",
      });
    },
  },
  {
    id: "createdAt",
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
