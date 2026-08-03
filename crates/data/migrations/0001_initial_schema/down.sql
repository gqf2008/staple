-- Reverse of 0001_initial_schema/up.sql. Drop order is reverse of creation
-- order so foreign keys never block a drop.

DROP TABLE IF EXISTS issue_documents;
DROP TABLE IF EXISTS document_revisions;
DROP TABLE IF EXISTS documents;
DROP TABLE IF EXISTS issue_attachments;
DROP TABLE IF EXISTS assets;
DROP TABLE IF EXISTS company_secret_versions;
DROP TABLE IF EXISTS company_secrets;
DROP TABLE IF EXISTS agent_memberships;
DROP TABLE IF EXISTS project_memberships;
DROP TABLE IF EXISTS activity_log;
DROP TABLE IF EXISTS approvals;
DROP TABLE IF EXISTS cost_events;
DROP TABLE IF EXISTS heartbeat_runs;
DROP TABLE IF EXISTS issue_comments;
DROP TABLE IF EXISTS issues;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS goals;
DROP TABLE IF EXISTS agent_api_keys;
DROP TABLE IF EXISTS agents;
DROP TABLE IF EXISTS companies;
