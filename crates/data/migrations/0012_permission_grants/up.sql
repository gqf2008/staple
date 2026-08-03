-- Principal permission grants (upstream §9.8 / principal_permission_grants.ts).
--
-- A grant is company-scoped and keyed by (principal_type, principal_id,
-- permission_key). `scope` holds a JSON object with recognized constraints:
--   project:  projectId | projectIds | allow:["project:<id>"]
--   agent:    agentId | agentIds | assigneeAgentId(s) | targetAgentId(s) | allow:["agent:<id>"]
--   subtree:  managerAgentId(s) | managedSubtreeAgentId(s) | subtreeAgentId(s)
--             | subtreeRootAgentId(s) | allow:["subtree:<id>"]
--   user:     userId | userIds
-- Multiple constraint families must all match; unknown keys do not constrain.

CREATE TABLE principal_permission_grants (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id),
  principal_type     TEXT NOT NULL CHECK (principal_type IN ('agent', 'user')),
  principal_id       TEXT NOT NULL,
  permission_key     TEXT NOT NULL,
  scope              TEXT,
  granted_by_user_id TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, principal_type, principal_id, permission_key),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_principal_permission_grants_company_permission
  ON principal_permission_grants (company_id, permission_key);
