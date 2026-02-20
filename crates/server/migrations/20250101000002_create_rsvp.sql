CREATE TABLE IF NOT EXISTS rsvps (
    id              TEXT PRIMARY KEY,
    guest_id        TEXT REFERENCES guests(id) ON DELETE SET NULL,
    song_request    TEXT,
    message         TEXT,
    submitted_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
