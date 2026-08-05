-- Column-level parity batch 1: pipeline_cases, activity_log, decision desk,
-- rate-limit counters, memberships, documents/work products, workspaces,
-- agents, projects. Table-level parity was complete; these ALTERs close the
-- remaining column gaps against upstream packages/db/src/schema.

-- pipeline_cases -----------------------------------------------------------
ALTER TABLE pipeline_cases ADD COLUMN parent_case_version INTEGER;
ALTER TABLE pipeline_cases ADD COLUMN request_key TEXT;
ALTER TABLE pipeline_cases ADD COLUMN automation_attempt_id TEXT;
ALTER TABLE pipeline_cases ADD COLUMN pending_suggestion TEXT;
ALTER TABLE pipeline_cases ADD COLUMN retired_by_attempt_id TEXT;
ALTER TABLE pipeline_cases ADD COLUMN hidden_from_board_at TEXT;
ALTER TABLE pipeline_cases ADD COLUMN origin_run_id TEXT;

CREATE INDEX idx_pipeline_cases_automation_attempt
  ON pipeline_cases (automation_attempt_id);
CREATE INDEX idx_pipeline_cases_lease_expires
  ON pipeline_cases (lease_expires_at) WHERE lease_expires_at IS NOT NULL;

-- activity_log -------------------------------------------------------------
ALTER TABLE activity_log ADD COLUMN agent_id TEXT;
ALTER TABLE activity_log ADD COLUMN run_id TEXT;
ALTER TABLE activity_log ADD COLUMN responsible_user_id TEXT;

CREATE INDEX idx_activity_log_company_agent_created
  ON activity_log (company_id, agent_id, created_at);
CREATE INDEX idx_activity_log_company_responsible_user_created
  ON activity_log (company_id, responsible_user_id, created_at);
CREATE INDEX idx_activity_log_run_id
  ON activity_log (run_id);
CREATE INDEX idx_activity_log_entity
  ON activity_log (entity_type, entity_id);

-- decision desk ------------------------------------------------------------
ALTER TABLE decision_archive_notification_outbox ADD COLUMN archive_version INTEGER NOT NULL DEFAULT 0;
ALTER TABLE decision_archive_notification_outbox ADD COLUMN delivered_at TEXT;
ALTER TABLE decision_archive_notification_outbox ADD COLUMN last_attempt_at TEXT;
ALTER TABLE decision_archive_notification_outbox ADD COLUMN origin_agent_id TEXT;
ALTER TABLE decision_archive_notification_outbox ADD COLUMN origin_issue_id TEXT;
ALTER TABLE decision_archive_notification_outbox ADD COLUMN source_id TEXT;
ALTER TABLE decision_archive_notification_outbox ADD COLUMN source_kind TEXT;
ALTER TABLE decision_archive_notification_outbox ADD COLUMN updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'));

ALTER TABLE decision_queues ADD COLUMN key TEXT;
ALTER TABLE decision_queues ADD COLUMN title TEXT;
ALTER TABLE decision_queues ADD COLUMN created_by_type TEXT;
ALTER TABLE decision_queues ADD COLUMN created_by_agent_id TEXT;
ALTER TABLE decision_queues ADD COLUMN created_by_user_id TEXT;
ALTER TABLE decision_queues ADD COLUMN created_by_run_id TEXT;
ALTER TABLE decision_queues ADD COLUMN created_by_agent_api_key_id TEXT;
ALTER TABLE decision_queues ADD COLUMN seed_rules TEXT NOT NULL DEFAULT '[]';
ALTER TABLE decision_queues ADD COLUMN seed_rules_enabled INTEGER NOT NULL DEFAULT 0;

ALTER TABLE decision_queue_items ADD COLUMN added_by_type TEXT;
ALTER TABLE decision_queue_items ADD COLUMN added_by_agent_id TEXT;
ALTER TABLE decision_queue_items ADD COLUMN added_by_user_id TEXT;
ALTER TABLE decision_queue_items ADD COLUMN added_by_run_id TEXT;
ALTER TABLE decision_queue_items ADD COLUMN added_by_agent_api_key_id TEXT;
ALTER TABLE decision_queue_items ADD COLUMN responsible_user_id TEXT;

ALTER TABLE decision_triage ADD COLUMN decide_by_date TEXT;
ALTER TABLE decision_triage ADD COLUMN set_by_type TEXT;
ALTER TABLE decision_triage ADD COLUMN set_by_agent_id TEXT;
ALTER TABLE decision_triage ADD COLUMN set_by_user_id TEXT;
ALTER TABLE decision_triage ADD COLUMN set_by_run_id TEXT;
ALTER TABLE decision_triage ADD COLUMN set_by_agent_api_key_id TEXT;
ALTER TABLE decision_triage ADD COLUMN responsible_user_id TEXT;
ALTER TABLE decision_triage ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE decision_triage ADD COLUMN created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'));

ALTER TABLE decision_triage_events ADD COLUMN queue_id TEXT;
ALTER TABLE decision_triage_events ADD COLUMN source_kind TEXT;
ALTER TABLE decision_triage_events ADD COLUMN source_id TEXT;
ALTER TABLE decision_triage_events ADD COLUMN action TEXT;
ALTER TABLE decision_triage_events ADD COLUMN actor_type TEXT;
ALTER TABLE decision_triage_events ADD COLUMN actor_agent_id TEXT;
ALTER TABLE decision_triage_events ADD COLUMN actor_user_id TEXT;
ALTER TABLE decision_triage_events ADD COLUMN actor_run_id TEXT;
ALTER TABLE decision_triage_events ADD COLUMN agent_api_key_id TEXT;
ALTER TABLE decision_triage_events ADD COLUMN responsible_user_id TEXT;
ALTER TABLE decision_triage_events ADD COLUMN details TEXT;

ALTER TABLE decision_retention ADD COLUMN source_activity_at TEXT;
ALTER TABLE decision_retention ADD COLUMN archived_by_type TEXT;
ALTER TABLE decision_retention ADD COLUMN archived_by_agent_id TEXT;
ALTER TABLE decision_retention ADD COLUMN archived_by_user_id TEXT;
ALTER TABLE decision_retention ADD COLUMN archived_by_run_id TEXT;
ALTER TABLE decision_retention ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE decision_retention ADD COLUMN archive_version INTEGER NOT NULL DEFAULT 0;

-- rate limit counters ------------------------------------------------------

-- memberships --------------------------------------------------------------
ALTER TABLE project_memberships ADD COLUMN starred_at TEXT;
ALTER TABLE agent_memberships ADD COLUMN starred_at TEXT;

-- documents / work products ------------------------------------------------
ALTER TABLE documents ADD COLUMN source_trust TEXT;
ALTER TABLE issue_work_products ADD COLUMN source_trust TEXT;

-- workspaces ---------------------------------------------------------------
ALTER TABLE execution_workspaces ADD COLUMN cleanup_eligible_at TEXT;
ALTER TABLE execution_workspaces ADD COLUMN cleanup_reason TEXT;
ALTER TABLE execution_workspaces ADD COLUMN metadata TEXT;

ALTER TABLE workspace_operations ADD COLUMN log_bytes INTEGER;
ALTER TABLE workspace_operations ADD COLUMN log_sha256 TEXT;
ALTER TABLE workspace_operations ADD COLUMN log_compressed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE workspace_operations ADD COLUMN stdout_excerpt TEXT;
ALTER TABLE workspace_operations ADD COLUMN stderr_excerpt TEXT;
ALTER TABLE workspace_operations ADD COLUMN metadata TEXT;
ALTER TABLE workspace_operations ADD COLUMN started_at TEXT;
ALTER TABLE workspace_operations ADD COLUMN finished_at TEXT;

ALTER TABLE workspace_runtime_services ADD COLUMN health_status TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE workspace_runtime_services ADD COLUMN started_at TEXT;
ALTER TABLE workspace_runtime_services ADD COLUMN stopped_at TEXT;
ALTER TABLE workspace_runtime_services ADD COLUMN last_used_at TEXT;
ALTER TABLE workspace_runtime_services ADD COLUMN stop_policy TEXT;

-- agents / projects --------------------------------------------------------
ALTER TABLE agents ADD COLUMN default_environment_id TEXT;
ALTER TABLE agents ADD COLUMN error_reason TEXT;

ALTER TABLE projects ADD COLUMN archived_at TEXT;
ALTER TABLE projects ADD COLUMN color TEXT;
ALTER TABLE projects ADD COLUMN icon TEXT;
ALTER TABLE projects ADD COLUMN pause_reason TEXT;
ALTER TABLE projects ADD COLUMN paused_at TEXT;
ALTER TABLE projects ADD COLUMN execution_workspace_policy TEXT;
