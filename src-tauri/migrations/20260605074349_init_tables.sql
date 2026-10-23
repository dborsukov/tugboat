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
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- unique natural key all brands are matched on
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    -- trimmed lowercase name for duplicate detection
    name_key TEXT NOT NULL UNIQUE,
    icon_slug TEXT NOT NULL DEFAULT '',
    is_custom INTEGER NOT NULL DEFAULT 0,
    archived_at TEXT
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

CREATE TABLE expense_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    name_key TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT 'slate',
    archived_at TEXT
);

CREATE TABLE expenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    amount TEXT NOT NULL,
    description TEXT NOT NULL,
    category_id INTEGER REFERENCES expense_categories (id) ON DELETE SET NULL,
    payment_method TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    archived_at TEXT,
    vat_rate TEXT NOT NULL DEFAULT '0',
    price_includes_vat INTEGER NOT NULL DEFAULT 1,
    vat_scheme TEXT NOT NULL DEFAULT 'standard',
    supplier_name TEXT,
    supplier_code TEXT,
    invoice_number TEXT,
    invoice_date TEXT,
    vat_credited INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE workers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    surname TEXT NOT NULL,
    patronymic TEXT,
    position TEXT,
    is_active INTEGER NOT NULL DEFAULT 1
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
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at TEXT,
    completed_at TEXT,
    paid_at TEXT,
    payment_method TEXT,
    tax_invoice_number TEXT,
    tax_invoice_date TEXT,
    exemption_code TEXT,
    vat_rate TEXT NOT NULL DEFAULT "0",
    prices_include_vat INTEGER NOT NULL DEFAULT 1,
    vat_scheme TEXT NOT NULL DEFAULT 'standard',
    archived_at TEXT,
    worker_id INTEGER REFERENCES workers (id) ON DELETE SET NULL,
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
    position INTEGER NOT NULL DEFAULT 0,
    -- NULL means the order's rate and treatment.
    vat_rate TEXT,
    vat_scheme TEXT,
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
    position INTEGER NOT NULL DEFAULT 0,
    -- NULL means the order's rate and treatment.
    vat_rate TEXT,
    vat_scheme TEXT,
    purchase_vat_rate TEXT NOT NULL DEFAULT '0',
    purchase_price_includes_vat INTEGER NOT NULL DEFAULT 1,
    purchase_scheme TEXT NOT NULL DEFAULT 'standard',
    supplier_name TEXT,
    supplier_code TEXT,
    invoice_number TEXT,
    invoice_date TEXT,
    vat_credited INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (work_order_id) REFERENCES work_orders (id) ON DELETE CASCADE
);

CREATE TABLE work_order_sequences (
    year_month TEXT PRIMARY KEY, -- 'YYYYMM'
    counter INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE work_order_document_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_order_id TEXT NOT NULL,
    document_kind TEXT NOT NULL CHECK (
        document_kind IN ('naryad_zamovlennya', 'akt_vykonanykh_robit')
    ),
    pdf_path TEXT NOT NULL,
    xlsx_path TEXT NOT NULL,
    source_hash TEXT NOT NULL,
    options TEXT NOT NULL,
    generated_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (work_order_id) REFERENCES work_orders (id) ON DELETE CASCADE
);

CREATE TABLE reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    report_type TEXT NOT NULL CHECK (report_type IN ('financial', 'vat')),
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    file_path TEXT NOT NULL,
    source_hash TEXT NOT NULL,
    generated_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE vat_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    direction TEXT NOT NULL DEFAULT 'output' CHECK (direction IN ('output', 'input')),
    -- The calendar day that decides the VAT period, in the business time zone
    tax_point TEXT NOT NULL CHECK (length (tax_point) = 10),
    source_kind TEXT NOT NULL CHECK (
        source_kind IN ('work_order', 'expense', 'part_item')
    ),
    -- Order id, expense id or part id, as text, deliberately not a
    -- foreign key because the tax record has to outlive the working record
    source_id TEXT NOT NULL,
    document_number TEXT,
    document_date TEXT,
    -- The benefit code printed on an exempt supply's tax invoice.
    exemption_code TEXT,
    -- The counterparty at the tax point, not as the client record reads today
    counterparty_name TEXT NOT NULL,
    counterparty_code TEXT,
    counterparty_vat_payer INTEGER NOT NULL DEFAULT 0,
    base TEXT NOT NULL,
    rate TEXT NOT NULL,
    amount TEXT NOT NULL,
    scheme TEXT NOT NULL DEFAULT 'standard' CHECK (
        scheme IN (
            'standard',
            'zero',
            'exempt',
            'not_object',
            'export'
        )
    ),
    -- Set on a reversal row only, pointing at what it cancels
    reverses_id INTEGER,
    -- Set on the original when it is reversed
    voided_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- A month whose VAT return has been filed. Nothing may be written into it,
-- corrections belong to the month they are made in.
CREATE TABLE vat_periods (
    -- 'YYYY-MM'
    month TEXT PRIMARY KEY CHECK (length (month) = 7),
    filed_at TEXT NOT NULL DEFAULT (strftime ('%Y-%m-%dT%H:%M:%SZ', 'now')),
    note TEXT
);

CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY NOT NULL DEFAULT 1 CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'system',
    ui_scale REAL NOT NULL DEFAULT 1.0,
    sidebar_metric TEXT NOT NULL DEFAULT 'financial_result',
    show_recent_items INTEGER NOT NULL DEFAULT 1,
    default_markup TEXT NOT NULL DEFAULT '0',
    default_vat_rate TEXT NOT NULL DEFAULT '0',
    default_prices_include_vat INTEGER NOT NULL DEFAULT 1,
    rate_variants TEXT NOT NULL DEFAULT '[]',
    vin_lookup_templates TEXT NOT NULL DEFAULT '[]',
    license_plate_lookup_templates TEXT NOT NULL DEFAULT '[]',
    timezone TEXT NOT NULL DEFAULT 'Europe/Kiev',
    default_phone_country_code TEXT NOT NULL DEFAULT 'UA',
    company_info TEXT,
    dashboard_layout TEXT NOT NULL DEFAULT '{}'
);

INSERT
OR IGNORE INTO app_settings (
    id,
    theme,
    ui_scale,
    sidebar_metric,
    show_recent_items,
    default_markup,
    default_vat_rate,
    default_prices_include_vat,
    rate_variants,
    vin_lookup_templates,
    license_plate_lookup_templates,
    timezone,
    default_phone_country_code,
    company_info,
    dashboard_layout
)
VALUES
    (
        1,
        'system',
        1.0,
        'financial_result',
        1,
        '0',
        1,
        '0',
        '[]',
        '[]',
        '[]',
        'Europe/Kiev',
        'UA',
        '',
        '{}'
    );

CREATE INDEX idx_cars_client_id ON cars (client_id);

CREATE INDEX idx_cars_brand_id ON cars (brand_id);

CREATE INDEX idx_car_brands_archived_at ON car_brands (archived_at);

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

CREATE UNIQUE INDEX idx_expense_categories_name_key_active ON expense_categories (name_key)
WHERE
    archived_at IS NULL;

CREATE INDEX idx_expense_categories_archived_at ON expense_categories (archived_at);

CREATE INDEX idx_expenses_date ON expenses (date);

CREATE INDEX idx_expenses_archived_at ON expenses (archived_at);

CREATE INDEX idx_expenses_category_id ON expenses (category_id);

CREATE INDEX idx_clients_archived_at ON clients (archived_at);

CREATE INDEX idx_client_details_archived_at ON client_details (archived_at);

CREATE INDEX idx_cars_archived_at ON cars (archived_at);

CREATE INDEX idx_work_orders_archived_at ON work_orders (archived_at);

CREATE INDEX idx_services_archived_at ON services (archived_at);

CREATE INDEX idx_workers_is_active ON workers (is_active);

CREATE INDEX idx_work_orders_car_id ON work_orders (car_id);

CREATE INDEX idx_work_orders_worker_id ON work_orders (worker_id);

CREATE INDEX idx_labor_items_service_id ON labor_items (service_id);

CREATE INDEX idx_labor_items_work_order_id ON labor_items (work_order_id);

CREATE INDEX idx_labor_items_work_order_position ON labor_items (work_order_id, position);

CREATE INDEX idx_part_items_work_order_id ON part_items (work_order_id);

CREATE INDEX idx_part_items_work_order_position ON part_items (work_order_id, position);

CREATE INDEX idx_work_order_document_versions_document ON work_order_document_versions (work_order_id, document_kind, generated_at);

CREATE INDEX idx_reports_type_period ON reports (report_type, period_start, period_end);

CREATE INDEX idx_vat_entries_tax_point ON vat_entries (tax_point);

CREATE INDEX idx_vat_entries_source ON vat_entries (source_kind, source_id);

-- One live entry per supply and VAT bucket
CREATE UNIQUE INDEX idx_vat_entries_live_source ON vat_entries (direction, source_kind, source_id, rate, scheme)
WHERE
    reverses_id IS NULL
    AND voided_at IS NULL;

CREATE TABLE recent_items (
    kind TEXT NOT NULL,
    item_id TEXT NOT NULL,
    viewed_at TEXT NOT NULL,
    PRIMARY KEY (kind, item_id)
);

CREATE INDEX idx_recent_items_viewed_at ON recent_items (viewed_at DESC);

CREATE TRIGGER recent_items_prune AFTER INSERT ON recent_items BEGIN
DELETE FROM recent_items
WHERE
    rowid NOT IN (
        SELECT
            rowid
        FROM
            recent_items
        ORDER BY
            viewed_at DESC
        LIMIT
            20
    );

END;

CREATE TRIGGER clients_touch_recent_insert AFTER INSERT ON clients BEGIN
DELETE FROM recent_items
WHERE
    kind = 'client'
    AND item_id = CAST(NEW.id AS TEXT);

INSERT INTO
    recent_items (kind, item_id, viewed_at)
VALUES
    (
        'client',
        CAST(NEW.id AS TEXT),
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

END;

CREATE TRIGGER clients_touch_recent_update AFTER
UPDATE ON clients WHEN NEW.archived_at IS NULL BEGIN
DELETE FROM recent_items
WHERE
    kind = 'client'
    AND item_id = CAST(NEW.id AS TEXT);

INSERT INTO
    recent_items (kind, item_id, viewed_at)
VALUES
    (
        'client',
        CAST(NEW.id AS TEXT),
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

END;

CREATE TRIGGER clients_forget_recent_delete AFTER DELETE ON clients BEGIN
DELETE FROM recent_items
WHERE
    kind = 'client'
    AND item_id = CAST(OLD.id AS TEXT);

END;

CREATE TRIGGER clients_forget_recent_archive AFTER
UPDATE ON clients WHEN NEW.archived_at IS NOT NULL BEGIN
DELETE FROM recent_items
WHERE
    kind = 'client'
    AND item_id = CAST(NEW.id AS TEXT);

END;

CREATE TRIGGER cars_touch_recent_insert AFTER INSERT ON cars BEGIN
DELETE FROM recent_items
WHERE
    kind = 'car'
    AND item_id = CAST(NEW.id AS TEXT);

INSERT INTO
    recent_items (kind, item_id, viewed_at)
VALUES
    (
        'car',
        CAST(NEW.id AS TEXT),
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

END;

CREATE TRIGGER cars_touch_recent_update AFTER
UPDATE ON cars WHEN NEW.archived_at IS NULL BEGIN
DELETE FROM recent_items
WHERE
    kind = 'car'
    AND item_id = CAST(NEW.id AS TEXT);

INSERT INTO
    recent_items (kind, item_id, viewed_at)
VALUES
    (
        'car',
        CAST(NEW.id AS TEXT),
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

END;

CREATE TRIGGER cars_forget_recent_delete AFTER DELETE ON cars BEGIN
DELETE FROM recent_items
WHERE
    kind = 'car'
    AND item_id = CAST(OLD.id AS TEXT);

END;

CREATE TRIGGER cars_forget_recent_archive AFTER
UPDATE ON cars WHEN NEW.archived_at IS NOT NULL BEGIN
DELETE FROM recent_items
WHERE
    kind = 'car'
    AND item_id = CAST(NEW.id AS TEXT);

END;

CREATE TRIGGER work_orders_touch_recent_insert AFTER INSERT ON work_orders BEGIN
DELETE FROM recent_items
WHERE
    kind = 'work_order'
    AND item_id = NEW.id;

INSERT INTO
    recent_items (kind, item_id, viewed_at)
VALUES
    (
        'work_order',
        NEW.id,
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

END;

CREATE TRIGGER work_orders_touch_recent_update AFTER
UPDATE ON work_orders WHEN NEW.archived_at IS NULL BEGIN
DELETE FROM recent_items
WHERE
    kind = 'work_order'
    AND item_id = NEW.id;

INSERT INTO
    recent_items (kind, item_id, viewed_at)
VALUES
    (
        'work_order',
        NEW.id,
        strftime ('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

END;

CREATE TRIGGER work_orders_forget_recent_delete AFTER DELETE ON work_orders BEGIN
DELETE FROM recent_items
WHERE
    kind = 'work_order'
    AND item_id = OLD.id;

END;

CREATE TRIGGER work_orders_forget_recent_archive AFTER
UPDATE ON work_orders WHEN NEW.archived_at IS NOT NULL BEGIN
DELETE FROM recent_items
WHERE
    kind = 'work_order'
    AND item_id = NEW.id;

END;
