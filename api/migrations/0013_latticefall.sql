-- Cinderheart (latticefall) asynchronous features, cloud saves, Marks, the speedrun
-- leaderboard, and opt-in telemetry. See docs/game/06-progression-and-systems.md.
-- Timestamps are Unix seconds so the 90-day cleanup is a plain integer comparison.

-- Messages: template and word IDs only, so there is no free text to moderate.
CREATE TABLE IF NOT EXISTS lf_messages (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  area        TEXT NOT NULL,
  x           REAL NOT NULL,
  y           REAL NOT NULL,
  z           REAL NOT NULL,
  template_id INTEGER NOT NULL,
  word_ids    TEXT NOT NULL,           -- JSON array of integers
  rating      INTEGER NOT NULL DEFAULT 0,
  anon_id     TEXT NOT NULL,
  user_id     INTEGER NOT NULL REFERENCES game_users(id),
  created     INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lf_messages_area ON lf_messages (area, rating DESC);
CREATE INDEX IF NOT EXISTS idx_lf_messages_created ON lf_messages (created);

CREATE TABLE IF NOT EXISTS lf_message_votes (
  message_id INTEGER NOT NULL REFERENCES lf_messages(id) ON DELETE CASCADE,
  user_id    INTEGER NOT NULL REFERENCES game_users(id),
  value      INTEGER NOT NULL,         -- 1 or -1
  PRIMARY KEY (message_id, user_id)
);

-- Afterimages: a short snippet of a player's last moments before a death.
CREATE TABLE IF NOT EXISTS lf_afterimages (
  id      INTEGER PRIMARY KEY AUTOINCREMENT,
  area    TEXT NOT NULL,
  x       REAL NOT NULL,
  y       REAL NOT NULL,
  z       REAL NOT NULL,
  cause   TEXT NOT NULL,
  snippet TEXT NOT NULL,               -- JSON array of [x, y, z, moveId], at most 150 entries
  anon_id TEXT NOT NULL,
  user_id INTEGER NOT NULL REFERENCES game_users(id),
  created INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lf_afterimages_area ON lf_afterimages (area);
CREATE INDEX IF NOT EXISTS idx_lf_afterimages_created ON lf_afterimages (created);

-- Wanderers: a short trail of another player walking through an area.
CREATE TABLE IF NOT EXISTS lf_wanderers (
  id      INTEGER PRIMARY KEY AUTOINCREMENT,
  area    TEXT NOT NULL,
  trail   TEXT NOT NULL,               -- JSON array of [x, y, z, moveId], at most 120 entries
  anon_id TEXT NOT NULL,
  user_id INTEGER NOT NULL REFERENCES game_users(id),
  created INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lf_wanderers_area ON lf_wanderers (area);
CREATE INDEX IF NOT EXISTS idx_lf_wanderers_created ON lf_wanderers (created);

-- Cloud save: one blob per user (the latest save wins). Kept until the player deletes it.
CREATE TABLE IF NOT EXISTS lf_saves (
  user_id  INTEGER PRIMARY KEY REFERENCES game_users(id),
  data     TEXT NOT NULL,              -- base64 of the postcard blob
  scenario TEXT NOT NULL,
  seed     TEXT NOT NULL,              -- decimal string (u64)
  hash     TEXT NOT NULL,
  saved_at INTEGER NOT NULL,           -- milliseconds, from the client
  updated  INTEGER NOT NULL
);

-- Marks (achievements) earned by a signed-in player. Sync merges as a union.
CREATE TABLE IF NOT EXISTS lf_marks (
  user_id   INTEGER NOT NULL REFERENCES game_users(id),
  mark_id   TEXT NOT NULL,
  earned_at INTEGER NOT NULL,          -- milliseconds, from the client
  PRIMARY KEY (user_id, mark_id)
);

-- Speedrun leaderboard: each player's best B12 clear, in in-game milliseconds.
CREATE TABLE IF NOT EXISTS lf_speedruns (
  user_id      INTEGER PRIMARY KEY REFERENCES game_users(id),
  best_ms      INTEGER NOT NULL,
  content_hash TEXT NOT NULL,
  updated      INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lf_speedruns_best ON lf_speedruns (best_ms);

-- Telemetry: one row per batch, anonymous, retained 90 days.
CREATE TABLE IF NOT EXISTS lf_telemetry (
  id       INTEGER PRIMARY KEY AUTOINCREMENT,
  anon_id  TEXT NOT NULL,
  session  TEXT NOT NULL,
  n_events INTEGER NOT NULL,
  events   TEXT NOT NULL,              -- JSON array
  created  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lf_telemetry_created ON lf_telemetry (created);

-- Fixed-window write rate limits (hourly) and the lazy cleanup clock.
CREATE TABLE IF NOT EXISTS lf_rate (
  key    TEXT PRIMARY KEY,
  window INTEGER NOT NULL,
  count  INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS lf_meta (
  key   TEXT PRIMARY KEY,
  value INTEGER NOT NULL
);
