PRAGMA foreign_keys = ON;

CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    name TEXT NOT NULL,
    phone TEXT UNIQUE
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
    registration TEXT UNIQUE,
    vin TEXT UNIQUE,
    FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE CASCADE,
    FOREIGN KEY (brand_id) REFERENCES car_brands (id) ON DELETE RESTRICT
);

CREATE TABLE services (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT,
    name TEXT NOT NULL,
    labor_hours REAL NOT NULL DEFAULT 1.0,
    is_archived INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE work_orders (
    id TEXT PRIMARY KEY,
    car_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at TEXT,
    completed_at TEXT,
    odometer_in INTEGER NOT NULL,
    customer_complaint TEXT,
    mechanic_notes TEXT,
    status TEXT NOT NULL,
    rate INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (car_id) REFERENCES cars (id) ON DELETE RESTRICT
);

CREATE TABLE work_order_labor_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER,
    work_order_id TEXT NOT NULL,
    code TEXT,
    name TEXT NOT NULL,
    labor_hours REAL NOT NULL DEFAULT 1.0,
    FOREIGN KEY (work_order_id) REFERENCES work_orders (id) ON DELETE CASCADE,
    FOREIGN KEY (service_id) REFERENCES services (id) ON DELETE SET NULL
);

CREATE INDEX idx_cars_client_id ON cars (client_id);

CREATE INDEX idx_cars_brand_id ON cars (brand_id);

CREATE INDEX idx_work_orders_car_id ON work_orders (car_id);

CREATE INDEX idx_work_order_labor_items_service_id ON work_order_labor_items (service_id);

CREATE INDEX idx_work_order_labor_items_work_order_id ON work_order_labor_items (work_order_id);
