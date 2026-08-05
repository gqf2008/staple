-- Roll back column-level parity batch 2.

ALTER TABLE routine_triggers DROP COLUMN updated_by_user_id;
ALTER TABLE routine_triggers DROP COLUMN updated_by_agent_id;
ALTER TABLE routine_triggers DROP COLUMN created_by_user_id;
ALTER TABLE routine_triggers DROP COLUMN created_by_agent_id;
ALTER TABLE routine_triggers DROP COLUMN last_result;
ALTER TABLE routine_triggers DROP COLUMN last_rotated_at;
ALTER TABLE routine_triggers DROP COLUMN replay_window_sec;
ALTER TABLE routine_triggers DROP COLUMN signing_mode;
ALTER TABLE routine_triggers DROP COLUMN secret_id;
ALTER TABLE routine_triggers DROP COLUMN public_id;
ALTER TABLE routine_triggers DROP COLUMN last_fired_at;
ALTER TABLE routine_triggers DROP COLUMN next_run_at;
ALTER TABLE routine_triggers DROP COLUMN timezone;
ALTER TABLE routine_triggers DROP COLUMN cron_expression;
ALTER TABLE routine_triggers DROP COLUMN label;
ALTER TABLE routine_triggers DROP COLUMN kind;

ALTER TABLE routine_runs DROP COLUMN completed_at;
ALTER TABLE routine_runs DROP COLUMN failure_reason;
ALTER TABLE routine_runs DROP COLUMN coalesced_into_run_id;
ALTER TABLE routine_runs DROP COLUMN linked_issue_id;
ALTER TABLE routine_runs DROP COLUMN dispatch_fingerprint;
ALTER TABLE routine_runs DROP COLUMN trigger_payload;
ALTER TABLE routine_runs DROP COLUMN idempotency_key;
ALTER TABLE routine_runs DROP COLUMN responsible_user_id;
ALTER TABLE routine_runs DROP COLUMN routine_revision_id;
ALTER TABLE routine_runs DROP COLUMN triggered_at;
ALTER TABLE routine_runs DROP COLUMN trigger_id;
ALTER TABLE routine_runs DROP COLUMN source;

ALTER TABLE routine_revisions DROP COLUMN created_by_run_id;
ALTER TABLE routine_revisions DROP COLUMN responsible_user_id;
ALTER TABLE routine_revisions DROP COLUMN restored_from_revision_id;
ALTER TABLE routine_revisions DROP COLUMN snapshot;
ALTER TABLE routine_revisions DROP COLUMN change_summary;

ALTER TABLE routines DROP COLUMN updated_by_user_id;
ALTER TABLE routines DROP COLUMN updated_by_agent_id;
ALTER TABLE routines DROP COLUMN folder_id;

ALTER TABLE issue_thread_interactions DROP COLUMN resolved_at;
ALTER TABLE issue_thread_interactions DROP COLUMN result;
ALTER TABLE issue_thread_interactions DROP COLUMN resolved_by_user_id;
ALTER TABLE issue_thread_interactions DROP COLUMN resolved_by_agent_id;
ALTER TABLE issue_thread_interactions DROP COLUMN created_by_user_id;
ALTER TABLE issue_thread_interactions DROP COLUMN created_by_agent_id;
ALTER TABLE issue_thread_interactions DROP COLUMN summary;
ALTER TABLE issue_thread_interactions DROP COLUMN title;
ALTER TABLE issue_thread_interactions DROP COLUMN source_run_id;
ALTER TABLE issue_thread_interactions DROP COLUMN source_comment_id;
ALTER TABLE issue_thread_interactions DROP COLUMN idempotency_key;
ALTER TABLE issue_thread_interactions DROP COLUMN continuation_policy;

ALTER TABLE issue_comments DROP COLUMN source_trust;
ALTER TABLE issue_comments DROP COLUMN deleted_by_run_id;
ALTER TABLE issue_comments DROP COLUMN deleted_by_user_id;
ALTER TABLE issue_comments DROP COLUMN deleted_by_agent_id;
ALTER TABLE issue_comments DROP COLUMN deleted_by_type;
ALTER TABLE issue_comments DROP COLUMN deleted_at;
ALTER TABLE issue_comments DROP COLUMN metadata;
ALTER TABLE issue_comments DROP COLUMN presentation;
ALTER TABLE issue_comments DROP COLUMN derived_author_source;
ALTER TABLE issue_comments DROP COLUMN derived_created_by_run_id;
ALTER TABLE issue_comments DROP COLUMN derived_author_agent_id;
ALTER TABLE issue_comments DROP COLUMN created_by_run_id;
ALTER TABLE issue_comments DROP COLUMN author_type;

ALTER TABLE document_revisions DROP COLUMN created_by_run_id;
ALTER TABLE document_revisions DROP COLUMN created_by_user_id;
ALTER TABLE document_revisions DROP COLUMN created_by_agent_id;
ALTER TABLE document_revisions DROP COLUMN format;
ALTER TABLE document_revisions DROP COLUMN title;

ALTER TABLE cost_events DROP COLUMN cached_input_tokens;
ALTER TABLE cost_events DROP COLUMN billing_type;
ALTER TABLE cost_events DROP COLUMN biller;
ALTER TABLE cost_events DROP COLUMN heartbeat_run_id;

ALTER TABLE agent_api_keys DROP COLUMN scope_config;
ALTER TABLE agent_api_keys DROP COLUMN responsible_user_id;
