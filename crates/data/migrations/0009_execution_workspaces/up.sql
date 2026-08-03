-- Execution workspace control plane (SPEC §7.16 addenda):
-- environments, project_workspaces, execution_workspaces,
-- workspace_runtime_services, workspace_operations.
--
-- `environments` is a global pool (no company_id, matching upstream:
-- unique name, single `local` driver). Everything else is company-scoped
-- with composite FKs so cross-company references are rejected.

CREATE TABLE environments (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  driver      TEXT NOT NULL DEFAULT 'local',
  status      TEXT NOT NULL DEFAULT 'active',
  config      TEXT NOT NULL DEFAULT '{}',
  env_vars    TEXT NOT NULL DEFAULT '{}',
  metadata    TEXT,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (name)
);

CREATE INDEX idx_environments_status ON environments (status);
CREATE UNIQUE INDEX environments_local_driver_idx ON environments (driver) WHERE driver = 'local';

CREATE TABLE project_workspaces (
  id                   TEXT PRIMARY KEY,
  company_id           TEXT NOT NULL REFERENCES companies(id),
  project_id           TEXT NOT NULL,
  name                 TEXT NOT NULL,
  source_type          TEXT NOT NULL DEFAULT 'local_path',
  cwd                  TEXT,
  repo_url             TEXT,
  repo_ref             TEXT,
  default_ref          TEXT,
  visibility           TEXT NOT NULL DEFAULT 'default',
  setup_command        TEXT,
  cleanup_command      TEXT,
  remote_provider      TEXT,
  remote_workspace_ref TEXT,
  shared_workspace_key TEXT,
  metadata             TEXT,
  is_primary           INTEGER NOT NULL DEFAULT 0,
  created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id)
);

CREATE TABLE execution_workspaces (
  id                               TEXT PRIMARY KEY,
  company_id                       TEXT NOT NULL REFERENCES companies(id),
  project_id                       TEXT NOT NULL,
  project_workspace_id             TEXT,
  source_issue_id                  TEXT,
  mode                             TEXT NOT NULL,
  strategy_type                    TEXT NOT NULL,
  name                             TEXT NOT NULL,
  status                           TEXT NOT NULL DEFAULT 'active',
  cwd                              TEXT,
  repo_url                         TEXT,
  base_ref                         TEXT,
  branch_name                      TEXT,
  provider_type                    TEXT NOT NULL DEFAULT 'local_fs',
  provider_ref                     TEXT,
  derived_from_execution_workspace_id TEXT,
  last_used_at                     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  opened_at                        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  closed_at                        TEXT,
  created_at                       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, project_workspace_id) REFERENCES project_workspaces (company_id, id),
  FOREIGN KEY (company_id, source_issue_id) REFERENCES issues (company_id, id)
);

CREATE INDEX idx_execution_workspaces_company_project ON execution_workspaces (company_id, project_id);
CREATE INDEX idx_execution_workspaces_company_status ON execution_workspaces (company_id, status);

CREATE TABLE workspace_runtime_services (
  id                    TEXT PRIMARY KEY,
  company_id            TEXT NOT NULL REFERENCES companies(id),
  project_id            TEXT,
  project_workspace_id  TEXT,
  execution_workspace_id TEXT,
  issue_id              TEXT,
  scope_type            TEXT NOT NULL,
  scope_id              TEXT,
  service_name          TEXT NOT NULL,
  status                TEXT NOT NULL,
  lifecycle             TEXT NOT NULL,
  reuse_key             TEXT,
  command               TEXT,
  cwd                   TEXT,
  port                  INTEGER,
  url                   TEXT,
  provider              TEXT NOT NULL,
  provider_ref          TEXT,
  owner_agent_id        TEXT,
  started_by_run_id     TEXT,
  created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, project_workspace_id) REFERENCES project_workspaces (company_id, id),
  FOREIGN KEY (company_id, execution_workspace_id) REFERENCES execution_workspaces (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, owner_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_runtime_services_company_scope ON workspace_runtime_services (company_id, scope_type, scope_id);

CREATE TABLE workspace_operations (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  execution_workspace_id TEXT,
  heartbeat_run_id       TEXT,
  issue_id               TEXT,
  phase                  TEXT NOT NULL,
  command                TEXT,
  cwd                    TEXT,
  status                 TEXT NOT NULL DEFAULT 'running',
  exit_code              INTEGER,
  log_store              TEXT,
  log_ref                TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, execution_workspace_id) REFERENCES execution_workspaces (company_id, id),
  FOREIGN KEY (company_id, heartbeat_run_id) REFERENCES heartbeat_runs (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id)
);

CREATE INDEX idx_workspace_operations_company_ws ON workspace_operations (company_id, execution_workspace_id);

-- Composite FK targets need unique indexes on the parent key.
CREATE UNIQUE INDEX heartbeat_runs_company_id_uq ON heartbeat_runs (company_id, id);
