-- Instruction documents and per-agent instruction file mounts
-- (upstream agent-instructions service parity: company document library +
-- managed agent bundle files with an entry-file flag).

CREATE TABLE instruction_documents (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id),
  name       TEXT NOT NULL,
  content    TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, name)
);

CREATE INDEX idx_instruction_documents_company_updated
  ON instruction_documents (company_id, updated_at);

CREATE TABLE agent_instruction_files (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id),
  agent_id   TEXT NOT NULL,
  path       TEXT NOT NULL,
  content    TEXT NOT NULL,
  is_entry   INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, agent_id, path),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_agent_instruction_files_company_agent
  ON agent_instruction_files (company_id, agent_id);
