-- Batch 3 of schema alignment: upstream decision domain
-- (decisions.ts + decision_training_examples.ts).

CREATE TABLE decision_bundles (
  id             TEXT PRIMARY KEY,
  company_id     TEXT NOT NULL REFERENCES companies(id),
  title          TEXT NOT NULL,
  summary        TEXT NOT NULL,
  origin_agent_id TEXT NOT NULL,
  origin_issue_id TEXT NOT NULL,
  origin_run_id  TEXT NOT NULL,
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, origin_agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, origin_issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, origin_run_id) REFERENCES heartbeat_runs (company_id, id)
);

CREATE INDEX idx_decision_bundles_company_created_at
  ON decision_bundles (company_id, created_at);

CREATE TABLE decisions (
  id                TEXT PRIMARY KEY,
  company_id        TEXT NOT NULL REFERENCES companies(id),
  bundle_id         TEXT,
  origin_agent_id   TEXT NOT NULL,
  origin_issue_id   TEXT NOT NULL,
  origin_run_id     TEXT NOT NULL,
  rule_key          TEXT,
  title             TEXT NOT NULL,
  body              TEXT NOT NULL,
  options           TEXT NOT NULL DEFAULT '[]',
  inputs            TEXT,
  status            TEXT NOT NULL DEFAULT 'open',
  execution_status  TEXT,
  chosen_option_id  TEXT,
  input_values      TEXT,
  decided_by_user_id TEXT,
  decided_at        TEXT,
  expires_at        TEXT NOT NULL,
  idempotency_key   TEXT,
  signed_spec       TEXT NOT NULL,
  target_snapshots  TEXT NOT NULL DEFAULT '{}',
  continuation_policy TEXT NOT NULL DEFAULT 'none',
  metadata          TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, bundle_id) REFERENCES decision_bundles (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, origin_agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, origin_issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, origin_run_id) REFERENCES heartbeat_runs (company_id, id)
);

CREATE INDEX idx_decisions_company_status_expires_at
  ON decisions (company_id, status, expires_at);
CREATE INDEX idx_decisions_bundle ON decisions (bundle_id);
CREATE INDEX idx_decisions_origin_issue ON decisions (origin_issue_id);
CREATE UNIQUE INDEX decisions_company_idempotency_uq
  ON decisions (company_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;

CREATE TABLE decision_target_issues (
  decision_id TEXT NOT NULL,
  issue_id    TEXT NOT NULL,
  company_id  TEXT NOT NULL REFERENCES companies(id),
  PRIMARY KEY (decision_id, issue_id),
  FOREIGN KEY (company_id, decision_id) REFERENCES decisions (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_decision_target_issues_decision ON decision_target_issues (decision_id);
CREATE INDEX idx_decision_target_issues_issue ON decision_target_issues (issue_id);

CREATE TABLE decision_effect_executions (
  id             TEXT PRIMARY KEY,
  company_id     TEXT NOT NULL REFERENCES companies(id),
  decision_id    TEXT NOT NULL,
  effect_index   INTEGER NOT NULL,
  effect_type    TEXT NOT NULL,
  target_issue_id TEXT NOT NULL,
  status         TEXT NOT NULL DEFAULT 'claimed',
  result         TEXT,
  error          TEXT,
  activity_log_id TEXT,
  executed_at    TEXT,
  UNIQUE (company_id, id),
  UNIQUE (decision_id, effect_index),
  FOREIGN KEY (company_id, decision_id) REFERENCES decisions (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, target_issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (activity_log_id) REFERENCES activity_log (id) ON DELETE SET NULL
);

CREATE INDEX idx_decision_effect_executions_target_issue
  ON decision_effect_executions (target_issue_id);

CREATE TABLE decision_training_examples (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  source_kind      TEXT NOT NULL,
  source_id        TEXT NOT NULL,
  issue_id         TEXT NOT NULL,
  cutoff_at        TEXT NOT NULL,
  notes            TEXT NOT NULL DEFAULT '',
  notes_history    TEXT NOT NULL DEFAULT '[]',
  decision_outcome TEXT,
  retention_policy TEXT NOT NULL DEFAULT 'scrub_deleted_comments_v1',
  snapshot         TEXT NOT NULL,
  created_by_user_id TEXT NOT NULL,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (source_kind, source_id, created_by_user_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_decision_training_examples_company_created_at
  ON decision_training_examples (company_id, created_at);
CREATE INDEX idx_decision_training_examples_issue
  ON decision_training_examples (issue_id);
