-- Meal tracker: one row per logged meal, scoped to a game_users owner.
-- Macros are estimates produced by a vision model and are user-editable.
CREATE TABLE meals (
  id           INTEGER PRIMARY KEY,
  owner_id     INTEGER NOT NULL,            -- references game_users.id
  eaten_on     TEXT NOT NULL,               -- 'YYYY-MM-DD' local day, sent by client
  eaten_at     TEXT DEFAULT (datetime('now')),
  description  TEXT,                         -- model's food summary (editable)
  calories     REAL,
  protein_g    REAL,
  carbs_g      REAL,
  fat_g        REAL,
  items_json   TEXT,                         -- raw per-item breakdown from the model
  photo_r2_key TEXT,                         -- null if photo discarded
  created      TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_meals_owner_day ON meals (owner_id, eaten_on);
