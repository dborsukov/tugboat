CREATE TABLE workers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    surname TEXT NOT NULL,
    patronymic TEXT,
    position TEXT,
    is_active INTEGER NOT NULL DEFAULT 1
);

ALTER TABLE work_orders
ADD COLUMN worker_id INTEGER REFERENCES workers (id) ON DELETE SET NULL;

CREATE INDEX idx_workers_is_active ON workers (is_active);

CREATE INDEX idx_work_orders_worker_id ON work_orders (worker_id);
