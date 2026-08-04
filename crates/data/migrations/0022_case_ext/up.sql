-- Batch 2 of schema alignment: case attachment tables (upstream cases.ts)
-- and the external-objects catalog + mentions (upstream external_objects.ts).

CREATE TABLE case_issue_links (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  case_id          TEXT NOT NULL,
  issue_id         TEXT NOT NULL,
  role             TEXT NOT NULL CHECK (role IN ('origin', 'work', 'reference')),
  created_by_run_id TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, case_id, issue_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_case_issue_links_case ON case_issue_links (company_id, case_id);
CREATE INDEX idx_case_issue_links_issue ON case_issue_links (issue_id);

CREATE TABLE case_events (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  case_id       TEXT NOT NULL,
  kind          TEXT NOT NULL
                CHECK (kind IN ('created', 'updated', 'fields_changed', 'status_changed',
                                'issue_linked', 'issue_unlinked', 'document_revised',
                                'child_linked', 'attachment_added', 'label_added',
                                'label_removed')),
  actor_type    TEXT NOT NULL CHECK (actor_type IN ('user', 'agent', 'system')),
  actor_user_id TEXT,
  actor_agent_id TEXT,
  run_id        TEXT,
  payload       TEXT NOT NULL DEFAULT '{}',
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, actor_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_case_events_case ON case_events (company_id, case_id, created_at);

CREATE TABLE case_documents (
  id          TEXT PRIMARY KEY,
  company_id  TEXT NOT NULL REFERENCES companies(id),
  case_id     TEXT NOT NULL,
  document_id TEXT NOT NULL,
  key         TEXT NOT NULL,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, case_id, key),
  UNIQUE (company_id, document_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_case_documents_case ON case_documents (company_id, case_id, updated_at);

CREATE TABLE case_labels (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id),
  case_id    TEXT NOT NULL,
  label_id   TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, case_id, label_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, label_id) REFERENCES labels (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_case_labels_case ON case_labels (company_id, case_id);
CREATE INDEX idx_case_labels_label ON case_labels (label_id);

CREATE TABLE case_attachments (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id),
  case_id    TEXT NOT NULL,
  asset_id   TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, asset_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, asset_id) REFERENCES assets (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_case_attachments_case ON case_attachments (company_id, case_id);

-- External object catalog (upstream external_objects.ts).
CREATE TABLE external_objects (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  provider_key           TEXT NOT NULL,
  plugin_id              TEXT,
  object_type            TEXT NOT NULL,
  external_id            TEXT NOT NULL,
  sanitized_canonical_url TEXT,
  canonical_identity_hash TEXT,
  display_key            TEXT,
  icon_key               TEXT,
  display_title          TEXT,
  status_key             TEXT,
  status_label           TEXT,
  status_icon_key        TEXT,
  status_category        TEXT NOT NULL DEFAULT 'unknown',
  status_tone            TEXT NOT NULL DEFAULT 'neutral',
  liveness               TEXT NOT NULL DEFAULT 'unknown',
  is_terminal            INTEGER NOT NULL DEFAULT 0,
  data                   TEXT NOT NULL DEFAULT '{}',
  remote_version         TEXT,
  etag                   TEXT,
  last_resolved_at       TEXT,
  last_changed_at        TEXT,
  last_error_at          TEXT,
  next_refresh_at        TEXT,
  refresh_started_at     TEXT,
  refresh_token          TEXT,
  last_error_code        TEXT,
  last_error_message     TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, provider_key, object_type, external_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (plugin_id) REFERENCES plugins (id) ON DELETE SET NULL
);

CREATE INDEX idx_external_objects_company_provider_object
  ON external_objects (company_id, provider_key, object_type);
CREATE INDEX idx_external_objects_company_provider_status
  ON external_objects (company_id, provider_key, status_category);
CREATE INDEX idx_external_objects_company_refresh
  ON external_objects (company_id, next_refresh_at);
CREATE UNIQUE INDEX external_objects_company_identity_uq
  ON external_objects (company_id, provider_key, object_type, canonical_identity_hash);

-- External object mentions (upstream external_object_mentions.ts).
CREATE TABLE external_object_mentions (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  source_issue_id        TEXT NOT NULL,
  source_kind            TEXT NOT NULL,
  source_record_id       TEXT,
  document_key           TEXT,
  property_key           TEXT,
  matched_text_redacted  TEXT,
  sanitized_display_url  TEXT,
  canonical_identity_hash TEXT,
  canonical_identity     TEXT,
  object_id              TEXT,
  provider_key           TEXT,
  detector_key           TEXT,
  object_type            TEXT,
  confidence             TEXT NOT NULL DEFAULT 'exact',
  created_by_plugin_id   TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, source_issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, object_id) REFERENCES external_objects (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (created_by_plugin_id) REFERENCES plugins (id) ON DELETE SET NULL
);

CREATE INDEX idx_external_object_mentions_source
  ON external_object_mentions (company_id, source_issue_id);
CREATE INDEX idx_external_object_mentions_object
  ON external_object_mentions (company_id, object_id);
CREATE INDEX idx_external_object_mentions_provider
  ON external_object_mentions (company_id, provider_key, object_type);

-- Upstream partial unique indexes for mentions (dedupe by source record).
CREATE UNIQUE INDEX external_object_mentions_company_source_record_uq
  ON external_object_mentions (company_id, source_issue_id, source_kind,
                               source_record_id, document_key, property_key,
                               canonical_identity_hash)
  WHERE source_record_id IS NOT NULL AND canonical_identity_hash IS NOT NULL;

CREATE UNIQUE INDEX external_object_mentions_company_source_null_record_uq
  ON external_object_mentions (company_id, source_issue_id, source_kind,
                               document_key, property_key, canonical_identity_hash)
  WHERE source_record_id IS NULL AND canonical_identity_hash IS NOT NULL;

