-- Workout tracker: training sessions owned by a Google-authenticated user, each
-- with per-exercise sets (reps + weight), plus a separate bodyweight log. Weights
-- are stored as INTEGER grams to avoid float drift (mirrors the splitter's cents).
CREATE TABLE IF NOT EXISTS workout_sessions (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  owner_id  INTEGER NOT NULL REFERENCES game_users(id),
  day_label TEXT NOT NULL DEFAULT '',     -- Push|Pull|Legs|'' (free text ok)
  date      TEXT NOT NULL,                 -- YYYY-MM-DD
  notes     TEXT NOT NULL DEFAULT '',
  created   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_workout_sessions_owner ON workout_sessions (owner_id);

CREATE TABLE IF NOT EXISTS workout_sets (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  INTEGER NOT NULL REFERENCES workout_sessions(id),
  exercise    TEXT NOT NULL,
  set_index   INTEGER NOT NULL,            -- 1-based order within the exercise
  reps        INTEGER NOT NULL DEFAULT 0,
  weight_g    INTEGER NOT NULL DEFAULT 0,  -- grams; 0 = bodyweight
  created     TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_workout_sets_session ON workout_sets (session_id);

CREATE TABLE IF NOT EXISTS bodyweight_logs (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  owner_id  INTEGER NOT NULL REFERENCES game_users(id),
  date      TEXT NOT NULL,                 -- YYYY-MM-DD
  weight_g  INTEGER NOT NULL,             -- grams
  created   TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_bodyweight_owner ON bodyweight_logs (owner_id);
