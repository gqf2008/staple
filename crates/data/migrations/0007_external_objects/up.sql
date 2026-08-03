-- External object links: stable associations with refreshable status.

CREATE TABLE issue_external_objects (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  issue_id         TEXT NOT NULL,
  kind             TEXT NOT NULL,
  external_id      TEXT NOT NULL,
  url              TEXT,
  status           TEXT NOT NULL DEFAULT 'pending',
  last_synced_at   TEXT,
  metadata         TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, issue_id, kind, external_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id)
);
