<script lang="ts" generics="TData">
  import { type Table } from "@tanstack/table-core";

  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import ChevronsLeftIcon from "@lucide/svelte/icons/chevrons-left";
  import ChevronsRightIcon from "@lucide/svelte/icons/chevrons-right";

  import { cn } from "$lib/utils";

  import { Button } from "$lib/components/ui/button";
  import { FlexRender } from "$lib/components/ui/data-table";
  import * as TableUI from "$lib/components/ui/table/index.js";

  let {
    table,
    emptyMessage = "Даних не знайдено...",
    headerClasses = {},
    rounded = true,
    onRowClick,
  }: {
    table: Table<TData>;
    emptyMessage?: string;
    headerClasses?: Record<string, string>;
    rounded?: boolean;
    onRowClick?: (row: TData) => void;
  } = $props();

  const showPagination = $derived(
    table.options.getPaginationRowModel !== undefined,
  );

  function handleRowKeyDown(e: KeyboardEvent, rowData: TData) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault(); // Prevent spacebar scrolling
      onRowClick?.(rowData);
    }
  }
</script>

<div class="flex flex-col h-full w-full max-w-full">
  <div
    class={cn(
      "flex-1 w-full overflow-auto border border-border",
      rounded && "rounded",
      showPagination && "border-b-0 rounded-b-none",
    )}
  >
    <TableUI.Root class="flex-1 w-full">
      <TableUI.Header>
        {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
          <TableUI.Row class="divide-x divide-border">
            {#each headerGroup.headers as header (header.id)}
              <TableUI.Head
                colspan={header.colSpan}
                class={cn(headerClasses[header.id])}
              >
                {#if !header.isPlaceholder}
                  <FlexRender
                    content={header.column.columnDef.header}
                    context={header.getContext()}
                  />
                {/if}
              </TableUI.Head>
            {/each}
          </TableUI.Row>
        {/each}
      </TableUI.Header>
      <TableUI.Body>
        {#each table.getRowModel().rows as row (row.id)}
          <TableUI.Row
            tabindex={onRowClick ? 0 : undefined}
            data-state={row.getIsSelected() && "selected"}
            class={cn(
              "divide-x divide-border",
              onRowClick &&
                "hover:cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:bg-muted/90",
            )}
            onclick={() => onRowClick?.(row.original)}
            onkeydown={(e) => handleRowKeyDown(e, row.original)}
          >
            {#each row.getVisibleCells() as cell (cell.id)}
              <TableUI.Cell>
                <FlexRender
                  content={cell.column.columnDef.cell}
                  context={cell.getContext()}
                />
              </TableUI.Cell>
            {/each}
          </TableUI.Row>
        {:else}
          <TableUI.Row>
            <TableUI.Cell
              colspan={table.getVisibleFlatColumns().length}
              class="h-24 text-center"
            >
              {emptyMessage}
            </TableUI.Cell>
          </TableUI.Row>
        {/each}
      </TableUI.Body>
    </TableUI.Root>
  </div>

  {#if showPagination}
    <div
      class={cn(
        "flex items-center justify-end sticky bottom-0 z-10 mt-auto p-4 border-t bg-background border-x border-b border-border",
        rounded && "rounded-b",
      )}
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
  {/if}
</div>
