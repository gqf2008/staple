-- Agent runtime: task sessions, runtime state, wakeup requests, and issue
-- recovery actions (upstream agent_task_sessions.ts / agent_runtime_state.ts /
-- agent_wakeup_requests.ts / issue_recovery_actions.ts).

-- Parent-table composite uniqueness needed by the new composite FKs below.
CREATE UNIQUE INDEX idx_heartbeat_runs_company_id ON heartbeat_runs (company_id, id);

CREATE TABLE agent_task_sessions (
  id                  TEXT PRIMARY KEY,
  company_id          TEXT NOT NULL REFERENCES companies(id),
  agent_id            TEXT NOT NULL,
  adapter_type        TEXT NOT NULL,
  task_key            TEXT NOT NULL,
  session_params_json TEXT,
  session_display_id  TEXT,
  last_run_id         TEXT,
  last_error          TEXT,
  created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, agent_id, adapter_type, task_key),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id),
  FOREIGN KEY (company_id, last_run_id) REFERENCES heartbeat_runs (company_id, id)
);

CREATE INDEX idx_agent_task_sessions_company_agent_updated
  ON agent_task_sessions (company_id, agent_id, updated_at);
CREATE INDEX idx_agent_task_sessions_company_task_updated
  ON agent_task_sessions (company_id, task_key, updated_at);

CREATE TABLE agent_runtime_state (
  agent_id               TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  adapter_type           TEXT NOT NULL,
  session_id             TEXT,
  state_json             TEXT NOT NULL DEFAULT '{}',
  last_run_id            TEXT,
  last_run_status        TEXT,
  total_input_tokens     INTEGER NOT NULL DEFAULT 0,
  total_output_tokens    INTEGER NOT NULL DEFAULT 0,
  total_cached_input_tokens INTEGER NOT NULL DEFAULT 0,
  total_cost_cents       INTEGER NOT NULL DEFAULT 0,
  last_error             TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, agent_id),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_agent_runtime_state_company_updated
  ON agent_runtime_state (company_id, updated_at);

CREATE TABLE agent_wakeup_requests (
  id                       TEXT PRIMARY KEY,
  company_id               TEXT NOT NULL REFERENCES companies(id),
  agent_id                 TEXT NOT NULL,
  source                   TEXT NOT NULL,
  trigger_detail           TEXT,
  reason                   TEXT,
  payload                  TEXT,
  status                   TEXT NOT NULL DEFAULT 'queued'
                           CHECK (status IN ('queued', 'claimed', 'finished', 'failed')),
  coalesced_count          INTEGER NOT NULL DEFAULT 0,
  requested_by_actor_type  TEXT,
  requested_by_actor_id    TEXT,
  idempotency_key          TEXT,
  run_id                   TEXT,
  requested_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  claimed_at               TEXT,
  finished_at              TEXT,
  error                    TEXT,
  created_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  UNIQUE (company_id, agent_id, idempotency_key),
  FOREIGN KEY (company_id, agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_agent_wakeup_requests_company_agent_status
  ON agent_wakeup_requests (company_id, agent_id, status);
CREATE INDEX idx_agent_wakeup_requests_company_requested
  ON agent_wakeup_requests (company_id, requested_at);
CREATE INDEX idx_agent_wakeup_requests_agent_requested
  ON agent_wakeup_requests (agent_id, requested_at);

CREATE TABLE issue_recovery_actions (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  source_issue_id        TEXT NOT NULL,
  recovery_issue_id      TEXT,
  kind                   TEXT NOT NULL,
  status                 TEXT NOT NULL DEFAULT 'active'
                         CHECK (status IN ('active', 'escalated', 'resolved', 'cancelled')),
  owner_type             TEXT NOT NULL DEFAULT 'agent',
  owner_agent_id         TEXT,
  owner_user_id          TEXT,
  previous_owner_agent_id TEXT,
  return_owner_agent_id  TEXT,
  cause                  TEXT NOT NULL,
  fingerprint            TEXT NOT NULL,
  evidence               TEXT NOT NULL DEFAULT '{}',
  next_action            TEXT NOT NULL,
  wake_policy            TEXT,
  monitor_policy         TEXT,
  attempt_count          INTEGER NOT NULL DEFAULT 0,
  max_attempts           INTEGER,
  timeout_at             TEXT,
  last_attempt_at        TEXT,
  outcome                TEXT,
  resolution_note        TEXT,
  resolved_at            TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, source_issue_id) REFERENCES issues (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, recovery_issue_id) REFERENCES issues (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, owner_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, previous_owner_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, return_owner_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_issue_recovery_actions_company_source_status
  ON issue_recovery_actions (company_id, source_issue_id, status);
CREATE INDEX idx_issue_recovery_actions_company_owner_status
  ON issue_recovery_actions (company_id, owner_agent_id, status);
CREATE INDEX idx_issue_recovery_actions_company_recovery_issue
  ON issue_recovery_actions (company_id, recovery_issue_id);
CREATE UNIQUE INDEX idx_issue_recovery_actions_active_source_uq
  ON issue_recovery_actions (company_id, source_issue_id)
  WHERE status IN ('active', 'escalated');
CREATE UNIQUE INDEX idx_issue_recovery_actions_active_fingerprint_uq
  ON issue_recovery_actions (company_id, source_issue_id, cause, fingerprint)
  WHERE status IN ('active', 'escalated');
