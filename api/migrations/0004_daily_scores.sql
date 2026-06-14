-- Daily-challenge leaderboard: best score per player per day. Kept separate
-- from the all-time game_scores table so the two boards don't intermingle.
CREATE TABLE IF NOT EXISTS daily_scores (
  user_id INTEGER NOT NULL REFERENCES game_users(id),
  day     TEXT NOT NULL,        -- YYYY-MM-DD
  score   INTEGER NOT NULL,
  updated TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (user_id, day)
);

CREATE INDEX IF NOT EXISTS idx_daily_scores_day_best ON daily_scores (day, score DESC);
