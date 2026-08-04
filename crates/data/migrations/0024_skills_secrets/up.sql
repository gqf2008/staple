-- Batch 4 of schema alignment: skills version system + secret
-- providers/bindings + user_secret_* tables.
--
-- Upstream references:
--   company_skills.ts (+ company_skill_policies.ts)
--   company_secret_bindings.ts + company_secret_provider_configs.ts
--   user_secret_definitions.ts + user_secret_declarations.ts + secret_access_events.ts
-- Also diffs existing company_skills / company_secrets /
-- company_secret_versions against upstream and adds missing columns.

-- ---------------------------------------------------------------------------
-- Skills version system
-- ---------------------------------------------------------------------------

-- company_skill_versions (upstream company_skills.ts).
CREATE TABLE company_skill_versions (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  company_skill_id TEXT NOT NULL,
  revision_number  INTEGER NOT NULL,
  label            TEXT,
  release_id       TEXT,
  release_name     TEXT,
  released_at      TEXT,
  file_inventory   TEXT NOT NULL DEFAULT '[]',
  author_agent_id  TEXT,
  author_user_id   TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_skill_id, revision_number),
  FOREIGN KEY (company_id, company_skill_id) REFERENCES company_skills (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, author_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX company_skill_versions_skill_release_uq
  ON company_skill_versions (company_skill_id, release_id)
  WHERE release_id IS NOT NULL;
CREATE INDEX company_skill_versions_company_skill_created_idx
  ON company_skill_versions (company_id, company_skill_id, created_at);

-- company_skill_stars (upstream company_skills.ts).
CREATE TABLE company_skill_stars (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  company_skill_id TEXT NOT NULL,
  agent_id         TEXT,
  user_id          TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_skill_id, agent_id),
  UNIQUE (company_skill_id, user_id),
  FOREIGN KEY (company_id, company_skill_id) REFERENCES company_skills (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE CASCADE
);

CREATE INDEX company_skill_stars_company_skill_created_idx
  ON company_skill_stars (company_id, company_skill_id, created_at);

-- company_skill_comments (upstream company_skills.ts).
CREATE TABLE company_skill_comments (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  company_skill_id TEXT NOT NULL,
  parent_comment_id TEXT,
  author_agent_id  TEXT,
  author_user_id   TEXT,
  body             TEXT NOT NULL,
  deleted_at       TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, company_skill_id) REFERENCES company_skills (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, parent_comment_id) REFERENCES company_skill_comments (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, author_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX company_skill_comments_company_skill_created_idx
  ON company_skill_comments (company_id, company_skill_id, created_at);
CREATE INDEX company_skill_comments_parent_idx
  ON company_skill_comments (parent_comment_id);

-- company_skill_test_inputs (upstream company_skills.ts).
CREATE TABLE company_skill_test_inputs (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  skill_id   TEXT NOT NULL,
  name       TEXT NOT NULL,
  content    TEXT NOT NULL,
  created_by TEXT,
  deleted_at TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, skill_id) REFERENCES company_skills (company_id, id) ON DELETE CASCADE
);

CREATE INDEX company_skill_test_inputs_company_skill_name_idx
  ON company_skill_test_inputs (company_id, skill_id, name);
CREATE INDEX company_skill_test_inputs_company_skill_active_idx
  ON company_skill_test_inputs (company_id, skill_id, deleted_at);

-- company_skill_test_run_templates (upstream company_skills.ts).
CREATE TABLE company_skill_test_run_templates (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  name                TEXT NOT NULL,
  description         TEXT,
  body                TEXT NOT NULL,
  created_by_agent_id TEXT,
  created_by_user_id  TEXT,
  updated_by_agent_id TEXT,
  updated_by_user_id  TEXT,
  deleted_at          TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, updated_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX company_skill_test_run_templates_company_active_idx
  ON company_skill_test_run_templates (company_id, deleted_at, name);

-- company_skill_test_runs (upstream company_skills.ts).
CREATE TABLE company_skill_test_runs (
  id                       TEXT PRIMARY KEY,
  company_id               TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  skill_id                 TEXT NOT NULL,
  input_id                 TEXT,
  input_snapshot           TEXT NOT NULL,
  skill_version_id         TEXT NOT NULL,
  agent_id                 TEXT NOT NULL,
  agent_config_snapshot    TEXT NOT NULL DEFAULT '{}',
  issue_id                 TEXT NOT NULL,
  template_id              TEXT,
  template_name            TEXT,
  template_body            TEXT,
  rendered_template_body   TEXT,
  harness_issue_description TEXT NOT NULL DEFAULT '',
  status                   TEXT NOT NULL DEFAULT 'queued',
  output_document_key      TEXT NOT NULL DEFAULT 'output',
  output_snapshot          TEXT NOT NULL DEFAULT '',
  error                    TEXT,
  deleted_at               TEXT,
  superseded_at            TEXT,
  harness_issue_expires_at TEXT,
  harness_issue_deleted_at TEXT,
  created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, issue_id),
  FOREIGN KEY (company_id, skill_id) REFERENCES company_skills (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, input_id) REFERENCES company_skill_test_inputs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, skill_version_id) REFERENCES company_skill_versions (company_id, id) ON DELETE RESTRICT,
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE RESTRICT,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE RESTRICT
);

CREATE INDEX company_skill_test_runs_company_skill_created_idx
  ON company_skill_test_runs (company_id, skill_id, created_at);
CREATE INDEX company_skill_test_runs_company_input_created_idx
  ON company_skill_test_runs (company_id, input_id, created_at);
CREATE INDEX company_skill_test_runs_company_status_idx
  ON company_skill_test_runs (company_id, status);
CREATE INDEX company_skill_test_runs_company_harness_expires_idx
  ON company_skill_test_runs (company_id, harness_issue_expires_at);

-- company_skill_policies (upstream company_skill_policies.ts).
CREATE TABLE company_skill_policies (
  company_id      TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
  schema_version  INTEGER NOT NULL DEFAULT 1,
  revision        INTEGER NOT NULL,
  default_effect  TEXT NOT NULL CHECK (default_effect IN ('allow', 'deny')),
  rules           TEXT NOT NULL DEFAULT '[]',
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

-- ---------------------------------------------------------------------------
-- Secret providers/bindings + user_secret_* tables
-- ---------------------------------------------------------------------------

-- company_secret_provider_configs (upstream company_secret_provider_configs.ts).
CREATE TABLE company_secret_provider_configs (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  provider           TEXT NOT NULL,
  display_name       TEXT NOT NULL,
  status             TEXT NOT NULL DEFAULT 'ready',
  is_default         INTEGER NOT NULL DEFAULT 0,
  config             TEXT NOT NULL DEFAULT '{}',
  health_status      TEXT,
  health_checked_at  TEXT,
  health_message     TEXT,
  health_details     TEXT,
  disabled_at        TEXT,
  created_by_agent_id TEXT,
  created_by_user_id TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX company_secret_provider_configs_company_idx
  ON company_secret_provider_configs (company_id);
CREATE INDEX company_secret_provider_configs_company_provider_idx
  ON company_secret_provider_configs (company_id, provider);
CREATE UNIQUE INDEX company_secret_provider_configs_default_uq
  ON company_secret_provider_configs (company_id, provider)
  WHERE is_default = 1;

-- user_secret_definitions (upstream user_secret_definitions.ts).
CREATE TABLE user_secret_definitions (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  key                 TEXT NOT NULL,
  name                TEXT NOT NULL,
  description         TEXT,
  status              TEXT NOT NULL DEFAULT 'active',
  provider            TEXT NOT NULL DEFAULT 'local_encrypted',
  managed_mode        TEXT NOT NULL DEFAULT 'paperclip_managed',
  provider_config_id  TEXT,
  provider_metadata   TEXT,
  usage_guidance      TEXT,
  created_by_agent_id TEXT,
  created_by_user_id  TEXT,
  updated_by_agent_id TEXT,
  updated_by_user_id  TEXT,
  deleted_at          TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, provider_config_id) REFERENCES company_secret_provider_configs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, updated_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX user_secret_definitions_company_status_idx
  ON user_secret_definitions (company_id, status);
CREATE INDEX user_secret_definitions_company_provider_idx
  ON user_secret_definitions (company_id, provider);
CREATE INDEX user_secret_definitions_provider_config_idx
  ON user_secret_definitions (provider_config_id);
CREATE UNIQUE INDEX user_secret_definitions_company_key_uq
  ON user_secret_definitions (company_id, key)
  WHERE deleted_at IS NULL;

-- user_secret_declarations (upstream user_secret_declarations.ts).
CREATE TABLE user_secret_declarations (
  id                      TEXT PRIMARY KEY,
  company_id              TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  user_secret_definition_id TEXT NOT NULL,
  target_type             TEXT NOT NULL,
  target_id               TEXT NOT NULL,
  config_path             TEXT NOT NULL,
  env_key                 TEXT NOT NULL,
  version_selector        TEXT NOT NULL DEFAULT 'latest',
  required                INTEGER NOT NULL DEFAULT 1,
  allow_missing_override  INTEGER NOT NULL DEFAULT 0,
  label                   TEXT,
  created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, target_type, target_id, config_path),
  FOREIGN KEY (company_id, user_secret_definition_id) REFERENCES user_secret_definitions (company_id, id) ON DELETE CASCADE
);

CREATE INDEX user_secret_declarations_company_idx
  ON user_secret_declarations (company_id);
CREATE INDEX user_secret_declarations_definition_idx
  ON user_secret_declarations (user_secret_definition_id);
CREATE INDEX user_secret_declarations_target_idx
  ON user_secret_declarations (company_id, target_type, target_id);
CREATE INDEX user_secret_declarations_company_required_idx
  ON user_secret_declarations (company_id, required);
CREATE INDEX user_secret_declarations_required_override_idx
  ON user_secret_declarations (company_id, allow_missing_override)
  WHERE allow_missing_override = 1;

-- company_secret_bindings (upstream company_secret_bindings.ts).
CREATE TABLE company_secret_bindings (
  id                      TEXT PRIMARY KEY,
  company_id              TEXT NOT NULL REFERENCES companies(id),
  secret_id               TEXT NOT NULL,
  target_type             TEXT NOT NULL,
  target_id               TEXT NOT NULL,
  config_path             TEXT NOT NULL,
  version_selector        TEXT NOT NULL DEFAULT 'latest',
  required                INTEGER NOT NULL DEFAULT 1,
  label                   TEXT,
  projection_class        TEXT NOT NULL DEFAULT 'unclassified',
  projection_allowlist_key TEXT,
  created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, target_type, target_id, config_path),
  FOREIGN KEY (company_id, secret_id) REFERENCES company_secrets (company_id, id) ON DELETE CASCADE
);

CREATE INDEX company_secret_bindings_company_idx
  ON company_secret_bindings (company_id);
CREATE INDEX company_secret_bindings_secret_idx
  ON company_secret_bindings (secret_id);
CREATE INDEX company_secret_bindings_target_idx
  ON company_secret_bindings (company_id, target_type, target_id);

-- secret_access_events (upstream secret_access_events.ts).
-- heartbeat_runs must expose UNIQUE (company_id, id) for the composite FK.
CREATE UNIQUE INDEX idx_heartbeat_runs_company_id_uq ON heartbeat_runs (company_id, id);

CREATE TABLE secret_access_events (
  id                        TEXT PRIMARY KEY,
  company_id                TEXT NOT NULL REFERENCES companies(id),
  secret_id                 TEXT,
  user_secret_definition_id TEXT,
  secret_scope              TEXT NOT NULL DEFAULT 'company',
  version                   INTEGER,
  provider                  TEXT NOT NULL,
  responsible_user_id       TEXT,
  credential_owner_user_id  TEXT,
  credential_subject_type   TEXT,
  credential_subject_id     TEXT,
  actor_type                TEXT NOT NULL,
  actor_id                  TEXT,
  consumer_type             TEXT NOT NULL,
  consumer_id               TEXT NOT NULL,
  config_path               TEXT,
  issue_id                  TEXT,
  heartbeat_run_id          TEXT,
  plugin_id                 TEXT,
  outcome                   TEXT NOT NULL,
  error_code                TEXT,
  created_at                TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, secret_id) REFERENCES company_secrets (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, user_secret_definition_id) REFERENCES user_secret_definitions (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, heartbeat_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (plugin_id) REFERENCES plugins (id) ON DELETE SET NULL
);

CREATE INDEX secret_access_events_company_created_idx
  ON secret_access_events (company_id, created_at);
CREATE INDEX secret_access_events_secret_created_idx
  ON secret_access_events (secret_id, created_at);
CREATE INDEX secret_access_events_user_definition_created_idx
  ON secret_access_events (user_secret_definition_id, created_at);
CREATE INDEX secret_access_events_company_credential_owner_idx
  ON secret_access_events (company_id, credential_owner_user_id, created_at);
CREATE INDEX secret_access_events_consumer_idx
  ON secret_access_events (company_id, consumer_type, consumer_id);
CREATE INDEX secret_access_events_run_idx
  ON secret_access_events (heartbeat_run_id);

-- ---------------------------------------------------------------------------
-- Existing table diffs
-- ---------------------------------------------------------------------------

-- company_skills: add missing upstream columns (company_skills.ts).
ALTER TABLE company_skills ADD COLUMN folder_id TEXT;
ALTER TABLE company_skills ADD COLUMN key TEXT NOT NULL DEFAULT '';
ALTER TABLE company_skills ADD COLUMN slug TEXT NOT NULL DEFAULT '';
ALTER TABLE company_skills ADD COLUMN markdown TEXT NOT NULL DEFAULT '';
ALTER TABLE company_skills ADD COLUMN source_type TEXT NOT NULL DEFAULT 'local_path';
ALTER TABLE company_skills ADD COLUMN source_locator TEXT;
ALTER TABLE company_skills ADD COLUMN source_ref TEXT;
ALTER TABLE company_skills ADD COLUMN trust_level TEXT NOT NULL DEFAULT 'markdown_only';
ALTER TABLE company_skills ADD COLUMN compatibility TEXT NOT NULL DEFAULT 'compatible';
ALTER TABLE company_skills ADD COLUMN file_inventory TEXT NOT NULL DEFAULT '[]';
ALTER TABLE company_skills ADD COLUMN icon_url TEXT;
ALTER TABLE company_skills ADD COLUMN color TEXT;
ALTER TABLE company_skills ADD COLUMN tagline TEXT;
ALTER TABLE company_skills ADD COLUMN author_name TEXT;
ALTER TABLE company_skills ADD COLUMN homepage_url TEXT;
ALTER TABLE company_skills ADD COLUMN categories TEXT NOT NULL DEFAULT '[]';
ALTER TABLE company_skills ADD COLUMN sharing_scope TEXT NOT NULL DEFAULT 'company';
ALTER TABLE company_skills ADD COLUMN public_share_token TEXT;
ALTER TABLE company_skills ADD COLUMN forked_from_skill_id TEXT REFERENCES company_skills(id) ON DELETE SET NULL;
ALTER TABLE company_skills ADD COLUMN forked_from_company_id TEXT REFERENCES companies(id) ON DELETE SET NULL;
ALTER TABLE company_skills ADD COLUMN star_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE company_skills ADD COLUMN install_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE company_skills ADD COLUMN fork_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE company_skills ADD COLUMN current_version_id TEXT REFERENCES company_skill_versions(id) ON DELETE SET NULL;
ALTER TABLE company_skills ADD COLUMN metadata TEXT;

CREATE UNIQUE INDEX company_skills_company_key_uq ON company_skills (company_id, key);
CREATE INDEX company_skills_company_name_idx ON company_skills (company_id, name);
CREATE INDEX company_skills_company_folder_idx ON company_skills (company_id, folder_id);
CREATE INDEX company_skills_company_sharing_scope_idx ON company_skills (company_id, sharing_scope);
CREATE INDEX company_skills_company_current_version_idx ON company_skills (company_id, current_version_id);
CREATE INDEX company_skills_company_forked_from_idx ON company_skills (company_id, forked_from_skill_id);

-- company_secrets: add missing upstream columns (company_secrets.ts).
ALTER TABLE company_secrets ADD COLUMN user_secret_definition_id TEXT REFERENCES user_secret_definitions(id) ON DELETE SET NULL;
ALTER TABLE company_secrets ADD COLUMN key TEXT NOT NULL DEFAULT '';
ALTER TABLE company_secrets ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE company_secrets ADD COLUMN managed_mode TEXT NOT NULL DEFAULT 'paperclip_managed';
ALTER TABLE company_secrets ADD COLUMN external_ref TEXT;
ALTER TABLE company_secrets ADD COLUMN provider_config_id TEXT REFERENCES company_secret_provider_configs(id) ON DELETE SET NULL;
ALTER TABLE company_secrets ADD COLUMN provider_metadata TEXT;
ALTER TABLE company_secrets ADD COLUMN latest_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE company_secrets ADD COLUMN description TEXT;
ALTER TABLE company_secrets ADD COLUMN last_resolved_at TEXT;
ALTER TABLE company_secrets ADD COLUMN last_rotated_at TEXT;
ALTER TABLE company_secrets ADD COLUMN deleted_at TEXT;
ALTER TABLE company_secrets ADD COLUMN created_by_agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL;
ALTER TABLE company_secrets ADD COLUMN created_by_user_id TEXT;

CREATE INDEX company_secrets_company_idx ON company_secrets (company_id);
CREATE INDEX company_secrets_company_scope_idx ON company_secrets (company_id, scope);
CREATE INDEX company_secrets_company_owner_idx ON company_secrets (company_id, owner_user_id);
CREATE INDEX company_secrets_user_definition_owner_idx
  ON company_secrets (company_id, user_secret_definition_id, owner_user_id);
CREATE INDEX company_secrets_company_provider_idx ON company_secrets (company_id, provider);
CREATE INDEX company_secrets_provider_config_idx ON company_secrets (provider_config_id);
CREATE UNIQUE INDEX company_secrets_company_name_uq
  ON company_secrets (company_id, name)
  WHERE scope = 'company' AND deleted_at IS NULL;
CREATE UNIQUE INDEX company_secrets_company_key_uq
  ON company_secrets (company_id, key)
  WHERE scope = 'company' AND deleted_at IS NULL;
CREATE UNIQUE INDEX company_secrets_user_definition_owner_uq
  ON company_secrets (company_id, user_secret_definition_id, owner_user_id)
  WHERE scope = 'user' AND deleted_at IS NULL;

-- company_secret_versions: add missing upstream columns
-- (company_secret_versions.ts).
ALTER TABLE company_secret_versions ADD COLUMN material TEXT NOT NULL DEFAULT '{}';
ALTER TABLE company_secret_versions ADD COLUMN value_sha256 TEXT NOT NULL DEFAULT '';
ALTER TABLE company_secret_versions ADD COLUMN provider_version_ref TEXT;
ALTER TABLE company_secret_versions ADD COLUMN status TEXT NOT NULL DEFAULT 'current';
ALTER TABLE company_secret_versions ADD COLUMN fingerprint_sha256 TEXT NOT NULL DEFAULT '';
ALTER TABLE company_secret_versions ADD COLUMN rotation_job_id TEXT;
ALTER TABLE company_secret_versions ADD COLUMN created_by_agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL;
ALTER TABLE company_secret_versions ADD COLUMN created_by_user_id TEXT;
ALTER TABLE company_secret_versions ADD COLUMN revoked_at TEXT;

CREATE INDEX company_secret_versions_secret_idx
  ON company_secret_versions (secret_id, created_at);
CREATE INDEX company_secret_versions_value_sha256_idx
  ON company_secret_versions (value_sha256);
CREATE INDEX company_secret_versions_fingerprint_idx
  ON company_secret_versions (fingerprint_sha256);
