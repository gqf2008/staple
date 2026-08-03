-- Issue work products, mirroring the upstream `issue_work_products` table.
-- FKs to execution workspaces / runtime services / heartbeat runs are
-- omitted until those tables land; the columns remain TEXT.

CREATE TABLE issue_work_products (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  project_id             TEXT,
  issue_id               TEXT NOT NULL,
  execution_workspace_id TEXT,
  runtime_service_id     TEXT,
  type                   TEXT NOT NULL,
  provider               TEXT NOT NULL,
  external_id            TEXT,
  title                  TEXT NOT NULL,
  url                    TEXT,
  status                 TEXT NOT NULL DEFAULT 'active',
  review_state           TEXT NOT NULL DEFAULT 'none',
  is_primary             INTEGER NOT NULL DEFAULT 0,
  health_status          TEXT NOT NULL DEFAULT 'unknown',
  summary                TEXT,
  metadata               TEXT,
  created_by_run_id      TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id)
);

CREATE INDEX idx_issue_work_products_company_issue_type
  ON issue_work_products (company_id, issue_id, type);
