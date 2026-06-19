-- Make workout sessions and bodyweight entries idempotent per day, so the merged
-- workout page can auto-save on every keystroke (upsert) instead of appending a
-- new row each time. One session per (owner, date, day_label); one bodyweight
-- entry per (owner, date).
CREATE UNIQUE INDEX IF NOT EXISTS idx_workout_sessions_owner_date_day
  ON workout_sessions (owner_id, date, day_label);

CREATE UNIQUE INDEX IF NOT EXISTS idx_bodyweight_owner_date
  ON bodyweight_logs (owner_id, date);
