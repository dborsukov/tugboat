PRAGMA foreign_keys = ON;
CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    phone TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE TABLE cars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id INTEGER NOT NULL,
    model TEXT NOT NULL,
    registration TEXT,
    vin TEXT,
    FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
);
CREATE TABLE services (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    unit_price INTEGER NOT NULL,
    is_archived INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    car_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    status TEXT NOT NULL DEFAULT 'draft',
    FOREIGN KEY (car_id) REFERENCES cars(id) ON DELETE RESTRICT
);
CREATE TABLE invoice_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL,
    -- Nullable for custom/one-off items
    service_id INTEGER,
    name TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price INTEGER NOT NULL,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
    FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE
    SET NULL
);
CREATE INDEX idx_cars_client_id ON cars(client_id);
CREATE INDEX idx_invoices_car_id ON invoices(car_id);
CREATE INDEX idx_invoice_items_invoice_id ON invoice_items(invoice_id);
