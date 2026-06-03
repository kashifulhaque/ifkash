-- Resume version history. Replaces the PocketBase `resumes` collection.
-- The rendered PDF is stored inline as base64 text (resume PDFs are small,
-- ~150 KB); swap to R2 later if files grow large.
CREATE TABLE IF NOT EXISTS resume_versions (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  version      TEXT,
  notes        TEXT,
  typst_source TEXT NOT NULL,
  pdf_base64   TEXT NOT NULL,
  created      TEXT NOT NULL,
  updated      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_resume_versions_created
  ON resume_versions (created DESC);
