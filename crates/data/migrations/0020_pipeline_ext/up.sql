-- Pipelines extension tables (upstream pipeline_cases.ts remainder).

CREATE TABLE pipeline_case_issue_links (
  id                    TEXT PRIMARY KEY,
  company_id            TEXT NOT NULL REFERENCES companies(id),
  case_id               TEXT NOT NULL,
  issue_id              TEXT NOT NULL,
  role                  TEXT NOT NULL CHECK (role IN ('origin', 'conversation', 'work', 'automation')),
  created_by_run_id     TEXT,
  automation_attempt_id TEXT,
  retired_at            TEXT,
  retired_by_attempt_id TEXT,
  retired_reason        TEXT,
  created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, case_id, issue_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES pipeline_cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_pipeline_case_issue_links_case ON pipeline_case_issue_links (company_id, case_id);
CREATE INDEX idx_pipeline_case_issue_links_issue ON pipeline_case_issue_links (issue_id);

CREATE TABLE pipeline_case_blockers (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  case_id          TEXT NOT NULL,
  blocked_by_case_id TEXT NOT NULL,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, case_id, blocked_by_case_id),
  UNIQUE (company_id, id),
  CHECK (case_id <> blocked_by_case_id),
  FOREIGN KEY (company_id, case_id) REFERENCES pipeline_cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, blocked_by_case_id) REFERENCES pipeline_cases (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_pipeline_case_blockers_case ON pipeline_case_blockers (company_id, case_id);
CREATE INDEX idx_pipeline_case_blockers_blocked_by ON pipeline_case_blockers (blocked_by_case_id);

CREATE TABLE pipeline_documents (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  pipeline_id   TEXT NOT NULL,
  document_id   TEXT NOT NULL,
  key           TEXT NOT NULL,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, pipeline_id, key),
  UNIQUE (company_id, document_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, pipeline_id) REFERENCES pipelines (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_pipeline_documents_pipeline ON pipeline_documents (company_id, pipeline_id, updated_at);

CREATE TABLE pipeline_case_documents (
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
  FOREIGN KEY (company_id, case_id) REFERENCES pipeline_cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_pipeline_case_documents_case ON pipeline_case_documents (company_id, case_id, updated_at);

CREATE TABLE pipeline_automation_executions (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  case_id                TEXT NOT NULL,
  automation_id          TEXT NOT NULL,
  triggering_event_id    TEXT NOT NULL,
  routine_id             TEXT NOT NULL,
  status                 TEXT NOT NULL CHECK (status IN ('succeeded', 'failed')),
  execution_issue_id     TEXT,
  retry_of_execution_id  TEXT,
  generation             INTEGER NOT NULL DEFAULT 1,
  error                  TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, case_id, automation_id, triggering_event_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES pipeline_cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, routine_id) REFERENCES routines (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, execution_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_pipeline_automation_executions_case ON pipeline_automation_executions (company_id, case_id);
CREATE INDEX idx_pipeline_automation_executions_routine ON pipeline_automation_executions (routine_id);
CREATE INDEX idx_pipeline_automation_executions_issue ON pipeline_automation_executions (execution_issue_id);
