-- Pipelines core (upstream pipelines.ts family): pipelines, stages,
-- transitions, pipeline cases, and case events.

CREATE TABLE pipelines (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id),
  project_id         TEXT,
  key                TEXT NOT NULL,
  name               TEXT NOT NULL,
  description        TEXT,
  enforce_transitions INTEGER NOT NULL DEFAULT 0,
  created_by_user_id TEXT,
  created_by_agent_id TEXT,
  archived_at        TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, key),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_pipelines_company ON pipelines (company_id);
CREATE INDEX idx_pipelines_company_project ON pipelines (company_id, project_id);

CREATE TABLE pipeline_stages (
  id          TEXT PRIMARY KEY,
  company_id  TEXT NOT NULL REFERENCES companies(id),
  pipeline_id TEXT NOT NULL,
  key         TEXT NOT NULL,
  name        TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('working', 'review', 'done', 'cancelled')),
  position    INTEGER NOT NULL,
  config      TEXT NOT NULL DEFAULT '{}',
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, pipeline_id, key),
  UNIQUE (company_id, pipeline_id, position),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, pipeline_id) REFERENCES pipelines (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_pipeline_stages_pipeline_position
  ON pipeline_stages (company_id, pipeline_id, position);

CREATE TABLE pipeline_transitions (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  pipeline_id   TEXT NOT NULL,
  from_stage_id TEXT NOT NULL,
  to_stage_id   TEXT NOT NULL,
  label         TEXT,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, pipeline_id, from_stage_id, to_stage_id),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, pipeline_id) REFERENCES pipelines (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, from_stage_id) REFERENCES pipeline_stages (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, to_stage_id) REFERENCES pipeline_stages (company_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_pipeline_transitions_pipeline_from
  ON pipeline_transitions (company_id, pipeline_id, from_stage_id);
CREATE INDEX idx_pipeline_transitions_pipeline_to
  ON pipeline_transitions (company_id, pipeline_id, to_stage_id);

CREATE TABLE pipeline_cases (
  id                 TEXT PRIMARY KEY,
  company_id         TEXT NOT NULL REFERENCES companies(id),
  pipeline_id        TEXT NOT NULL,
  stage_id           TEXT NOT NULL,
  case_key           TEXT NOT NULL,
  title              TEXT NOT NULL,
  summary            TEXT,
  fields             TEXT NOT NULL DEFAULT '{}',
  workspace_ref      TEXT,
  parent_case_id     TEXT,
  version            INTEGER NOT NULL DEFAULT 1,
  lease_owner_type   TEXT CHECK (lease_owner_type IN ('user', 'agent')),
  lease_agent_id     TEXT,
  lease_user_id      TEXT,
  lease_token        TEXT,
  lease_expires_at   TEXT,
  terminal_kind      TEXT CHECK (terminal_kind IN ('done', 'cancelled')),
  terminal_at        TEXT,
  retired_at         TEXT,
  retired_reason     TEXT,
  child_count        INTEGER NOT NULL DEFAULT 0,
  terminal_child_count INTEGER NOT NULL DEFAULT 0,
  created_by_user_id TEXT,
  created_by_agent_id TEXT,
  created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, pipeline_id, case_key),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, pipeline_id) REFERENCES pipelines (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, stage_id) REFERENCES pipeline_stages (company_id, id),
  FOREIGN KEY (company_id, parent_case_id) REFERENCES pipeline_cases (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, lease_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, created_by_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_pipeline_cases_company ON pipeline_cases (company_id);
CREATE INDEX idx_pipeline_cases_pipeline_stage ON pipeline_cases (company_id, pipeline_id, stage_id);
CREATE INDEX idx_pipeline_cases_parent ON pipeline_cases (parent_case_id);
CREATE INDEX idx_pipeline_cases_retired ON pipeline_cases (company_id, retired_at);

CREATE TABLE pipeline_case_events (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  case_id       TEXT NOT NULL,
  type          TEXT NOT NULL
                CHECK (type IN ('ingested', 'updated', 'claimed', 'lease_released',
                                'lease_expired', 'transitioned', 'transition_forced',
                                'transition_suggested', 'suggestion_resolved',
                                'review_decided', 'conversation_opened', 'issue_linked',
                                'issue_unlinked', 'automation_executed', 'automation_failed',
                                'automation_retry_requested', 'automation_effects_retired',
                                'automation_retry_dispatched', 'blockers_set',
                                'blockers_resolved', 'children_terminal', 'upstream_drift',
                                'drift_acknowledged')),
  actor_type    TEXT NOT NULL CHECK (actor_type IN ('user', 'agent', 'system')),
  actor_user_id TEXT,
  actor_agent_id TEXT,
  run_id        TEXT,
  from_stage_id TEXT,
  to_stage_id   TEXT,
  payload       TEXT NOT NULL DEFAULT '{}',
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, case_id) REFERENCES pipeline_cases (company_id, id) ON DELETE CASCADE,
  FOREIGN KEY (company_id, actor_agent_id) REFERENCES agents (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, from_stage_id) REFERENCES pipeline_stages (company_id, id) ON DELETE SET NULL,
  FOREIGN KEY (company_id, to_stage_id) REFERENCES pipeline_stages (company_id, id) ON DELETE SET NULL
);

CREATE INDEX idx_pipeline_case_events_case_created
  ON pipeline_case_events (case_id, created_at);
CREATE INDEX idx_pipeline_case_events_company_case
  ON pipeline_case_events (company_id, case_id);
