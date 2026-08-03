-- Access & operations: memberships, instance roles, invites/join requests,
-- board API keys + CLI auth challenges, budget policies/incidents, sidebar
-- preferences, and company logos (upstream schema parity).

CREATE TABLE company_memberships (
  id              TEXT PRIMARY KEY,
  company_id      TEXT NOT NULL REFERENCES companies(id),
  principal_type  TEXT NOT NULL CHECK (principal_type IN ('agent', 'user')),
  principal_id    TEXT NOT NULL,
  status          TEXT NOT NULL DEFAULT 'active'
                  CHECK (status IN ('active', 'inactive', 'pending', 'removed')),
  membership_role TEXT CHECK (membership_role IN ('owner', 'admin', 'operator', 'viewer')),
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, principal_type, principal_id),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_company_memberships_company_status
  ON company_memberships (company_id, status);
CREATE INDEX idx_company_memberships_principal_status
  ON company_memberships (principal_type, principal_id, status);

CREATE TABLE instance_user_roles (
  id         TEXT PRIMARY KEY,
  user_id    TEXT NOT NULL,
  role       TEXT NOT NULL DEFAULT 'instance_admin'
             CHECK (role IN ('instance_admin')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (user_id, role)
);

CREATE INDEX idx_instance_user_roles_role ON instance_user_roles (role);

CREATE TABLE invites (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id),
  invite_type        TEXT NOT NULL DEFAULT 'company_join'
                     CHECK (invite_type IN ('company_join', 'bootstrap_ceo')),
  token_hash         TEXT NOT NULL,
  allowed_join_types TEXT NOT NULL DEFAULT 'both'
                      CHECK (allowed_join_types IN ('human', 'agent', 'both')),
  defaults_payload   TEXT,
  expires_at         TEXT NOT NULL,
  invited_by_user_id TEXT,
  revoked_at         TEXT,
  accepted_at        TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (token_hash),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_invites_company_state
  ON invites (company_id, invite_type, revoked_at, expires_at);

CREATE TABLE join_requests (
  id                    TEXT PRIMARY KEY,
  invite_id             TEXT NOT NULL,
  company_id            TEXT NOT NULL REFERENCES companies(id),
  request_type          TEXT NOT NULL CHECK (request_type IN ('human', 'agent')),
  status                TEXT NOT NULL DEFAULT 'pending_approval'
                        CHECK (status IN ('pending_approval', 'approved', 'rejected',
                                          'expired', 'cancelled')),
  request_ip            TEXT NOT NULL DEFAULT '',
  requesting_user_id    TEXT,
  request_email_snapshot TEXT,
  agent_name            TEXT,
  adapter_type          TEXT,
  capabilities          TEXT,
  agent_defaults_payload TEXT,
  claim_secret_hash     TEXT,
  claim_secret_expires_at TEXT,
  claim_secret_consumed_at TEXT,
  created_agent_id      TEXT,
  approved_by_user_id   TEXT,
  approved_at           TEXT,
  rejected_by_user_id   TEXT,
  rejected_at           TEXT,
  created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (invite_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, invite_id) REFERENCES invites (company_id, id)
);

CREATE INDEX idx_join_requests_company_status
  ON join_requests (company_id, status, request_type, created_at);

CREATE TABLE board_api_keys (
  id           TEXT PRIMARY KEY,
  user_id      TEXT NOT NULL,
  name         TEXT NOT NULL,
  key_hash     TEXT NOT NULL UNIQUE,
  last_used_at TEXT,
  revoked_at   TEXT,
  expires_at   TEXT,
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_board_api_keys_user ON board_api_keys (user_id);

CREATE TABLE cli_auth_challenges (
  id                   TEXT PRIMARY KEY,
  secret_hash          TEXT NOT NULL,
  command              TEXT NOT NULL,
  client_name          TEXT,
  requested_access     TEXT NOT NULL DEFAULT 'board',
  requested_company_id TEXT REFERENCES companies(id),
  pending_key_hash     TEXT NOT NULL,
  pending_key_name     TEXT NOT NULL,
  approved_by_user_id  TEXT,
  board_api_key_id     TEXT,
  approved_at          TEXT,
  cancelled_at         TEXT,
  expires_at           TEXT NOT NULL,
  created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX idx_cli_auth_challenges_secret ON cli_auth_challenges (secret_hash);
CREATE INDEX idx_cli_auth_challenges_company ON cli_auth_challenges (requested_company_id);

CREATE TABLE budget_policies (
  id                TEXT PRIMARY KEY,
  company_id        TEXT NOT NULL REFERENCES companies(id),
  scope_type        TEXT NOT NULL CHECK (scope_type IN ('company', 'agent', 'project')),
  scope_id          TEXT NOT NULL,
  metric            TEXT NOT NULL DEFAULT 'billed_cents',
  window_kind       TEXT NOT NULL CHECK (window_kind IN ('calendar_month_utc', 'rolling_30d')),
  amount            INTEGER NOT NULL DEFAULT 0,
  warn_percent      INTEGER NOT NULL DEFAULT 80,
  hard_stop_enabled INTEGER NOT NULL DEFAULT 1,
  notify_enabled    INTEGER NOT NULL DEFAULT 1,
  is_active         INTEGER NOT NULL DEFAULT 1,
  created_by_user_id TEXT,
  updated_by_user_id TEXT,
  created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, scope_type, scope_id, metric, window_kind),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_budget_policies_company_scope
  ON budget_policies (company_id, scope_type, scope_id, is_active);
CREATE INDEX idx_budget_policies_company_window
  ON budget_policies (company_id, window_kind, metric);

CREATE TABLE budget_incidents (
  id              TEXT PRIMARY KEY,
  company_id      TEXT NOT NULL REFERENCES companies(id),
  policy_id       TEXT NOT NULL,
  scope_type      TEXT NOT NULL,
  scope_id        TEXT NOT NULL,
  metric          TEXT NOT NULL,
  window_kind     TEXT NOT NULL,
  window_start    TEXT NOT NULL,
  window_end      TEXT NOT NULL,
  threshold_type  TEXT NOT NULL CHECK (threshold_type IN ('warn', 'hard_stop')),
  amount_limit    INTEGER NOT NULL,
  amount_observed INTEGER NOT NULL,
  status          TEXT NOT NULL DEFAULT 'open'
                  CHECK (status IN ('open', 'resolved', 'dismissed')),
  approval_id     TEXT,
  resolved_at     TEXT,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (policy_id, window_start, threshold_type),
  FOREIGN KEY (company_id, policy_id) REFERENCES budget_policies (company_id, id)
    ON DELETE CASCADE
);

CREATE INDEX idx_budget_incidents_company_status
  ON budget_incidents (company_id, status);
CREATE INDEX idx_budget_incidents_company_scope
  ON budget_incidents (company_id, scope_type, scope_id, status);

CREATE TABLE company_user_sidebar_preferences (
  id           TEXT PRIMARY KEY,
  company_id   TEXT NOT NULL REFERENCES companies(id),
  user_id      TEXT NOT NULL,
  project_order TEXT NOT NULL DEFAULT '[]',
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, user_id),
  UNIQUE (company_id, id)
);

CREATE INDEX idx_sidebar_prefs_company ON company_user_sidebar_preferences (company_id);
CREATE INDEX idx_sidebar_prefs_user ON company_user_sidebar_preferences (user_id);

CREATE TABLE company_logos (
  id         TEXT PRIMARY KEY,
  company_id TEXT NOT NULL REFERENCES companies(id),
  asset_id   TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id),
  UNIQUE (asset_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, asset_id) REFERENCES assets (company_id, id)
);
