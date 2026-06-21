PRAGMA foreign_keys = ON;

CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    name TEXT NOT NULL,
    phone TEXT
);

CREATE TABLE car_brands (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    icon_slug TEXT NOT NULL
);

CREATE TABLE car_model_suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    brand_id INTEGER NOT NULL,
    model_name TEXT NOT NULL,
    FOREIGN KEY (brand_id) REFERENCES car_brands (id) ON DELETE CASCADE,
    UNIQUE (brand_id, model_name)
);

CREATE TABLE cars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id INTEGER NOT NULL,
    brand_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    model TEXT NOT NULL,
    year INTEGER NOT NULL,
    registration TEXT,
    vin TEXT,
    FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE CASCADE,
    FOREIGN KEY (brand_id) REFERENCES car_brands (id) ON DELETE RESTRICT
);

CREATE TABLE work_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    car_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    status TEXT NOT NULL DEFAULT 'draft',
    FOREIGN KEY (car_id) REFERENCES cars (id) ON DELETE RESTRICT
);

CREATE TABLE work_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_order_id INTEGER NOT NULL,
    -- Nullable for custom/one-off items
    service_id INTEGER,
    name TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price INTEGER NOT NULL,
    FOREIGN KEY (work_order_id) REFERENCES work_oders (id) ON DELETE CASCADE,
    FOREIGN KEY (service_id) REFERENCES services (id) ON DELETE SET NULL
);

CREATE TABLE services (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    unit_price INTEGER NOT NULL,
    is_archived INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_cars_client_id ON cars (client_id);

CREATE INDEX idx_cars_brand_id ON cars (brand_id);

CREATE INDEX idx_model_suggestions_lookup ON car_model_suggestions (brand_id, model_name);

CREATE INDEX idx_work_orders_car_id ON work_orders (car_id);

CREATE INDEX idx_work_order_items_work_order_id ON work_order_items (work_order_id);
