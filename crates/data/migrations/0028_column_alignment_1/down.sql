-- Roll back column-level parity batch 1.

ALTER TABLE projects DROP COLUMN execution_workspace_policy;
ALTER TABLE projects DROP COLUMN paused_at;
ALTER TABLE projects DROP COLUMN pause_reason;
ALTER TABLE projects DROP COLUMN icon;
ALTER TABLE projects DROP COLUMN color;
ALTER TABLE projects DROP COLUMN archived_at;

ALTER TABLE agents DROP COLUMN error_reason;
ALTER TABLE agents DROP COLUMN default_environment_id;

ALTER TABLE workspace_runtime_services DROP COLUMN stop_policy;
ALTER TABLE workspace_runtime_services DROP COLUMN last_used_at;
ALTER TABLE workspace_runtime_services DROP COLUMN stopped_at;
ALTER TABLE workspace_runtime_services DROP COLUMN started_at;
ALTER TABLE workspace_runtime_services DROP COLUMN health_status;

ALTER TABLE workspace_operations DROP COLUMN finished_at;
ALTER TABLE workspace_operations DROP COLUMN started_at;
ALTER TABLE workspace_operations DROP COLUMN metadata;
ALTER TABLE workspace_operations DROP COLUMN stderr_excerpt;
ALTER TABLE workspace_operations DROP COLUMN stdout_excerpt;
ALTER TABLE workspace_operations DROP COLUMN log_compressed;
ALTER TABLE workspace_operations DROP COLUMN log_sha256;
ALTER TABLE workspace_operations DROP COLUMN log_bytes;

ALTER TABLE execution_workspaces DROP COLUMN metadata;
ALTER TABLE execution_workspaces DROP COLUMN cleanup_reason;
ALTER TABLE execution_workspaces DROP COLUMN cleanup_eligible_at;

ALTER TABLE issue_work_products DROP COLUMN source_trust;
ALTER TABLE documents DROP COLUMN source_trust;

ALTER TABLE agent_memberships DROP COLUMN starred_at;
ALTER TABLE project_memberships DROP COLUMN starred_at;


ALTER TABLE decision_retention DROP COLUMN archive_version;
ALTER TABLE decision_retention DROP COLUMN version;
ALTER TABLE decision_retention DROP COLUMN archived_by_run_id;
ALTER TABLE decision_retention DROP COLUMN archived_by_user_id;
ALTER TABLE decision_retention DROP COLUMN archived_by_agent_id;
ALTER TABLE decision_retention DROP COLUMN archived_by_type;
ALTER TABLE decision_retention DROP COLUMN source_activity_at;

ALTER TABLE decision_triage_events DROP COLUMN details;
ALTER TABLE decision_triage_events DROP COLUMN responsible_user_id;
ALTER TABLE decision_triage_events DROP COLUMN agent_api_key_id;
ALTER TABLE decision_triage_events DROP COLUMN actor_run_id;
ALTER TABLE decision_triage_events DROP COLUMN actor_user_id;
ALTER TABLE decision_triage_events DROP COLUMN actor_agent_id;
ALTER TABLE decision_triage_events DROP COLUMN actor_type;
ALTER TABLE decision_triage_events DROP COLUMN action;
ALTER TABLE decision_triage_events DROP COLUMN source_id;
ALTER TABLE decision_triage_events DROP COLUMN source_kind;
ALTER TABLE decision_triage_events DROP COLUMN queue_id;

ALTER TABLE decision_triage DROP COLUMN created_at;
ALTER TABLE decision_triage DROP COLUMN version;
ALTER TABLE decision_triage DROP COLUMN responsible_user_id;
ALTER TABLE decision_triage DROP COLUMN set_by_agent_api_key_id;
ALTER TABLE decision_triage DROP COLUMN set_by_run_id;
ALTER TABLE decision_triage DROP COLUMN set_by_user_id;
ALTER TABLE decision_triage DROP COLUMN set_by_agent_id;
ALTER TABLE decision_triage DROP COLUMN set_by_type;
ALTER TABLE decision_triage DROP COLUMN decide_by_date;

ALTER TABLE decision_queue_items DROP COLUMN responsible_user_id;
ALTER TABLE decision_queue_items DROP COLUMN added_by_agent_api_key_id;
ALTER TABLE decision_queue_items DROP COLUMN added_by_run_id;
ALTER TABLE decision_queue_items DROP COLUMN added_by_user_id;
ALTER TABLE decision_queue_items DROP COLUMN added_by_agent_id;
ALTER TABLE decision_queue_items DROP COLUMN added_by_type;

ALTER TABLE decision_archive_notification_outbox DROP COLUMN updated_at;
ALTER TABLE decision_archive_notification_outbox DROP COLUMN source_kind;
ALTER TABLE decision_archive_notification_outbox DROP COLUMN source_id;
ALTER TABLE decision_archive_notification_outbox DROP COLUMN origin_issue_id;
ALTER TABLE decision_archive_notification_outbox DROP COLUMN origin_agent_id;
ALTER TABLE decision_archive_notification_outbox DROP COLUMN last_attempt_at;
ALTER TABLE decision_archive_notification_outbox DROP COLUMN delivered_at;
ALTER TABLE decision_archive_notification_outbox DROP COLUMN archive_version;

ALTER TABLE decision_queues DROP COLUMN seed_rules_enabled;
ALTER TABLE decision_queues DROP COLUMN seed_rules;
ALTER TABLE decision_queues DROP COLUMN created_by_agent_api_key_id;
ALTER TABLE decision_queues DROP COLUMN created_by_run_id;
ALTER TABLE decision_queues DROP COLUMN created_by_user_id;
ALTER TABLE decision_queues DROP COLUMN created_by_agent_id;
ALTER TABLE decision_queues DROP COLUMN created_by_type;
ALTER TABLE decision_queues DROP COLUMN title;
ALTER TABLE decision_queues DROP COLUMN key;

DROP INDEX IF EXISTS idx_activity_log_entity;
DROP INDEX IF EXISTS idx_activity_log_run_id;
DROP INDEX IF EXISTS idx_activity_log_company_responsible_user_created;
DROP INDEX IF EXISTS idx_activity_log_company_agent_created;

ALTER TABLE activity_log DROP COLUMN responsible_user_id;
ALTER TABLE activity_log DROP COLUMN run_id;
ALTER TABLE activity_log DROP COLUMN agent_id;

DROP INDEX IF EXISTS idx_pipeline_cases_lease_expires;
DROP INDEX IF EXISTS idx_pipeline_cases_automation_attempt;

ALTER TABLE pipeline_cases DROP COLUMN origin_run_id;
ALTER TABLE pipeline_cases DROP COLUMN hidden_from_board_at;
ALTER TABLE pipeline_cases DROP COLUMN retired_by_attempt_id;
ALTER TABLE pipeline_cases DROP COLUMN pending_suggestion;
ALTER TABLE pipeline_cases DROP COLUMN automation_attempt_id;
ALTER TABLE pipeline_cases DROP COLUMN request_key;
ALTER TABLE pipeline_cases DROP COLUMN parent_case_version;
