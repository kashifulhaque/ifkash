CREATE TABLE IF NOT EXISTS edge_signals (
  colo TEXT PRIMARY KEY CHECK (length(colo) = 3),
  signals INTEGER NOT NULL DEFAULT 0 CHECK (signals >= 0),
  updated TEXT NOT NULL DEFAULT (datetime('now'))
);
