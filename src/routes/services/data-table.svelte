<script lang="ts" generics="TData, TValue">
  import { rankItem } from "@tanstack/match-sorter-utils";
  import {
    getCoreRowModel,
    getFilteredRowModel,
    getPaginationRowModel,
    getSortedRowModel,
    type ColumnDef,
    type ColumnFiltersState,
    type FilterFn,
    type PaginationState,
    type SortingState,
  } from "@tanstack/table-core";

  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import ChevronsLeftIcon from "@lucide/svelte/icons/chevrons-left";
  import ChevronsRightIcon from "@lucide/svelte/icons/chevrons-right";

  import { cn } from "$lib/utils";

  import { Button } from "$lib/components/ui/button";
  import { createSvelteTable, FlexRender } from "$lib/components/ui/data-table";
  import * as Table from "$lib/components/ui/table/index.js";

  let {
    data,
    columns,
    searchTerm = $bindable(""),
  }: {
    data: TData[];
    columns: ColumnDef<TData, TValue>[];
    searchTerm?: string;
  } = $props();

  let sorting = $state<SortingState>([]);
  let columnFilters = $state<ColumnFiltersState>([]);
  let pagination = $state<PaginationState>({ pageIndex: 0, pageSize: 10 });

  const fuzzyFilter: FilterFn<any> = (row, columnId, value, addMeta) => {
    const itemRank = rankItem(row.getValue(columnId), value);
    addMeta({ itemRank });
    return itemRank.passed;
  };

  let table = createSvelteTable({
    get data() {
      return data;
    },
    get columns() {
      return columns;
    },
    filterFns: {
      fuzzy: fuzzyFilter,
    },
    // @ts-expect-error API and docs mismatch?
    globalFilterFn: "fuzzy",
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onSortingChange: (updater) => {
      sorting = typeof updater === "function" ? updater(sorting) : updater;
    },
    onColumnFiltersChange: (updater) => {
      columnFilters =
        typeof updater === "function" ? updater(columnFilters) : updater;
    },
    onPaginationChange: (updater) => {
      pagination =
        typeof updater === "function" ? updater(pagination) : updater;
    },
    onGlobalFilterChange: (updater) => {
      searchTerm =
        typeof updater === "function" ? updater(searchTerm) : updater;
    },
    state: {
      get sorting() {
        return sorting;
      },
      get columnFilters() {
        return columnFilters;
      },
      get pagination() {
        return pagination;
      },
      get globalFilter() {
        return searchTerm;
      },
    },
  });
</script>

<div class="flex flex-col h-full">
  <Table.Root class="flex-1">
    <Table.Header>
      {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
        <Table.Row>
          {#each headerGroup.headers as header (header.id)}
            <Table.Head
              colspan={header.colSpan}
              class={cn(header.id === "name" && "w-full")}
            >
              {#if !header.isPlaceholder}
                <FlexRender
                  content={header.column.columnDef.header}
                  context={header.getContext()}
                />
              {/if}
            </Table.Head>
          {/each}
        </Table.Row>
      {/each}
    </Table.Header>
    <Table.Body>
      {#each table.getRowModel().rows as row (row.id)}
        <Table.Row data-state={row.getIsSelected() && "selected"}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <Table.Cell>
              <FlexRender
                content={cell.column.columnDef.cell}
                context={cell.getContext()}
              />
            </Table.Cell>
          {/each}
        </Table.Row>
      {:else}
        <Table.Row>
          <Table.Cell colspan={columns.length} class="h-24 text-center">
            Сервісів не знайдено...
          </Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
  <div
    class="flex items-center justify-end sticky bottom-0 z-10 mt-auto p-4 border-t"
  >
    <p class="font-medium text-sm mr-8">
      Сторінка {table.getState().pagination.pageIndex + 1} з {table.getPageCount()}
    </p>
    <div class="flex gap-2">
      <Button
        variant="outline"
        size="icon"
        onclick={() => table.firstPage()}
        disabled={!table.getCanPreviousPage()}
      >
        <ChevronsLeftIcon />
      </Button>
      <Button
        variant="outline"
        size="icon"
        onclick={() => table.previousPage()}
        disabled={!table.getCanPreviousPage()}
      >
        <ChevronLeftIcon />
      </Button>
      <Button
        variant="outline"
        size="icon"
        onclick={() => table.nextPage()}
        disabled={!table.getCanNextPage()}
      >
        <ChevronRightIcon />
      </Button>
      <Button
        variant="outline"
        size="icon"
        onclick={() => table.lastPage()}
        disabled={!table.getCanNextPage()}
      >
        <ChevronsRightIcon />
      </Button>
    </div>
  </div>
</div>
