-- A session minted from a Google ID token. The ID token itself dies after
-- about an hour, and a phone with the screen off often misses the timer that
-- would have refreshed it, so a long gym session gets signed out. This row
-- is what the Worker accepts instead, and it does not call Google on each
-- request.
CREATE TABLE IF NOT EXISTS auth_sessions (
  token      TEXT PRIMARY KEY,
  google_sub TEXT NOT NULL,
  email      TEXT NOT NULL DEFAULT '',
  name       TEXT NOT NULL DEFAULT '',
  picture    TEXT NOT NULL DEFAULT '',
  expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_auth_sessions_expires
  ON auth_sessions (expires_at);
