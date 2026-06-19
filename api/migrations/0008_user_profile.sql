-- Per-user body profile for the fitness metrics (BMI / BMR / TDEE / targets).
-- One row per user; the workout and meals pages both read/write it via /api/profile.
CREATE TABLE IF NOT EXISTS user_profile (
  owner_id   INTEGER PRIMARY KEY REFERENCES game_users(id),
  height_cm  INTEGER NOT NULL DEFAULT 179,
  sex        TEXT    NOT NULL DEFAULT 'male',          -- 'male' | 'female'
  age_years  INTEGER NOT NULL DEFAULT 28,
  activity   REAL    NOT NULL DEFAULT 1.55,            -- TDEE multiplier over BMR
  goal       TEXT    NOT NULL DEFAULT 'cut_moderate',  -- cut_moderate | cut_aggressive | maintain
  updated    TEXT    NOT NULL DEFAULT (datetime('now'))
);
