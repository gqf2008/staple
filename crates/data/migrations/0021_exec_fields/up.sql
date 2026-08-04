-- Execution-surface field alignment with upstream heartbeat_runs.ts and
-- issues.ts (columns only; behavior is unchanged).

ALTER TABLE heartbeat_runs ADD COLUMN responsible_user_id TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN wakeup_request_id TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN exit_code INTEGER;
ALTER TABLE heartbeat_runs ADD COLUMN signal TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN usage_json TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN result_json TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN session_id_before TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN session_id_after TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN log_store TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN log_ref TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN log_sha256 TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN log_compressed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE heartbeat_runs ADD COLUMN stdout_excerpt TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN stderr_excerpt TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN error_code TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN process_pid INTEGER;
ALTER TABLE heartbeat_runs ADD COLUMN process_group_id INTEGER;
ALTER TABLE heartbeat_runs ADD COLUMN process_started_at TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN last_output_at TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN last_output_seq INTEGER;
ALTER TABLE heartbeat_runs ADD COLUMN last_output_stream TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN last_output_bytes INTEGER;
ALTER TABLE heartbeat_runs ADD COLUMN retry_of_run_id TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN process_loss_retry_count INTEGER;
ALTER TABLE heartbeat_runs ADD COLUMN scheduled_retry_at TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN scheduled_retry_attempt INTEGER;
ALTER TABLE heartbeat_runs ADD COLUMN scheduled_retry_reason TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN issue_comment_status TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN issue_comment_satisfied_by_comment_id TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN issue_comment_retry_queued_at TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN liveness_state TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN liveness_reason TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN continuation_attempt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE heartbeat_runs ADD COLUMN last_useful_action_at TEXT;
ALTER TABLE heartbeat_runs ADD COLUMN next_action TEXT;

CREATE INDEX IF NOT EXISTS idx_heartbeat_runs_company_status ON heartbeat_runs (company_id, status);
CREATE INDEX IF NOT EXISTS idx_heartbeat_runs_company_liveness ON heartbeat_runs (company_id, liveness_state);
CREATE INDEX IF NOT EXISTS idx_heartbeat_runs_company_responsible_user ON heartbeat_runs (company_id, responsible_user_id);
CREATE INDEX IF NOT EXISTS idx_heartbeat_runs_scheduled_retry ON heartbeat_runs (scheduled_retry_at);

ALTER TABLE issues ADD COLUMN project_workspace_id TEXT;
ALTER TABLE issues ADD COLUMN harness_kind TEXT;
ALTER TABLE issues ADD COLUMN responsible_user_id TEXT;
ALTER TABLE issues ADD COLUMN monitor_next_check_at TEXT;
ALTER TABLE issues ADD COLUMN monitor_wake_requested_at TEXT;
ALTER TABLE issues ADD COLUMN monitor_last_triggered_at TEXT;
ALTER TABLE issues ADD COLUMN monitor_attempt_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE issues ADD COLUMN monitor_notes TEXT;
ALTER TABLE issues ADD COLUMN monitor_scheduled_by TEXT;
ALTER TABLE issues ADD COLUMN execution_workspace_id TEXT;
ALTER TABLE issues ADD COLUMN execution_workspace_preference TEXT;
ALTER TABLE issues ADD COLUMN execution_workspace_settings TEXT;
ALTER TABLE issues ADD COLUMN source_trust TEXT;
ALTER TABLE issues ADD COLUMN unblock_descriptor TEXT;
ALTER TABLE issues ADD COLUMN blocked_transition_at TEXT;
ALTER TABLE issues ADD COLUMN blocked_owner_notified_at TEXT;

CREATE INDEX IF NOT EXISTS idx_issues_company_status ON issues (company_id, status);
CREATE INDEX IF NOT EXISTS idx_issues_company_monitor ON issues (company_id, monitor_next_check_at);
