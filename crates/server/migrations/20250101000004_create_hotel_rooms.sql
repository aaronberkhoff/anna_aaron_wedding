CREATE TABLE IF NOT EXISTS hotel_rooms (
    id          TEXT PRIMARY KEY,
    hotel_name  TEXT NOT NULL,
    room_type   TEXT NOT NULL,
    capacity    INTEGER NOT NULL,
    price_usd   REAL,
    block_code  TEXT,
    booking_url TEXT,
    notes       TEXT
);

CREATE TABLE IF NOT EXISTS hotel_bookings (
    id          TEXT PRIMARY KEY,
    guest_id    TEXT REFERENCES guests(id) ON DELETE SET NULL,
    room_id     TEXT NOT NULL REFERENCES hotel_rooms(id) ON DELETE CASCADE,
    check_in    TEXT,
    check_out   TEXT
);
