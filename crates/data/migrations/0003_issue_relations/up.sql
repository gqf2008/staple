-- Issue relations (blockers), mirroring the upstream `issue_relations` table.
--
-- `type = 'blocks'` means `issue_id` blocks `related_issue_id`; the
-- `issue_id` column is the source (blocking) issue.

CREATE TABLE issue_relations (
  id                   TEXT PRIMARY KEY,
  company_id           TEXT NOT NULL REFERENCES companies(id),
  issue_id             TEXT NOT NULL,
  related_issue_id     TEXT NOT NULL,
  type                 TEXT NOT NULL DEFAULT 'blocks' CHECK (type IN ('blocks')),
  created_by_agent_id  TEXT,
  created_by_user_id   TEXT,
  created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, issue_id, related_issue_id, type),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, related_issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_issue_relations_company_issue ON issue_relations (company_id, issue_id);
CREATE INDEX idx_issue_relations_company_related ON issue_relations (company_id, related_issue_id);
CREATE INDEX idx_issue_relations_company_type ON issue_relations (company_id, type);
