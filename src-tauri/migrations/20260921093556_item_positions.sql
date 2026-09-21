-- Display and document order of work order items. Items are ordered per work
-- order by `position`, with `id` as a tie breaker, and new items are appended
-- at MAX(position) + 1.
ALTER TABLE labor_items
ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

ALTER TABLE part_items
ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

-- Backfill to the order the app displayed until now: insertion order within a
-- work order (the queries had no ORDER BY and returned index order).
UPDATE labor_items
SET
    position = (
        SELECT
            COUNT(*)
        FROM
            labor_items AS earlier
        WHERE
            earlier.work_order_id = labor_items.work_order_id
            AND earlier.id < labor_items.id
    );

UPDATE part_items
SET
    position = (
        SELECT
            COUNT(*)
        FROM
            part_items AS earlier
        WHERE
            earlier.work_order_id = part_items.work_order_id
            AND earlier.id < part_items.id
    );

CREATE INDEX idx_labor_items_work_order_position ON labor_items (work_order_id, position);

CREATE INDEX idx_part_items_work_order_position ON part_items (work_order_id, position);
