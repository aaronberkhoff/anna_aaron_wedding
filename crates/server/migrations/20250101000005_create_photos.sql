CREATE TABLE IF NOT EXISTS photos (
    id          TEXT PRIMARY KEY,
    filename    TEXT NOT NULL UNIQUE,
    caption     TEXT,
    taken_at    TEXT,
    uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
    size_bytes  INTEGER
);
