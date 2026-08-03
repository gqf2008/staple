-- Decision desk: durable queues, queue membership, and triage state.
-- Mirrors the upstream sidecar model (SPEC §7.16).

CREATE TABLE decision_queues (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  name             TEXT NOT NULL,
  description      TEXT,
  retention_days   INTEGER,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, name)
);

CREATE TABLE decision_queue_items (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  queue_id         TEXT NOT NULL,
  source_kind      TEXT NOT NULL,
  source_id        TEXT NOT NULL,
  payload          TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, queue_id, source_kind, source_id),
  FOREIGN KEY (company_id, queue_id) REFERENCES decision_queues (company_id, id)
);

CREATE TABLE decision_triage (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  source_kind      TEXT NOT NULL,
  source_id        TEXT NOT NULL,
  decide_by        TEXT,
  snoozed_until    TEXT,
  decision         TEXT,
  decided_by_user_id TEXT,
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, source_kind, source_id)
);
