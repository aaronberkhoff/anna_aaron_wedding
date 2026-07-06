-- Multiple guests in the same party now share an invite code.
-- The unique constraint was for the old single-primary-guest model.
DROP INDEX IF EXISTS idx_guests_invite_code;
