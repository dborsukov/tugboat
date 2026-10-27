<script lang="ts">
  import { DragDropProvider } from "@dnd-kit/svelte";
  import Big from "big.js";
  import type { Snippet } from "svelte";

  import AppTooltip from "$lib/components/app-tooltip.svelte";
  import {
    moveRowIds,
    type RowDragEndEvent,
    type RowDragOverEvent,
    type RowReorderOptions,
  } from "$lib/components/data-table/data-table-reorder";
  import DataTableSelectCheckbox from "$lib/components/data-table/data-table-select-checkbox.svelte";

  import type { LaborItemDto } from "$lib/bindings";
  import { cn } from "$lib/utils";

  import DiscountTooltip from "./discount-tooltip.svelte";
  import LaborTableActions from "./labor-table-actions-cell.svelte";
  import SortableGridRow from "./sortable-grid-row.svelte";
  import WorkOrderDragHandle from "./work-order-drag-handle.svelte";
  import {
    decimalText,
    discountText,
    hasDiscount,
    laborGridStyle,
    leftTruncateClass,
    moneyText,
  } from "./work-order-table";

  let {
    items,
    total,
    readOnly = false,
    selectedIds = $bindable<number[]>([]),
    rowReorder,
    onEdit,
    onDelete,
    onMoveUp,
    onMoveDown,
    actions,
    selectionActions,
    emptyMessage = "Сервісів не знайдено...",
  }: {
    items: LaborItemDto[];
    total: string;
    readOnly?: boolean;
    selectedIds?: number[];
    rowReorder?: RowReorderOptions;
    onEdit: (id: number) => void;
    onDelete: (id: number) => void;
    onMoveUp: (id: number) => void;
    onMoveDown: (id: number) => void;
    actions?: Snippet;
    selectionActions?: Snippet;
    emptyMessage?: string;
  } = $props();

  const gridStyle = $derived(laborGridStyle(readOnly));
  const rowIds = $derived(items.map((item) => String(item.id)));
  const allSelected = $derived(
    items.length > 0 && items.every((item) => selectedIds.includes(item.id)),
  );
  const someSelected = $derived(
    !allSelected && items.some((item) => selectedIds.includes(item.id)),
  );

  $effect(() => {
    const live = new Set(items.map((item) => item.id));
    const next = selectedIds.filter((id) => live.has(id));
    if (next.length !== selectedIds.length) selectedIds = next;
  });

  function toggleAll(checked: boolean) {
    selectedIds = checked ? items.map((item) => item.id) : [];
  }

  function toggleItem(id: number, checked: boolean) {
    selectedIds = checked
      ? [...selectedIds, id]
      : selectedIds.filter((selected) => selected !== id);
  }

  function handleDragOver(event: RowDragOverEvent) {
    if (!rowReorder) return;
    const order = moveRowIds(rowIds, event);
    if (order !== rowIds) rowReorder.onMove?.(order);
  }

  function handleDragEnd(event: RowDragEndEvent) {
    if (!rowReorder) return;
    if (event.canceled) rowReorder.onCancel?.();
    else rowReorder.onCommit?.();
  }

  function priceOf(item: LaborItemDto) {
    return item.priceType === "Hourly"
      ? new Big(item.totals.price).times(item.laborHours).toFixed(2)
      : item.totals.price;
  }

  function priceNote(item: LaborItemDto) {
    return item.priceType === "Hourly"
      ? `${decimalText(item.laborHours)} × ${moneyText(item.totals.price)}`
      : "фіксована";
  }

  function hoursText(item: LaborItemDto) {
    return item.priceType === "Hourly" ? decimalText(item.laborHours) : "—";
  }

  function discountTitle(item: LaborItemDto) {
    return item.discountType === "Percentage"
      ? `Знижка ${decimalText(item.discountValue)}%`
      : `Знижка ${moneyText(item.discountValue)} грн.`;
  }

  function discountDetails(item: LaborItemDto) {
    const amount = new Big(item.totals.discount);
    const after = new Big(item.totals.total);
    const before = after.plus(amount);
    const pair = `було ${moneyText(before.toFixed(2))}, стало ${moneyText(after.toFixed(2))}`;

    if (item.discountType === "Percentage") {
      return `−${moneyText(amount.toFixed(2))} грн. · ${pair}`;
    }

    const percent = before.eq(0) ? new Big(0) : amount.div(before).times(100);
    return `−${percent.toFixed(1).replace(".", ",")}% · ${pair}`;
  }
</script>

<div
  class="mt-4 w-full min-w-0 overflow-hidden rounded border border-border bg-card"
>
  <div
    class="flex min-h-12 flex-wrap items-center justify-between gap-x-3 gap-y-2 px-4 py-2"
  >
    <div class="flex min-w-0 flex-wrap items-baseline gap-x-2 gap-y-1">
      <p class="text-xl font-medium">Робота</p>
      <span class="text-base text-muted-foreground tabular-nums">
        {moneyText(total)} грн.
      </span>
    </div>
    {#if actions}
      <div class="flex flex-wrap items-center gap-2">
        {@render actions()}
      </div>
    {/if}
  </div>

  <DragDropProvider
    onDragStart={() => rowReorder?.onStart?.()}
    onDragOver={handleDragOver}
    onDragEnd={handleDragEnd}
  >
    <div
      class={cn(
        "grid h-9 items-center border-y border-border pr-4 text-xs font-medium text-muted-foreground",
        readOnly ? "pl-4" : "pl-2",
      )}
      style={gridStyle}
    >
      {#if !readOnly}
        <div class="flex min-w-0 items-center gap-0.5">
          <span class="size-6 shrink-0" aria-hidden="true"></span>
          <DataTableSelectCheckbox
            title="Обрати всі"
            checkboxClass="rounded-[4px]"
            checked={allSelected}
            indeterminate={someSelected}
            onCheckedChange={toggleAll}
          />
        </div>
      {/if}
      <div class="min-w-0 truncate">Назва</div>
      <div class="text-right">Н-год</div>
      <div class="truncate text-right">Ціна, грн.</div>
      <div class="truncate text-right">Сума, грн.</div>
      {#if !readOnly}
        <div></div>
      {/if}
    </div>

    {#each items as item, index (item.id)}
      <SortableGridRow
        id={String(item.id)}
        {index}
        reorder={rowReorder}
        selected={selectedIds.includes(item.id)}
        class={cn("cursor-default", readOnly ? "pl-4" : "pl-2")}
        style={gridStyle}
      >
        {#if !readOnly}
          <div class="flex min-w-0 items-center gap-0.5">
            <WorkOrderDragHandle label={item.name} />
            <DataTableSelectCheckbox
              title="Обрати рядок"
              checkboxClass="rounded-[4px]"
              checked={selectedIds.includes(item.id)}
              onCheckedChange={(checked) => toggleItem(item.id, checked)}
            />
          </div>
        {/if}

        <div class="min-w-0 pr-2">
          <p class="truncate text-sm font-medium" title={item.name}>
            {item.name}
          </p>
          <p
            class={cn(
              "truncate font-mono text-xs text-muted-foreground",
              !item.code && "font-sans",
            )}
            title={item.code ?? undefined}
          >
            {item.code ?? "—"}
          </p>
        </div>

        <div
          class={cn(
            "text-right text-sm tabular-nums",
            item.priceType === "Hourly" ? "" : "text-muted-foreground",
          )}
        >
          {hoursText(item)}
        </div>

        <div
          class="flex min-w-0 flex-col items-end justify-center leading-tight"
        >
          <span class={leftTruncateClass} title={moneyText(priceOf(item))}>
            <span class="inline-block [direction:ltr] [unicode-bidi:isolate]"
              >{moneyText(priceOf(item))}</span
            >
          </span>
          <span
            class="max-w-full truncate text-xs text-muted-foreground"
            title={priceNote(item)}
          >
            {priceNote(item)}
          </span>
        </div>

        <div
          class="flex min-w-0 flex-col items-end justify-center text-right leading-tight"
        >
          {#if item.isFree}
            <span
              class="inline-flex h-6 items-center rounded-full bg-amber-100 px-2.5 text-xs font-medium whitespace-nowrap text-amber-800 dark:bg-amber-500/15 dark:text-amber-300"
            >
              Безкоштовно
            </span>
          {:else}
            <span
              class={cn(
                leftTruncateClass,
                item.totals.belowCost && "text-rose-600 dark:text-rose-400",
              )}
              title={item.totals.belowCost
                ? undefined
                : moneyText(item.totals.total)}
            >
              {#if item.totals.belowCost}
                <AppTooltip
                  heading="Сума нижча за собівартість"
                  content={`Собівартість ${moneyText(
                    item.totals.cost,
                  )} грн., сума ${moneyText(item.totals.total)} грн.`}
                >
                  {#snippet trigger({ props })}
                    <button
                      {...props}
                      type="button"
                      class="inline-block rounded-[4px] bg-rose-100 px-1.5 py-0.5 text-rose-700 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none dark:bg-rose-500/15 dark:text-rose-300"
                    >
                      {moneyText(item.totals.total)}
                    </button>
                  {/snippet}
                </AppTooltip>
              {:else}
                <span
                  class="inline-block [direction:ltr] [unicode-bidi:isolate]"
                  >{moneyText(item.totals.total)}</span
                >
              {/if}
            </span>
            {#if hasDiscount(item.discountType, item.discountValue)}
              <DiscountTooltip
                first={discountTitle(item)}
                second={discountDetails(item)}
                triggerClass={cn(
                  "block max-w-full truncate text-right text-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                  item.totals.belowCost
                    ? "text-amber-600 dark:text-amber-400"
                    : "text-muted-foreground",
                )}
              >
                {discountText(item.discountType, item.discountValue)}
              </DiscountTooltip>
            {/if}
          {/if}
        </div>

        {#if !readOnly}
          <div class="flex items-center justify-center">
            <LaborTableActions
              id={item.id}
              {onEdit}
              {onDelete}
              {onMoveUp}
              {onMoveDown}
              canMoveUp={index > 0}
              canMoveDown={index < items.length - 1}
            />
          </div>
        {/if}
      </SortableGridRow>
    {:else}
      <div
        class="flex h-[60px] items-center px-4 text-sm text-muted-foreground"
      >
        {emptyMessage}
      </div>
    {/each}
  </DragDropProvider>

  {#if !readOnly && selectedIds.length > 0}
    <div class="flex h-10 items-center gap-3 px-4">
      <p class="text-sm font-medium">
        Обрано {selectedIds.length} з {items.length}
      </p>
      {#if selectionActions}
        {@render selectionActions()}
      {/if}
    </div>
  {/if}
</div>
