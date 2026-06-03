-- Switch PDF storage from inline base64 to R2 object keys.
-- Safe to recreate: no resume versions have been saved yet.
DROP TABLE IF EXISTS resume_versions;

CREATE TABLE resume_versions (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  version      TEXT,
  notes        TEXT,
  typst_source TEXT NOT NULL,
  pdf_key      TEXT NOT NULL,
  created      TEXT NOT NULL,
  updated      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_resume_versions_created
  ON resume_versions (created DESC);
