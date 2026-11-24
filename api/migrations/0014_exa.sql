-- Exa-backed features: the research radar, ask-my-portfolio, site search, and the
-- workout research notes. Every Exa call goes through handlers/exa.rs, which uses
-- these tables for its response cache, per-address rate limit, and daily spending cap.

-- Cached Exa answers, keyed by feature and normalized query.
CREATE TABLE IF NOT EXISTS exa_cache (
  key     TEXT PRIMARY KEY,
  body    TEXT NOT NULL,
  expires INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS exa_cache_expires ON exa_cache (expires);

-- Hourly call counters per feature and client address.
CREATE TABLE IF NOT EXISTS exa_rate (
  key    TEXT PRIMARY KEY,
  window INTEGER NOT NULL,
  count  INTEGER NOT NULL
);

-- What Exa reported spending each UTC day, in millionths of a dollar.
CREATE TABLE IF NOT EXISTS exa_spend (
  day    TEXT PRIMARY KEY,
  micros INTEGER NOT NULL,
  calls  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS exa_meta (
  key   TEXT PRIMARY KEY,
  value INTEGER NOT NULL
);

-- Papers the research radar found, one row per URL.
CREATE TABLE IF NOT EXISTS radar_items (
  url       TEXT PRIMARY KEY,
  topic     TEXT NOT NULL,
  title     TEXT NOT NULL,
  authors   TEXT NOT NULL,
  published TEXT NOT NULL,
  summary   TEXT NOT NULL,
  found     INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS radar_items_published ON radar_items (published);
