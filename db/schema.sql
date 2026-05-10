PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS breweries (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  notes TEXT,
  website TEXT,
  address TEXT,
  lat REAL,
  lng REAL,
  drink_menu TEXT,
  food_schedule TEXT,
  created_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS vendors (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  normalized_name TEXT,
  notes TEXT,
  website TEXT,
  menu TEXT,
  needs_review INTEGER NOT NULL DEFAULT 0,
  match_method TEXT,
  match_score REAL,
  created_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS schedule_entries (
  id TEXT PRIMARY KEY,
  brewery_id TEXT NOT NULL REFERENCES breweries(id),
  vendor_id TEXT NOT NULL REFERENCES vendors(id),
  start_at TEXT NOT NULL,
  end_at TEXT NOT NULL,
  source TEXT NOT NULL,
  source_url TEXT,
  scrape_run_id TEXT,
  is_active INTEGER NOT NULL DEFAULT 1,
  superseded_at TEXT,
  scraped_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS vendors_normalized_name_idx ON vendors(normalized_name);
CREATE UNIQUE INDEX IF NOT EXISTS schedule_entries_source_identity_idx
  ON schedule_entries(source, brewery_id, vendor_id, start_at, end_at);
CREATE INDEX IF NOT EXISTS schedule_entries_time_idx ON schedule_entries(start_at, end_at);
CREATE INDEX IF NOT EXISTS schedule_entries_brewery_idx ON schedule_entries(brewery_id);
CREATE INDEX IF NOT EXISTS schedule_entries_vendor_idx ON schedule_entries(vendor_id);
CREATE INDEX IF NOT EXISTS schedule_entries_active_idx ON schedule_entries(source, brewery_id, is_active);
