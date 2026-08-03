-- Staple initial schema (V1 data model).
--
-- Mirrors doc/SPEC-implementation.md §7.1-§7.15 with Turso/SQLite types:
--   uuid       -> TEXT (RFC 4122 string)
--   jsonb      -> TEXT (JSON document)
--   timestamptz -> TEXT (ISO 8601 UTC, strftime('%Y-%m-%dT%H:%M:%fZ','now'))
--   boolean    -> INTEGER (0/1)
--   enum       -> TEXT + CHECK
--
-- Company isolation is enforced in SQL:
--   * every business table has company_id REFERENCES companies(id)
--   * parent tables expose UNIQUE(company_id, id) and children use composite
--     foreign keys (company_id, <parent>_id), so a row can never reference a
--     parent owned by a different company.
--
-- Required indexes follow §7.14.

CREATE TABLE companies (
  id                              TEXT PRIMARY KEY,
  name                            TEXT NOT NULL,
  description                     TEXT,
  status                          TEXT NOT NULL DEFAULT 'active'
                                  CHECK (status IN ('active', 'paused', 'archived')),
  pause_reason                    TEXT,
  paused_at                       TEXT,
  issue_prefix                    TEXT NOT NULL,
  issue_counter                   INTEGER NOT NULL DEFAULT 0,
  budget_monthly_cents            INTEGER NOT NULL DEFAULT 0,
  spent_monthly_cents             INTEGER NOT NULL DEFAULT 0,
  attachment_max_bytes            INTEGER NOT NULL,
  require_board_approval_for_new_agents INTEGER NOT NULL DEFAULT 0,
  brand_color                     TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE agents (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  name                            TEXT NOT NULL,
  role                            TEXT NOT NULL,
  title                           TEXT,
  icon                            TEXT,
  status                          TEXT NOT NULL DEFAULT 'active'
                                  CHECK (status IN ('active', 'paused', 'idle', 'running',
                                                    'error', 'pending_approval', 'terminated')),
  reports_to                      TEXT REFERENCES agents(id),
  capabilities                    TEXT,
  adapter_type                    TEXT NOT NULL,
  adapter_config                  TEXT NOT NULL DEFAULT '{}',
  runtime_config                  TEXT NOT NULL DEFAULT '{}',
  context_mode                    TEXT NOT NULL DEFAULT 'thin'
                                  CHECK (context_mode IN ('thin', 'fat')),
  budget_monthly_cents            INTEGER NOT NULL DEFAULT 0,
  spent_monthly_cents             INTEGER NOT NULL DEFAULT 0,
  pause_reason                    TEXT,
  paused_at                       TEXT,
  permissions                     TEXT NOT NULL DEFAULT '{}',
  last_heartbeat_at               TEXT,
  metadata                        TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_agents_company_status ON agents (company_id, status);
CREATE INDEX idx_agents_company_reports_to ON agents (company_id, reports_to);

CREATE TABLE agent_api_keys (
  id                              TEXT PRIMARY KEY,
  agent_id                        TEXT NOT NULL,
  company_id                      TEXT NOT NULL,
  name                            TEXT NOT NULL,
  key_hash                        TEXT NOT NULL,
  last_used_at                    TEXT,
  revoked_at                      TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id)
);

CREATE TABLE goals (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  title                           TEXT NOT NULL,
  description                     TEXT,
  level                           TEXT NOT NULL
                                  CHECK (level IN ('company', 'team', 'agent', 'task')),
  parent_id                       TEXT,
  owner_agent_id                  TEXT,
  status                          TEXT NOT NULL DEFAULT 'planned'
                                  CHECK (status IN ('planned', 'active', 'achieved', 'cancelled')),
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, parent_id) REFERENCES goals (company_id, id),
  FOREIGN KEY (company_id, owner_agent_id) REFERENCES agents (company_id, id)
);

CREATE TABLE projects (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  goal_id                         TEXT,
  name                            TEXT NOT NULL,
  description                     TEXT,
  status                          TEXT NOT NULL DEFAULT 'backlog'
                                  CHECK (status IN ('backlog', 'planned', 'in_progress',
                                                    'completed', 'cancelled')),
  lead_agent_id                   TEXT,
  target_date                     TEXT,
  env                             TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, goal_id) REFERENCES goals (company_id, id),
  FOREIGN KEY (company_id, lead_agent_id) REFERENCES agents (company_id, id)
);

CREATE TABLE issues (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  project_id                      TEXT,
  goal_id                         TEXT,
  parent_id                       TEXT,
  title                           TEXT NOT NULL,
  description                     TEXT,
  status                          TEXT NOT NULL DEFAULT 'backlog'
                                  CHECK (status IN ('backlog', 'todo', 'in_progress', 'in_review',
                                                    'done', 'blocked', 'cancelled')),
  priority                        TEXT NOT NULL DEFAULT 'medium'
                                  CHECK (priority IN ('critical', 'high', 'medium', 'low')),
  assignee_agent_id               TEXT,
  assignee_user_id                TEXT,
  checkout_run_id                 TEXT,
  execution_run_id                TEXT,
  execution_agent_name_key        TEXT,
  execution_locked_at             TEXT,
  created_by_agent_id             TEXT,
  created_by_user_id              TEXT,
  issue_number                    INTEGER NOT NULL,
  identifier                      TEXT NOT NULL,
  origin_kind                     TEXT,
  origin_id                       TEXT,
  origin_run_id                   TEXT,
  origin_fingerprint              TEXT,
  request_depth                   INTEGER NOT NULL DEFAULT 0,
  work_mode                       TEXT NOT NULL DEFAULT 'standard'
                                  CHECK (work_mode IN ('standard', 'ask', 'planning')),
  billing_code                    TEXT,
  assignee_adapter_overrides      TEXT,
  execution_policy                TEXT,
  execution_state                 TEXT,
  started_at                      TEXT,
  completed_at                    TEXT,
  cancelled_at                    TEXT,
  hidden_at                       TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, issue_number),
  UNIQUE (company_id, identifier),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, goal_id) REFERENCES goals (company_id, id),
  FOREIGN KEY (company_id, parent_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, assignee_agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_issues_company_status ON issues (company_id, status);
CREATE INDEX idx_issues_company_assignee_status ON issues (company_id, assignee_agent_id, status);
CREATE INDEX idx_issues_company_parent ON issues (company_id, parent_id);
CREATE INDEX idx_issues_company_project ON issues (company_id, project_id);

CREATE TABLE issue_comments (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  issue_id                        TEXT NOT NULL,
  author_agent_id                 TEXT,
  author_user_id                  TEXT,
  body                            TEXT NOT NULL,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, author_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_issue_comments_company_issue ON issue_comments (company_id, issue_id);

CREATE TABLE heartbeat_runs (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  agent_id                        TEXT NOT NULL,
  invocation_source               TEXT NOT NULL
                                  CHECK (invocation_source IN ('scheduler', 'manual', 'callback')),
  status                          TEXT NOT NULL DEFAULT 'queued'
                                  CHECK (status IN ('queued', 'running', 'succeeded', 'failed',
                                                    'cancelled', 'timed_out')),
  started_at                      TEXT,
  finished_at                     TEXT,
  error                           TEXT,
  external_run_id                 TEXT,
  context_snapshot                TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_heartbeat_runs_company_agent_started
  ON heartbeat_runs (company_id, agent_id, started_at DESC);

CREATE TABLE cost_events (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  agent_id                        TEXT NOT NULL,
  issue_id                        TEXT,
  project_id                      TEXT,
  goal_id                         TEXT,
  billing_code                    TEXT,
  provider                        TEXT NOT NULL,
  model                           TEXT NOT NULL,
  cost_status                     TEXT NOT NULL DEFAULT 'reported'
                                  CHECK (cost_status IN ('reported', 'unpriced')),
  input_tokens                    INTEGER NOT NULL DEFAULT 0,
  output_tokens                   INTEGER NOT NULL DEFAULT 0,
  cost_cents                      INTEGER NOT NULL,
  occurred_at                     TEXT NOT NULL,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, goal_id) REFERENCES goals (company_id, id)
);

CREATE INDEX idx_cost_events_company_occurred ON cost_events (company_id, occurred_at);
CREATE INDEX idx_cost_events_company_agent_occurred
  ON cost_events (company_id, agent_id, occurred_at);

CREATE TABLE approvals (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  type                            TEXT NOT NULL
                                  CHECK (type IN ('hire_agent', 'approve_ceo_strategy',
                                                  'budget_override_required',
                                                  'request_board_approval')),
  requested_by_agent_id           TEXT,
  requested_by_user_id            TEXT,
  status                          TEXT NOT NULL DEFAULT 'pending'
                                  CHECK (status IN ('pending', 'revision_requested',
                                                    'approved', 'rejected', 'cancelled')),
  payload                         TEXT NOT NULL DEFAULT '{}',
  decision_note                   TEXT,
  decided_by_user_id              TEXT,
  decided_at                      TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY (company_id, requested_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_approvals_company_status_type ON approvals (company_id, status, type);

CREATE TABLE activity_log (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  actor_type                      TEXT NOT NULL CHECK (actor_type IN ('agent', 'user', 'system')),
  actor_id                        TEXT NOT NULL,
  action                          TEXT NOT NULL,
  entity_type                     TEXT NOT NULL,
  entity_id                       TEXT NOT NULL,
  details                         TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_activity_log_company_created ON activity_log (company_id, created_at DESC);

CREATE TABLE project_memberships (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  project_id                      TEXT NOT NULL,
  user_id                         TEXT NOT NULL,
  state                           TEXT NOT NULL DEFAULT 'joined'
                                  CHECK (state IN ('joined', 'left')),
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, user_id, project_id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id)
);

CREATE INDEX idx_project_memberships_company_user ON project_memberships (company_id, user_id);

CREATE TABLE agent_memberships (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  agent_id                        TEXT NOT NULL,
  user_id                         TEXT NOT NULL,
  state                           TEXT NOT NULL DEFAULT 'joined'
                                  CHECK (state IN ('joined', 'left')),
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, user_id, agent_id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_agent_memberships_company_user ON agent_memberships (company_id, user_id);

CREATE TABLE company_secrets (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  name                            TEXT NOT NULL,
  scope                           TEXT NOT NULL DEFAULT 'company'
                                  CHECK (scope IN ('company', 'user')),
  provider                        TEXT NOT NULL DEFAULT 'local_encrypted',
  owner_user_id                   TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, name)
);

CREATE TABLE company_secret_versions (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  secret_id                       TEXT NOT NULL,
  version                         INTEGER NOT NULL,
  encrypted_value                 TEXT NOT NULL,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (secret_id, version),
  FOREIGN KEY (company_id, secret_id) REFERENCES company_secrets (company_id, id)
);

CREATE TABLE assets (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  provider                        TEXT NOT NULL,
  object_key                      TEXT NOT NULL,
  content_type                    TEXT NOT NULL,
  byte_size                       INTEGER NOT NULL,
  sha256                          TEXT NOT NULL,
  original_filename               TEXT,
  created_by_agent_id             TEXT,
  created_by_user_id              TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, object_key),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_assets_company_created ON assets (company_id, created_at DESC);

CREATE TABLE issue_attachments (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  issue_id                        TEXT NOT NULL,
  asset_id                        TEXT NOT NULL,
  issue_comment_id                TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, asset_id) REFERENCES assets (company_id, id),
  FOREIGN KEY (company_id, issue_comment_id) REFERENCES issue_comments (company_id, id)
);

CREATE INDEX idx_issue_attachments_company_issue ON issue_attachments (company_id, issue_id);

CREATE TABLE documents (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  title                           TEXT,
  format                          TEXT NOT NULL DEFAULT 'markdown',
  latest_body                     TEXT NOT NULL DEFAULT '',
  latest_revision_id              TEXT,
  latest_revision_number          INTEGER NOT NULL DEFAULT 0,
  created_by_agent_id             TEXT,
  created_by_user_id              TEXT,
  updated_by_agent_id             TEXT,
  updated_by_user_id              TEXT,
  locked_at                       TEXT,
  locked_by_agent_id              TEXT,
  locked_by_user_id               TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, updated_by_agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, locked_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE TABLE document_revisions (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  document_id                     TEXT NOT NULL,
  revision_number                 INTEGER NOT NULL,
  body                            TEXT NOT NULL,
  change_summary                  TEXT,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, document_id, revision_number),
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id)
);

CREATE TABLE issue_documents (
  id                              TEXT PRIMARY KEY,
  company_id                      TEXT NOT NULL REFERENCES companies(id),
  issue_id                        TEXT NOT NULL,
  document_id                     TEXT NOT NULL,
  key                             TEXT NOT NULL,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, issue_id, key),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id)
);
