-- Column-level parity batch 2: agent_api_keys, cost_events,
-- document_revisions, issue_comments, issue_thread_interactions, routines,
-- routine_revisions, routine_runs, routine_triggers. Closes the remaining
-- column gaps against upstream packages/db/src/schema.

-- agent_api_keys -----------------------------------------------------------
ALTER TABLE agent_api_keys ADD COLUMN responsible_user_id TEXT;
ALTER TABLE agent_api_keys ADD COLUMN scope_config TEXT;

-- cost_events --------------------------------------------------------------
ALTER TABLE cost_events ADD COLUMN heartbeat_run_id TEXT;
ALTER TABLE cost_events ADD COLUMN biller TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE cost_events ADD COLUMN billing_type TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE cost_events ADD COLUMN cached_input_tokens INTEGER NOT NULL DEFAULT 0;

-- document_revisions -------------------------------------------------------
ALTER TABLE document_revisions ADD COLUMN title TEXT;
ALTER TABLE document_revisions ADD COLUMN format TEXT NOT NULL DEFAULT 'markdown';
ALTER TABLE document_revisions ADD COLUMN created_by_agent_id TEXT;
ALTER TABLE document_revisions ADD COLUMN created_by_user_id TEXT;
ALTER TABLE document_revisions ADD COLUMN created_by_run_id TEXT;

-- issue_comments -----------------------------------------------------------
ALTER TABLE issue_comments ADD COLUMN author_type TEXT;
ALTER TABLE issue_comments ADD COLUMN created_by_run_id TEXT;
ALTER TABLE issue_comments ADD COLUMN derived_author_agent_id TEXT;
ALTER TABLE issue_comments ADD COLUMN derived_created_by_run_id TEXT;
ALTER TABLE issue_comments ADD COLUMN derived_author_source TEXT;
ALTER TABLE issue_comments ADD COLUMN presentation TEXT;
ALTER TABLE issue_comments ADD COLUMN metadata TEXT;
ALTER TABLE issue_comments ADD COLUMN deleted_at TEXT;
ALTER TABLE issue_comments ADD COLUMN deleted_by_type TEXT;
ALTER TABLE issue_comments ADD COLUMN deleted_by_agent_id TEXT;
ALTER TABLE issue_comments ADD COLUMN deleted_by_user_id TEXT;
ALTER TABLE issue_comments ADD COLUMN deleted_by_run_id TEXT;
ALTER TABLE issue_comments ADD COLUMN source_trust TEXT;

-- issue_thread_interactions ------------------------------------------------
ALTER TABLE issue_thread_interactions ADD COLUMN continuation_policy TEXT NOT NULL DEFAULT 'wake_assignee';
ALTER TABLE issue_thread_interactions ADD COLUMN idempotency_key TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN source_comment_id TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN source_run_id TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN title TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN summary TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN created_by_agent_id TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN created_by_user_id TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN resolved_by_agent_id TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN resolved_by_user_id TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN result TEXT;
ALTER TABLE issue_thread_interactions ADD COLUMN resolved_at TEXT;

-- routines -----------------------------------------------------------------
ALTER TABLE routines ADD COLUMN folder_id TEXT;
ALTER TABLE routines ADD COLUMN updated_by_agent_id TEXT;
ALTER TABLE routines ADD COLUMN updated_by_user_id TEXT;

-- routine_revisions --------------------------------------------------------
ALTER TABLE routine_revisions ADD COLUMN change_summary TEXT;
ALTER TABLE routine_revisions ADD COLUMN snapshot TEXT;
ALTER TABLE routine_revisions ADD COLUMN restored_from_revision_id TEXT;
ALTER TABLE routine_revisions ADD COLUMN responsible_user_id TEXT;
ALTER TABLE routine_revisions ADD COLUMN created_by_run_id TEXT;

-- routine_runs -------------------------------------------------------------
ALTER TABLE routine_runs ADD COLUMN source TEXT;
ALTER TABLE routine_runs ADD COLUMN trigger_id TEXT;
ALTER TABLE routine_runs ADD COLUMN triggered_at TEXT;
ALTER TABLE routine_runs ADD COLUMN routine_revision_id TEXT;
ALTER TABLE routine_runs ADD COLUMN responsible_user_id TEXT;
ALTER TABLE routine_runs ADD COLUMN idempotency_key TEXT;
ALTER TABLE routine_runs ADD COLUMN trigger_payload TEXT;
ALTER TABLE routine_runs ADD COLUMN dispatch_fingerprint TEXT;
ALTER TABLE routine_runs ADD COLUMN linked_issue_id TEXT;
ALTER TABLE routine_runs ADD COLUMN coalesced_into_run_id TEXT;
ALTER TABLE routine_runs ADD COLUMN failure_reason TEXT;
ALTER TABLE routine_runs ADD COLUMN completed_at TEXT;

-- routine_triggers ---------------------------------------------------------
ALTER TABLE routine_triggers ADD COLUMN kind TEXT;
ALTER TABLE routine_triggers ADD COLUMN label TEXT;
ALTER TABLE routine_triggers ADD COLUMN cron_expression TEXT;
ALTER TABLE routine_triggers ADD COLUMN timezone TEXT;
ALTER TABLE routine_triggers ADD COLUMN next_run_at TEXT;
ALTER TABLE routine_triggers ADD COLUMN last_fired_at TEXT;
ALTER TABLE routine_triggers ADD COLUMN public_id TEXT;
ALTER TABLE routine_triggers ADD COLUMN secret_id TEXT;
ALTER TABLE routine_triggers ADD COLUMN signing_mode TEXT;
ALTER TABLE routine_triggers ADD COLUMN replay_window_sec INTEGER;
ALTER TABLE routine_triggers ADD COLUMN last_rotated_at TEXT;
ALTER TABLE routine_triggers ADD COLUMN last_result TEXT;
ALTER TABLE routine_triggers ADD COLUMN created_by_agent_id TEXT;
ALTER TABLE routine_triggers ADD COLUMN created_by_user_id TEXT;
ALTER TABLE routine_triggers ADD COLUMN updated_by_agent_id TEXT;
ALTER TABLE routine_triggers ADD COLUMN updated_by_user_id TEXT;
