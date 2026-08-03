-- Decision desk completion: immutable triage events, retention (keep marker +
-- archive version), and the archive-notification outbox with dedupe
-- (SPEC §7.16 decision-desk).

-- Parent-table composite uniqueness needed by the new composite FKs below.
CREATE UNIQUE INDEX idx_decision_triage_company_id ON decision_triage (company_id, id);

CREATE TABLE decision_triage_events (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  triage_id           TEXT NOT NULL,
  event_type          TEXT NOT NULL
                      CHECK (event_type IN ('decided', 'snoozed', 'kept', 'archived', 'restored')),
  decision            TEXT,
  decided_by_user_id  TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, triage_id) REFERENCES decision_triage (company_id, id)
);

CREATE INDEX idx_decision_triage_events_company_triage
  ON decision_triage_events (company_id, triage_id, created_at);

CREATE TABLE decision_retention (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  triage_id     TEXT NOT NULL,
  source_kind   TEXT NOT NULL,
  source_id     TEXT NOT NULL,
  keep          INTEGER NOT NULL DEFAULT 0,
  archived      INTEGER NOT NULL DEFAULT 0,
  archived_at   TEXT,
  archived_reason TEXT,
  restored_at   TEXT,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, triage_id),
  UNIQUE (company_id, source_kind, source_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, triage_id) REFERENCES decision_triage (company_id, id)
);

CREATE TABLE decision_archive_notification_outbox (
  id                TEXT PRIMARY KEY,
  company_id        TEXT NOT NULL REFERENCES companies(id),
  triage_id         TEXT NOT NULL,
  notification_kind TEXT NOT NULL DEFAULT 'archive',
  recipient_user_id TEXT,
  status            TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'sent', 'failed')),
  attempt_count     INTEGER NOT NULL DEFAULT 0,
  last_error        TEXT,
  dedupe_key        TEXT NOT NULL,
  created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  sent_at           TEXT,
  UNIQUE (company_id, dedupe_key),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, triage_id) REFERENCES decision_triage (company_id, id)
);

CREATE INDEX idx_decision_outbox_company_status
  ON decision_archive_notification_outbox (company_id, status, created_at);
