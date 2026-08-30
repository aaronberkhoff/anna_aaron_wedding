CREATE TABLE IF NOT EXISTS registry_funds (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL,
    description    TEXT NOT NULL,
    venmo_username TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fund_contributions (
    id                TEXT PRIMARY KEY,
    fund_id           TEXT NOT NULL REFERENCES registry_funds(id) ON DELETE CASCADE,
    contributor_name  TEXT NOT NULL,
    amount_usd        REAL NOT NULL,
    created_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_fund_contributions_fund_id ON fund_contributions(fund_id);

INSERT INTO registry_funds (id, name, description, venmo_username) VALUES
  ('a1a1a1a1-1111-4111-8111-111111111111', 'New Home Fund',  'Help us settle into our first home together.', '@Aaron-Berkhoff'),
  ('b2b2b2b2-2222-4222-8222-222222222222', 'Honeymoon Fund', 'Help send us on our honeymoon adventure.',      '@Aaron-Berkhoff');
