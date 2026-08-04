-- Cases (upstream cases.ts): company-scoped case records with a per-company
-- case number, identifier, typed key, free-form fields, and self-referencing
-- parent.

CREATE TABLE cases (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id),
  project_id         TEXT,
  case_number        INTEGER NOT NULL,
  identifier         TEXT NOT NULL,
  case_type          TEXT NOT NULL,
  key                TEXT,
  title              TEXT NOT NULL,
  summary            TEXT,
  status             TEXT NOT NULL DEFAULT 'draft'
                     CHECK (status IN ('draft', 'in_progress', 'in_review',
                                       'approved', 'done', 'cancelled')),
  fields             TEXT NOT NULL DEFAULT '{}',
  parent_case_id     TEXT,
  created_by_agent_id TEXT,
  created_by_user_id TEXT,
  completed_at       TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, case_number),
  UNIQUE (company_id, identifier),
  UNIQUE (company_id, case_type, key),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, parent_case_id) REFERENCES cases (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_cases_company_status ON cases (company_id, status);
CREATE INDEX idx_cases_company_type ON cases (company_id, case_type);
CREATE INDEX idx_cases_company_project ON cases (company_id, project_id);
CREATE INDEX idx_cases_parent ON cases (parent_case_id);
