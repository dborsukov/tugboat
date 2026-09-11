CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    surname TEXT,
    first_name TEXT,
    patronymic TEXT,
    phone TEXT,
    email TEXT,
    client_type TEXT NOT NULL DEFAULT 'individual' CHECK (client_type IN ('individual', 'fop', 'company')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    archived_at TEXT
);

CREATE TABLE client_details (
    client_id INTEGER PRIMARY KEY,
    legal_name TEXT,
    legal_address TEXT,
    edrpou TEXT CHECK (
        edrpou IS NULL
        OR length (edrpou) = 8
    ),
    rnokpp TEXT CHECK (
        rnokpp IS NULL
        OR length (rnokpp) = 10
    ),
    director_surname TEXT,
    director_first_name TEXT,
    director_patronymic TEXT,
    contact_surname TEXT,
    contact_first_name TEXT,
    contact_patronymic TEXT,
    contact_phone TEXT,
    contact_email TEXT,
    iban TEXT,
    bank_name TEXT,
    vat_payer INTEGER NOT NULL DEFAULT 0,
    archived_at TEXT,
    FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE CASCADE
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
    license_plate TEXT,
    vin TEXT,
    engine_code TEXT,
    engine_displacement REAL,
    fuel_type TEXT,
    drive_type TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    archived_at TEXT,
    FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE CASCADE,
    FOREIGN KEY (brand_id) REFERENCES car_brands (id) ON DELETE RESTRICT
);

CREATE TABLE services (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT,
    name TEXT NOT NULL,
    price_type TEXT NOT NULL,
    fixed_price TEXT NOT NULL DEFAULT '0',
    labor_hours TEXT NOT NULL DEFAULT "1.0",
    archived_at TEXT
);

CREATE TABLE work_orders (
    id TEXT PRIMARY KEY,
    car_id INTEGER NOT NULL,
    odometer_in INTEGER NOT NULL,
    odometer_out INTEGER,
    description TEXT,
    notes TEXT,
    condition_report TEXT,
    status TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'regular',
    warranty_of TEXT,
    payer_client_id INTEGER REFERENCES clients (id) ON DELETE SET NULL,
    customer_client_id INTEGER REFERENCES clients (id) ON DELETE SET NULL,
    driver_surname TEXT,
    driver_first_name TEXT,
    driver_patronymic TEXT,
    driver_phone TEXT,
    rate TEXT NOT NULL DEFAULT "0",
    labor_cost_type TEXT NOT NULL,
    labor_cost_value TEXT NOT NULL DEFAULT "0",
    discount_type TEXT NOT NULL,
    discount_value TEXT NOT NULL DEFAULT "0",
    vat_rate TEXT NOT NULL DEFAULT "0",
    prices_include_vat INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at TEXT,
    completed_at TEXT,
    paid_at TEXT,
    payment_method TEXT,
    archived_at TEXT,
    FOREIGN KEY (car_id) REFERENCES cars (id) ON DELETE CASCADE,
    FOREIGN KEY (warranty_of) REFERENCES work_orders (id) ON DELETE SET NULL
);

CREATE TABLE labor_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER,
    work_order_id TEXT NOT NULL,
    code TEXT,
    name TEXT NOT NULL,
    price_type TEXT NOT NULL,
    fixed_price TEXT NOT NULL DEFAULT '0',
    labor_hours TEXT NOT NULL DEFAULT "1.0",
    discount_type TEXT NOT NULL,
    discount_value TEXT NOT NULL DEFAULT "0",
    is_free INTEGER NOT NULL DEFAULT 0,
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
    discount_value TEXT NOT NULL DEFAULT "0",
    is_free INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (work_order_id) REFERENCES work_orders (id) ON DELETE CASCADE
);

CREATE TABLE work_order_sequences (
    year_month TEXT PRIMARY KEY, -- 'YYYYMM'
    counter INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_cars_client_id ON cars (client_id);

CREATE INDEX idx_cars_brand_id ON cars (brand_id);

CREATE UNIQUE INDEX idx_clients_phone_active ON clients (phone)
WHERE
    phone IS NOT NULL
    AND archived_at IS NULL;

CREATE UNIQUE INDEX idx_cars_license_plate_active ON cars (license_plate)
WHERE
    license_plate IS NOT NULL
    AND archived_at IS NULL;

CREATE UNIQUE INDEX idx_cars_vin_active ON cars (vin)
WHERE
    vin IS NOT NULL
    AND archived_at IS NULL;

CREATE UNIQUE INDEX idx_client_details_edrpou ON client_details (edrpou)
WHERE
    edrpou IS NOT NULL
    AND archived_at IS NULL;

CREATE UNIQUE INDEX idx_client_details_rnokpp ON client_details (rnokpp)
WHERE
    rnokpp IS NOT NULL
    AND archived_at IS NULL;

CREATE INDEX idx_clients_archived_at ON clients (archived_at);

CREATE INDEX idx_client_details_archived_at ON client_details (archived_at);

CREATE INDEX idx_cars_archived_at ON cars (archived_at);

CREATE INDEX idx_work_orders_archived_at ON work_orders (archived_at);

CREATE INDEX idx_services_archived_at ON services (archived_at);

CREATE INDEX idx_work_orders_car_id ON work_orders (car_id);

CREATE INDEX idx_labor_items_service_id ON labor_items (service_id);

CREATE INDEX idx_labor_items_work_order_id ON labor_items (work_order_id);

CREATE INDEX idx_part_items_work_order_id ON part_items (work_order_id);
