-- Managed checkout materialization state for execution workspaces
-- (upstream managed-checkout: server-side clone/fetch with company-secret
-- credentials).

ALTER TABLE execution_workspaces ADD COLUMN materialized INTEGER NOT NULL DEFAULT 0;
ALTER TABLE execution_workspaces ADD COLUMN materialized_at TEXT;
ALTER TABLE execution_workspaces ADD COLUMN materialize_error TEXT;
ALTER TABLE execution_workspaces ADD COLUMN credential_secret_name TEXT;
