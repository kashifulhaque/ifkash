-- Expense splitter: groups owned by a Google-authenticated bookkeeper, with
-- plain named members (friends do NOT log in), expenses, and resolved per-member
-- owed shares. All money is stored as INTEGER cents to avoid float drift.
CREATE TABLE IF NOT EXISTS split_groups (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  owner_id  INTEGER NOT NULL REFERENCES game_users(id),
  name      TEXT NOT NULL,
  currency  TEXT NOT NULL DEFAULT 'INR',
  created   TEXT NOT NULL DEFAULT (datetime('now')),
  updated   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_split_groups_owner ON split_groups (owner_id);

CREATE TABLE IF NOT EXISTS split_members (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  group_id  INTEGER NOT NULL REFERENCES split_groups(id),
  name      TEXT NOT NULL,
  created   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_split_members_group ON split_members (group_id);

CREATE TABLE IF NOT EXISTS split_expenses (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  group_id     INTEGER NOT NULL REFERENCES split_groups(id),
  payer_id     INTEGER NOT NULL REFERENCES split_members(id),
  description  TEXT NOT NULL DEFAULT '',
  amount_cents INTEGER NOT NULL,
  split_type   TEXT NOT NULL DEFAULT 'equal', -- equal|exact|percent|shares
  created      TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_split_expenses_group ON split_expenses (group_id);

-- Resolved owed amount per member per expense (in cents, normalized so the
-- rows for an expense sum to its amount_cents).
CREATE TABLE IF NOT EXISTS split_shares (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  expense_id  INTEGER NOT NULL REFERENCES split_expenses(id),
  member_id   INTEGER NOT NULL REFERENCES split_members(id),
  owed_cents  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_split_shares_expense ON split_shares (expense_id);
