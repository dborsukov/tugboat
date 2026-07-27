CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY NOT NULL DEFAULT 1 CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'system'
);

-- Seed the initial default settings row
INSERT
OR IGNORE INTO app_settings (id, theme)
VALUES
    (1, 'system');
