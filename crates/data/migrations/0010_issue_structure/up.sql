-- Issue structure extensions (SPEC §7.16 addenda): labels, thread
-- interactions, read states, issue approvals, execution decisions.

CREATE TABLE labels (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id),
  name       TEXT NOT NULL,
  color      TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, name)
);

CREATE INDEX idx_labels_company ON labels (company_id);

CREATE TABLE issue_labels (
  issue_id   TEXT NOT NULL,
  label_id   TEXT NOT NULL,
  company_id TEXT NOT NULL REFERENCES companies(id),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (issue_id, label_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, label_id) REFERENCES labels (company_id, id)
);

CREATE INDEX idx_issue_labels_issue ON issue_labels (issue_id);
CREATE INDEX idx_issue_labels_label ON issue_labels (label_id);
CREATE INDEX idx_issue_labels_company ON issue_labels (company_id);

CREATE TABLE issue_thread_interactions (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id),
  issue_id   TEXT NOT NULL,
  kind       TEXT NOT NULL,
  status     TEXT NOT NULL DEFAULT 'pending',
  payload    TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id)
);

CREATE TABLE issue_read_states (
  id           TEXT PRIMARY KEY,
  company_id   TEXT NOT NULL REFERENCES companies(id),
  issue_id     TEXT NOT NULL,
  user_id      TEXT NOT NULL,
  last_read_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, issue_id, user_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id)
);

CREATE INDEX idx_issue_read_states_company_issue ON issue_read_states (company_id, issue_id);
CREATE INDEX idx_issue_read_states_company_user ON issue_read_states (company_id, user_id);

CREATE TABLE issue_approvals (
  issue_id          TEXT NOT NULL,
  approval_id       TEXT NOT NULL,
  company_id        TEXT NOT NULL REFERENCES companies(id),
  linked_by_agent_id TEXT,
  linked_by_user_id TEXT,
  created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (issue_id, approval_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, approval_id) REFERENCES approvals (company_id, id),
  FOREIGN KEY (company_id, linked_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_issue_approvals_issue ON issue_approvals (issue_id);
CREATE INDEX idx_issue_approvals_approval ON issue_approvals (approval_id);

CREATE TABLE issue_execution_decisions (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  issue_id         TEXT NOT NULL,
  stage_id         TEXT NOT NULL,
  stage_type       TEXT NOT NULL,
  actor_agent_id   TEXT,
  actor_user_id    TEXT,
  outcome          TEXT NOT NULL,
  body             TEXT NOT NULL DEFAULT '',
  created_by_run_id TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, actor_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_issue_execution_decisions_company_issue
  ON issue_execution_decisions (company_id, issue_id);
CREATE INDEX idx_issue_execution_decisions_stage
  ON issue_execution_decisions (issue_id, stage_id, created_at);

-- Composite FK target for issue_approvals.
CREATE UNIQUE INDEX approvals_company_id_uq ON approvals (company_id, id);
