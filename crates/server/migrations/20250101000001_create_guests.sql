CREATE TABLE IF NOT EXISTS guests (
    id                TEXT PRIMARY KEY,          -- UUID stored as text
    first_name        TEXT NOT NULL,
    last_name         TEXT NOT NULL,
    email             TEXT,
    phone             TEXT,
    rsvp_status       TEXT NOT NULL DEFAULT 'pending',   -- pending | attending | declined
    dietary           TEXT NOT NULL DEFAULT 'none',       -- none | vegetarian | vegan | gluten_free | halal_kosher | other:...
    plus_one          INTEGER NOT NULL DEFAULT 0,         -- boolean (0|1)
    plus_one_name     TEXT,
    invite_sent       INTEGER NOT NULL DEFAULT 0,
    notes             TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_guests_last_name ON guests(last_name);
CREATE INDEX IF NOT EXISTS idx_guests_rsvp_status ON guests(rsvp_status);
