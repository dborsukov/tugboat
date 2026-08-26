CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY NOT NULL DEFAULT 1 CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'system',
    ui_scale REAL NOT NULL DEFAULT 1.0,
    default_markup TEXT NOT NULL DEFAULT '0',
    rate_variants TEXT NOT NULL DEFAULT '[]',
    vin_lookup_templates TEXT NOT NULL DEFAULT '[]',
    license_plate_lookup_templates TEXT NOT NULL DEFAULT '[]'
);

-- Seed the initial default settings row
INSERT
OR IGNORE INTO app_settings (
    id,
    theme,
    ui_scale,
    default_markup,
    rate_variants,
    vin_lookup_templates,
    license_plate_lookup_templates
)
VALUES
    (1, 'system', 1.0, '0', '[]', '[]', '[]');
