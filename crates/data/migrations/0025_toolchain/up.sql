-- Batch 5 of schema alignment: upstream toolchain domain
-- (tool_access.ts): tool_* + connection_* tables.

-- tool_applications (no tool-domain parents; referenced by many tables below).
CREATE TABLE tool_applications (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  application_key  TEXT,
  name             TEXT NOT NULL,
  description      TEXT,
  type             TEXT NOT NULL,
  status           TEXT NOT NULL DEFAULT 'active',
  plugin_id        TEXT REFERENCES plugins(id) ON DELETE SET NULL,
  owner_agent_id   TEXT,
  owner_user_id    TEXT,
  metadata         TEXT NOT NULL DEFAULT '{}',
  archived_at      TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, owner_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_applications_company_idx ON tool_applications (company_id);
CREATE INDEX tool_applications_company_status_idx ON tool_applications (company_id, status);
CREATE UNIQUE INDEX tool_applications_company_name_uq ON tool_applications (company_id, name);
CREATE UNIQUE INDEX tool_applications_company_key_uq ON tool_applications (company_id, application_key);

-- tool_connections.
CREATE TABLE tool_connections (
  id                       TEXT PRIMARY KEY,
  company_id               TEXT NOT NULL REFERENCES companies(id),
  application_id           TEXT NOT NULL,
  name                     TEXT NOT NULL,
  uid                      TEXT NOT NULL,
  connection_kind          TEXT NOT NULL DEFAULT 'managed',
  ownership                TEXT NOT NULL DEFAULT 'customer',
  transport                TEXT NOT NULL,
  auth_kind                TEXT NOT NULL DEFAULT 'none',
  status                   TEXT NOT NULL DEFAULT 'draft',
  enabled                  INTEGER NOT NULL DEFAULT 0,
  config                   TEXT NOT NULL DEFAULT '{}',
  transport_config         TEXT NOT NULL DEFAULT '{}',
  credential_refs          TEXT NOT NULL DEFAULT '[]',
  credential_secret_refs   TEXT NOT NULL DEFAULT '[]',
  health_status            TEXT NOT NULL DEFAULT 'unchecked',
  health_message           TEXT,
  health_checked_at        TEXT,
  last_health_at           TEXT,
  last_catalog_refresh_at  TEXT,
  last_error               TEXT,
  created_by_agent_id      TEXT,
  created_by_user_id       TEXT,
  created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  CHECK (ownership IN ('platform_shared', 'platform_provisioned', 'customer', 'dcr')),
  CHECK (transport IN ('mcp_remote', 'rest_api', 'local_stdio')),
  CHECK (auth_kind IN ('oauth', 'api_key', 'none')),
  FOREIGN KEY (company_id, application_id) REFERENCES tool_applications (company_id, id)
);

CREATE INDEX tool_connections_company_idx ON tool_connections (company_id);
CREATE INDEX tool_connections_application_idx ON tool_connections (application_id);
CREATE INDEX tool_connections_company_enabled_idx ON tool_connections (company_id, enabled);
CREATE UNIQUE INDEX tool_connections_company_name_uq ON tool_connections (company_id, name);
CREATE UNIQUE INDEX tool_connections_company_uid_uq ON tool_connections (company_id, uid);

-- connection_grants.
CREATE TABLE connection_grants (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  connection_id          TEXT NOT NULL,
  kind                   TEXT NOT NULL,
  subject_user_id        TEXT,
  provider_tenant        TEXT,
  credential_secret_refs TEXT NOT NULL DEFAULT '[]',
  status                 TEXT NOT NULL DEFAULT 'active',
  is_default             INTEGER NOT NULL DEFAULT 0,
  created_by_agent_id    TEXT,
  created_by_user_id     TEXT,
  revoked_at             TEXT,
  revoked_by_agent_id    TEXT,
  revoked_by_user_id     TEXT,
  last_used_at           TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  CHECK (kind IN ('workspace', 'user')),
  CHECK (status IN ('active', 'revoked', 'expired', 'needs_reauthorization')),
  CHECK ((kind = 'user' AND subject_user_id IS NOT NULL)
         OR (kind = 'workspace' AND subject_user_id IS NULL)),
  CHECK (is_default = 0 OR kind = 'workspace'),
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, revoked_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX connection_grants_company_connection_idx ON connection_grants (company_id, connection_id);
CREATE INDEX connection_grants_subject_user_idx ON connection_grants (company_id, subject_user_id);
CREATE UNIQUE INDEX connection_grants_user_uq ON connection_grants (connection_id, subject_user_id);
CREATE UNIQUE INDEX connection_grants_default_uq ON connection_grants (connection_id)
  WHERE is_default = 1 AND kind = 'workspace';

-- tool_connection_installs.
CREATE TABLE tool_connection_installs (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  connection_id       TEXT NOT NULL,
  target_type         TEXT NOT NULL,
  target_id           TEXT NOT NULL,
  created_by_agent_id TEXT,
  created_by_user_id  TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  CHECK (target_type IN ('company', 'agent')),
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_connection_installs_company_target_idx
  ON tool_connection_installs (company_id, target_type, target_id);
CREATE INDEX tool_connection_installs_connection_idx
  ON tool_connection_installs (company_id, connection_id);
CREATE UNIQUE INDEX tool_connection_installs_target_uq
  ON tool_connection_installs (company_id, connection_id, target_type, target_id);

-- tool_oauth_states (state is the primary key; issue/interaction ids are
-- plain TEXT columns in upstream, not foreign keys).
CREATE TABLE tool_oauth_states (
  state                 TEXT PRIMARY KEY,
  company_id            TEXT NOT NULL REFERENCES companies(id),
  connection_id         TEXT NOT NULL,
  code_verifier         TEXT NOT NULL,
  created_by_actor_type TEXT,
  created_by_actor_id   TEXT,
  created_by_session_id TEXT,
  subject_user_id       TEXT,
  requested_scopes      TEXT,
  return_to             TEXT,
  issue_id              TEXT,
  interaction_id        TEXT,
  expires_at            TEXT NOT NULL,
  created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE CASCADE
);

CREATE INDEX tool_oauth_states_company_idx ON tool_oauth_states (company_id);
CREATE INDEX tool_oauth_states_connection_idx ON tool_oauth_states (connection_id);
CREATE INDEX tool_oauth_states_actor_idx ON tool_oauth_states (created_by_actor_type, created_by_actor_id);
CREATE INDEX tool_oauth_states_expires_at_idx ON tool_oauth_states (expires_at);

-- tool_catalog_entries.
CREATE TABLE tool_catalog_entries (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  application_id      TEXT,
  connection_id       TEXT NOT NULL,
  entry_kind          TEXT NOT NULL DEFAULT 'tool',
  name                TEXT NOT NULL,
  tool_name           TEXT NOT NULL,
  title               TEXT,
  description         TEXT,
  input_schema        TEXT NOT NULL DEFAULT '{}',
  output_schema       TEXT,
  annotations         TEXT NOT NULL DEFAULT '{}',
  risk_level          TEXT NOT NULL DEFAULT 'read',
  is_read_only        INTEGER NOT NULL DEFAULT 1,
  is_write            INTEGER NOT NULL DEFAULT 0,
  is_destructive      INTEGER NOT NULL DEFAULT 0,
  status              TEXT NOT NULL DEFAULT 'active',
  version             TEXT,
  version_hash        TEXT NOT NULL,
  schema_hash         TEXT,
  first_seen_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  last_seen_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  reviewed_at         TEXT,
  reviewed_by_agent_id TEXT,
  reviewed_by_user_id TEXT,
  quarantined_at      TEXT,
  quarantine_reason   TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, application_id) REFERENCES tool_applications (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, reviewed_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_catalog_entries_company_idx ON tool_catalog_entries (company_id);
CREATE INDEX tool_catalog_entries_application_idx ON tool_catalog_entries (application_id);
CREATE INDEX tool_catalog_entries_connection_idx ON tool_catalog_entries (connection_id);
CREATE INDEX tool_catalog_entries_company_status_idx ON tool_catalog_entries (company_id, status);
CREATE UNIQUE INDEX tool_catalog_entries_connection_name_uq
  ON tool_catalog_entries (connection_id, name);
-- tool_profiles.
CREATE TABLE tool_profiles (
  id                    TEXT PRIMARY KEY,
  company_id            TEXT NOT NULL REFERENCES companies(id),
  profile_key           TEXT NOT NULL,
  name                  TEXT NOT NULL,
  description           TEXT,
  status                TEXT NOT NULL DEFAULT 'active',
  default_action        TEXT NOT NULL DEFAULT 'deny',
  new_tools_reviewed_at TEXT,
  metadata              TEXT NOT NULL DEFAULT '{}',
  created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id)
);

CREATE INDEX tool_profiles_company_status_idx ON tool_profiles (company_id, status);
CREATE UNIQUE INDEX tool_profiles_company_key_uq ON tool_profiles (company_id, profile_key);
CREATE UNIQUE INDEX tool_profiles_company_name_uq ON tool_profiles (company_id, name);

-- tool_profile_entries.
CREATE TABLE tool_profile_entries (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  profile_id       TEXT NOT NULL,
  selector_type    TEXT NOT NULL,
  effect           TEXT NOT NULL DEFAULT 'include',
  application_id   TEXT,
  connection_id    TEXT,
  catalog_entry_id TEXT,
  tool_name        TEXT,
  risk_level       TEXT,
  conditions       TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, profile_id) REFERENCES tool_profiles (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, application_id) REFERENCES tool_applications (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, catalog_entry_id) REFERENCES tool_catalog_entries (company_id, id) ON DELETE CASCADE
);

CREATE INDEX tool_profile_entries_company_profile_idx ON tool_profile_entries (company_id, profile_id);
CREATE INDEX tool_profile_entries_application_idx ON tool_profile_entries (company_id, application_id);
CREATE INDEX tool_profile_entries_connection_idx ON tool_profile_entries (company_id, connection_id);
CREATE INDEX tool_profile_entries_catalog_entry_idx ON tool_profile_entries (company_id, catalog_entry_id);

-- tool_profile_bindings.
CREATE TABLE tool_profile_bindings (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  profile_id          TEXT NOT NULL,
  target_type         TEXT NOT NULL,
  target_id           TEXT NOT NULL,
  priority            INTEGER NOT NULL DEFAULT 100,
  metadata            TEXT NOT NULL DEFAULT '{}',
  created_by_agent_id TEXT,
  created_by_user_id  TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, profile_id) REFERENCES tool_profiles (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_profile_bindings_company_target_idx
  ON tool_profile_bindings (company_id, target_type, target_id);
CREATE UNIQUE INDEX tool_profile_bindings_target_profile_uq
  ON tool_profile_bindings (company_id, target_type, target_id, profile_id);

-- tool_mcp_gateways.
CREATE TABLE tool_mcp_gateways (
  id                   TEXT PRIMARY KEY,
  company_id           TEXT NOT NULL REFERENCES companies(id),
  gateway_public_id    TEXT NOT NULL DEFAULT ('gw_' || lower(hex(randomblob(16)))),
  name                 TEXT NOT NULL,
  slug                 TEXT NOT NULL,
  display_slug         TEXT NOT NULL DEFAULT '',
  description          TEXT,
  status               TEXT NOT NULL DEFAULT 'active',
  profile_id           TEXT NOT NULL,
  default_profile_mode TEXT NOT NULL DEFAULT 'gateway_only',
  context_scope_type   TEXT NOT NULL DEFAULT 'none',
  context_scope_id     TEXT,
  agent_id             TEXT,
  project_id           TEXT,
  issue_id             TEXT,
  approval_issue_id    TEXT,
  auth_config          TEXT NOT NULL DEFAULT '{"version":1,"bearer":{"enabled":true,"tokenPrefix":"pcgw","defaultTtlSeconds":7776000,"requireFiniteExpiry":true,"longLivedTokenRequiresOverride":true},"oauth":{"enabled":false,"reservedFor":"v1_5","dynamicClientRegistration":false,"authorizationCodePkce":false}}',
  header_policy        TEXT NOT NULL DEFAULT '{"version":1,"callerPassthrough":{"enabled":false,"allowedHeaders":[]},"staticHeaders":[],"generatedMetadata":{"enabled":false,"allowedHeaders":[]},"responseHeaders":{"forwardMcpRequiredHeaders":true,"forwardSafeCacheHeaders":true}}',
  metadata_policy      TEXT NOT NULL DEFAULT '{"version":1,"forwardCompanyId":false,"forwardGatewayId":false,"forwardProjectId":false,"forwardIssueId":false,"forwardAgentId":false,"forwardRunId":false,"forwardCorrelationId":true}',
  on_demand_tools_config TEXT NOT NULL DEFAULT '{"enabled":false,"searchToolName":"search_tools","runToolName":"run_tool"}',
  metadata             TEXT NOT NULL DEFAULT '{}',
  created_by_agent_id  TEXT,
  created_by_user_id   TEXT,
  archived_at          TEXT,
  created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, profile_id) REFERENCES tool_profiles (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, approval_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_mcp_gateways_company_idx ON tool_mcp_gateways (company_id);
CREATE INDEX tool_mcp_gateways_company_status_idx ON tool_mcp_gateways (company_id, status);
CREATE INDEX tool_mcp_gateways_profile_idx ON tool_mcp_gateways (company_id, profile_id);
CREATE UNIQUE INDEX tool_mcp_gateways_public_id_uq ON tool_mcp_gateways (gateway_public_id);
CREATE UNIQUE INDEX tool_mcp_gateways_company_slug_uq ON tool_mcp_gateways (company_id, slug);
CREATE UNIQUE INDEX tool_mcp_gateways_company_name_uq ON tool_mcp_gateways (company_id, name);

-- tool_mcp_gateway_tokens.
CREATE TABLE tool_mcp_gateway_tokens (
  id                         TEXT PRIMARY KEY,
  company_id                 TEXT NOT NULL REFERENCES companies(id),
  gateway_id                 TEXT NOT NULL,
  name                       TEXT NOT NULL,
  token_hash                 TEXT NOT NULL,
  token_prefix               TEXT NOT NULL DEFAULT '',
  subject_type               TEXT NOT NULL DEFAULT 'gateway_client',
  subject_id                 TEXT,
  client_label               TEXT NOT NULL DEFAULT '',
  owner_note                 TEXT NOT NULL DEFAULT '',
  allowed_actions            TEXT NOT NULL DEFAULT '["tools/list","tools/call"]',
  expires_at                 TEXT,
  expiry_override_reason     TEXT,
  expiry_override_by_user_id TEXT,
  expiry_override_by_agent_id TEXT,
  expiry_override_at         TEXT,
  last_used_at               TEXT,
  revoked_at                 TEXT,
  created_by_agent_id        TEXT,
  created_by_user_id         TEXT,
  created_at                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, gateway_id) REFERENCES tool_mcp_gateways (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, expiry_override_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX tool_mcp_gateway_tokens_token_hash_uq ON tool_mcp_gateway_tokens (token_hash);
CREATE INDEX tool_mcp_gateway_tokens_gateway_idx ON tool_mcp_gateway_tokens (company_id, gateway_id);
CREATE INDEX tool_mcp_gateway_tokens_subject_idx
  ON tool_mcp_gateway_tokens (company_id, subject_type, subject_id);
CREATE INDEX tool_mcp_gateway_tokens_company_expires_idx
  ON tool_mcp_gateway_tokens (company_id, expires_at);

-- tool_policies.
CREATE TABLE tool_policies (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  name                TEXT NOT NULL,
  description         TEXT,
  policy_type         TEXT NOT NULL,
  priority            INTEGER NOT NULL DEFAULT 100,
  enabled             INTEGER NOT NULL DEFAULT 1,
  selectors           TEXT NOT NULL DEFAULT '{}',
  conditions          TEXT,
  config              TEXT,
  created_by_agent_id TEXT,
  created_by_user_id  TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_policies_company_enabled_idx ON tool_policies (company_id, enabled);
CREATE INDEX tool_policies_company_type_idx ON tool_policies (company_id, policy_type);
CREATE UNIQUE INDEX tool_policies_company_name_uq ON tool_policies (company_id, name);

-- tool_runtime_slots.
CREATE TABLE tool_runtime_slots (
  id                      TEXT PRIMARY KEY,
  company_id              TEXT NOT NULL REFERENCES companies(id),
  application_id          TEXT,
  connection_id           TEXT,
  project_workspace_id    TEXT,
  execution_workspace_id  TEXT,
  issue_id                TEXT,
  owner_scope_type        TEXT NOT NULL DEFAULT 'connection',
  owner_scope_id          TEXT,
  runtime_kind            TEXT NOT NULL DEFAULT 'local_stdio',
  slot_key                TEXT NOT NULL,
  status                  TEXT NOT NULL DEFAULT 'stopped',
  reuse_key               TEXT,
  workspace_scope         TEXT,
  credential_scope_hash   TEXT,
  provider                TEXT,
  provider_ref            TEXT,
  process_id              INTEGER,
  command_template_key    TEXT,
  health_status           TEXT NOT NULL DEFAULT 'unchecked',
  health_message          TEXT,
  last_health_check_at    TEXT,
  last_started_at         TEXT,
  started_at              TEXT,
  stopped_at              TEXT,
  last_used_at            TEXT,
  idle_expires_at         TEXT,
  idle_deadline_at        TEXT,
  last_error              TEXT,
  metadata                TEXT NOT NULL DEFAULT '{}',
  created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, application_id) REFERENCES tool_applications (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, project_workspace_id) REFERENCES project_workspaces (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, execution_workspace_id) REFERENCES execution_workspaces (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_runtime_slots_company_idx ON tool_runtime_slots (company_id);
CREATE INDEX tool_runtime_slots_connection_idx ON tool_runtime_slots (connection_id);
CREATE INDEX tool_runtime_slots_execution_workspace_idx
  ON tool_runtime_slots (company_id, execution_workspace_id);
CREATE UNIQUE INDEX tool_runtime_slots_slot_key_uq ON tool_runtime_slots (company_id, slot_key);

-- tool_stdio_command_templates.
CREATE TABLE tool_stdio_command_templates (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  template_key        TEXT NOT NULL,
  name                TEXT NOT NULL,
  description         TEXT,
  status              TEXT NOT NULL DEFAULT 'active',
  command             TEXT NOT NULL,
  args                TEXT NOT NULL DEFAULT '[]',
  env_keys            TEXT NOT NULL DEFAULT '[]',
  tools               TEXT NOT NULL DEFAULT '[]',
  created_by_agent_id TEXT,
  created_by_user_id  TEXT,
  disabled_at         TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_stdio_command_templates_company_idx ON tool_stdio_command_templates (company_id);
CREATE INDEX tool_stdio_command_templates_company_status_idx
  ON tool_stdio_command_templates (company_id, status);
CREATE UNIQUE INDEX tool_stdio_command_templates_company_key_uq
  ON tool_stdio_command_templates (company_id, template_key);

-- tool_gateway_sessions.
CREATE TABLE tool_gateway_sessions (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  agent_id            TEXT NOT NULL,
  run_id              TEXT NOT NULL,
  issue_id            TEXT,
  project_id          TEXT,
  gateway_id          TEXT,
  gateway_token_id    TEXT,
  gateway_public_id   TEXT,
  client_subject_type TEXT,
  client_subject_id   TEXT,
  client_name         TEXT,
  mcp_session_id      TEXT,
  correlation_id      TEXT,
  token_hash          TEXT NOT NULL,
  expires_at          TEXT NOT NULL,
  last_used_at        TEXT,
  revoked_at          TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, gateway_id) REFERENCES tool_mcp_gateways (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, gateway_token_id) REFERENCES tool_mcp_gateway_tokens (company_id, id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX tool_gateway_sessions_token_hash_uq ON tool_gateway_sessions (token_hash);
CREATE INDEX tool_gateway_sessions_company_agent_idx ON tool_gateway_sessions (company_id, agent_id);
CREATE INDEX tool_gateway_sessions_company_expires_idx ON tool_gateway_sessions (company_id, expires_at);
CREATE INDEX tool_gateway_sessions_run_idx ON tool_gateway_sessions (company_id, run_id);
CREATE INDEX tool_gateway_sessions_issue_idx ON tool_gateway_sessions (company_id, issue_id);
CREATE INDEX tool_gateway_sessions_gateway_idx ON tool_gateway_sessions (company_id, gateway_id);

-- tool_gateway_rate_limit_counters.
CREATE TABLE tool_gateway_rate_limit_counters (
  id              TEXT PRIMARY KEY,
  company_id      TEXT NOT NULL REFERENCES companies(id),
  counter_key     TEXT NOT NULL,
  window_start_at TEXT NOT NULL,
  window_ms       INTEGER NOT NULL,
  "limit"        INTEGER NOT NULL,
  count           INTEGER NOT NULL DEFAULT 0,
  reset_at        TEXT NOT NULL,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id)
);

CREATE INDEX tool_gateway_rate_limit_counters_company_idx
  ON tool_gateway_rate_limit_counters (company_id);
CREATE UNIQUE INDEX tool_gateway_rate_limit_counters_window_uq
  ON tool_gateway_rate_limit_counters (company_id, counter_key, window_start_at);
-- tool_invocations.
CREATE TABLE tool_invocations (
  id                       TEXT PRIMARY KEY,
  company_id               TEXT NOT NULL REFERENCES companies(id),
  idempotency_key          TEXT,
  actor_type               TEXT NOT NULL DEFAULT 'system',
  actor_id                 TEXT,
  agent_id                 TEXT,
  issue_id                 TEXT,
  run_id                   TEXT,
  gateway_id               TEXT,
  gateway_token_id         TEXT,
  gateway_public_id        TEXT,
  client_subject_type      TEXT,
  client_subject_id        TEXT,
  client_name              TEXT,
  mcp_session_id           TEXT,
  correlation_id           TEXT,
  application_id           TEXT,
  connection_id            TEXT,
  catalog_entry_id         TEXT,
  catalog_version_hash     TEXT,
  catalog_schema_hash      TEXT,
  provider_type            TEXT,
  application_key          TEXT,
  upstream_tool_name       TEXT,
  risk_level               TEXT,
  tool_name                TEXT NOT NULL,
  arguments_hash           TEXT,
  arguments_summary        TEXT,
  policy_decision          TEXT,
  matched_policy_ids       TEXT NOT NULL DEFAULT '[]',
  policy_explanation       TEXT,
  credential_scope_summary TEXT,
  header_policy_summary    TEXT,
  approval_state           TEXT NOT NULL DEFAULT 'not_required',
  status                   TEXT NOT NULL DEFAULT 'pending',
  upstream_request_id      TEXT,
  result_hash              TEXT,
  result_summary           TEXT,
  result_size_bytes        INTEGER,
  result_artifact_id       TEXT,
  error_code               TEXT,
  error_message            TEXT,
  started_at               TEXT,
  completed_at             TEXT,
  created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, gateway_id) REFERENCES tool_mcp_gateways (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, gateway_token_id) REFERENCES tool_mcp_gateway_tokens (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, application_id) REFERENCES tool_applications (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, catalog_entry_id) REFERENCES tool_catalog_entries (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_invocations_company_created_idx ON tool_invocations (company_id, created_at);
CREATE INDEX tool_invocations_run_idx ON tool_invocations (company_id, run_id);
CREATE INDEX tool_invocations_issue_idx ON tool_invocations (company_id, issue_id);
CREATE INDEX tool_invocations_gateway_idx ON tool_invocations (company_id, gateway_id);
CREATE UNIQUE INDEX tool_invocations_company_idempotency_uq
  ON tool_invocations (company_id, idempotency_key);

-- tool_action_requests.
CREATE TABLE tool_action_requests (
  id                          TEXT PRIMARY KEY,
  company_id                  TEXT NOT NULL REFERENCES companies(id),
  invocation_id               TEXT NOT NULL,
  issue_id                    TEXT,
  interaction_id              TEXT,
  approval_id                 TEXT,
  status                      TEXT NOT NULL DEFAULT 'pending',
  canonical_arguments_hash    TEXT NOT NULL,
  canonical_arguments_summary TEXT NOT NULL,
  signed_arguments            TEXT,
  preview_markdown            TEXT,
  requested_by_agent_id       TEXT,
  requested_by_user_id        TEXT,
  resolved_by_agent_id        TEXT,
  resolved_by_user_id         TEXT,
  decided_by_agent_id         TEXT,
  decided_by_user_id          TEXT,
  decided_at                  TEXT,
  expires_at                  TEXT,
  resolved_at                 TEXT,
  created_at                  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, invocation_id) REFERENCES tool_invocations (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, interaction_id) REFERENCES issue_thread_interactions (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, approval_id) REFERENCES approvals (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, requested_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, resolved_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, decided_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_action_requests_company_status_idx ON tool_action_requests (company_id, status);
CREATE INDEX tool_action_requests_invocation_idx ON tool_action_requests (invocation_id);
CREATE INDEX tool_action_requests_issue_idx ON tool_action_requests (company_id, issue_id);

-- tool_call_events.
CREATE TABLE tool_call_events (
  id                       TEXT PRIMARY KEY,
  company_id               TEXT NOT NULL REFERENCES companies(id),
  event_type               TEXT NOT NULL,
  actor_type               TEXT NOT NULL DEFAULT 'system',
  actor_id                 TEXT,
  agent_id                 TEXT,
  run_id                   TEXT,
  issue_id                 TEXT,
  gateway_id               TEXT,
  gateway_token_id         TEXT,
  gateway_public_id        TEXT,
  client_subject_type      TEXT,
  client_subject_id        TEXT,
  client_name              TEXT,
  mcp_session_id           TEXT,
  correlation_id           TEXT,
  application_id           TEXT,
  connection_id            TEXT,
  catalog_entry_id         TEXT,
  invocation_id            TEXT,
  action_request_id        TEXT,
  runtime_slot_id          TEXT,
  tool_name                TEXT,
  decision                 TEXT,
  matched_policy_ids       TEXT NOT NULL DEFAULT '[]',
  reason_code              TEXT,
  policy_explanation       TEXT,
  credential_scope_summary TEXT,
  header_policy_summary    TEXT,
  outcome                  TEXT NOT NULL DEFAULT 'pending',
  latency_ms               INTEGER,
  arguments_summary        TEXT,
  request_hash             TEXT,
  request_summary          TEXT,
  result_hash              TEXT,
  result_summary           TEXT,
  result_size_bytes        INTEGER,
  redaction_plan           TEXT,
  rate_limit_state         TEXT,
  metadata                 TEXT,
  error_code               TEXT,
  error_message            TEXT,
  created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, gateway_id) REFERENCES tool_mcp_gateways (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, gateway_token_id) REFERENCES tool_mcp_gateway_tokens (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, application_id) REFERENCES tool_applications (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, catalog_entry_id) REFERENCES tool_catalog_entries (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, invocation_id) REFERENCES tool_invocations (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, action_request_id) REFERENCES tool_action_requests (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, runtime_slot_id) REFERENCES tool_runtime_slots (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_call_events_company_created_idx ON tool_call_events (company_id, created_at);
CREATE INDEX tool_call_events_run_idx ON tool_call_events (company_id, run_id);
CREATE INDEX tool_call_events_issue_idx ON tool_call_events (company_id, issue_id);
CREATE INDEX tool_call_events_invocation_idx ON tool_call_events (invocation_id);
CREATE INDEX tool_call_events_gateway_idx ON tool_call_events (company_id, gateway_id);

-- tool_access_audit_events.
CREATE TABLE tool_access_audit_events (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  gateway_id       TEXT,
  gateway_token_id TEXT,
  gateway_public_id TEXT,
  client_name      TEXT,
  correlation_id   TEXT,
  connection_id    TEXT,
  catalog_entry_id TEXT,
  actor_type       TEXT NOT NULL DEFAULT 'system',
  actor_id         TEXT,
  action           TEXT NOT NULL,
  outcome          TEXT NOT NULL,
  reason_code      TEXT,
  details          TEXT NOT NULL DEFAULT '{}',
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, gateway_id) REFERENCES tool_mcp_gateways (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, gateway_token_id) REFERENCES tool_mcp_gateway_tokens (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, catalog_entry_id) REFERENCES tool_catalog_entries (company_id, id) ON DELETE SET NULL
);

CREATE INDEX tool_access_audit_company_created_idx ON tool_access_audit_events (company_id, created_at);
CREATE INDEX tool_access_audit_connection_idx ON tool_access_audit_events (connection_id);
CREATE INDEX tool_access_audit_gateway_idx ON tool_access_audit_events (company_id, gateway_id);

-- connection_token_issuances.
CREATE TABLE connection_token_issuances (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  application_id      TEXT,
  connection_id       TEXT NOT NULL,
  agent_id            TEXT NOT NULL,
  run_id              TEXT,
  issue_id            TEXT,
  project_id          TEXT,
  responsible_user_id TEXT,
  path                TEXT NOT NULL,
  requested_scope     TEXT NOT NULL DEFAULT '[]',
  issued_scope        TEXT NOT NULL DEFAULT '[]',
  ttl_seconds         INTEGER,
  expires_at          TEXT,
  token_hash          TEXT,
  outcome             TEXT NOT NULL,
  error_code          TEXT,
  metadata            TEXT NOT NULL DEFAULT '{}',
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  CHECK (path IN ('exchange', 'oauth_access', 'static')),
  CHECK (outcome IN ('success', 'denied', 'rate_limited', 'use_env_lease', 'upstream_error', 'failure')),
  CHECK (ttl_seconds IS NULL OR (ttl_seconds >= 1 AND ttl_seconds <= 900)),
  CHECK (token_hash IS NULL OR token_hash GLOB '[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]'),
  FOREIGN KEY (company_id, application_id) REFERENCES tool_applications (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, connection_id) REFERENCES tool_connections (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id) ON DELETE SET NULL
);

CREATE INDEX connection_token_issuances_company_created_idx
  ON connection_token_issuances (company_id, created_at);
CREATE INDEX connection_token_issuances_connection_created_idx
  ON connection_token_issuances (company_id, connection_id, created_at);
CREATE INDEX connection_token_issuances_agent_connection_idx
  ON connection_token_issuances (company_id, agent_id, connection_id, created_at);
CREATE INDEX connection_token_issuances_run_idx ON connection_token_issuances (company_id, run_id);

-- tool_rate_limit_counters.
CREATE TABLE tool_rate_limit_counters (
  id              TEXT PRIMARY KEY,
  company_id      TEXT NOT NULL REFERENCES companies(id),
  policy_id       TEXT NOT NULL,
  counter_key     TEXT NOT NULL,
  scope_type      TEXT NOT NULL,
  scope_id        TEXT NOT NULL,
  window_kind     TEXT NOT NULL,
  window_start_at TEXT NOT NULL,
  "limit"        INTEGER NOT NULL,
  remaining       INTEGER NOT NULL,
  reset_at        TEXT NOT NULL,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, policy_id) REFERENCES tool_policies (company_id, id) ON DELETE CASCADE
);

CREATE INDEX tool_rate_limit_counters_company_idx ON tool_rate_limit_counters (company_id);
CREATE UNIQUE INDEX tool_rate_limit_counters_window_uq
  ON tool_rate_limit_counters (company_id, policy_id, counter_key, window_kind, window_start_at);

-- tool_runtime_metric_counters.
CREATE TABLE tool_runtime_metric_counters (
  id              TEXT PRIMARY KEY,
  company_id      TEXT NOT NULL REFERENCES companies(id),
  metric          TEXT NOT NULL,
  bucket_start_at TEXT NOT NULL,
  count           INTEGER NOT NULL DEFAULT 0,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  CHECK (count >= 0)
);

CREATE INDEX tool_runtime_metric_counters_company_metric_idx
  ON tool_runtime_metric_counters (company_id, metric, bucket_start_at);
CREATE UNIQUE INDEX tool_runtime_metric_counters_bucket_uq
  ON tool_runtime_metric_counters (company_id, metric, bucket_start_at);

-- approvals already exposes UNIQUE (company_id, id) via migration 0010
-- (approvals_company_id_uq), which tool_action_requests references.
