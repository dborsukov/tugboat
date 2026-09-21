# Reordering labor and part items

Analysis of what it takes to let users reorder the **Робота** (labor) and
**Запчастини** (parts) tables on a work order — with `Вгору` / `Вниз` row actions
and a drag handle — and to have that order flow into generated documents.

Everything below was checked against the code as it stands today (no order
concept exists anywhere yet).

## 0. Status: implemented with `@dnd-kit/svelte`

Built as described in §3–§6, with `@dnd-kit/svelte` 0.5.0 as the drag engine (§7).
Three things the plan could not know and the implementation settled:

- **The Svelte adapter deliberately drops `OptimisticSortingPlugin`**
  (`createSortable.svelte.js`: "reorders DOM nodes directly during drag, which
  breaks Svelte 5's `{#each}` reconciler"). So the host _must_ reorder its own
  state on every `dragover`; the library only animates. That is the flow in §6.2.
- **The library never reparents the dragged row.** It clones the element as a
  placeholder (a `<tr>` clone, so column widths are untouched) and promotes the
  original row to the top layer with `popover`, moving it with the CSS `translate`
  property. No `DragOverlay` is needed, and `overflow-auto` on the table wrapper
  cannot clip the dragged row.
- **The destination comes from the drop target, never from `source.index`.** See
  §0.1; this was the bug the first manual run found.
- **The dragged row has to be put back into the top layer by us.** Chromium
  closes an element's popover when that element is detached and re-inserted,
  which is how a keyed `{#each}` moves a row, and it does not fire `beforetoggle`
  while doing it — so the library never re-opens it and the row goes invisible
  mid-drag. See §0.2.
- **The drag is locked to the vertical axis.** Rows only reorder up and down, so
  the pointer's horizontal travel is dropped rather than sliding the row out of
  its columns (§6.6).

Files, all verified with `cargo clippy -D warnings`, `svelte-check` (0 errors,
0 warnings) and `prettier`:

| Area                                 | File                                                                                                                               |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Migration                            | `src-tauri/migrations/20260921093556_item_positions.sql`                                                                           |
| Order storage, reorder commands      | `src-tauri/src/database/work_orders/{structs.rs,mod.rs}`, `src-tauri/src/lib.rs`                                                   |
| Drag plumbing, row component, handle | `src/lib/components/data-table/{data-table.svelte,data-table-row.svelte,data-table-drag-handle-cell.svelte,data-table-reorder.ts}` |
| Order helpers                        | `src/routes/work_orders/[workorderId]/item-order.ts`                                                                               |
| Columns, row actions                 | `src/routes/work_orders/[workorderId]/{labor,part}-columns.ts`, `{labor,part}-table-actions-cell.svelte`                           |
| Wiring                               | `src/routes/work_orders/[workorderId]/+page.svelte`, `src/routes/work_orders/queries.ts`                                           |

### 0.1 The drop destination is the target's index, not the source's

**Symptom (first manual run):** `Вгору`/`Вниз` worked, but a dragged row floated
under the pointer and snapped straight back — nothing could be dropped anywhere.

**Cause:** the first implementation read the destination from `source.index`, the
way dnd-kit's own examples do. That is only correct while something keeps
projecting the dragged item's index as the pointer travels — the job of
`OptimisticSortingPlugin`, which the Svelte adapter filters out. With nothing
projecting it, `createSortable`'s `$effect.pre` keeps `sortable.index` equal to
the row's rendered index, so `source.index` is always the source's current
position: every computed "move" was `moveEntry(ids, i, i)`, a no-op. The order
never changed, so no rows moved and the drop persisted an unchanged order.

**Fix:** compute the move from `event.operation.target` (`moveRowIds` in
`data-table-reorder.ts`), which matches what the library's own `move()` helper
falls back to when `source.index` does not differ from the source's position:

```js
// @dnd-kit/dom/sortable.js, the `move`/`mutate` behind `@dnd-kit/helpers`
if (!canceled && typeof source.index === "number") {
  if (source.index !== sourceIndex)
    return arrayMove(items, sourceIndex, source.index);
}
return arrayMove(items, sourceIndex, targetIndex);
```

Reading the target works because the source's own droppable is **proxied**: the
feedback plugin clones the row into the placeholder slot, hands that clone to the
source's droppable (`droppable.proxy`), and `Droppable.element` returns
`proxy ?? element`. So while the original row floats in the top layer, the source
droppable still measures its old slot — the pointer meets a neighbour's rect, not
the dragged row's, and `pointerIntersection` (priority `High`) reports the row
genuinely under the pointer. Dropping the row back onto its own slot resolves to
the source and is a no-op, which is what stops a drag from oscillating.

**Consequence for `dragend`:** `onCommit` no longer receives ids. The drag end
event describes the move that led to the drop, so applying it again would undo
it; the order to keep is the one `onMove` last applied, which the host already
holds (the page persists its own `laborItemIds`/`partItemIds`). A drop that
changed nothing is skipped rather than sent to the backend.

`moveRowIds` is pure and covered by an executable harness (temp file, not
checked in) that walks a pointer over a 5-row table one pixel at a time and
applies the order per target change: a full drag down/up lands the row last/first,
one boundary crossing moves it one place, a wander past the boundary and back
returns the original order, and self/no-target/foreign-target/canceled events
return the input array by identity so no update is queued.

### 0.2 The dragged row must be put back into the top layer

**Symptom (second manual run):** reordering worked, but the dragged row vanished
part-way through a drag — the placeholder kept its slot, no row followed the
pointer, and it stayed invisible until the drag ended.

**Mechanism**, measured on a probe route driven by a real pointer through the
DevTools protocol in Edge 153 (the same Chromium as the app's WebView2), reading
the dragged row's computed style after every pointer move:

```
pressed      : open=true  display=block  rect=[273,125,966,37]
move  4      : open=false display=none   rect=[0,0,0,0]    <- first reorder
move  8 … 24 : open=false display=none                    <- for the rest of the drag
mutation log : REMOVED the dragged row from <tbody> / ADDED the dragged row to <tbody>
```

The library lifts the dragged row out of the table with the `popover` attribute,
i.e. into the top layer. A popover is closed when its element is removed from the
document, and Svelte's keyed `{#each}` moves a row with `dest.before(node)` — a
remove plus an insert, which is a removal. Two details make that permanent:

- **No `beforetoggle` is fired for that close,** so the library's
  `preventPopoverClose` re-show, which listens for exactly that event, never
  runs. Measured directly: `remove()` then insert leaves `:popover-open` false
  with `beforetoggle fired=0`, while moving a _different_ row past it leaves the
  popover open.
- **The UA stylesheet paints a closed popover as `display: none`**
  (`[popover]:not(:popover-open)`), so the row stops being painted at all — its
  rect is 0×0 while its column widths and translate vars stay correct.

Reordering is what triggers it, which is why the row only began disappearing once
§0.1 made reordering work.

**Fix:** `data-table-row.svelte` re-opens the popover. While the row is the drag
source it watches its `<tbody>` for child list changes and calls `showPopover()`
when the library's `popover` attribute is present and the popover is closed. A
MutationObserver callback runs after the rows have been moved, which is what
`showPopover()` needs — it throws on a disconnected element. The repair is scoped
to `isDragging` so it cannot fight the drop animation, and it cannot outlive the
library's cleanup either: removing the `popover` attribute hides the popover, so
the library stays authoritative.

**Verified** on the patched code in the same harness: the row stays
`open=true, display=block` with its rect following the pointer through every
reorder, the drop lands where it was dragged, and an Escape cancel restores the
original order leaving 0 elements with `data-dnd-dragging`, 0 placeholders and 0
open popovers.

The harness is deliberately not checked in, but it is worth recreating for any
future drag change: a throwaway route rendering `DataTable` with `rowReorder`
over local data (no backend), `vite dev` on a spare port, then headless Edge with
`--remote-debugging-port` driven over CDP — `Input.dispatchMouseEvent` produces
trusted pointer events, which is what the library's `setPointerCapture` needs, so
a synthetic `Runtime.evaluate` drag will not do. Two things make the app render
outside Tauri: stub `window.__TAURI_INTERNALS__` via
`Page.addScriptToEvaluateOnNewDocument` (the shell calls Tauri on mount, and the
crash handler logs through the Tauri log plugin, so it throws a second time), and
bind the dev server to `127.0.0.1` (`--host 127.0.0.1`) since Vite otherwise
listens on `::1` only and the browser gets `ERR_CONNECTION_REFUSED`.

Still needs a manual run (`pnpm tauri dev`): how the drag actually feels in the
real window, the animation and edge auto-scroll with a long list, and the
migration applying to a real database.

## 1. Recommendation in one paragraph

Store an explicit integer `position` per item row, ordered `position, id`, and
append new items at `MAX(position) + 1`. Feed the screens and the document
builder from the same ordered query, so documents follow the table for free —
that part needs **zero** changes in `src-tauri/src/documents/`. The UI needs a
new leading "drag" column, two extra dropdown items, and one opt-in capability in
the shared `DataTable` so it can host a drag zone on its own `<tbody>`. Use a
**pointer-event** drag library, never HTML5 DnD — Tauri's native file-drop handler
interferes with `dragover`/`drop` in the webview (§7).

For smooth animation specifically, `@dnd-kit/svelte` is the better engine: it is
transform-only, so a dragged `<tr>` cannot lose its table-computed column widths
mid-drag, and it offers a `DragOverlay` for a properly styled drag preview.
`svelte-dnd-action` is the cheaper fallback (one action on the body plus one per
handle, zero dependencies, stable 0.9.x API) and its FLIP animation is also
transform-based — but it physically relocates the row to build the placeholder,
which is the one place this UI can visibly jank. The Up/Down buttons double as the
accessible/keyboard path, so the drag library only has to get the mouse/touch
case right.

## 2. Facts that decide the design

| Fact                                                                             | Where                                                                                                    | Consequence                                                                                                                                                                                           |
| -------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Item queries have **no** `ORDER BY`; `LaborItem`/`PartItem` have no order column | `src-tauri/src/database/work_orders/mod.rs:146-231`, `migrations/20260605074349_init_tables.sql:119-149` | Order today is whatever SQLite returns — in practice insert order (`id`). Order is undefined, so a column + explicit `ORDER BY` is required before anything else.                                     |
| Both tables already render in **array order**                                    | `labor-columns.ts`, `part-columns.ts`, `+page.svelte:182-229`                                            | `disableSorting`, `disableFiltering`, `disablePagination` are all on for these two tables, so row index == array index == drop index. No index remapping, no "displayed order vs stored order" split. |
| Documents iterate the DTO arrays in order and number rows `i + 1`                | `documents/model.rs:474-560`, `pdf.rs:769`, `xlsx.rs:129-141`                                            | Reordering rows renumbers the document automatically. `build_labor_table` / `build_parts_table` only filter (`labor_is_billed`, `part_is_billed`), they never sort.                                   |
| `generate_document` calls the same `fetch_work_order_details_dto`                | `documents/mod.rs:214`                                                                                   | One `ORDER BY` fixes the table, the PDF and the XLSX at once.                                                                                                                                         |
| Rows are `<tr>` children of a `<tbody>` owned by `DataTable`                     | `data-table.svelte:271-304`                                                                              | A drag zone must be attached to an element this component renders, not to the page's markup. See §6.3.                                                                                                |
| Nothing in the app uses drag and drop today; no Tauri file-drop handler          | grep for `ondrop`/`onDragDropEvent` → no matches                                                         | Free choice of library; `dragDropEnabled` is untouched (relevant in §7).                                                                                                                              |
| Backend guards item writes with `ensure_work_order_active` + `begin_immediate`   | `mod.rs:1103-1141`, `1206`, `1314`, `1416`, `1476`                                                       | The reorder command should follow the same shape: immediate transaction, archived check, no status check (the Paid/Completed lock is a frontend-only concept today).                                  |

## 3. Data model

New migration `src-tauri/migrations/<timestamp>_item_positions.sql` (must sort
**after** `20260914110421_init_workers.sql`; never edit an existing migration —
`sqlx::migrate!` checksums them and the pre-commit config warns about it):

```sql
ALTER TABLE labor_items
ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

ALTER TABLE part_items
ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

-- Backfill to today's visible order: insertion order within a work order.
UPDATE labor_items
SET
  position = (
    SELECT
      COUNT(*)
    FROM
      labor_items x
    WHERE
      x.work_order_id = labor_items.work_order_id
      AND x.id < labor_items.id
  );

UPDATE part_items
SET
  position = (
    SELECT
      COUNT(*)
    FROM
      part_items x
    WHERE
      x.work_order_id = part_items.work_order_id
      AND x.id < part_items.id
  );

CREATE INDEX idx_labor_items_wo_position ON labor_items (work_order_id, position);

CREATE INDEX idx_part_items_wo_position ON part_items (work_order_id, position);
```

Decisions:

- **Name** `position`. `sort_order` is the alternative; `position` reads better in
  `ORDER BY position` and does not collide with anything in these tables
  (`workers.position` is a different table and never joined here).
- **No `UNIQUE (work_order_id, position)`.** SQLite cannot defer unique
  constraints, so a swap would need a temporary sentinel range. Reorders write a
  dense `0..n-1` inside one transaction anyway, and gaps (after a delete) are
  harmless because reads use `ORDER BY position, id`.
- **Gaps are not normalised.** After deleting item at index 2 of 5 the stored
  positions are `0,1,3,4`; ordering is still correct, the next reorder rewrites
  them densely, and new items keep landing at the end via `MAX(position) + 1`.
- **`position` is exposed to the frontend** (it becomes part of `LaborItem` /
  `PartItem` in the generated `bindings.ts`). It is not needed for correctness,
  but it makes the frontend able to detect drift and to sort defensively. If it
  is unwanted in the API, `#[serde(skip)]` on the field is enough; `FromRow`
  still populates it.

## 4. Backend changes

### 4.1 `structs.rs`

Add `#[specta(type = specta_typescript::Number)] pub position: i32` to `LaborItem`
(line 210) and `PartItem` (line 277) plus the matching `row.try_get("position")?`
in both `FromRow` impls. Every struct literal must then be updated:

- `preview_labor_item` — `LaborItem { … }` at `mod.rs:1603`
- `preview_part_item` — `PartItem { … }` at `mod.rs:1675`

### 4.2 Every query that feeds the struct

`position` must be added to each `SELECT`/`RETURNING` list, otherwise `FromRow`
panics at runtime (it reads by column name):

| Function                              | Line (approx.) | Change                                                         |
| ------------------------------------- | -------------- | -------------------------------------------------------------- |
| `fetch_labor_items_by_work_order_ids` | 146            | `+ position`, `+ ORDER BY work_order_id, position, id`         |
| `fetch_part_items_by_work_order_ids`  | 189            | same                                                           |
| `fetch_labor_items_list`              | 1145           | `+ position`                                                   |
| `fetch_labor_item_by_id`              | 1172           | `+ position`                                                   |
| `create_labor_item`                   | 1202           | `+ position` in INSERT (see 4.3) and in `RETURNING`            |
| `update_labor_item`                   | 1258           | `+ position` in `RETURNING` only (edits must not move the row) |
| `fetch_part_items_list`               | 1355           | `+ position`                                                   |
| `fetch_part_item_by_id`               | 1381           | `+ position`                                                   |
| `create_part_item`                    | 1412           | `+ position` in INSERT and `RETURNING`                         |
| `update_part_item`                    | 1471           | `+ position` in `RETURNING` only                               |

The two `by_work_order_ids` helpers are shared by `fetch_work_order_details_dto`,
`…_list`, `…_by_car_id` and `…_by_client_id`, so all four entry points get the
order together. `src-tauri/src/lib.rs` needs no change for these.

### 4.3 Appending new items

Inside the existing `begin_immediate` transaction, extend the INSERT so a new
item lands last:

```sql
INSERT INTO labor_items (service_id, work_order_id, code, name, price_type,
                         fixed_price, labor_hours, discount_type, discount_value,
                         is_free, position)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
        (SELECT COALESCE(MAX(position), -1) + 1
           FROM labor_items WHERE work_order_id = $2))
RETURNING …, position
```

Same for `create_part_item`. Reusing `$2` for the subquery is fine with sqlx's
numbered parameters, and `begin_immediate` (write lock taken up front) means a
concurrent reorder cannot interleave.

### 4.4 Two new commands

`reorder_labor_items(work_order_id: String, ordered_ids: Vec<i32>)` and
`reorder_part_items(...)`, sharing one helper that reuses the existing
`ItemTable` enum (`mod.rs:27-39`):

```rust
async fn reorder_items(
    state: &State<'_, crate::AppState>,
    table: ItemTable,
    work_order_id: &str,
    ordered_ids: &[i32],
) -> AppResult<()> {
    let mut tx = begin_immediate(&state.db_pool).await?;
    ensure_work_order_active(&mut *tx, work_order_id).await?;

    // The submitted order must be a permutation of this work order's items:
    // no foreign ids, none missing, no duplicates.
    let mut existing: Vec<i32> = sqlx::query_scalar(&format!(
        "SELECT id FROM {} WHERE work_order_id = $1",
        table.table_name()
    ))
    .bind(work_order_id)
    .fetch_all(&mut *tx)
    .await?;

    let mut submitted = ordered_ids.to_vec();
    existing.sort_unstable();
    submitted.sort_unstable();
    if existing != submitted {
        return Err(AppError::validation(
            "Порядок позицій не відповідає вмісту замовлення",
        ));
    }

    for (index, id) in ordered_ids.iter().enumerate() {
        sqlx::query(&format!(
            "UPDATE {} SET position = $2 WHERE id = $1",
            table.table_name()
        ))
        .bind(id)
        .bind(index as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

Notes:

- **One "write the whole order" command, not `move_up`/`move_down`.** Both
  buttons and the drop handler can express their intent as a full ordered id
  list, so one command covers everything, is idempotent, and cannot drift out of
  sync with the UI. n is small (tens), so n `UPDATE`s in one transaction is
  cheap; a single `CASE` statement would be premature.
- The permutation check turns a buggy or malicious client into a clear error
  instead of silently corrupting another order.
- Register both in `lib.rs`'s `collect_commands![ … ]` next to
  `update_labor_items_discount` / `update_part_items_discount` — that is what
  generates the `commands.reorderLaborItems` binding.
- clippy runs with `-D warnings` in pre-commit; `format!` on the table name is
  the same pattern `ensure_item_work_order_active` already uses.

## 5. Generated documents

**No changes required** in `documents/mod.rs`, `model.rs`, `pdf.rs`, `xlsx.rs`.
The chain is already order-preserving:

```
generate_document → fetch_work_order_details_dto → fetch_*_items_by_work_order_ids (ORDER BY position)
  → WorkOrderDetailsDto.labor_items / .part_items
    → build_labor_table / build_parts_table (filter only, keep order)
      → DocTable.rows → pdf row numbers (i+1) / xlsx row numbers (i+1)
```

Two things worth stating explicitly because they are easy to assume wrongly:

- Documents are **files on disk**, generated at a point in time. Reordering after
  a document was generated does not (and should not) rewrite it — you regenerate.
- The **filters stay filters**: `labor_is_billed`, `part_is_billed` and
  `part_is_projected_billed` decide which rows appear, and the numbering is
  recomputed over the surviving rows. Reordering therefore changes the visible
  sequence and its numbering, not which rows are included.

The only risk is a future caller that builds a `WorkOrderDetailsDto` without
going through `fetch_*_items_by_work_order_ids`; there is none today.

## 6. Frontend

### 6.1 What the UI needs

- **Drag handle column**, first column, before `select`:
  `{ id: "drag", header: <empty>, cell: renderComponent(DataTableDragHandleCell, {...}) }`
  in both `labor-columns.ts` and `part-columns.ts`. It must be dropped together
  with `select`/`actions` when the order is read-only — the page already filters
  columns by id in the `columns` getters (`+page.svelte:186-197, 214-218`), so add
  `c.id !== "drag"` to that filter. Add `drag: "w-8 p-0"` to `headerClasses`
  (and `cellClasses`) next to the existing `select: "w-10 p-0"`.
- **Handle component** (new, shared):
  `src/lib/components/data-table/data-table-drag-handle-cell.svelte` — a
  `Button variant="ghost" size="icon-sm"` with `<GripVerticalIcon />`,
  `class="cursor-grab active:cursor-grabbing touch-none select-none"`,
  `aria-label="Перемістити рядок"`, carrying the library's handle attachment
  (§6.3).
- **Row actions**: add to `labor-table-actions-cell.svelte` and
  `part-table-actions-cell.svelte`, between the existing items:

  ```svelte
  <DropdownMenu.Separator />
  <DropdownMenu.Item disabled={!canMoveUp} onclick={onMoveUp}>
    <ArrowUpIcon /> Вгору
  </DropdownMenu.Item>
  <DropdownMenu.Item disabled={!canMoveDown} onclick={onMoveDown}>
    <ArrowDownIcon /> Вниз
  </DropdownMenu.Item>
  <DropdownMenu.Separator />
  ```

  `DropdownMenuPrimitive.ItemProps` already supports `disabled` and the wrapper
  styles `data-disabled:opacity-50`, so boundary rows get a greyed-out item for
  free. Boundaries are known locally in the column def: `row.index === 0` and
  `row.index === table.getRowModel().rows.length - 1` — safe here precisely
  because sorting/filtering/pagination are disabled for these tables.

- **Meta plumbing**: extend `LaborTableMeta` / `PartsTableMeta` with
  `onMoveUp: (id: number) => void` and `onMoveDown: (id: number) => void`, and
  pass them from the page's `buildTable({ meta })` factories next to
  `onEdit`/`onDelete`. The page stays the single source of truth for the order;
  the cells never reorder anything themselves.

### 6.2 The data flow for a move

```
button/drop → new ordered id array → optimistic cache write (instant UI)
            → reorder_*_items command → onSettled: invalidate work-orders queries
```

What was built is this flow with the cache write moved earlier: a drag writes the
order straight into the query cache from the drag's `onMove` (no mutation until
the drop, so no `onMutate`/`onError` rollback is involved), and the drop persists
it — §0.1 has that contract. The buttons use the diagram as written.

- Add `createReorderLaborItemsMutation` / `createReorderPartItemsMutation` to
  `src/routes/work_orders/queries.ts`, following the existing factory pattern,
  but with optimistic updates because a drag must feel immediate:

  ```ts
  onMutate: async ({ workOrderId, orderedIds }) => {
    await queryClient.cancelQueries({ queryKey: keys.workOrders.detail(workOrderId) });
    const previous = queryClient.getQueryData(keys.workOrders.detail(workOrderId));
    queryClient.setQueryData(keys.workOrders.detail(workOrderId), (old) =>
      old ? { ...old, laborItems: applyOrder(old.laborItems, orderedIds) } : old);
    return { previous, workOrderId };
  },
  onError: (error, _vars, context) => {
    queryClient.setQueryData(keys.workOrders.detail(context.workOrderId), context.previous);
    showMutationErrorToast(error);
  },
  onSettled: () => invalidateWorkOrderLists(queryClient),
  ```

  Reordering cannot change any total (`pricing.rs` sums over the whole array), so
  the optimistic patch only has to reshuffle `laborItems` / `partItems`. It must
  return a **new array** rather than splicing in place: TanStack rebuilds its row
  model off the data reference, and an in-place `splice` would leave the DOM
  showing the old order.

- Pure helpers (new file, e.g. `src/routes/work_orders/[workorderId]/item-order.ts`):
  `applyOrder(items, orderedIds)` (ids that are not in the list keep their
  relative order at the end — which is exactly what the backend does for items
  created meanwhile), `moveBy(items, id, delta)`, `swap(items, from, to)`. No test
  runner is configured in this repo (no `test` script, no vitest config), so
  these are verified by hand or by adding a runner first — worth deciding.
- During a drag, the same `applyOrder` is applied to the cache on every
  `consider` event (**no DB write**); the command fires once on `finalize`.

### 6.3 Hosting a drag zone inside `DataTable`

This is the only genuinely intricate part. It is written against
`svelte-dnd-action`'s API, but the contract below is library-agnostic — `@dnd-kit/svelte`
has the same shape (`items` array, live reorder during drag, commit on drop), with
`DragOverlay` replacing the shadow-item mechanism (§7.1). Constraints coming from
the library:

1. The zone is an element whose children are **exactly** the items, in the same
   order, keyed by item id — no extra or missing children.
2. The host must update the items list on **both** `consider` and `finalize`.
3. Item ids must be unique **within a zone type**, and zones of the same type can
   exchange items.

Constraint 3 is a trap on this page: both tables live side by side, so they must
get different `type`s (e.g. `"work-order-labor-items"` /
`"work-order-part-items"`), otherwise labor rows could be dropped into the parts
table even though the two tables have overlapping numeric ids.

Constraints 1–2 map onto `DataTable` like this:

- The page passes a `rowReorder` prop, and `DataTable` derives the items array
  from the very rows it renders:

  ```ts
  const rows = $derived(table.getRowModel().rows);
  const zoneOptions = $derived({
    items: rows.map((row) => ({ id: row.id })), // row.id === String(item.id)
    type: reorder.type,
    flipDurationMs: 150,
    dragDisabled: reorder.disabled,
    zoneTabIndex: -1, // don't add a tab stop per table
    zoneItemTabIndex: -1, // rows contain their own controls
  });
  ```

  `row.id` already is `String(item.id)` thanks to `getRowId` in the page, so no
  extra id plumbing is needed.

- **The empty table is an edge case**: when a table has no rows `DataTable`
  renders a single "nothing here" `<tr>` (`data-table.svelte:294-303`), which
  would be one child for zero items. Attach the zone only when
  `rows.length > 0` — a conditional attachment is fine, because `attach()`
  simply does nothing while the value is falsy.

- **The placeholder ("shadow") row.** While dragging, the library injects a
  temporary item marked with `SHADOW_ITEM_MARKER_PROPERTY_NAME` into the array it
  hands to `consider`, and expects the consumer to render it as the gap where the
  row will land. `DataTable` must therefore (a) keep its own
  `placeholderIndex` state derived from that event, (b) insert the marker item
  into `zoneOptions.items` at that index, and (c) render one extra "ghost" `<tr>`
  at that position so children and items still match 1:1. It is roughly 25 lines
  in `data-table.svelte`, and it is what makes the drop target visually correct.
  The simpler v1 alternative — filter the shadow item out and never render a gap
  — is worth a 10-minute spike first; if the drop index and the animation still
  behave, the ghost row can be skipped.

- **Attaching an action to a component-owned `<tbody>`.** `DataTable` renders
  `<TableUI.Body>` (a vendored shadcn component), and Svelte 5 does not allow
  `use:` on a component. Svelte 5.32+/5.56 has the answer built in:
  `fromAction()` from `svelte/attachments` converts a library action into an
  attachment, and `{@attach …}` on a component becomes a symbol-keyed prop that
  the component's `{...restProps}` spread applies to its DOM element.
  `src/lib/components/ui/table/table-body.svelte` already does
  `{...restProps}` on `<tbody>`, and `flex-render.svelte` already carries an
  (unused) `attach` prop of exactly this kind — so no vendored file needs to be
  touched (which also keeps `docs/overlays.md`'s "vendored files get overwritten
  by `shadcn-svelte add`" warning out of play):

  ```svelte
  <TableUI.Body bind:ref={bodyRef} {@attach fromAction(dragHandleZone, () => zoneOptions)}>
  ```

  The handle is simpler: it lives in our own component, so
  `{@attach fromAction(dragHandle)}` on the `Button` (`button.svelte` spreads
  `{...restProps}` onto `<button>`) or a plain `<button use:dragHandle>` both
  work.

- `consider` / `finalize` are dispatched as custom events on the zone element.
  Since `DataTable` owns that element via `bind:ref={bodyRef}`, the least
  fragile option is to add the listeners imperatively in an `$effect` on
  `bodyRef` rather than pushing `onconsider` props through a vendored component
  (both work; the imperative version keeps the typing honest).

- **Svelte 5 `$state` proxy caveat.** `svelte-dnd-action` 0.9.58 regressed on
  zones whose items array is a `$state()` proxy
  ([#637](https://github.com/isaacHagoel/svelte-dnd-action/issues/637)), and
  fast optimistic updates are a known rough edge
  ([#625](https://github.com/isaacHagoel/svelte-dnd-action/issues/625)). Pass a
  plain snapshot (`$state.snapshot(...)`) into `zoneOptions.items` and keep the
  reactive source in the query cache, not in the array handed to the library.

### 6.4 Page wiring (`[workorderId]/+page.svelte`)

- `moveLaborItem(id, delta)` / `movePartItem(id, delta)` read the current id order
  from `workOrderDetails`, compute the new order with the pure helpers, and call
  the mutation. Guard with `isReadOnly` (already derived from `isLocked ||
isArchived`).
- `handleLaborConsider` / `handleLaborFinalize` (and parts equivalents): map the
  event's items to ids, dropping the shadow item, patch the cache, and on
  `finalize` fire the mutation.
- Pass `rowReorder={{ type, disabled: isReadOnly, onConsider, onFinalize }}` to
  both `DataTable` instances.
- Bulk selection, bulk discount targets and the delete dialogs all key off item
  ids (`laborBulkRowsById`, `selectedLaborIds`), so they are order-independent and
  need no changes. `getFilteredSelectedRowModel()` keeps working because row
  selection is keyed by row id.

### 6.5 Library setup details

- `pnpm add -D svelte-dnd-action` — devDependency is consistent with how
  `bits-ui`, `@tanstack/table-core` and `formsnap` are declared here;
  `sort-package-json` (pre-commit) keeps the key order.
- Screen-reader strings are English by default. The app is Ukrainian-only, so
  either call `setAriaStrings({ … })` once at startup with Ukrainian wording, or
  pass `autoAriaDisabled: true` and rely on the `aria-label`s plus the Up/Down
  buttons for accessibility.
- Consider `delayTouchStart: true` if touch scrolling on the order page ever
  starts a drag.

### 6.6 Animation tuning that matters either way

The library only supplies the motion; these are the things in _this_ codebase that
will make it look janky regardless of which one is picked:

- **The vendored row has a hover transition.** `ui/table/table-row.svelte:18` is
  `hover:bg-muted/50 … transition-colors`, so while a row is being dragged the
  highlight keeps firing under the pointer and paints over the animation. Suppress
  row hover while a drag is active (e.g. a `data-dragging` attribute on the scroll
  wrapper plus one override rule).
- **The vendored button nudges itself on `:active`.** `buttonVariants` carries
  `active:not-aria-[haspopup]:translate-y-px` (`ui/button/button.svelte:11`), which
  on the drag handle is a 1px jump at the exact moment the drag starts. Add
  `active:translate-y-0` to the handle's class.
- **Keep it to `transform`/`opacity`.** Do not animate the row's `border-b`, the
  `divide-x` dividers or the zebra fill — those are paint-bound. Both libraries
  already restrict themselves to transforms; the risk is our own CSS.
- **Duration:** 150–200 ms with an ease-out curve for the "make room" motion
  (`flipDurationMs` for svelte-dnd-action, the transition option for dnd-kit).
  Anything under ~120 ms reads as a snap, anything over ~250 ms as lag.
- **`will-change: transform` on the dragged row only** — promoting every row for the
  lifetime of the page wastes compositor memory for no benefit.
- **Handle styling:** `touch-none select-none cursor-grab active:cursor-grabbing`.
- **Respect `prefers-reduced-motion`:** set the duration options to `0` and skip
  `animate:flip` when it matches.
- **Auto-scroll:** the table lives in an `overflow-auto` wrapper
  (`data-table.svelte:243-250`). Both libraries auto-scroll a scrollable ancestor
  during a drag (dnd-kit through a configurable plugin); verify it with 30+ rows,
  because a long order is exactly where a missing auto-scroll becomes obvious.
- **Lock the drag to the vertical axis.** A row only ever reorders up and down, so
  letting the pointer's horizontal travel through slides it out of its columns for
  no reason. `data-table-row.svelte` passes `RestrictToVerticalAxis` (from
  `@dnd-kit/abstract/modifiers`) to `createSortable`, which zeroes the transform's
  `x`. Modifiers are per-entity on purpose: the manager's `modifiers` option takes
  instances, while a sortable accepts the ready-made descriptors
  (`{plugin, options}`) that `*.configure()` returns. Verified with a diagonal drag
  — the pointer 288 px to the right, `translate` stayed `0px <y>` and the row's
  `x`/width never moved, while the reorder kept following the pointer's `y`.

## 7. Library choice

|                          | `svelte-dnd-action` 0.9.79                                                                                                                                      | `@dnd-kit/svelte` 0.5.0                                                         | `@atlaskit/pragmatic-drag-and-drop` / SortableJS / hand-rolled |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Svelte 5                 | ✅ peer `>=3.23.0 \|\| ^5.0.0-next.0`; docs cover `onconsider`/`onfinalize` for Svelte 5                                                                        | ✅ peer `^5.29.0`, rune-native                                                  | framework-agnostic glue                                        |
| Maturity                 | 0.9.x since 2020, actively maintained, single maintainer, no breaking-change policy violations in years                                                         | 0.x "next" line of dnd-kit; 0.2.3 → 0.5.0 inside a few months, API still moving | mature libs                                                    |
| Dependencies             | none                                                                                                                                                            | `@dnd-kit/abstract`, `@dnd-kit/dom`, `@dnd-kit/state`, `tslib`                  | varies                                                         |
| Input model              | **pointer events** (explicitly not HTML5 DnD), touch, keyboard                                                                                                  | pointer/keyboard                                                                | Pragmatic DnD and SortableJS use **native HTML5 DnD**          |
| Drag handles             | first-class: `dragHandleZone` + `dragHandle` since 0.9.46                                                                                                       | doable, more wiring                                                             | varies                                                         |
| Integration surface here | 1 action on `<tbody>` + 1 per handle                                                                                                                            | provider + per-row draggable/droppable attachments + sensors                    | varies                                                         |
| Table rows               | designed for arbitrary containers; a fixed `<tr>`-styles bug exists ([#167](https://github.com/isaacHagoel/svelte-dnd-action/issues/167)) → verify in the spike | transform-based, never reparents nodes, so table layout is inherently safer     | varies                                                         |
| Known friction           | `$state()` arrays (#637), fast optimistic updates (#625)                                                                                                        | young port, thin docs/examples for tables                                       | —                                                              |

### 7.1 Animation: which engine is actually smoother

Smooth dragging is a stated requirement, and this is where the two libraries differ
architecturally rather than cosmetically.

| Animation aspect            | `svelte-dnd-action`                                                                                                                     | `@dnd-kit/svelte`                                                                                                                          |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| "Make room" while dragging  | FLIP — `animate:flip` from `svelte/animate` plus a `flipDurationMs` that must match the zone option, driven by the order array changing | inline `transform` + CSS `transition` maintained by the library on every sortable item, duration/easing configurable                       |
| Motion of the dragged row   | `transform` driven by the pointer                                                                                                       | `transform` driven by the pointer, **or** a `DragOverlay` clone outside the table                                                          |
| DOM ownership during a drag | the library **relocates/duplicates the node** to synthesise the placeholder ("morph"), then the host re-renders the same order          | never reparents anything; only writes `transform`/`transition` inline styles, and sorting is committed by calling `move()` in `onDragOver` |
| Cost per frame              | compositor transform, plus a possible table layout recalculation when the row is pulled out of the `<table>`                            | compositor transform only — column widths cannot change mid-drag                                                                           |
| Drop animation              | built in (`dropAnimationDisabled` to opt out)                                                                                           | built in, or animate the `DragOverlay` back yourself                                                                                       |
| Escape hatches              | `morphDisabled`, `dropAnimationDisabled`, `transformDraggedElement`, `centreDraggedOnCursor`                                            | `DragOverlay`, plugins, modifiers, custom drop animation                                                                                   |

**Verdict: `@dnd-kit/svelte` is the better animation engine**, and since smooth
dragging is now a requirement it is the primary choice. Both libraries animate with
transforms rather than layout properties, so at ~10 rows this table will look the
same under either — the difference is the failure mode. dnd-kit is transform-only by
construction, so the `<tr>` keeps the widths the table layout gave it, and its
`DragOverlay` is a first-class answer to "the dragged row must not look broken".

Cost of that choice, all of it contained inside `DataTable`:

- it is 0.x (0.5.0 is `latest`, 0.5.1-beta-\* was published days later) while
  `svelte-dnd-action`'s 0.9.x API has been stable for years — expect to pin a
  version and re-verify on upgrades
- four transitive deps instead of none (`@dnd-kit/abstract`, `@dnd-kit/dom`,
  `@dnd-kit/state`, `tslib`)
- `createSortable` must be called **once per item component**; calling it inline in
  an `{#each}` "creates a fresh `Sortable` instance per render and breaks sort
  transition animations"
  ([docs](https://dndkit.com/svelte/primitives/create-sortable/)). The row loop in
  `data-table.svelte:272-293` therefore has to be extracted into a
  `data-table-row.svelte` component — a clean refactor, but not free
- a `DragDropProvider` must wrap the table, and `createDraggable`/`createSortable`
  may only be called from components rendered _inside_ it
- none of §6.2–§6.4 changes: the items-array contract, the empty-table guard, the
  optimistic order in the query cache, distinct types per table and the
  consider→finalize split all apply to dnd-kit too; only the svelte-dnd-action
  specifics (`SHADOW_ITEM_MARKER_PROPERTY_NAME`, `SHADOW_PLACEHOLDER_ITEM_ID`,
  `zoneTabIndex`, `setAriaStrings`) are replaced by dnd-kit's own equivalents
  (drag overlay, sensors, announcements)

Keep `svelte-dnd-action` as the documented fallback: if the 0.x churn or the
per-row component requirement costs more than the animation is worth, it is one
action, zero dependencies, and it will very likely look identical on a 10-row
table. Two caveats are structural rather than cosmetic:

1. **`svelte-dnd-action` physically relocates the dragged `<tr>`.** A `<tr>` moved
   out of its `<table>` loses the column widths the table layout gave it, which is
   why it is now the fallback rather than the primary choice. If it is chosen
   anyway, `transformDraggedElement` can rebuild the preview as a fixed-position
   block.
2. **HTML5 DnD is a bad idea in this app.** `tauri.conf.json` does not set
   `dragDropEnabled`, so Tauri's native file-drop handler is active; on Windows it
   is a well-known source of swallowed `dragover`/`drop` events inside the
   webview. Pointer-event libraries sidestep that entirely; a hand-rolled
   `dragstart`/`dragover` implementation would need `"dragDropEnabled": false` on
   the main window to work reliably (harmless today — nothing handles file drops
   — but it is a real config change, not a detail).

Given that the Up/Down buttons already provide the keyboard and screen-reader
path, the library only has to nail the pointer case, which is the least risky
part of the whole feature.

## 8. Edge cases to get right

| Case                                               | Expected behaviour                                                                                          |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Read-only order (archived, or Paid/Completed lock) | No drag column, no `Вгору`/`Вниз`, `dragDisabled: true`. Mirrors the existing `select`/`actions` filtering. |
| Empty table                                        | Zone not attached (one empty-state `<tr>` would break the children ↔ items invariant).                      |
| Single row                                         | Both buttons disabled; drag handle inert.                                                                   |
| First / last row                                   | `Вгору` / `Вниз` disabled respectively.                                                                     |
| New item created (sheet or "Додати з бази")        | Appended last, by `MAX(position) + 1`.                                                                      |
| Item deleted                                       | Gaps in `position` are fine; order unaffected.                                                              |
| Selection / bulk discount active while reordering  | Id-keyed, unaffected.                                                                                       |
| Drag canceled (Escape / dropped outside)           | `finalize` with the original order → the optimistic patch is a no-op, no DB write.                          |
| Reorder + concurrent create                        | Serialised by `begin_immediate`; the permutation check rejects any stale submission.                        |
| Reorder while a document dialog is open            | Dialog reads `workOrderDetails` at generation time; regeneration uses the new order.                        |
| Duplicate positions from an external/older DB      | `ORDER BY position, id` keeps it deterministic.                                                             |
| Two tables, same numeric item ids                  | Distinct zone `type`s (or `dropFromOthersDisabled`) prevent cross-table drops.                              |

Non-goals worth confirming: reordering on the **work orders list** page, reordering
**within** the generated document independently of the table, and persisting an
order for the services dictionary.

## 9. Verification plan

- `cargo clippy -- -D warnings` and `cargo +nightly fmt` (both enforced by
  pre-commit), `pnpm check` (svelte-check) for the frontend, `npx prettier --write`.
- `src/lib/bindings.ts` is regenerated automatically by specta in a debug build;
  confirm `position` and the two `reorder*` commands appear there before wiring
  the frontend (the file is excluded from prettier and from the pre-commit
  `exclude` list, so it should not be hand-edited).
- Manual matrix: buttons on first/middle/last row; drag first→last, last→first,
  adjacent; both tables; archived + paid order; empty + single-row table; reload
  the order to confirm persistence; then generate **both** document kinds in
  **both** formats and check row order and the 1..n numbering, including the case
  where some parts are `Returned`/`Quoted` and therefore filtered out.
- Migration check: open an existing order before and after the migration and
  confirm the visible order is unchanged (backfill reproduces `id` order).
- Animation check (in the real WebView2, not a browser): record a drag in the
  Performance panel and confirm no layout/paint work during the drag itself, that
  the dragged row keeps its column widths, that `prefers-reduced-motion` is
  honoured, and that a 30+ row order auto-scrolls at the wrapper edge.

## 10. Work breakdown

| Step                                                                                                                                                                                                                                                                                                                                   | Files                                                                                     | Size |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ---- |
| 1. Migration + `position` on both structs and every query/RETURNING                                                                                                                                                                                                                                                                    | migration, `work_orders/structs.rs`, `work_orders/mod.rs`                                 | S    |
| 2. Append-on-create (`MAX(position) + 1`)                                                                                                                                                                                                                                                                                              | `work_orders/mod.rs`                                                                      | S    |
| 3. `reorder_labor_items` / `reorder_part_items` + registration                                                                                                                                                                                                                                                                         | `work_orders/mod.rs`, `lib.rs`                                                            | S    |
| 4. Verify documents pick the order up (no code)                                                                                                                                                                                                                                                                                        | —                                                                                         | XS   |
| 5. Frontend mutations + pure order helpers                                                                                                                                                                                                                                                                                             | `work_orders/queries.ts`, new `item-order.ts`                                             | S    |
| 6. `DataTable` reorder capability (zone, placeholder row, events, empty-table guard)                                                                                                                                                                                                                                                   | `data-table/data-table.svelte`, new handle cell                                           | M    |
| 7. Columns, action cells, page wiring, read-only gating                                                                                                                                                                                                                                                                                | `labor-columns.ts`, `part-columns.ts`, both `*-table-actions-cell.svelte`, `+page.svelte` | M    |
| 8. Bake-off before committing to a library: throwaway page with the real row markup (~30 rows) in the actual WebView2, one drag implementation per library; judge with a Performance recording (no layout/paint during the drag, steady frame rate), check the dragged row keeps its column widths, and check `prefers-reduced-motion` | throwaway                                                                                 | M    |

Rough total: about half a day of Rust, one to one-and-a-half days of frontend,
with step 8 deciding whether the preview needs the `transformDraggedElement`
workaround or a switch to `@dnd-kit/svelte`.

## 11. Open questions

1. Should the Up/Down actions also get keyboard shortcuts (e.g. `Alt+↑`/`Alt+↓`)
   on the focused row, or is the dropdown enough?
2. Should drag reordering be disabled while a bulk discount dialog or a mutation
   sheet is open, to avoid the optimistic order and a stale form disagreeing?
3. Is `position` wanted in the public bindings, or hidden with `#[serde(skip)]`?
4. Does the technician need to reorder the parts table by installation sequence
   independently from the document order (e.g. "show pending parts last") — i.e.
   is one order enough, or is a second "document order" needed later?
