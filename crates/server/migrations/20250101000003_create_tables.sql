CREATE TABLE IF NOT EXISTS seating_tables (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    capacity    INTEGER NOT NULL,
    notes       TEXT
);

-- Each guest can only be assigned to one table (PRIMARY KEY on guest_id).
CREATE TABLE IF NOT EXISTS table_assignments (
    guest_id    TEXT NOT NULL REFERENCES guests(id) ON DELETE CASCADE,
    table_id    TEXT NOT NULL REFERENCES seating_tables(id) ON DELETE CASCADE,
    seat_number INTEGER,
    PRIMARY KEY (guest_id)
);

CREATE INDEX IF NOT EXISTS idx_table_assignments_table_id ON table_assignments(table_id);
