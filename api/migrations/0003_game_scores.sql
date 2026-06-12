-- Game leaderboard: Google-authenticated players and their best scores.
CREATE TABLE IF NOT EXISTS game_users (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  google_sub  TEXT NOT NULL UNIQUE,
  email       TEXT NOT NULL,
  name        TEXT NOT NULL,
  picture     TEXT,
  created     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS game_scores (
  user_id      INTEGER PRIMARY KEY REFERENCES game_users(id),
  best_score   INTEGER NOT NULL DEFAULT 0,
  games_played INTEGER NOT NULL DEFAULT 0,
  updated      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_game_scores_best ON game_scores (best_score DESC);
