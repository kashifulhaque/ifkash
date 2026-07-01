-- Cardio logging per workout session. Each session can carry any number of
-- cardio bouts (cycle, crosstrainer, treadmill …), each with a duration and the
-- calories burnt. kcal is the value that matters; minutes is context. Rows are
-- replaced wholesale on every session upsert, mirroring workout_sets.
CREATE TABLE IF NOT EXISTS workout_cardio (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  INTEGER NOT NULL REFERENCES workout_sessions(id),
  kind        TEXT NOT NULL DEFAULT '',    -- Cycle|Crosstrainer|Treadmill|… (free text ok)
  minutes     INTEGER NOT NULL DEFAULT 0,  -- duration in minutes
  kcal        INTEGER NOT NULL DEFAULT 0,  -- calories burnt (the point of the log)
  entry_index INTEGER NOT NULL,            -- 1-based order within the session
  created     TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_workout_cardio_session ON workout_cardio (session_id);
