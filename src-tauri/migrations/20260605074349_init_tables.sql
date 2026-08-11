CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    phone TEXT UNIQUE,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now'))
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
    model TEXT NOT NULL,
    year INTEGER NOT NULL,
    license_plate TEXT UNIQUE,
    vin TEXT UNIQUE,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE CASCADE,
    FOREIGN KEY (brand_id) REFERENCES car_brands (id) ON DELETE RESTRICT
);

CREATE TABLE services (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT,
    name TEXT NOT NULL,
    labor_hours TEXT NOT NULL DEFAULT "1.0"
);

CREATE TABLE work_orders (
    id TEXT PRIMARY KEY,
    car_id INTEGER NOT NULL,
    odometer_in INTEGER NOT NULL,
    customer_complaint TEXT,
    mechanic_notes TEXT,
    status TEXT NOT NULL,
    rate TEXT NOT NULL DEFAULT "0",
    discount_type TEXT NOT NULL,
    discount_amount TEXT NOT NULL DEFAULT "0",
    discount_percent TEXT NOT NULL DEFAULT "0",
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at TEXT,
    completed_at TEXT,
    paid_at TEXT,
    payment_method TEXT,
    FOREIGN KEY (car_id) REFERENCES cars (id) ON DELETE RESTRICT
);

CREATE TABLE labor_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER,
    work_order_id TEXT NOT NULL,
    code TEXT,
    name TEXT NOT NULL,
    labor_hours TEXT NOT NULL DEFAULT "1.0",
    discount_type TEXT NOT NULL,
    discount_amount TEXT NOT NULL DEFAULT "0",
    discount_percent TEXT NOT NULL DEFAULT "0",
    FOREIGN KEY (work_order_id) REFERENCES work_orders (id) ON DELETE CASCADE,
    FOREIGN KEY (service_id) REFERENCES services (id) ON DELETE SET NULL
);

CREATE TABLE part_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_order_id TEXT NOT NULL,
    status TEXT NOT NULL,
    code TEXT,
    name TEXT NOT NULL,
    quantity TEXT NOT NULL DEFAULT "1.0",
    unit_type TEXT NOT NULL DEFAULT "шт",
    base_cost TEXT NOT NULL DEFAULT "0",
    markup TEXT NOT NULL DEFAULT "0",
    discount_type TEXT NOT NULL,
    discount_amount TEXT NOT NULL DEFAULT "0",
    discount_percent TEXT NOT NULL DEFAULT "0",
    FOREIGN KEY (work_order_id) REFERENCES work_orders (id) ON DELETE CASCADE
);

CREATE INDEX idx_cars_client_id ON cars (client_id);

CREATE INDEX idx_cars_brand_id ON cars (brand_id);

CREATE INDEX idx_work_orders_car_id ON work_orders (car_id);

CREATE INDEX idx_labor_items_service_id ON labor_items (service_id);

CREATE INDEX idx_labor_items_work_order_id ON labor_items (work_order_id);

CREATE INDEX idx_part_items_work_order_id ON part_items (work_order_id);
