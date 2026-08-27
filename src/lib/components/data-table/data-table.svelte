<script lang="ts" generics="TData">
  import { type Table } from "@tanstack/table-core";
  import { type Snippet } from "svelte";

  import { browser } from "$app/environment";

  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import ChevronsLeftIcon from "@lucide/svelte/icons/chevrons-left";
  import ChevronsRightIcon from "@lucide/svelte/icons/chevrons-right";

  import { Button } from "$lib/components/ui/button";
  import { FlexRender } from "$lib/components/ui/data-table";
  import * as Select from "$lib/components/ui/select/index.js";
  import * as TableUI from "$lib/components/ui/table/index.js";

  import { cn } from "$lib/utils";

  const MIN_PAGE_SIZE = 5;
  const pageSizeOptions = [5, 10, 15, 20, 25, 50];

  let {
    table,
    emptyMessage = "Даних не знайдено...",
    headerClasses = {},
    cellClasses = {},
    rounded = true,
    onRowClick,
    selectionActions,
    autoPageSize = false,
    class: className,
  }: {
    table: Table<TData>;
    emptyMessage?: string;
    headerClasses?: Record<string, string>;
    cellClasses?: Record<string, string>;
    rounded?: boolean;
    onRowClick?: (row: TData) => void;
    selectionActions?: Snippet;
    autoPageSize?: boolean;
    class?: string;
  } = $props();

  const showPagination = $derived(
    table.options.getPaginationRowModel !== undefined,
  );

  const selectedRowCount = $derived(
    table.getFilteredSelectedRowModel().rows.length,
  );

  const totalRowCount = $derived(table.getFilteredRowModel().rows.length);

  const showSelectionBar = $derived(!showPagination && selectedRowCount > 0);

  const footerVisible = $derived(showPagination || showSelectionBar);

  const currentPageSize = $derived(table.getState().pagination.pageSize);

  let wrapperRef = $state<HTMLDivElement | null>(null);
  let headerRef = $state<HTMLTableSectionElement | null>(null);
  let bodyRef = $state<HTMLTableSectionElement | null>(null);

  let measureRafId: number | undefined;

  // The wrapper's own height tracks its content, so the available space
  // must be measured from the nearest scroll container instead.
  function findScrollContainer(el: HTMLElement): HTMLElement {
    let node: HTMLElement | null = el.parentElement;
    while (node) {
      const overflowY = getComputedStyle(node).overflowY;
      if (
        overflowY === "auto" ||
        overflowY === "scroll" ||
        overflowY === "overlay"
      ) {
        return node;
      }
      node = node.parentElement;
    }
    return document.documentElement;
  }

  function measureBelowWrapper(wrapper: HTMLElement): number {
    let total = 0;
    let sibling = wrapper.nextElementSibling;
    while (sibling) {
      if (sibling instanceof HTMLElement) total += sibling.offsetHeight;
      sibling = sibling.nextElementSibling;
    }
    return total;
  }

  // Fit as many rows as the available viewport space allows. Recomputations
  // are requestAnimationFrame-coalesced and only applied when the computed
  // size actually differs, so there is no feedback loop.
  function scheduleMeasure() {
    if (!autoPageSize || !showPagination || !browser) return;
    const wrapper = wrapperRef;
    const thead = headerRef;
    const body = bodyRef;
    if (!wrapper || !thead || !body) return;
    // No rows to fit yet
    if (table.getRowModel().rows.length === 0) return;
    // <tbody> children are always <tr> elements (HTML invariant), so the
    // first child is the first data row.
    const firstRow = body.children[0];
    if (!(firstRow instanceof HTMLTableRowElement)) return;

    if (measureRafId !== undefined) cancelAnimationFrame(measureRafId);
    measureRafId = requestAnimationFrame(() => {
      measureRafId = undefined;
      const scrollContainer = findScrollContainer(wrapper);
      const wrapperRect = wrapper.getBoundingClientRect();
      const containerRect = scrollContainer.getBoundingClientRect();
      // Adding scrollTop keeps the offset correct regardless of the scroll
      // position (also covers the documentElement fallback, where the page
      // itself scrolls and the wrapper's viewport offset would drift).
      const offsetFromContainerTop =
        wrapperRect.top - containerRect.top + scrollContainer.scrollTop;
      const availableHeight =
        scrollContainer.clientHeight -
        offsetFromContainerTop -
        measureBelowWrapper(wrapper);
      if (availableHeight <= 0) return;
      const pageSize = Math.max(
        MIN_PAGE_SIZE,
        Math.floor(
          (availableHeight - thead.offsetHeight) / firstRow.offsetHeight,
        ),
      );
      if (pageSize !== table.getState().pagination.pageSize) {
        table.setPageSize(pageSize);
      }
    });
  }

  // Set up the resize observation once per auto-mode activation: the wrapper
  // (its height changes with content), the scroll container (window resizes),
  // plus a window resize listener as a fallback for the documentElement case.
  $effect(() => {
    if (!autoPageSize || !showPagination || !browser) return;
    const wrapper = wrapperRef;
    if (!wrapper) return;

    const scrollContainer = findScrollContainer(wrapper);
    const resizeObserver = new ResizeObserver(scheduleMeasure);
    resizeObserver.observe(wrapper);
    resizeObserver.observe(scrollContainer);
    window.addEventListener("resize", scheduleMeasure);

    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("resize", scheduleMeasure);
      if (measureRafId !== undefined) cancelAnimationFrame(measureRafId);
    };
  });

  // Re-measure whenever the rendered rows change (data load, search, page).
  // Kept separate from the ResizeObserver lifecycle so it doesn't churn it.
  $effect(() => {
    if (!autoPageSize || !showPagination || !browser) return;
    table.getRowModel().rows.length;
    scheduleMeasure();
  });

  function handleRowKeyDown(e: KeyboardEvent, rowData: TData) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault(); // Prevent spacebar scrolling
      onRowClick?.(rowData);
    }
  }
</script>

{#snippet selectionInfo()}
  <div class="flex items-center gap-3">
    {#if selectedRowCount > 0}
      <p class="font-medium text-sm">
        Обрано {selectedRowCount} з {totalRowCount}
      </p>
      {@render selectionActions?.()}
    {/if}
  </div>
{/snippet}

<div class={cn("flex flex-col h-full w-full max-w-full", className)}>
  <div
    bind:this={wrapperRef}
    class={cn(
      "flex-1 w-full overflow-auto border border-border",
      rounded && "rounded",
      footerVisible && "rounded-b-none",
    )}
  >
    <TableUI.Root class="flex-1 w-full">
      <TableUI.Header bind:ref={headerRef}>
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
      <TableUI.Body bind:ref={bodyRef}>
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
              <TableUI.Cell class={cn("h-px", cellClasses[cell.column.id])}>
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

  {#if showSelectionBar}
    <div
      class={cn(
        "flex items-center justify-between sticky bottom-0 z-10 mt-auto p-2 bg-card border-x border-b border-border",
        rounded && "rounded-b",
      )}
    >
      {@render selectionInfo()}
    </div>
  {/if}

  {#if showPagination}
    <div
      class={cn(
        "flex items-center justify-between sticky bottom-0 z-10 mt-auto p-2 bg-card border-x border-b border-border",
        rounded && "rounded-b",
      )}
    >
      {@render selectionInfo()}
      <div class="flex items-center gap-4">
        <div class="flex gap-1 items-center">
          <p class="font-medium text-sm">Рядків на сторінці:</p>
          <Select.Root
            type="single"
            value={autoPageSize ? "auto" : String(currentPageSize)}
            onValueChange={(value) => {
              if (value !== "auto") table.setPageSize(Number(value));
            }}
            disabled={autoPageSize}
          >
            <Select.Trigger size="sm" class="w-fit">
              {autoPageSize ? "Авто" : currentPageSize}
            </Select.Trigger>
            <Select.Content>
              {#if autoPageSize}
                <Select.Item value="auto">Авто</Select.Item>
              {:else}
                {#each pageSizeOptions as option (option)}
                  <Select.Item value={String(option)}>{option}</Select.Item>
                {/each}
              {/if}
            </Select.Content>
          </Select.Root>
        </div>
        <p class="font-medium text-sm">
          Сторінка {table.getState().pagination.pageIndex + 1} з {table.getPageCount()}
        </p>
        <div class="flex gap-2">
          <Button
            variant="outline"
            size="icon-sm"
            onclick={() => table.firstPage()}
            disabled={!table.getCanPreviousPage()}
          >
            <ChevronsLeftIcon />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            onclick={() => table.previousPage()}
            disabled={!table.getCanPreviousPage()}
          >
            <ChevronLeftIcon />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            onclick={() => table.nextPage()}
            disabled={!table.getCanNextPage()}
          >
            <ChevronRightIcon />
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            onclick={() => table.lastPage()}
            disabled={!table.getCanNextPage()}
          >
            <ChevronsRightIcon />
          </Button>
        </div>
      </div>
    </div>
  {/if}
</div>
