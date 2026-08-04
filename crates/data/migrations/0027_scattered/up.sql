-- Batch 7 (收尾) of schema alignment: status cards, summary slots,
-- smoke runs, feedback exports/votes, finance events, and document
-- annotations (upstream status_cards.ts + summary_slots.ts + smoke_lab.ts
-- + feedback_exports.ts + feedback_votes.ts + finance_events.ts +
-- document_annotation_*.ts).

-- finance_events needs a company-scoped composite FK target on cost_events.
CREATE UNIQUE INDEX cost_events_company_id_uq ON cost_events (company_id, id);

CREATE TABLE status_cards (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  created_by_user_id     TEXT,
  created_by_agent_id    TEXT,
  title                  TEXT,
  title_pinned           INTEGER NOT NULL DEFAULT 0,
  interest_prompt        TEXT NOT NULL,
  queries                TEXT NOT NULL DEFAULT '[]',
  query_version          INTEGER NOT NULL DEFAULT 0,
  query_compiled_at      TEXT,
  query_compiled_by_agent_id TEXT,
  agent_id               TEXT,
  refresh_policy         TEXT NOT NULL,
  state                  TEXT NOT NULL DEFAULT 'compiling'
                         CHECK (state IN ('compiling', 'active', 'error', 'paused_budget', 'paused_hours')),
  pending_change_count   INTEGER NOT NULL DEFAULT 0,
  pending_change_hash    TEXT,
  last_change_at         TEXT,
  fingerprint            TEXT,
  fingerprint_at         TEXT,
  mentioned_issue_ids    TEXT NOT NULL DEFAULT '[]',
  document_id            TEXT,
  last_update_run_kind   TEXT CHECK (last_update_run_kind IN ('full', 'incremental')),
  last_generated_at      TEXT,
  last_model             TEXT,
  generating_issue_id    TEXT,
  failure_reason         TEXT,
  next_eval_at           TEXT,
  archived_at            TEXT,
  archived_by_user_id    TEXT,
  archived_by_agent_id   TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, query_compiled_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, generating_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, archived_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_status_cards_company_archived ON status_cards (company_id, archived_at);
CREATE INDEX idx_status_cards_company_next_eval ON status_cards (company_id, next_eval_at);

CREATE TABLE status_card_updates (
  id                  TEXT PRIMARY KEY,
  card_id             TEXT NOT NULL REFERENCES status_cards(id) ON DELETE CASCADE,
  kind                TEXT NOT NULL CHECK (kind IN ('compile', 'full', 'incremental')),
  trigger             TEXT NOT NULL CHECK (trigger IN ('manual', 'interval', 'reactive', 'restore')),
  generation_issue_id TEXT REFERENCES issues(id) ON DELETE SET NULL,
  run_id              TEXT REFERENCES heartbeat_runs(id) ON DELETE SET NULL,
  changes             TEXT NOT NULL DEFAULT '[]',
  input_tokens        INTEGER NOT NULL DEFAULT 0,
  output_tokens       INTEGER NOT NULL DEFAULT 0,
  cost_cents          INTEGER NOT NULL DEFAULT 0,
  model               TEXT,
  query_version       INTEGER,
  change_summary      TEXT,
  started_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  finished_at         TEXT,
  status              TEXT NOT NULL CHECK (status IN ('running', 'ok', 'failed')),
  error               TEXT
);

CREATE INDEX idx_status_card_updates_card_started
  ON status_card_updates (card_id, started_at);
CREATE INDEX idx_status_card_updates_generation_issue
  ON status_card_updates (generation_issue_id);

CREATE TABLE summary_slots (
  id                      TEXT PRIMARY KEY,
  company_id              TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  scope_kind              TEXT NOT NULL,
  scope_id                TEXT,
  slot_key                TEXT NOT NULL,
  document_id             TEXT,
  status                  TEXT NOT NULL DEFAULT 'idle',
  failure_reason          TEXT,
  generating_issue_id     TEXT,
  last_generated_at       TEXT,
  last_generated_by_agent_id TEXT,
  last_model              TEXT,
  created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, generating_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, last_generated_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX summary_slots_company_scope_slot_uq
  ON summary_slots (company_id, scope_kind, COALESCE(scope_id, ''), slot_key);
CREATE UNIQUE INDEX summary_slots_document_uq ON summary_slots (document_id);
CREATE INDEX idx_summary_slots_company_scope
  ON summary_slots (company_id, scope_kind, scope_id);
CREATE INDEX idx_summary_slots_company_generating_issue
  ON summary_slots (company_id, generating_issue_id);
CREATE INDEX idx_summary_slots_company_updated
  ON summary_slots (company_id, updated_at);

CREATE TABLE smoke_runs (
  id          TEXT PRIMARY KEY,
  company_id  TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  trigger     TEXT NOT NULL,
  status      TEXT NOT NULL DEFAULT 'running',
  started_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  finished_at TEXT,
  summary     TEXT NOT NULL DEFAULT '{}',
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_smoke_runs_company_started ON smoke_runs (company_id, started_at);
CREATE INDEX idx_smoke_runs_company_status ON smoke_runs (company_id, status);

CREATE TABLE smoke_run_steps (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  run_id                 TEXT NOT NULL,
  path                   TEXT NOT NULL,
  scenario_step          TEXT NOT NULL,
  status                 TEXT NOT NULL,
  detail                 TEXT,
  screenshot_artifact_ref TEXT,
  duration_ms            INTEGER,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, run_id) REFERENCES smoke_runs (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_smoke_run_steps_company_run ON smoke_run_steps (company_id, run_id);
CREATE INDEX idx_smoke_run_steps_company_path ON smoke_run_steps (company_id, path);

CREATE TABLE feedback_votes (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  issue_id         TEXT NOT NULL,
  target_type      TEXT NOT NULL,
  target_id        TEXT NOT NULL,
  author_user_id   TEXT NOT NULL,
  vote             TEXT NOT NULL,
  reason           TEXT,
  shared_with_labs INTEGER NOT NULL DEFAULT 0,
  shared_at        TEXT,
  consent_version  TEXT,
  redaction_summary TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, target_type, target_id, author_user_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id)
);

CREATE INDEX idx_feedback_votes_company_issue ON feedback_votes (company_id, issue_id);
CREATE INDEX idx_feedback_votes_issue_target
  ON feedback_votes (issue_id, target_type, target_id);
CREATE INDEX idx_feedback_votes_author ON feedback_votes (author_user_id, created_at);

CREATE TABLE feedback_exports (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  feedback_vote_id TEXT NOT NULL,
  issue_id         TEXT NOT NULL,
  project_id       TEXT,
  author_user_id   TEXT NOT NULL,
  target_type      TEXT NOT NULL,
  target_id        TEXT NOT NULL,
  vote             TEXT NOT NULL,
  status           TEXT NOT NULL DEFAULT 'local_only',
  destination      TEXT,
  export_id        TEXT,
  consent_version  TEXT,
  schema_version   TEXT NOT NULL DEFAULT 'paperclip-feedback-envelope-v2',
  bundle_version   TEXT NOT NULL DEFAULT 'paperclip-feedback-bundle-v2',
  payload_version  TEXT NOT NULL DEFAULT 'paperclip-feedback-v1',
  payload_digest   TEXT,
  payload_snapshot TEXT,
  target_summary   TEXT NOT NULL,
  redaction_summary TEXT,
  attempt_count    INTEGER NOT NULL DEFAULT 0,
  last_attempted_at TEXT,
  exported_at      TEXT,
  failure_reason   TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (feedback_vote_id),
  FOREIGN KEY (company_id, feedback_vote_id) REFERENCES feedback_votes (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_feedback_exports_company_created
  ON feedback_exports (company_id, created_at);
CREATE INDEX idx_feedback_exports_company_status
  ON feedback_exports (company_id, status, created_at);
CREATE INDEX idx_feedback_exports_company_issue
  ON feedback_exports (company_id, issue_id, created_at);
CREATE INDEX idx_feedback_exports_company_project
  ON feedback_exports (company_id, project_id, created_at);
CREATE INDEX idx_feedback_exports_company_author
  ON feedback_exports (company_id, author_user_id, created_at);

CREATE TABLE finance_events (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  agent_id               TEXT,
  issue_id               TEXT,
  project_id             TEXT,
  goal_id                TEXT,
  heartbeat_run_id       TEXT,
  cost_event_id          TEXT,
  billing_code           TEXT,
  description            TEXT,
  event_kind             TEXT NOT NULL,
  direction              TEXT NOT NULL DEFAULT 'debit',
  biller                 TEXT NOT NULL,
  provider               TEXT,
  execution_adapter_type TEXT,
  pricing_tier           TEXT,
  region                 TEXT,
  model                  TEXT,
  quantity               INTEGER,
  unit                   TEXT,
  amount_cents           INTEGER NOT NULL,
  currency               TEXT NOT NULL DEFAULT 'USD',
  estimated              INTEGER NOT NULL DEFAULT 0,
  external_invoice_id    TEXT,
  metadata_json          TEXT,
  occurred_at            TEXT NOT NULL,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, goal_id) REFERENCES goals (company_id, id),
  FOREIGN KEY (company_id, heartbeat_run_id) REFERENCES heartbeat_runs (company_id, id),
  FOREIGN KEY (company_id, cost_event_id) REFERENCES cost_events (company_id, id)
);

CREATE INDEX idx_finance_events_company_occurred
  ON finance_events (company_id, occurred_at);
CREATE INDEX idx_finance_events_company_biller_occurred
  ON finance_events (company_id, biller, occurred_at);
CREATE INDEX idx_finance_events_company_kind_occurred
  ON finance_events (company_id, event_kind, occurred_at);
CREATE INDEX idx_finance_events_company_direction_occurred
  ON finance_events (company_id, direction, occurred_at);
CREATE INDEX idx_finance_events_company_heartbeat_run
  ON finance_events (company_id, heartbeat_run_id);
CREATE INDEX idx_finance_events_company_cost_event
  ON finance_events (company_id, cost_event_id);

CREATE TABLE document_annotation_threads (
  id                       TEXT PRIMARY KEY,
  company_id               TEXT NOT NULL REFERENCES companies(id),
  issue_id                 TEXT,
  routine_id               TEXT,
  case_id                  TEXT,
  document_id              TEXT NOT NULL,
  document_key             TEXT NOT NULL,
  status                   TEXT NOT NULL DEFAULT 'open',
  anchor_state             TEXT NOT NULL DEFAULT 'active',
  original_revision_id     TEXT REFERENCES document_revisions(id) ON DELETE SET NULL,
  original_revision_number INTEGER NOT NULL,
  current_revision_id      TEXT REFERENCES document_revisions(id) ON DELETE SET NULL,
  current_revision_number  INTEGER NOT NULL,
  selected_text            TEXT NOT NULL,
  prefix_text              TEXT NOT NULL DEFAULT '',
  suffix_text              TEXT NOT NULL DEFAULT '',
  normalized_start         INTEGER NOT NULL,
  normalized_end           INTEGER NOT NULL,
  markdown_start           INTEGER NOT NULL,
  markdown_end             INTEGER NOT NULL,
  anchor_confidence        TEXT NOT NULL DEFAULT 'exact',
  anchor_selector          TEXT NOT NULL,
  created_by_agent_id      TEXT,
  created_by_user_id       TEXT,
  resolved_by_agent_id     TEXT,
  resolved_by_user_id      TEXT,
  resolved_at              TEXT,
  created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  CHECK ((issue_id IS NOT NULL) + (routine_id IS NOT NULL) + (case_id IS NOT NULL) = 1),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, routine_id) REFERENCES routines (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, case_id) REFERENCES cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, resolved_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_doc_annotation_threads_company_document_status
  ON document_annotation_threads (company_id, document_id, status);
CREATE INDEX idx_doc_annotation_threads_company_issue_status
  ON document_annotation_threads (company_id, issue_id, status);
CREATE INDEX idx_doc_annotation_threads_company_routine_status
  ON document_annotation_threads (company_id, routine_id, status);
CREATE INDEX idx_doc_annotation_threads_company_case_status
  ON document_annotation_threads (company_id, case_id, status);
CREATE INDEX idx_doc_annotation_threads_company_current_revision_open
  ON document_annotation_threads (company_id, document_id, current_revision_id, status);
CREATE INDEX idx_doc_annotation_threads_company_anchor_state
  ON document_annotation_threads (company_id, anchor_state);

CREATE TABLE document_annotation_comments (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  thread_id        TEXT NOT NULL,
  issue_id         TEXT,
  routine_id       TEXT,
  case_id          TEXT,
  document_id      TEXT NOT NULL,
  body             TEXT NOT NULL,
  author_type      TEXT NOT NULL,
  author_agent_id  TEXT,
  author_user_id   TEXT,
  created_by_run_id TEXT,
  issue_comment_id TEXT REFERENCES issue_comments(id) ON DELETE SET NULL,
  source_trust     TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  CHECK ((issue_id IS NOT NULL) + (routine_id IS NOT NULL) + (case_id IS NOT NULL) = 1),
  FOREIGN KEY (company_id, thread_id) REFERENCES document_annotation_threads (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, routine_id) REFERENCES routines (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, case_id) REFERENCES cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, author_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_doc_annotation_comments_company_thread_created
  ON document_annotation_comments (company_id, thread_id, created_at);
CREATE INDEX idx_doc_annotation_comments_company_issue_created
  ON document_annotation_comments (company_id, issue_id, created_at);
CREATE INDEX idx_doc_annotation_comments_company_routine_created
  ON document_annotation_comments (company_id, routine_id, created_at);
CREATE INDEX idx_doc_annotation_comments_company_case_created
  ON document_annotation_comments (company_id, case_id, created_at);
CREATE INDEX idx_doc_annotation_comments_company_document_created
  ON document_annotation_comments (company_id, document_id, created_at);
CREATE INDEX idx_doc_annotation_comments_issue_comment
  ON document_annotation_comments (issue_comment_id);

CREATE TABLE document_annotation_anchor_snapshots (
  id                TEXT PRIMARY KEY,
  company_id        TEXT NOT NULL REFERENCES companies(id),
  thread_id         TEXT NOT NULL,
  document_id       TEXT NOT NULL,
  from_revision_id  TEXT REFERENCES document_revisions(id) ON DELETE SET NULL,
  from_revision_number INTEGER,
  to_revision_id    TEXT REFERENCES document_revisions(id) ON DELETE SET NULL,
  to_revision_number INTEGER NOT NULL,
  previous_anchor   TEXT NOT NULL,
  next_anchor       TEXT,
  anchor_state      TEXT NOT NULL,
  anchor_confidence TEXT NOT NULL,
  failure_reason    TEXT,
  created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, thread_id) REFERENCES document_annotation_threads (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_doc_annotation_snapshots_company_thread_created
  ON document_annotation_anchor_snapshots (company_id, thread_id, created_at);
CREATE INDEX idx_doc_annotation_snapshots_company_document_revision
  ON document_annotation_anchor_snapshots (company_id, document_id, to_revision_number);
