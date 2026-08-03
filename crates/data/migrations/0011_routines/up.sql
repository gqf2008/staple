-- Routines: routine definitions, append-only revisions, triggers, and runs
-- (upstream §7.16 addenda; schema follows routines.ts plus the revision/
-- trigger/run extension tables).

CREATE TABLE routines (
  id                     TEXT PRIMARY KEY,
  company_id             TEXT NOT NULL REFERENCES companies(id),
  project_id             TEXT,
  goal_id                TEXT,
  parent_issue_id        TEXT,
  title                  TEXT NOT NULL,
  description            TEXT,
  assignee_agent_id      TEXT,
  priority               TEXT NOT NULL DEFAULT 'medium',
  status                 TEXT NOT NULL DEFAULT 'active',
  concurrency_policy     TEXT NOT NULL DEFAULT 'coalesce_if_active',
  catch_up_policy        TEXT NOT NULL DEFAULT 'skip_missed',
  activity_gate_policy   TEXT NOT NULL DEFAULT 'always',
  activity_gate_scope    TEXT NOT NULL DEFAULT 'company',
  origin_kind            TEXT NOT NULL DEFAULT 'manual',
  origin_id              TEXT,
  variables              TEXT NOT NULL DEFAULT '[]',
  env                    TEXT,
  latest_revision_id     TEXT,
  latest_revision_number INTEGER NOT NULL DEFAULT 1,
  created_by_agent_id    TEXT,
  created_by_user_id     TEXT,
  responsible_user_id    TEXT,
  last_triggered_at      TEXT,
  last_enqueued_at       TEXT,
  created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, project_id) REFERENCES projects (company_id, id),
  FOREIGN KEY (company_id, goal_id) REFERENCES goals (company_id, id),
  FOREIGN KEY (company_id, parent_issue_id) REFERENCES issues (company_id, id),
  FOREIGN KEY (company_id, assignee_agent_id) REFERENCES agents (company_id, id)
);

CREATE INDEX idx_routines_company_status ON routines (company_id, status);
CREATE INDEX idx_routines_company_assignee ON routines (company_id, assignee_agent_id);

CREATE TABLE routine_revisions (
  id               TEXT PRIMARY KEY,
  company_id       TEXT NOT NULL REFERENCES companies(id),
  routine_id       TEXT NOT NULL,
  revision_number  INTEGER NOT NULL,
  title            TEXT NOT NULL,
  description      TEXT,
  assignee_agent_id TEXT,
  priority         TEXT NOT NULL DEFAULT 'medium',
  variables        TEXT NOT NULL DEFAULT '[]',
  env              TEXT,
  created_by_agent_id TEXT,
  created_by_user_id  TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, routine_id, revision_number),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, routine_id) REFERENCES routines (company_id, id)
);

CREATE TABLE routine_triggers (
  id            TEXT PRIMARY KEY,
  company_id    TEXT NOT NULL REFERENCES companies(id),
  routine_id    TEXT NOT NULL,
  schedule_kind TEXT NOT NULL DEFAULT 'manual'
                CHECK (schedule_kind IN ('manual', 'cron', 'webhook')),
  schedule_expr TEXT,
  enabled       INTEGER NOT NULL DEFAULT 1,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, routine_id, schedule_kind),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, routine_id) REFERENCES routines (company_id, id)
);

CREATE TABLE routine_runs (
  id           TEXT PRIMARY KEY,
  company_id   TEXT NOT NULL REFERENCES companies(id),
  routine_id   TEXT NOT NULL,
  revision_id  TEXT,
  status       TEXT NOT NULL DEFAULT 'queued'
               CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
  triggered_by TEXT,
  issue_id     TEXT,
  error        TEXT,
  started_at   TEXT,
  finished_at  TEXT,
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  UNIQUE (company_id, id),
  FOREIGN KEY (company_id, routine_id) REFERENCES routines (company_id, id),
  FOREIGN KEY (company_id, revision_id) REFERENCES routine_revisions (company_id, id),
  FOREIGN KEY (company_id, issue_id) REFERENCES issues (company_id, id)
);

CREATE INDEX idx_routine_runs_company_routine ON routine_runs (company_id, routine_id, created_at);
