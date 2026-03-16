-- Add invite code and rehearsal invite flag to the existing guests table.
ALTER TABLE guests ADD COLUMN invite_code TEXT;
ALTER TABLE guests ADD COLUMN rehearsal_invited INTEGER NOT NULL DEFAULT 0;

-- Unique index for invite codes. WHERE IS NOT NULL prevents conflicts
-- among guests who have not yet been assigned a code.
CREATE UNIQUE INDEX IF NOT EXISTS idx_guests_invite_code
    ON guests(invite_code) WHERE invite_code IS NOT NULL;

-- Party members pre-loaded from the invite spreadsheet and linked to a
-- primary guest. Attendance is recorded here when the primary guest RSVPs.
CREATE TABLE IF NOT EXISTS party_members (
    id                  TEXT PRIMARY KEY,
    guest_id            TEXT NOT NULL REFERENCES guests(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    dietary             TEXT NOT NULL DEFAULT 'none',
    attending_reception INTEGER,       -- NULL = not yet RSVPed, 1 = yes, 0 = no
    attending_rehearsal INTEGER        -- NULL = not invited or not yet answered
);

CREATE INDEX IF NOT EXISTS idx_party_members_guest_id ON party_members(guest_id);

-- Extend the rsvps table with per-event attendance and seating preference.
ALTER TABLE rsvps ADD COLUMN attending_reception INTEGER;
ALTER TABLE rsvps ADD COLUMN attending_rehearsal  INTEGER;    -- NULL if not invited
ALTER TABLE rsvps ADD COLUMN known_guests         TEXT;       -- JSON array of names
