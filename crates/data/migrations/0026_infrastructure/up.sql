-- Batch 6 of schema alignment: upstream infrastructure domain
-- (auth.ts, instance_settings.ts, folders.ts, issue watchdogs/holds,
--  heartbeat events, environment images/leases, user preferences).

-- document_revisions needs an explicit (company_id, id) unique target so
-- issue_plan_decompositions can use a company-scoped composite FK.
CREATE UNIQUE INDEX document_revisions_company_id_uq ON document_revisions (company_id, id);

-- Auth (upstream auth.ts: user, session, account, verification).
CREATE TABLE user (
  id             TEXT PRIMARY KEY,
  name           TEXT NOT NULL,
  email          TEXT NOT NULL,
  email_verified INTEGER NOT NULL DEFAULT 0,
  image          TEXT,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);

CREATE TABLE session (
  id         TEXT PRIMARY KEY,
  expires_at TEXT NOT NULL,
  token      TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  ip_address TEXT,
  user_agent TEXT,
  user_id    TEXT NOT NULL REFERENCES user(id) ON DELETE CASCADE
);

CREATE TABLE account (
  id                      TEXT PRIMARY KEY,
  account_id              TEXT NOT NULL,
  provider_id             TEXT NOT NULL,
  user_id                 TEXT NOT NULL REFERENCES user(id) ON DELETE CASCADE,
  access_token            TEXT,
  refresh_token           TEXT,
  id_token                TEXT,
  access_token_expires_at TEXT,
  refresh_token_expires_at TEXT,
  scope                   TEXT,
  password                TEXT,
  created_at              TEXT NOT NULL,
  updated_at              TEXT NOT NULL
);

CREATE TABLE verification (
  id         TEXT PRIMARY KEY,
  identifier TEXT NOT NULL,
  value      TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  created_at TEXT,
  updated_at TEXT
);

-- Instance settings (upstream instance_settings.ts).
CREATE TABLE instance_settings (
  id                    TEXT PRIMARY KEY,
  singleton_key         TEXT NOT NULL DEFAULT 'default',
  default_environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  general               TEXT NOT NULL DEFAULT '{}',
  experimental          TEXT NOT NULL DEFAULT '{}',
  created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE UNIQUE INDEX instance_settings_singleton_key_idx ON instance_settings (singleton_key);

-- Folders (upstream folders.ts).
CREATE TABLE folders (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  kind       TEXT NOT NULL,
  parent_id  TEXT REFERENCES folders(id) ON DELETE RESTRICT,
  name       TEXT NOT NULL,
  slug       TEXT NOT NULL,
  system_key TEXT,
  color      TEXT,
  position   INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_folders_company_kind_position
  ON folders (company_id, kind, position, name);
CREATE UNIQUE INDEX folders_company_kind_root_slug_uq
  ON folders (company_id, kind, slug) WHERE parent_id IS NULL;
CREATE UNIQUE INDEX folders_company_kind_parent_slug_uq
  ON folders (company_id, kind, parent_id, slug) WHERE parent_id IS NOT NULL;
CREATE UNIQUE INDEX folders_company_kind_system_key_uq
  ON folders (company_id, kind, system_key) WHERE system_key IS NOT NULL;
CREATE INDEX idx_folders_company_kind_parent_position
  ON folders (company_id, kind, parent_id, position, name);

-- Agent config revisions (upstream agent_config_revisions.ts).
CREATE TABLE agent_config_revisions (
  id                       TEXT PRIMARY KEY,
  company_id               TEXT NOT NULL REFERENCES companies(id),
  agent_id                 TEXT NOT NULL,
  created_by_agent_id      TEXT,
  created_by_user_id       TEXT,
  source                   TEXT NOT NULL DEFAULT 'patch',
  rolled_back_from_revision_id TEXT,
  changed_keys             TEXT NOT NULL DEFAULT '[]',
  before_config            TEXT NOT NULL,
  after_config             TEXT NOT NULL,
  created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (rolled_back_from_revision_id) REFERENCES agent_config_revisions (id)
);

CREATE INDEX idx_agent_config_revisions_company_agent_created
  ON agent_config_revisions (company_id, agent_id, created_at);
CREATE INDEX idx_agent_config_revisions_agent_created
  ON agent_config_revisions (agent_id, created_at);

-- Inbox dismissals (upstream inbox_dismissals.ts).
CREATE TABLE inbox_dismissals (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  user_id       TEXT NOT NULL,
  item_key      TEXT NOT NULL,
  kind          TEXT NOT NULL DEFAULT 'dismiss',
  dismissed_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  snoozed_until TEXT,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, user_id, item_key)
);

CREATE INDEX idx_inbox_dismissals_company_user ON inbox_dismissals (company_id, user_id);
CREATE INDEX idx_inbox_dismissals_company_item ON inbox_dismissals (company_id, item_key);

-- Project goals (upstream project_goals.ts).
CREATE TABLE project_goals (
  project_id TEXT NOT NULL,
  goal_id    TEXT NOT NULL,
  company_id TEXT NOT NULL REFERENCES companies(id),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  PRIMARY KEY (project_id, goal_id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, goal_id) REFERENCES goals (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_project_goals_project ON project_goals (project_id);
CREATE INDEX idx_project_goals_goal ON project_goals (goal_id);
CREATE INDEX idx_project_goals_company ON project_goals (company_id);

-- Document memberships (upstream document_memberships.ts).
CREATE TABLE document_memberships (
  id          TEXT PRIMARY KEY,
  company_id  TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  document_id TEXT NOT NULL,
  user_id     TEXT NOT NULL,
  starred_at  TEXT,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, user_id, document_id),
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_document_memberships_company_user_starred
  ON document_memberships (company_id, user_id, starred_at);

-- Routine documents (upstream routine_documents.ts).
CREATE TABLE routine_documents (
  id          TEXT PRIMARY KEY,
  company_id  TEXT NOT NULL REFERENCES companies(id),
  routine_id  TEXT NOT NULL,
  document_id TEXT NOT NULL,
  key         TEXT NOT NULL,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, routine_id, key),
  UNIQUE (document_id),
  FOREIGN KEY (company_id, routine_id) REFERENCES routines (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, document_id) REFERENCES documents (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_routine_documents_company_routine_updated
  ON routine_documents (company_id, routine_id, updated_at);

-- Approval comments (upstream approval_comments.ts).
CREATE TABLE approval_comments (
  id             TEXT PRIMARY KEY,
  company_id     TEXT NOT NULL REFERENCES companies(id),
  approval_id    TEXT NOT NULL,
  author_agent_id TEXT,
  author_user_id TEXT,
  body           TEXT NOT NULL,
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, approval_id) REFERENCES approvals (company_id, id),
  FOREIGN KEY (company_id, author_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_approval_comments_company ON approval_comments (company_id);
CREATE INDEX idx_approval_comments_approval ON approval_comments (approval_id);
CREATE INDEX idx_approval_comments_approval_created
  ON approval_comments (approval_id, created_at);

-- Built-in managed resources (upstream built_in_managed_resources.ts).
CREATE TABLE built_in_managed_resources (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  bundle_key    TEXT NOT NULL,
  resource_kind TEXT NOT NULL,
  resource_key  TEXT NOT NULL,
  resource_id   TEXT NOT NULL,
  stock_version TEXT NOT NULL,
  stock_hash    TEXT NOT NULL,
  defaults_json TEXT NOT NULL DEFAULT '{}',
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, bundle_key, resource_kind, resource_key)
);

CREATE INDEX idx_built_in_managed_resources_company ON built_in_managed_resources (company_id);
CREATE INDEX idx_built_in_managed_resources_resource
  ON built_in_managed_resources (resource_kind, resource_id);

-- Issue create idempotency keys (upstream issue_create_idempotency_keys.ts).
CREATE TABLE issue_create_idempotency_keys (
  id              TEXT PRIMARY KEY,
  company_id      TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  idempotency_key TEXT NOT NULL,
  issue_id        TEXT NOT NULL,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, idempotency_key),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_issue_create_idempotency_keys_issue
  ON issue_create_idempotency_keys (issue_id);
CREATE INDEX idx_issue_create_idempotency_keys_company_created
  ON issue_create_idempotency_keys (company_id, created_at);

-- Issue inbox archives (upstream issue_inbox_archives.ts).
CREATE TABLE issue_inbox_archives (
  id                    TEXT PRIMARY KEY,
  company_id            TEXT NOT NULL REFERENCES companies(id),
  issue_id              TEXT NOT NULL,
  user_id               TEXT NOT NULL,
  archived_by_actor_type TEXT NOT NULL DEFAULT 'user'
                         CHECK (archived_by_actor_type IN ('user', 'agent')),
  archived_by_agent_id  TEXT,
  archived_by_run_id    TEXT,
  archived_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, issue_id, user_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, archived_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, archived_by_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_issue_inbox_archives_company_issue
  ON issue_inbox_archives (company_id, issue_id);
CREATE INDEX idx_issue_inbox_archives_company_user
  ON issue_inbox_archives (company_id, user_id);

-- Issue plan decompositions (upstream issue_plan_decompositions.ts).
CREATE TABLE issue_plan_decompositions (
  id                        TEXT PRIMARY KEY,
  company_id                TEXT NOT NULL REFERENCES companies(id),
  source_issue_id           TEXT NOT NULL,
  accepted_plan_revision_id TEXT NOT NULL,
  accepted_interaction_id   TEXT,
  status                    TEXT NOT NULL DEFAULT 'in_flight',
  request_fingerprint       TEXT NOT NULL,
  requested_child_count     INTEGER NOT NULL DEFAULT 0,
  requested_children        TEXT NOT NULL DEFAULT '[]',
  child_issue_ids           TEXT NOT NULL DEFAULT '[]',
  owner_agent_id            TEXT,
  owner_user_id             TEXT,
  owner_run_id              TEXT,
  completed_at              TEXT,
  created_at                TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, source_issue_id, accepted_plan_revision_id),
  FOREIGN KEY (company_id, source_issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, accepted_plan_revision_id) REFERENCES document_revisions (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, accepted_interaction_id) REFERENCES issue_thread_interactions (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, owner_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, owner_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_issue_plan_decompositions_company_source_status
  ON issue_plan_decompositions (company_id, source_issue_id, status);
CREATE INDEX idx_issue_plan_decompositions_active_owner
  ON issue_plan_decompositions (company_id, owner_agent_id) WHERE status = 'in_flight';

-- Issue reference mentions (upstream issue_reference_mentions.ts).
CREATE TABLE issue_reference_mentions (
  id              TEXT PRIMARY KEY,
  company_id      TEXT NOT NULL REFERENCES companies(id),
  source_issue_id TEXT NOT NULL,
  target_issue_id TEXT NOT NULL,
  source_kind     TEXT NOT NULL CHECK (source_kind IN ('title', 'description', 'comment', 'document')),
  source_record_id TEXT,
  document_key    TEXT,
  matched_text    TEXT,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, source_issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, target_issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_issue_reference_mentions_company_source_issue
  ON issue_reference_mentions (company_id, source_issue_id);
CREATE INDEX idx_issue_reference_mentions_company_target_issue
  ON issue_reference_mentions (company_id, target_issue_id);
CREATE INDEX idx_issue_reference_mentions_company_issue_pair
  ON issue_reference_mentions (company_id, source_issue_id, target_issue_id);
CREATE UNIQUE INDEX issue_reference_mentions_company_source_mention_record_uq
  ON issue_reference_mentions (company_id, source_issue_id, target_issue_id, source_kind, source_record_id)
  WHERE source_record_id IS NOT NULL;
CREATE UNIQUE INDEX issue_reference_mentions_company_source_mention_null_record_uq
  ON issue_reference_mentions (company_id, source_issue_id, target_issue_id, source_kind)
  WHERE source_record_id IS NULL;

-- Issue tree holds (upstream issue_tree_holds.ts).
CREATE TABLE issue_tree_holds (
  id                      TEXT PRIMARY KEY,
  company_id              TEXT NOT NULL REFERENCES companies(id),
  root_issue_id           TEXT NOT NULL,
  mode                    TEXT NOT NULL,
  status                  TEXT NOT NULL DEFAULT 'active',
  reason                  TEXT,
  release_policy          TEXT,
  created_by_actor_type   TEXT NOT NULL DEFAULT 'system',
  created_by_agent_id     TEXT,
  created_by_user_id      TEXT,
  created_by_run_id       TEXT,
  released_at             TEXT,
  released_by_actor_type  TEXT,
  released_by_agent_id    TEXT,
  released_by_user_id     TEXT,
  released_by_run_id      TEXT,
  release_reason          TEXT,
  release_metadata        TEXT,
  created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, root_issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, released_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, released_by_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_issue_tree_holds_company_root_status
  ON issue_tree_holds (company_id, root_issue_id, status);
CREATE INDEX idx_issue_tree_holds_company_status_mode
  ON issue_tree_holds (company_id, status, mode);

-- Issue tree hold members (upstream issue_tree_hold_members.ts).
CREATE TABLE issue_tree_hold_members (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  hold_id          TEXT NOT NULL,
  issue_id         TEXT NOT NULL,
  parent_issue_id  TEXT,
  depth            INTEGER NOT NULL DEFAULT 0,
  issue_identifier TEXT,
  issue_title      TEXT NOT NULL,
  issue_status     TEXT NOT NULL,
  assignee_agent_id TEXT,
  assignee_user_id TEXT,
  active_run_id    TEXT,
  active_run_status TEXT,
  skipped          INTEGER NOT NULL DEFAULT 0,
  skip_reason      TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (hold_id, issue_id),
  FOREIGN KEY (company_id, hold_id) REFERENCES issue_tree_holds (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, parent_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, assignee_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, active_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_issue_tree_hold_members_company_issue
  ON issue_tree_hold_members (company_id, issue_id);
CREATE INDEX idx_issue_tree_hold_members_hold_depth
  ON issue_tree_hold_members (hold_id, depth);

-- Issue watchdogs (upstream issue_watchdogs.ts).
CREATE TABLE issue_watchdogs (
  id                         TEXT PRIMARY KEY,
  company_id                 TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  issue_id                   TEXT NOT NULL,
  watchdog_agent_id          TEXT NOT NULL,
  instructions               TEXT,
  status                     TEXT NOT NULL DEFAULT 'active',
  watchdog_issue_id          TEXT,
  last_observed_fingerprint  TEXT,
  last_reviewed_fingerprint  TEXT,
  last_observed_stop_snapshot TEXT,
  last_reviewed_stop_snapshot TEXT,
  last_triggered_at          TEXT,
  last_completed_at          TEXT,
  trigger_count              INTEGER NOT NULL DEFAULT 0,
  created_by_agent_id        TEXT,
  created_by_user_id         TEXT,
  created_by_run_id          TEXT,
  updated_by_agent_id        TEXT,
  updated_by_user_id         TEXT,
  updated_by_run_id          TEXT,
  created_at                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, issue_id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, watchdog_agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, watchdog_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, updated_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, updated_by_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_issue_watchdogs_company_status ON issue_watchdogs (company_id, status);
CREATE INDEX idx_issue_watchdogs_company_agent ON issue_watchdogs (company_id, watchdog_agent_id);
CREATE UNIQUE INDEX issue_watchdogs_company_watchdog_issue_uq
  ON issue_watchdogs (company_id, watchdog_issue_id) WHERE watchdog_issue_id IS NOT NULL;

-- Heartbeat run events (upstream heartbeat_run_events.ts).
CREATE TABLE heartbeat_run_events (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  company_id TEXT NOT NULL REFERENCES companies(id),
  run_id     TEXT NOT NULL,
  agent_id   TEXT NOT NULL,
  seq        INTEGER NOT NULL,
  event_type TEXT NOT NULL,
  stream     TEXT,
  level      TEXT,
  color      TEXT,
  message    TEXT,
  payload    TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  FOREIGN KEY (company_id, run_id) REFERENCES heartbeat_runs (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_heartbeat_run_events_run_seq ON heartbeat_run_events (run_id, seq);
CREATE INDEX idx_heartbeat_run_events_company_run
  ON heartbeat_run_events (company_id, run_id);
CREATE INDEX idx_heartbeat_run_events_company_created
  ON heartbeat_run_events (company_id, created_at);

-- Heartbeat run watchdog decisions (upstream heartbeat_run_watchdog_decisions.ts).
CREATE TABLE heartbeat_run_watchdog_decisions (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id),
  run_id             TEXT NOT NULL,
  evaluation_issue_id TEXT,
  decision           TEXT NOT NULL,
  snoozed_until      TEXT,
  reason             TEXT,
  created_by_agent_id TEXT,
  created_by_user_id TEXT,
  created_by_run_id  TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, evaluation_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_heartbeat_run_watchdog_decisions_company_run_created
  ON heartbeat_run_watchdog_decisions (company_id, run_id, created_at);
CREATE INDEX idx_heartbeat_run_watchdog_decisions_company_run_snooze
  ON heartbeat_run_watchdog_decisions (company_id, run_id, snoozed_until);

-- Environment custom image templates (upstream environment_custom_image_templates.ts).
CREATE TABLE environment_custom_image_templates (
  id                                 TEXT PRIMARY KEY,
  environment_id                     TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
  provider                           TEXT NOT NULL,
  template_kind                      TEXT NOT NULL DEFAULT 'unknown',
  template_ref                       TEXT NOT NULL,
  source_template_ref                TEXT,
  source_environment_config_fingerprint TEXT,
  status                             TEXT NOT NULL DEFAULT 'active',
  created_by_user_id                 TEXT,
  created_by_agent_id                TEXT REFERENCES agents(id) ON DELETE SET NULL,
  captured_at                        TEXT,
  last_used_at                       TEXT,
  superseded_by_template_id          TEXT REFERENCES environment_custom_image_templates(id) ON DELETE SET NULL,
  metadata                           TEXT,
  created_at                         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at                         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_env_custom_image_templates_environment_status
  ON environment_custom_image_templates (environment_id, status);
CREATE INDEX idx_env_custom_image_templates_env_provider_status
  ON environment_custom_image_templates (environment_id, provider, status);
CREATE UNIQUE INDEX environment_custom_image_templates_environment_active_uq
  ON environment_custom_image_templates (environment_id) WHERE status = 'active';
CREATE INDEX idx_env_custom_image_templates_superseded_by
  ON environment_custom_image_templates (superseded_by_template_id);
CREATE INDEX idx_env_custom_image_templates_last_used
  ON environment_custom_image_templates (last_used_at);

-- Environment leases (upstream environment_leases.ts).
CREATE TABLE environment_leases (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  environment_id         TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
  execution_workspace_id TEXT,
  issue_id               TEXT,
  heartbeat_run_id       TEXT,
  status                 TEXT NOT NULL DEFAULT 'active',
  lease_policy           TEXT NOT NULL DEFAULT 'ephemeral',
  provider               TEXT,
  provider_lease_id      TEXT,
  acquired_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  last_used_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  expires_at             TEXT,
  released_at            TEXT,
  failure_reason         TEXT,
  cleanup_status         TEXT,
  metadata               TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, execution_workspace_id) REFERENCES execution_workspaces (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, heartbeat_run_id) REFERENCES heartbeat_runs (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_environment_leases_company_environment_status
  ON environment_leases (company_id, environment_id, status);
CREATE INDEX idx_environment_leases_company_execution_workspace
  ON environment_leases (company_id, execution_workspace_id);
CREATE INDEX idx_environment_leases_company_issue
  ON environment_leases (company_id, issue_id);
CREATE INDEX idx_environment_leases_heartbeat_run
  ON environment_leases (heartbeat_run_id);
CREATE INDEX idx_environment_leases_company_last_used
  ON environment_leases (company_id, last_used_at);
CREATE INDEX idx_environment_leases_provider_lease
  ON environment_leases (provider_lease_id);

-- Environment custom image setup sessions (upstream
-- environment_custom_image_setup_sessions.ts).
CREATE TABLE environment_custom_image_setup_sessions (
  id                  TEXT PRIMARY KEY,
  environment_id      TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
  template_id         TEXT REFERENCES environment_custom_image_templates(id) ON DELETE SET NULL,
  promoted_template_id TEXT REFERENCES environment_custom_image_templates(id) ON DELETE SET NULL,
  provider            TEXT NOT NULL,
  provider_lease_id   TEXT,
  environment_lease_id TEXT REFERENCES environment_leases(id) ON DELETE SET NULL,
  status              TEXT NOT NULL DEFAULT 'starting',
  started_by_user_id  TEXT,
  started_by_agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
  base_template_ref   TEXT,
  expires_at          TEXT,
  finished_at         TEXT,
  failure_reason      TEXT,
  connection_summary  TEXT,
  connection_secret_ref TEXT,
  metadata            TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_env_setup_sessions_environment_status
  ON environment_custom_image_setup_sessions (environment_id, status);
CREATE UNIQUE INDEX environment_custom_image_setup_sessions_environment_active_uq
  ON environment_custom_image_setup_sessions (environment_id)
  WHERE status IN ('starting', 'waiting_for_user', 'capturing');
CREATE INDEX idx_env_setup_sessions_template
  ON environment_custom_image_setup_sessions (template_id);
CREATE INDEX idx_env_setup_sessions_promoted_template
  ON environment_custom_image_setup_sessions (promoted_template_id);
CREATE INDEX idx_env_setup_sessions_expires
  ON environment_custom_image_setup_sessions (expires_at);
CREATE INDEX idx_env_setup_sessions_provider_lease
  ON environment_custom_image_setup_sessions (provider, provider_lease_id);

-- User inbox agent policies (upstream user_inbox_agent_policies.ts).
CREATE TABLE user_inbox_agent_policies (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
  user_id          TEXT NOT NULL,
  mode             TEXT NOT NULL DEFAULT 'open'
                   CHECK (mode IN ('open', 'allowlist', 'disabled')),
  allowed_agent_ids TEXT NOT NULL DEFAULT '[]',
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, user_id)
);

-- User sidebar preferences (upstream user_sidebar_preferences.ts).
CREATE TABLE user_sidebar_preferences (
  id            TEXT PRIMARY KEY,
  user_id       TEXT NOT NULL,
  company_order TEXT NOT NULL DEFAULT '[]',
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (user_id)
);
