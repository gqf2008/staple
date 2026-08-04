//! Infrastructure domain: auth, instance settings, folders, watchdogs,
//! holds, heartbeat events, environment images/leases, and user
//! preferences (upstream auth.ts + infrastructure family).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A user (upstream auth.ts `user`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a user.
#[derive(Debug, Clone)]
pub struct NewUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
}

/// A session (upstream auth.ts `session`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub id: String,
    pub expires_at: String,
    pub token: String,
    pub created_at: String,
    pub updated_at: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: String,
}

/// Input for creating a session.
#[derive(Debug, Clone)]
pub struct NewSession {
    pub id: String,
    pub expires_at: String,
    pub token: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub user_id: String,
}

/// A folder (upstream folders.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderRecord {
    pub id: String,
    pub company_id: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub slug: String,
    pub system_key: Option<String>,
    pub color: Option<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a folder.
#[derive(Debug, Clone)]
pub struct NewFolder {
    pub company_id: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub slug: String,
    pub system_key: Option<String>,
    pub color: Option<String>,
    pub position: i64,
}

/// An agent config revision (upstream agent_config_revisions.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfigRevisionRecord {
    pub id: String,
    pub company_id: String,
    pub agent_id: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub source: String,
    pub rolled_back_from_revision_id: Option<String>,
    pub changed_keys: serde_json::Value,
    pub before_config: serde_json::Value,
    pub after_config: serde_json::Value,
    pub created_at: String,
}

/// Input for creating an agent config revision.
#[derive(Debug, Clone)]
pub struct NewAgentConfigRevision {
    pub company_id: String,
    pub agent_id: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub source: String,
    pub rolled_back_from_revision_id: Option<String>,
    pub changed_keys: serde_json::Value,
    pub before_config: serde_json::Value,
    pub after_config: serde_json::Value,
}

/// An inbox dismissal (upstream inbox_dismissals.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxDismissalRecord {
    pub id: String,
    pub company_id: String,
    pub user_id: String,
    pub item_key: String,
    pub kind: String,
    pub dismissed_at: String,
    pub snoozed_until: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting an inbox dismissal.
#[derive(Debug, Clone)]
pub struct NewInboxDismissal {
    pub company_id: String,
    pub user_id: String,
    pub item_key: String,
    pub kind: String,
    pub snoozed_until: Option<String>,
}

/// A document membership (upstream document_memberships.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMembershipRecord {
    pub id: String,
    pub company_id: String,
    pub document_id: String,
    pub user_id: String,
    pub starred_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting a document membership.
#[derive(Debug, Clone)]
pub struct NewDocumentMembership {
    pub company_id: String,
    pub document_id: String,
    pub user_id: String,
    pub starred_at: Option<String>,
}

/// A routine document link (upstream routine_documents.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineDocumentRecord {
    pub id: String,
    pub company_id: String,
    pub routine_id: String,
    pub document_id: String,
    pub key: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for linking a routine document.
#[derive(Debug, Clone)]
pub struct NewRoutineDocument {
    pub company_id: String,
    pub routine_id: String,
    pub document_id: String,
    pub key: String,
}

/// An approval comment (upstream approval_comments.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalCommentRecord {
    pub id: String,
    pub company_id: String,
    pub approval_id: String,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an approval comment.
#[derive(Debug, Clone)]
pub struct NewApprovalComment {
    pub company_id: String,
    pub approval_id: String,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub body: String,
}

/// A built-in managed resource (upstream built_in_managed_resources.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltInResourceRecord {
    pub id: String,
    pub company_id: String,
    pub bundle_key: String,
    pub resource_kind: String,
    pub resource_key: String,
    pub resource_id: String,
    pub stock_version: String,
    pub stock_hash: String,
    pub defaults_json: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting a built-in managed resource.
#[derive(Debug, Clone)]
pub struct NewBuiltInResource {
    pub company_id: String,
    pub bundle_key: String,
    pub resource_kind: String,
    pub resource_key: String,
    pub resource_id: String,
    pub stock_version: String,
    pub stock_hash: String,
    pub defaults_json: serde_json::Value,
}

/// An issue create idempotency key (upstream issue_create_idempotency_keys.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueIdempotencyRecord {
    pub id: String,
    pub company_id: String,
    pub idempotency_key: String,
    pub issue_id: String,
    pub created_at: String,
}

/// An issue inbox archive (upstream issue_inbox_archives.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueInboxArchiveRecord {
    pub id: String,
    pub company_id: String,
    pub issue_id: String,
    pub user_id: String,
    pub archived_by_actor_type: String,
    pub archived_by_agent_id: Option<String>,
    pub archived_by_run_id: Option<String>,
    pub archived_at: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for archiving an issue in a user inbox.
#[derive(Debug, Clone)]
pub struct NewIssueInboxArchive {
    pub company_id: String,
    pub issue_id: String,
    pub user_id: String,
    pub archived_by_actor_type: String,
    pub archived_by_agent_id: Option<String>,
    pub archived_by_run_id: Option<String>,
}

/// An issue plan decomposition (upstream issue_plan_decompositions.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuePlanDecompositionRecord {
    pub id: String,
    pub company_id: String,
    pub source_issue_id: String,
    pub accepted_plan_revision_id: String,
    pub accepted_interaction_id: Option<String>,
    pub status: String,
    pub request_fingerprint: String,
    pub requested_child_count: i64,
    pub requested_children: serde_json::Value,
    pub child_issue_ids: serde_json::Value,
    pub owner_agent_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub owner_run_id: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an issue plan decomposition.
#[derive(Debug, Clone)]
pub struct NewIssuePlanDecomposition {
    pub company_id: String,
    pub source_issue_id: String,
    pub accepted_plan_revision_id: String,
    pub accepted_interaction_id: Option<String>,
    pub status: String,
    pub request_fingerprint: String,
    pub requested_child_count: i64,
    pub requested_children: serde_json::Value,
    pub child_issue_ids: serde_json::Value,
    pub owner_agent_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub owner_run_id: Option<String>,
}

/// An issue reference mention (upstream issue_reference_mentions.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueReferenceMentionRecord {
    pub id: String,
    pub company_id: String,
    pub source_issue_id: String,
    pub target_issue_id: String,
    pub source_kind: String,
    pub source_record_id: Option<String>,
    pub document_key: Option<String>,
    pub matched_text: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an issue reference mention.
#[derive(Debug, Clone)]
pub struct NewIssueReferenceMention {
    pub company_id: String,
    pub source_issue_id: String,
    pub target_issue_id: String,
    pub source_kind: String,
    pub source_record_id: Option<String>,
    pub document_key: Option<String>,
    pub matched_text: Option<String>,
}

/// An issue tree hold (upstream issue_tree_holds.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueTreeHoldRecord {
    pub id: String,
    pub company_id: String,
    pub root_issue_id: String,
    pub mode: String,
    pub status: String,
    pub reason: Option<String>,
    pub release_policy: Option<serde_json::Value>,
    pub created_by_actor_type: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
    pub released_at: Option<String>,
    pub released_by_actor_type: Option<String>,
    pub released_by_agent_id: Option<String>,
    pub released_by_user_id: Option<String>,
    pub released_by_run_id: Option<String>,
    pub release_reason: Option<String>,
    pub release_metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an issue tree hold.
#[derive(Debug, Clone)]
pub struct NewIssueTreeHold {
    pub company_id: String,
    pub root_issue_id: String,
    pub mode: String,
    pub status: String,
    pub reason: Option<String>,
    pub release_policy: Option<serde_json::Value>,
    pub created_by_actor_type: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
}

/// An issue tree hold member (upstream issue_tree_hold_members.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueTreeHoldMemberRecord {
    pub id: String,
    pub company_id: String,
    pub hold_id: String,
    pub issue_id: String,
    pub parent_issue_id: Option<String>,
    pub depth: i64,
    pub issue_identifier: Option<String>,
    pub issue_title: String,
    pub issue_status: String,
    pub assignee_agent_id: Option<String>,
    pub assignee_user_id: Option<String>,
    pub active_run_id: Option<String>,
    pub active_run_status: Option<String>,
    pub skipped: bool,
    pub skip_reason: Option<String>,
    pub created_at: String,
}

/// Input for adding an issue tree hold member.
#[derive(Debug, Clone)]
pub struct NewIssueTreeHoldMember {
    pub company_id: String,
    pub hold_id: String,
    pub issue_id: String,
    pub parent_issue_id: Option<String>,
    pub depth: i64,
    pub issue_identifier: Option<String>,
    pub issue_title: String,
    pub issue_status: String,
    pub assignee_agent_id: Option<String>,
    pub assignee_user_id: Option<String>,
    pub active_run_id: Option<String>,
    pub active_run_status: Option<String>,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

/// An issue watchdog (upstream issue_watchdogs.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWatchdogRecord {
    pub id: String,
    pub company_id: String,
    pub issue_id: String,
    pub watchdog_agent_id: String,
    pub instructions: Option<String>,
    pub status: String,
    pub watchdog_issue_id: Option<String>,
    pub last_observed_fingerprint: Option<String>,
    pub last_reviewed_fingerprint: Option<String>,
    pub last_observed_stop_snapshot: Option<serde_json::Value>,
    pub last_reviewed_stop_snapshot: Option<serde_json::Value>,
    pub last_triggered_at: Option<String>,
    pub last_completed_at: Option<String>,
    pub trigger_count: i64,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
    pub updated_by_agent_id: Option<String>,
    pub updated_by_user_id: Option<String>,
    pub updated_by_run_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an issue watchdog.
#[derive(Debug, Clone)]
pub struct NewIssueWatchdog {
    pub company_id: String,
    pub issue_id: String,
    pub watchdog_agent_id: String,
    pub instructions: Option<String>,
    pub status: String,
    pub watchdog_issue_id: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
}

/// A heartbeat run event (upstream heartbeat_run_events.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRunEventRecord {
    pub id: i64,
    pub company_id: String,
    pub run_id: String,
    pub agent_id: String,
    pub seq: i64,
    pub event_type: String,
    pub stream: Option<String>,
    pub level: Option<String>,
    pub color: Option<String>,
    pub message: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub created_at: String,
}

/// Input for appending a heartbeat run event.
#[derive(Debug, Clone)]
pub struct NewHeartbeatRunEvent {
    pub company_id: String,
    pub run_id: String,
    pub agent_id: String,
    pub seq: i64,
    pub event_type: String,
    pub stream: Option<String>,
    pub level: Option<String>,
    pub color: Option<String>,
    pub message: Option<String>,
    pub payload: Option<serde_json::Value>,
}

/// A heartbeat run watchdog decision (upstream
/// heartbeat_run_watchdog_decisions.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatWatchdogDecisionRecord {
    pub id: String,
    pub company_id: String,
    pub run_id: String,
    pub evaluation_issue_id: Option<String>,
    pub decision: String,
    pub snoozed_until: Option<String>,
    pub reason: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
    pub created_at: String,
}

/// Input for creating a heartbeat run watchdog decision.
#[derive(Debug, Clone)]
pub struct NewHeartbeatWatchdogDecision {
    pub company_id: String,
    pub run_id: String,
    pub evaluation_issue_id: Option<String>,
    pub decision: String,
    pub snoozed_until: Option<String>,
    pub reason: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_by_run_id: Option<String>,
}

/// An environment custom image template (upstream
/// environment_custom_image_templates.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvImageTemplateRecord {
    pub id: String,
    pub environment_id: String,
    pub provider: String,
    pub template_kind: String,
    pub template_ref: String,
    pub source_template_ref: Option<String>,
    pub source_environment_config_fingerprint: Option<String>,
    pub status: String,
    pub created_by_user_id: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub captured_at: Option<String>,
    pub last_used_at: Option<String>,
    pub superseded_by_template_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an environment custom image template.
#[derive(Debug, Clone)]
pub struct NewEnvImageTemplate {
    pub environment_id: String,
    pub provider: String,
    pub template_kind: String,
    pub template_ref: String,
    pub source_template_ref: Option<String>,
    pub source_environment_config_fingerprint: Option<String>,
    pub status: String,
    pub created_by_user_id: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// An environment lease (upstream environment_leases.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvLeaseRecord {
    pub id: String,
    pub company_id: String,
    pub environment_id: String,
    pub execution_workspace_id: Option<String>,
    pub issue_id: Option<String>,
    pub heartbeat_run_id: Option<String>,
    pub status: String,
    pub lease_policy: String,
    pub provider: Option<String>,
    pub provider_lease_id: Option<String>,
    pub acquired_at: String,
    pub last_used_at: String,
    pub expires_at: Option<String>,
    pub released_at: Option<String>,
    pub failure_reason: Option<String>,
    pub cleanup_status: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an environment lease.
#[derive(Debug, Clone)]
pub struct NewEnvLease {
    pub company_id: String,
    pub environment_id: String,
    pub execution_workspace_id: Option<String>,
    pub issue_id: Option<String>,
    pub heartbeat_run_id: Option<String>,
    pub status: String,
    pub lease_policy: String,
    pub provider: Option<String>,
    pub provider_lease_id: Option<String>,
    pub expires_at: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// An environment custom image setup session (upstream
/// environment_custom_image_setup_sessions.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvSetupSessionRecord {
    pub id: String,
    pub environment_id: String,
    pub template_id: Option<String>,
    pub promoted_template_id: Option<String>,
    pub provider: String,
    pub provider_lease_id: Option<String>,
    pub environment_lease_id: Option<String>,
    pub status: String,
    pub started_by_user_id: Option<String>,
    pub started_by_agent_id: Option<String>,
    pub base_template_ref: Option<String>,
    pub expires_at: Option<String>,
    pub finished_at: Option<String>,
    pub failure_reason: Option<String>,
    pub connection_summary: Option<serde_json::Value>,
    pub connection_secret_ref: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an environment custom image setup session.
#[derive(Debug, Clone)]
pub struct NewEnvSetupSession {
    pub environment_id: String,
    pub template_id: Option<String>,
    pub promoted_template_id: Option<String>,
    pub provider: String,
    pub provider_lease_id: Option<String>,
    pub environment_lease_id: Option<String>,
    pub status: String,
    pub started_by_user_id: Option<String>,
    pub started_by_agent_id: Option<String>,
    pub base_template_ref: Option<String>,
    pub expires_at: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// A user inbox agent policy (upstream user_inbox_agent_policies.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxAgentPolicyRecord {
    pub id: String,
    pub company_id: String,
    pub user_id: String,
    pub mode: String,
    pub allowed_agent_ids: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting a user inbox agent policy.
#[derive(Debug, Clone)]
pub struct NewInboxAgentPolicy {
    pub company_id: String,
    pub user_id: String,
    pub mode: String,
    pub allowed_agent_ids: serde_json::Value,
}

/// Instance settings (upstream instance_settings.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSettingsRecord {
    pub id: String,
    pub singleton_key: String,
    pub default_environment_id: Option<String>,
    pub general: serde_json::Value,
    pub experimental: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// A user sidebar preference (upstream user_sidebar_preferences.ts).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSidebarPreferenceRecord {
    pub id: String,
    pub user_id: String,
    pub company_order: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for releasing an issue tree hold.
#[derive(Debug, Clone)]
pub struct ReleaseTreeHold {
    /// Owning company id.
    pub company_id: String,
    /// Hold id.
    pub hold_id: String,
    /// Releasing actor type.
    pub released_by_actor_type: Option<String>,
    /// Releasing agent id.
    pub released_by_agent_id: Option<String>,
    /// Releasing user id.
    pub released_by_user_id: Option<String>,
    /// Releasing run id.
    pub released_by_run_id: Option<String>,
    /// Release reason.
    pub release_reason: Option<String>,
    /// Release metadata JSON.
    pub release_metadata: Option<serde_json::Value>,
}

/// Infrastructure repository errors.
#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    #[error("company not found")]
    CompanyNotFound,
    #[error("reference not found")]
    ReferenceNotFound,
    #[error("record already exists")]
    AlreadyExists,
    #[error("record not found")]
    NotFound,
}

/// Infrastructure persistence contract.
#[async_trait]
pub trait InfrastructureRepository: Send + Sync {
    // Auth ---------------------------------------------------------------
    async fn create_user(&self, input: NewUser) -> Result<UserRecord, InfrastructureError>;
    async fn get_user(&self, id: &str) -> Result<Option<UserRecord>, InfrastructureError>;
    async fn list_users(&self) -> Result<Vec<UserRecord>, InfrastructureError>;
    async fn create_session(&self, input: NewSession)
    -> Result<SessionRecord, InfrastructureError>;
    async fn list_sessions(&self, user_id: &str)
    -> Result<Vec<SessionRecord>, InfrastructureError>;

    // Instance settings ---------------------------------------------------
    async fn get_instance_settings(&self) -> Result<InstanceSettingsRecord, InfrastructureError>;
    async fn update_instance_settings(
        &self,
        default_environment_id: Option<Option<String>>,
        general: Option<serde_json::Value>,
        experimental: Option<serde_json::Value>,
    ) -> Result<InstanceSettingsRecord, InfrastructureError>;

    // Folders -------------------------------------------------------------
    async fn create_folder(&self, input: NewFolder) -> Result<FolderRecord, InfrastructureError>;
    async fn list_folders(
        &self,
        company_id: &str,
        kind: Option<&str>,
    ) -> Result<Vec<FolderRecord>, InfrastructureError>;
    async fn delete_folder(&self, company_id: &str, id: &str) -> Result<bool, InfrastructureError>;

    // Agent config revisions ----------------------------------------------
    async fn create_agent_config_revision(
        &self,
        input: NewAgentConfigRevision,
    ) -> Result<AgentConfigRevisionRecord, InfrastructureError>;
    async fn list_agent_config_revisions(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Vec<AgentConfigRevisionRecord>, InfrastructureError>;

    // Inbox dismissals -----------------------------------------------------
    async fn set_inbox_dismissal(
        &self,
        input: NewInboxDismissal,
    ) -> Result<InboxDismissalRecord, InfrastructureError>;
    async fn list_inbox_dismissals(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Vec<InboxDismissalRecord>, InfrastructureError>;
    async fn remove_inbox_dismissal(
        &self,
        company_id: &str,
        user_id: &str,
        item_key: &str,
    ) -> Result<bool, InfrastructureError>;

    // Document memberships -------------------------------------------------
    async fn set_document_membership(
        &self,
        input: NewDocumentMembership,
    ) -> Result<DocumentMembershipRecord, InfrastructureError>;
    async fn list_document_memberships(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Vec<DocumentMembershipRecord>, InfrastructureError>;

    // Routine documents ----------------------------------------------------
    async fn link_routine_document(
        &self,
        input: NewRoutineDocument,
    ) -> Result<RoutineDocumentRecord, InfrastructureError>;
    async fn list_routine_documents(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<Vec<RoutineDocumentRecord>, InfrastructureError>;

    // Approval comments ----------------------------------------------------
    async fn create_approval_comment(
        &self,
        input: NewApprovalComment,
    ) -> Result<ApprovalCommentRecord, InfrastructureError>;
    async fn list_approval_comments(
        &self,
        company_id: &str,
        approval_id: &str,
    ) -> Result<Vec<ApprovalCommentRecord>, InfrastructureError>;

    // Built-in managed resources -------------------------------------------
    async fn upsert_built_in_resource(
        &self,
        input: NewBuiltInResource,
    ) -> Result<BuiltInResourceRecord, InfrastructureError>;
    async fn list_built_in_resources(
        &self,
        company_id: &str,
    ) -> Result<Vec<BuiltInResourceRecord>, InfrastructureError>;

    // Issue create idempotency keys ----------------------------------------
    async fn reserve_issue_idempotency_key(
        &self,
        company_id: &str,
        idempotency_key: &str,
        issue_id: &str,
    ) -> Result<IssueIdempotencyRecord, InfrastructureError>;
    async fn get_issue_by_idempotency_key(
        &self,
        company_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<String>, InfrastructureError>;

    // Issue inbox archives -------------------------------------------------
    async fn archive_issue_inbox(
        &self,
        input: NewIssueInboxArchive,
    ) -> Result<IssueInboxArchiveRecord, InfrastructureError>;
    async fn list_issue_inbox_archives(
        &self,
        company_id: &str,
        issue_id: &str,
    ) -> Result<Vec<IssueInboxArchiveRecord>, InfrastructureError>;

    // Issue plan decompositions --------------------------------------------
    async fn create_plan_decomposition(
        &self,
        input: NewIssuePlanDecomposition,
    ) -> Result<IssuePlanDecompositionRecord, InfrastructureError>;
    async fn list_plan_decompositions(
        &self,
        company_id: &str,
        source_issue_id: &str,
    ) -> Result<Vec<IssuePlanDecompositionRecord>, InfrastructureError>;

    // Issue reference mentions ---------------------------------------------
    async fn create_reference_mention(
        &self,
        input: NewIssueReferenceMention,
    ) -> Result<IssueReferenceMentionRecord, InfrastructureError>;
    async fn list_reference_mentions(
        &self,
        company_id: &str,
        source_issue_id: &str,
    ) -> Result<Vec<IssueReferenceMentionRecord>, InfrastructureError>;

    // Issue tree holds -----------------------------------------------------
    async fn create_tree_hold(
        &self,
        input: NewIssueTreeHold,
    ) -> Result<IssueTreeHoldRecord, InfrastructureError>;
    async fn list_tree_holds(
        &self,
        company_id: &str,
        root_issue_id: &str,
    ) -> Result<Vec<IssueTreeHoldRecord>, InfrastructureError>;
    async fn release_tree_hold(
        &self,
        input: ReleaseTreeHold,
    ) -> Result<Option<IssueTreeHoldRecord>, InfrastructureError>;
    async fn add_tree_hold_member(
        &self,
        input: NewIssueTreeHoldMember,
    ) -> Result<IssueTreeHoldMemberRecord, InfrastructureError>;
    async fn list_tree_hold_members(
        &self,
        company_id: &str,
        hold_id: &str,
    ) -> Result<Vec<IssueTreeHoldMemberRecord>, InfrastructureError>;

    // Issue watchdogs ------------------------------------------------------
    async fn create_watchdog(
        &self,
        input: NewIssueWatchdog,
    ) -> Result<IssueWatchdogRecord, InfrastructureError>;
    async fn list_watchdogs(
        &self,
        company_id: &str,
        issue_id: &str,
    ) -> Result<Vec<IssueWatchdogRecord>, InfrastructureError>;
    async fn update_watchdog_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        last_triggered_at: Option<&str>,
        last_completed_at: Option<&str>,
    ) -> Result<Option<IssueWatchdogRecord>, InfrastructureError>;

    // Heartbeat run events -------------------------------------------------
    /// Resolves the owning company of a heartbeat run.
    async fn heartbeat_run_company(
        &self,
        run_id: &str,
    ) -> Result<Option<String>, InfrastructureError>;
    async fn append_heartbeat_event(
        &self,
        input: NewHeartbeatRunEvent,
    ) -> Result<HeartbeatRunEventRecord, InfrastructureError>;
    async fn list_heartbeat_events(
        &self,
        company_id: &str,
        run_id: &str,
    ) -> Result<Vec<HeartbeatRunEventRecord>, InfrastructureError>;

    // Heartbeat run watchdog decisions -------------------------------------
    async fn create_heartbeat_watchdog_decision(
        &self,
        input: NewHeartbeatWatchdogDecision,
    ) -> Result<HeartbeatWatchdogDecisionRecord, InfrastructureError>;
    async fn list_heartbeat_watchdog_decisions(
        &self,
        company_id: &str,
        run_id: &str,
    ) -> Result<Vec<HeartbeatWatchdogDecisionRecord>, InfrastructureError>;

    // Environment images/leases/sessions -----------------------------------
    async fn create_env_image_template(
        &self,
        input: NewEnvImageTemplate,
    ) -> Result<EnvImageTemplateRecord, InfrastructureError>;
    async fn list_env_image_templates(
        &self,
        environment_id: &str,
    ) -> Result<Vec<EnvImageTemplateRecord>, InfrastructureError>;
    async fn create_env_lease(
        &self,
        input: NewEnvLease,
    ) -> Result<EnvLeaseRecord, InfrastructureError>;
    async fn list_env_leases(
        &self,
        company_id: &str,
        environment_id: &str,
    ) -> Result<Vec<EnvLeaseRecord>, InfrastructureError>;
    async fn release_env_lease(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        released_at: Option<&str>,
        failure_reason: Option<&str>,
    ) -> Result<Option<EnvLeaseRecord>, InfrastructureError>;
    async fn create_env_setup_session(
        &self,
        input: NewEnvSetupSession,
    ) -> Result<EnvSetupSessionRecord, InfrastructureError>;
    async fn list_env_setup_sessions(
        &self,
        environment_id: &str,
    ) -> Result<Vec<EnvSetupSessionRecord>, InfrastructureError>;

    // User preferences -----------------------------------------------------
    async fn set_inbox_agent_policy(
        &self,
        input: NewInboxAgentPolicy,
    ) -> Result<InboxAgentPolicyRecord, InfrastructureError>;
    async fn get_inbox_agent_policy(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Option<InboxAgentPolicyRecord>, InfrastructureError>;
    async fn set_user_sidebar_preference(
        &self,
        user_id: &str,
        company_order: serde_json::Value,
    ) -> Result<UserSidebarPreferenceRecord, InfrastructureError>;
    async fn get_user_sidebar_preference(
        &self,
        user_id: &str,
    ) -> Result<Option<UserSidebarPreferenceRecord>, InfrastructureError>;
}

/// Turso/libSQL implementation of [`InfrastructureRepository`].
#[derive(Debug)]
pub struct TursoInfrastructureRepository {
    db: Database,
}

impl TursoInfrastructureRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InfrastructureRepository for TursoInfrastructureRepository {
    async fn create_user(&self, input: NewUser) -> Result<UserRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        conn.execute(
            &format!(
                "INSERT INTO user (id, name, email, email_verified, image, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, {now}, {now})",
                now = now
            ),
            libsql::params![
                input.id.clone(),
                input.name,
                input.email,
                i64::from(input.email_verified),
                input.image
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, name, email, email_verified, image, created_at, updated_at
                 FROM user WHERE id = ?1",
                libsql::params![input.id],
            )
            .await?;
        let row = rows.next().await?.expect("user was just inserted");
        Ok(row_to_user(&row)?)
    }

    async fn get_user(&self, id: &str) -> Result<Option<UserRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, name, email, email_verified, image, created_at, updated_at
                 FROM user WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn list_users(&self) -> Result<Vec<UserRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, name, email, email_verified, image, created_at, updated_at
                 FROM user ORDER BY created_at",
                (),
            )
            .await?;
        let mut users = Vec::new();
        while let Some(row) = rows.next().await? {
            users.push(row_to_user(&row)?);
        }
        Ok(users)
    }

    async fn create_session(
        &self,
        input: NewSession,
    ) -> Result<SessionRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        conn.execute(
            &format!(
                "INSERT INTO session (id, expires_at, token, created_at, updated_at, ip_address,
                                     user_agent, user_id)
                 VALUES (?1, ?2, ?3, {now}, {now}, ?4, ?5, ?6)",
                now = now
            ),
            libsql::params![
                input.id.clone(),
                input.expires_at,
                input.token,
                input.ip_address,
                input.user_agent,
                input.user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, expires_at, token, created_at, updated_at, ip_address, user_agent,
                        user_id
                 FROM session WHERE id = ?1",
                libsql::params![input.id],
            )
            .await?;
        let row = rows.next().await?.expect("session was just inserted");
        Ok(row_to_session(&row)?)
    }

    async fn list_sessions(
        &self,
        user_id: &str,
    ) -> Result<Vec<SessionRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, expires_at, token, created_at, updated_at, ip_address, user_agent,
                        user_id
                 FROM session WHERE user_id = ?1 ORDER BY created_at DESC",
                libsql::params![user_id],
            )
            .await?;
        let mut sessions = Vec::new();
        while let Some(row) = rows.next().await? {
            sessions.push(row_to_session(&row)?);
        }
        Ok(sessions)
    }

    async fn get_instance_settings(&self) -> Result<InstanceSettingsRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, singleton_key, default_environment_id, general, experimental,
                        created_at, updated_at
                 FROM instance_settings WHERE singleton_key = 'default' LIMIT 1",
                (),
            )
            .await?;
        if let Some(row) = rows.next().await? {
            return Ok(row_to_instance_settings(&row)?);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        conn.execute(
            &format!(
                "INSERT INTO instance_settings (id, singleton_key, general, experimental,
                                                created_at, updated_at)
                 VALUES (?1, 'default', '{{}}', '{{}}', {now}, {now})",
                now = now
            ),
            libsql::params![id.clone()],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, singleton_key, default_environment_id, general, experimental,
                        created_at, updated_at
                 FROM instance_settings WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("settings were just inserted");
        Ok(row_to_instance_settings(&row)?)
    }

    async fn update_instance_settings(
        &self,
        default_environment_id: Option<Option<String>>,
        general: Option<serde_json::Value>,
        experimental: Option<serde_json::Value>,
    ) -> Result<InstanceSettingsRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let current = self.get_instance_settings().await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut push = |column: &str, value: Option<libsql::Value>| {
            if let Some(value) = value {
                sets.push(format!("{column} = ?{}", values.len() + 1));
                values.push(value);
            }
        };
        push(
            "default_environment_id",
            default_environment_id.flatten().map(libsql::Value::from),
        );
        push(
            "general",
            general.map(|v| libsql::Value::from(v.to_string())),
        );
        push(
            "experimental",
            experimental.map(|v| libsql::Value::from(v.to_string())),
        );
        if !sets.is_empty() {
            let param = values.len() + 1;
            values.push(libsql::Value::from(current.id.clone()));
            conn.execute(
                &format!(
                    "UPDATE instance_settings SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE id = ?{param}",
                    sets.join(", ")
                ),
                values,
            )
            .await?;
        }
        let mut rows = conn
            .query(
                "SELECT id, singleton_key, default_environment_id, general, experimental,
                        created_at, updated_at
                 FROM instance_settings WHERE id = ?1",
                libsql::params![current.id],
            )
            .await?;
        let row = rows.next().await?.expect("settings exist");
        Ok(row_to_instance_settings(&row)?)
    }

    async fn create_folder(&self, input: NewFolder) -> Result<FolderRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(InfrastructureError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO folders (id, company_id, kind, parent_id, name, slug, system_key,
                                      color, position, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.kind,
                    input.parent_id,
                    input.name,
                    input.slug,
                    input.system_key,
                    input.color,
                    input.position
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, kind, parent_id, name, slug, system_key, color,
                                position, created_at, updated_at
                         FROM folders WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("folder was just inserted");
                Ok(row_to_folder(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_folders(
        &self,
        company_id: &str,
        kind: Option<&str>,
    ) -> Result<Vec<FolderRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let kind_filter = kind.map(|_| "AND kind = ?2").unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(kind) = kind {
            params.push(kind.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT id, company_id, kind, parent_id, name, slug, system_key, color,
                            position, created_at, updated_at
                     FROM folders WHERE company_id = ?1 {kind_filter}
                     ORDER BY position, name"
                ),
                params,
            )
            .await?;
        let mut folders = Vec::new();
        while let Some(row) = rows.next().await? {
            folders.push(row_to_folder(&row)?);
        }
        Ok(folders)
    }

    async fn delete_folder(&self, company_id: &str, id: &str) -> Result<bool, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let deleted = conn
            .execute(
                "DELETE FROM folders WHERE company_id = ?1 AND id = ?2",
                libsql::params![company_id, id],
            )
            .await?;
        Ok(deleted > 0)
    }

    async fn create_agent_config_revision(
        &self,
        input: NewAgentConfigRevision,
    ) -> Result<AgentConfigRevisionRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "agents", &input.agent_id, &input.company_id)
            .await?
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO agent_config_revisions (id, company_id, agent_id, created_by_agent_id,
                                                 created_by_user_id, source,
                                                 rolled_back_from_revision_id, changed_keys,
                                                 before_config, after_config, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.agent_id,
                input.created_by_agent_id,
                input.created_by_user_id,
                input.source,
                input.rolled_back_from_revision_id,
                input.changed_keys.to_string(),
                input.before_config.to_string(),
                input.after_config.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, agent_id, created_by_agent_id, created_by_user_id, source,
                        rolled_back_from_revision_id, changed_keys, before_config, after_config,
                        created_at
                 FROM agent_config_revisions WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("revision was just inserted");
        Ok(row_to_agent_config_revision(&row)?)
    }

    async fn list_agent_config_revisions(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Vec<AgentConfigRevisionRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, agent_id, created_by_agent_id, created_by_user_id, source,
                        rolled_back_from_revision_id, changed_keys, before_config, after_config,
                        created_at
                 FROM agent_config_revisions
                 WHERE company_id = ?1 AND agent_id = ?2 ORDER BY created_at DESC",
                libsql::params![company_id, agent_id],
            )
            .await?;
        let mut revisions = Vec::new();
        while let Some(row) = rows.next().await? {
            revisions.push(row_to_agent_config_revision(&row)?);
        }
        Ok(revisions)
    }

    async fn set_inbox_dismissal(
        &self,
        input: NewInboxDismissal,
    ) -> Result<InboxDismissalRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(InfrastructureError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_user_id = input.user_id.clone();
        let key_item_key = input.item_key.clone();
        conn.execute(
            &format!(
                "INSERT INTO inbox_dismissals (id, company_id, user_id, item_key, kind,
                                               dismissed_at, snoozed_until, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, {now}, ?6, {now}, {now})
                 ON CONFLICT (company_id, user_id, item_key) DO UPDATE SET
                   kind = excluded.kind,
                   snoozed_until = excluded.snoozed_until,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id.clone(),
                input.company_id,
                input.user_id,
                input.item_key,
                input.kind,
                input.snoozed_until
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, user_id, item_key, kind, dismissed_at, snoozed_until,
                        created_at, updated_at
                 FROM inbox_dismissals
                 WHERE company_id = ?1 AND user_id = ?2 AND item_key = ?3",
                libsql::params![key_company_id, key_user_id, key_item_key],
            )
            .await?;
        let row = rows.next().await?.expect("dismissal was just upserted");
        Ok(row_to_inbox_dismissal(&row)?)
    }

    async fn list_inbox_dismissals(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Vec<InboxDismissalRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, user_id, item_key, kind, dismissed_at, snoozed_until,
                        created_at, updated_at
                 FROM inbox_dismissals WHERE company_id = ?1 AND user_id = ?2
                 ORDER BY dismissed_at DESC",
                libsql::params![company_id, user_id],
            )
            .await?;
        let mut dismissals = Vec::new();
        while let Some(row) = rows.next().await? {
            dismissals.push(row_to_inbox_dismissal(&row)?);
        }
        Ok(dismissals)
    }

    async fn remove_inbox_dismissal(
        &self,
        company_id: &str,
        user_id: &str,
        item_key: &str,
    ) -> Result<bool, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let deleted = conn
            .execute(
                "DELETE FROM inbox_dismissals
                 WHERE company_id = ?1 AND user_id = ?2 AND item_key = ?3",
                libsql::params![company_id, user_id, item_key],
            )
            .await?;
        Ok(deleted > 0)
    }

    async fn set_document_membership(
        &self,
        input: NewDocumentMembership,
    ) -> Result<DocumentMembershipRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "documents",
            &input.document_id,
            &input.company_id,
        )
        .await?
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_user_id = input.user_id.clone();
        let key_document_id = input.document_id.clone();
        conn.execute(
            &format!(
                "INSERT INTO document_memberships (id, company_id, document_id, user_id,
                                                   starred_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, {now}, {now})
                 ON CONFLICT (company_id, user_id, document_id) DO UPDATE SET
                   starred_at = excluded.starred_at,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id.clone(),
                input.company_id,
                input.document_id,
                input.user_id,
                input.starred_at
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, document_id, user_id, starred_at, created_at, updated_at
                 FROM document_memberships
                 WHERE company_id = ?1 AND user_id = ?2 AND document_id = ?3",
                libsql::params![key_company_id, key_user_id, key_document_id],
            )
            .await?;
        let row = rows.next().await?.expect("membership was just upserted");
        Ok(row_to_document_membership(&row)?)
    }

    async fn list_document_memberships(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Vec<DocumentMembershipRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, document_id, user_id, starred_at, created_at, updated_at
                 FROM document_memberships WHERE company_id = ?1 AND user_id = ?2
                 ORDER BY starred_at DESC",
                libsql::params![company_id, user_id],
            )
            .await?;
        let mut memberships = Vec::new();
        while let Some(row) = rows.next().await? {
            memberships.push(row_to_document_membership(&row)?);
        }
        Ok(memberships)
    }

    async fn link_routine_document(
        &self,
        input: NewRoutineDocument,
    ) -> Result<RoutineDocumentRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "routines", &input.routine_id, &input.company_id)
            .await?
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        if helpers::row_company(&conn, "documents", &input.document_id).await?
            != Some(input.company_id.clone())
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO routine_documents (id, company_id, routine_id, document_id, key,
                                                created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.routine_id,
                    input.document_id,
                    input.key
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, routine_id, document_id, key, created_at, updated_at
                         FROM routine_documents WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("link was just inserted");
                Ok(row_to_routine_document(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_routine_documents(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<Vec<RoutineDocumentRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, routine_id, document_id, key, created_at, updated_at
                 FROM routine_documents WHERE company_id = ?1 AND routine_id = ?2
                 ORDER BY updated_at",
                libsql::params![company_id, routine_id],
            )
            .await?;
        let mut documents = Vec::new();
        while let Some(row) = rows.next().await? {
            documents.push(row_to_routine_document(&row)?);
        }
        Ok(documents)
    }

    async fn create_approval_comment(
        &self,
        input: NewApprovalComment,
    ) -> Result<ApprovalCommentRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "approvals",
            &input.approval_id,
            &input.company_id,
        )
        .await?
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO approval_comments (id, company_id, approval_id, author_agent_id,
                                            author_user_id, body, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.approval_id,
                input.author_agent_id,
                input.author_user_id,
                input.body
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, approval_id, author_agent_id, author_user_id, body,
                        created_at, updated_at
                 FROM approval_comments WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("comment was just inserted");
        Ok(row_to_approval_comment(&row)?)
    }

    async fn list_approval_comments(
        &self,
        company_id: &str,
        approval_id: &str,
    ) -> Result<Vec<ApprovalCommentRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, approval_id, author_agent_id, author_user_id, body,
                        created_at, updated_at
                 FROM approval_comments WHERE company_id = ?1 AND approval_id = ?2
                 ORDER BY created_at",
                libsql::params![company_id, approval_id],
            )
            .await?;
        let mut comments = Vec::new();
        while let Some(row) = rows.next().await? {
            comments.push(row_to_approval_comment(&row)?);
        }
        Ok(comments)
    }

    async fn upsert_built_in_resource(
        &self,
        input: NewBuiltInResource,
    ) -> Result<BuiltInResourceRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(InfrastructureError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_bundle_key = input.bundle_key.clone();
        let key_resource_kind = input.resource_kind.clone();
        let key_resource_key = input.resource_key.clone();
        conn.execute(
            &format!(
                "INSERT INTO built_in_managed_resources (id, company_id, bundle_key,
                                                         resource_kind, resource_key,
                                                         resource_id, stock_version, stock_hash,
                                                         defaults_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, {now}, {now})
                 ON CONFLICT (company_id, bundle_key, resource_kind, resource_key) DO UPDATE SET
                   resource_id = excluded.resource_id,
                   stock_version = excluded.stock_version,
                   stock_hash = excluded.stock_hash,
                   defaults_json = excluded.defaults_json,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id.clone(),
                input.company_id,
                input.bundle_key,
                input.resource_kind,
                input.resource_key,
                input.resource_id,
                input.stock_version,
                input.stock_hash,
                input.defaults_json.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, bundle_key, resource_kind, resource_key, resource_id,
                        stock_version, stock_hash, defaults_json, created_at, updated_at
                 FROM built_in_managed_resources
                 WHERE company_id = ?1 AND bundle_key = ?2 AND resource_kind = ?3
                   AND resource_key = ?4",
                libsql::params![
                    key_company_id,
                    key_bundle_key,
                    key_resource_kind,
                    key_resource_key
                ],
            )
            .await?;
        let row = rows.next().await?.expect("resource was just upserted");
        Ok(row_to_built_in_resource(&row)?)
    }

    async fn list_built_in_resources(
        &self,
        company_id: &str,
    ) -> Result<Vec<BuiltInResourceRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, bundle_key, resource_kind, resource_key, resource_id,
                        stock_version, stock_hash, defaults_json, created_at, updated_at
                 FROM built_in_managed_resources WHERE company_id = ?1 ORDER BY bundle_key",
                libsql::params![company_id],
            )
            .await?;
        let mut resources = Vec::new();
        while let Some(row) = rows.next().await? {
            resources.push(row_to_built_in_resource(&row)?);
        }
        Ok(resources)
    }

    async fn reserve_issue_idempotency_key(
        &self,
        company_id: &str,
        idempotency_key: &str,
        issue_id: &str,
    ) -> Result<IssueIdempotencyRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "issues", issue_id).await? != Some(company_id.to_owned()) {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_create_idempotency_keys (id, company_id, idempotency_key,
                                                            issue_id, created_at)
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, idempotency_key, issue_id],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, idempotency_key, issue_id, created_at
                         FROM issue_create_idempotency_keys WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("key was just inserted");
                Ok(row_to_issue_idempotency(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn get_issue_by_idempotency_key(
        &self,
        company_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<String>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT issue_id FROM issue_create_idempotency_keys
                 WHERE company_id = ?1 AND idempotency_key = ?2",
                libsql::params![company_id, idempotency_key],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(helpers::row_text(&row, 0)?),
            None => Ok(None),
        }
    }

    async fn archive_issue_inbox(
        &self,
        input: NewIssueInboxArchive,
    ) -> Result<IssueInboxArchiveRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "issues", &input.issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_issue_id = input.issue_id.clone();
        let key_user_id = input.user_id.clone();
        conn.execute(
            &format!(
                "INSERT INTO issue_inbox_archives (id, company_id, issue_id, user_id,
                                                   archived_by_actor_type, archived_by_agent_id,
                                                   archived_by_run_id, archived_at, created_at,
                                                   updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, {now}, {now}, {now})
                 ON CONFLICT (company_id, issue_id, user_id) DO UPDATE SET
                   archived_by_actor_type = excluded.archived_by_actor_type,
                   archived_by_agent_id = excluded.archived_by_agent_id,
                   archived_by_run_id = excluded.archived_by_run_id,
                   archived_at = {now},
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id.clone(),
                input.company_id,
                input.issue_id,
                input.user_id,
                input.archived_by_actor_type,
                input.archived_by_agent_id,
                input.archived_by_run_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, user_id, archived_by_actor_type,
                        archived_by_agent_id, archived_by_run_id, archived_at, created_at,
                        updated_at
                 FROM issue_inbox_archives
                 WHERE company_id = ?1 AND issue_id = ?2 AND user_id = ?3",
                libsql::params![key_company_id, key_issue_id, key_user_id],
            )
            .await?;
        let row = rows.next().await?.expect("archive was just upserted");
        Ok(row_to_issue_inbox_archive(&row)?)
    }

    async fn list_issue_inbox_archives(
        &self,
        company_id: &str,
        issue_id: &str,
    ) -> Result<Vec<IssueInboxArchiveRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, user_id, archived_by_actor_type,
                        archived_by_agent_id, archived_by_run_id, archived_at, created_at,
                        updated_at
                 FROM issue_inbox_archives WHERE company_id = ?1 AND issue_id = ?2
                 ORDER BY archived_at",
                libsql::params![company_id, issue_id],
            )
            .await?;
        let mut archives = Vec::new();
        while let Some(row) = rows.next().await? {
            archives.push(row_to_issue_inbox_archive(&row)?);
        }
        Ok(archives)
    }

    async fn create_plan_decomposition(
        &self,
        input: NewIssuePlanDecomposition,
    ) -> Result<IssuePlanDecompositionRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "issues", &input.source_issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_plan_decompositions (id, company_id, source_issue_id,
                                                        accepted_plan_revision_id,
                                                        accepted_interaction_id, status,
                                                        request_fingerprint,
                                                        requested_child_count,
                                                        requested_children, child_issue_ids,
                                                        owner_agent_id, owner_user_id,
                                                        owner_run_id, created_at,
                                                        updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.source_issue_id,
                    input.accepted_plan_revision_id,
                    input.accepted_interaction_id,
                    input.status,
                    input.request_fingerprint,
                    input.requested_child_count,
                    input.requested_children.to_string(),
                    input.child_issue_ids.to_string(),
                    input.owner_agent_id,
                    input.owner_user_id,
                    input.owner_run_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, source_issue_id, accepted_plan_revision_id,
                                accepted_interaction_id, status, request_fingerprint,
                                requested_child_count, requested_children, child_issue_ids,
                                owner_agent_id, owner_user_id, owner_run_id, completed_at,
                                created_at, updated_at
                         FROM issue_plan_decompositions WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("decomposition was just inserted");
                Ok(row_to_plan_decomposition(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_plan_decompositions(
        &self,
        company_id: &str,
        source_issue_id: &str,
    ) -> Result<Vec<IssuePlanDecompositionRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, source_issue_id, accepted_plan_revision_id,
                        accepted_interaction_id, status, request_fingerprint,
                        requested_child_count, requested_children, child_issue_ids,
                        owner_agent_id, owner_user_id, owner_run_id, completed_at,
                        created_at, updated_at
                 FROM issue_plan_decompositions
                 WHERE company_id = ?1 AND source_issue_id = ?2 ORDER BY created_at DESC",
                libsql::params![company_id, source_issue_id],
            )
            .await?;
        let mut decompositions = Vec::new();
        while let Some(row) = rows.next().await? {
            decompositions.push(row_to_plan_decomposition(&row)?);
        }
        Ok(decompositions)
    }

    async fn create_reference_mention(
        &self,
        input: NewIssueReferenceMention,
    ) -> Result<IssueReferenceMentionRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "issues", &input.source_issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        if helpers::row_company(&conn, "issues", &input.target_issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_reference_mentions (id, company_id, source_issue_id,
                                                       target_issue_id, source_kind,
                                                       source_record_id, document_key,
                                                       matched_text, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.source_issue_id,
                    input.target_issue_id,
                    input.source_kind,
                    input.source_record_id,
                    input.document_key,
                    input.matched_text
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, source_issue_id, target_issue_id, source_kind,
                                source_record_id, document_key, matched_text, created_at,
                                updated_at
                         FROM issue_reference_mentions WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("mention was just inserted");
                Ok(row_to_reference_mention(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_reference_mentions(
        &self,
        company_id: &str,
        source_issue_id: &str,
    ) -> Result<Vec<IssueReferenceMentionRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, source_issue_id, target_issue_id, source_kind,
                        source_record_id, document_key, matched_text, created_at, updated_at
                 FROM issue_reference_mentions
                 WHERE company_id = ?1 AND source_issue_id = ?2 ORDER BY created_at",
                libsql::params![company_id, source_issue_id],
            )
            .await?;
        let mut mentions = Vec::new();
        while let Some(row) = rows.next().await? {
            mentions.push(row_to_reference_mention(&row)?);
        }
        Ok(mentions)
    }

    async fn create_tree_hold(
        &self,
        input: NewIssueTreeHold,
    ) -> Result<IssueTreeHoldRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "issues", &input.root_issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO issue_tree_holds (id, company_id, root_issue_id, mode, status, reason,
                                           release_policy, created_by_actor_type,
                                           created_by_agent_id, created_by_user_id,
                                           created_by_run_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.root_issue_id,
                input.mode,
                input.status,
                input.reason,
                input.release_policy.map(|v| v.to_string()),
                input.created_by_actor_type,
                input.created_by_agent_id,
                input.created_by_user_id,
                input.created_by_run_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, root_issue_id, mode, status, reason, release_policy,
                        created_by_actor_type, created_by_agent_id, created_by_user_id,
                        created_by_run_id, released_at, released_by_actor_type,
                        released_by_agent_id, released_by_user_id, released_by_run_id,
                        release_reason, release_metadata, created_at, updated_at
                 FROM issue_tree_holds WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("hold was just inserted");
        Ok(row_to_tree_hold(&row)?)
    }

    async fn list_tree_holds(
        &self,
        company_id: &str,
        root_issue_id: &str,
    ) -> Result<Vec<IssueTreeHoldRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, root_issue_id, mode, status, reason, release_policy,
                        created_by_actor_type, created_by_agent_id, created_by_user_id,
                        created_by_run_id, released_at, released_by_actor_type,
                        released_by_agent_id, released_by_user_id, released_by_run_id,
                        release_reason, release_metadata, created_at, updated_at
                 FROM issue_tree_holds
                 WHERE company_id = ?1 AND root_issue_id = ?2 ORDER BY created_at",
                libsql::params![company_id, root_issue_id],
            )
            .await?;
        let mut holds = Vec::new();
        while let Some(row) = rows.next().await? {
            holds.push(row_to_tree_hold(&row)?);
        }
        Ok(holds)
    }

    async fn release_tree_hold(
        &self,
        input: ReleaseTreeHold,
    ) -> Result<Option<IssueTreeHoldRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "issue_tree_holds",
            &input.hold_id,
            &input.company_id,
        )
        .await?
        {
            return Ok(None);
        }
        conn.execute(
            "UPDATE issue_tree_holds
             SET status = 'released',
                 released_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 released_by_actor_type = ?1,
                 released_by_agent_id = ?2,
                 released_by_user_id = ?3,
                 released_by_run_id = ?4,
                 release_reason = ?5,
                 release_metadata = ?6,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?7 AND company_id = ?8",
            libsql::params![
                input.released_by_actor_type,
                input.released_by_agent_id,
                input.released_by_user_id,
                input.released_by_run_id,
                input.release_reason,
                input.release_metadata.map(|v| v.to_string()),
                input.hold_id.clone(),
                input.company_id.clone()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, root_issue_id, mode, status, reason, release_policy,
                        created_by_actor_type, created_by_agent_id, created_by_user_id,
                        created_by_run_id, released_at, released_by_actor_type,
                        released_by_agent_id, released_by_user_id, released_by_run_id,
                        release_reason, release_metadata, created_at, updated_at
                 FROM issue_tree_holds WHERE id = ?1",
                libsql::params![input.hold_id],
            )
            .await?;
        let row = rows.next().await?.expect("hold exists");
        Ok(Some(row_to_tree_hold(&row)?))
    }

    async fn add_tree_hold_member(
        &self,
        input: NewIssueTreeHoldMember,
    ) -> Result<IssueTreeHoldMemberRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "issue_tree_holds",
            &input.hold_id,
            &input.company_id,
        )
        .await?
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_tree_hold_members (id, company_id, hold_id, issue_id,
                                                      parent_issue_id, depth, issue_identifier,
                                                      issue_title, issue_status,
                                                      assignee_agent_id, assignee_user_id,
                                                      active_run_id, active_run_status, skipped,
                                                      skip_reason, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.hold_id,
                    input.issue_id,
                    input.parent_issue_id,
                    input.depth,
                    input.issue_identifier,
                    input.issue_title,
                    input.issue_status,
                    input.assignee_agent_id,
                    input.assignee_user_id,
                    input.active_run_id,
                    input.active_run_status,
                    i64::from(input.skipped),
                    input.skip_reason
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, hold_id, issue_id, parent_issue_id, depth,
                                issue_identifier, issue_title, issue_status, assignee_agent_id,
                                assignee_user_id, active_run_id, active_run_status, skipped,
                                skip_reason, created_at
                         FROM issue_tree_hold_members WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("member was just inserted");
                Ok(row_to_tree_hold_member(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_tree_hold_members(
        &self,
        company_id: &str,
        hold_id: &str,
    ) -> Result<Vec<IssueTreeHoldMemberRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, hold_id, issue_id, parent_issue_id, depth,
                        issue_identifier, issue_title, issue_status, assignee_agent_id,
                        assignee_user_id, active_run_id, active_run_status, skipped, skip_reason,
                        created_at
                 FROM issue_tree_hold_members
                 WHERE company_id = ?1 AND hold_id = ?2 ORDER BY depth",
                libsql::params![company_id, hold_id],
            )
            .await?;
        let mut members = Vec::new();
        while let Some(row) = rows.next().await? {
            members.push(row_to_tree_hold_member(&row)?);
        }
        Ok(members)
    }

    async fn create_watchdog(
        &self,
        input: NewIssueWatchdog,
    ) -> Result<IssueWatchdogRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if helpers::row_company(&conn, "issues", &input.issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(InfrastructureError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_watchdogs (id, company_id, issue_id, watchdog_agent_id,
                                              instructions, status, watchdog_issue_id,
                                              created_by_agent_id, created_by_user_id,
                                              created_by_run_id, trigger_count, created_at,
                                              updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.issue_id,
                    input.watchdog_agent_id,
                    input.instructions,
                    input.status,
                    input.watchdog_issue_id,
                    input.created_by_agent_id,
                    input.created_by_user_id,
                    input.created_by_run_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, issue_id, watchdog_agent_id, instructions, status,
                                watchdog_issue_id, last_observed_fingerprint,
                                last_reviewed_fingerprint, last_observed_stop_snapshot,
                                last_reviewed_stop_snapshot, last_triggered_at,
                                last_completed_at, trigger_count, created_by_agent_id,
                                created_by_user_id, created_by_run_id, updated_by_agent_id,
                                updated_by_user_id, updated_by_run_id, created_at, updated_at
                         FROM issue_watchdogs WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("watchdog was just inserted");
                Ok(row_to_watchdog(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_watchdogs(
        &self,
        company_id: &str,
        issue_id: &str,
    ) -> Result<Vec<IssueWatchdogRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, watchdog_agent_id, instructions, status,
                        watchdog_issue_id, last_observed_fingerprint,
                        last_reviewed_fingerprint, last_observed_stop_snapshot,
                        last_reviewed_stop_snapshot, last_triggered_at, last_completed_at,
                        trigger_count, created_by_agent_id, created_by_user_id,
                        created_by_run_id, updated_by_agent_id, updated_by_user_id,
                        updated_by_run_id, created_at, updated_at
                 FROM issue_watchdogs
                 WHERE company_id = ?1 AND issue_id = ?2 ORDER BY created_at",
                libsql::params![company_id, issue_id],
            )
            .await?;
        let mut watchdogs = Vec::new();
        while let Some(row) = rows.next().await? {
            watchdogs.push(row_to_watchdog(&row)?);
        }
        Ok(watchdogs)
    }

    async fn update_watchdog_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        last_triggered_at: Option<&str>,
        last_completed_at: Option<&str>,
    ) -> Result<Option<IssueWatchdogRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "issue_watchdogs", id, company_id).await? {
            return Ok(None);
        }
        conn.execute(
            "UPDATE issue_watchdogs
             SET status = ?1, last_triggered_at = ?2, last_completed_at = ?3,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?4 AND company_id = ?5",
            libsql::params![status, last_triggered_at, last_completed_at, id, company_id],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, watchdog_agent_id, instructions, status,
                        watchdog_issue_id, last_observed_fingerprint,
                        last_reviewed_fingerprint, last_observed_stop_snapshot,
                        last_reviewed_stop_snapshot, last_triggered_at, last_completed_at,
                        trigger_count, created_by_agent_id, created_by_user_id,
                        created_by_run_id, updated_by_agent_id, updated_by_user_id,
                        updated_by_run_id, created_at, updated_at
                 FROM issue_watchdogs WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("watchdog exists");
        Ok(Some(row_to_watchdog(&row)?))
    }

    async fn heartbeat_run_company(
        &self,
        run_id: &str,
    ) -> Result<Option<String>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT company_id FROM heartbeat_runs WHERE id = ?1",
                libsql::params![run_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(helpers::row_text(&row, 0)?),
            None => Ok(None),
        }
    }

    async fn append_heartbeat_event(
        &self,
        input: NewHeartbeatRunEvent,
    ) -> Result<HeartbeatRunEventRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        conn.execute(
            "INSERT INTO heartbeat_run_events (company_id, run_id, agent_id, seq, event_type,
                                               stream, level, color, message, payload,
                                               created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                input.company_id,
                input.run_id,
                input.agent_id,
                input.seq,
                input.event_type,
                input.stream,
                input.level,
                input.color,
                input.message,
                input.payload.map(|v| v.to_string())
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, run_id, agent_id, seq, event_type, stream, level, color,
                        message, payload, created_at
                 FROM heartbeat_run_events WHERE id = last_insert_rowid()",
                (),
            )
            .await?;
        let row = rows.next().await?.expect("event was just inserted");
        Ok(row_to_heartbeat_event(&row)?)
    }

    async fn list_heartbeat_events(
        &self,
        company_id: &str,
        run_id: &str,
    ) -> Result<Vec<HeartbeatRunEventRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, run_id, agent_id, seq, event_type, stream, level, color,
                        message, payload, created_at
                 FROM heartbeat_run_events WHERE company_id = ?1 AND run_id = ?2
                 ORDER BY seq",
                libsql::params![company_id, run_id],
            )
            .await?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().await? {
            events.push(row_to_heartbeat_event(&row)?);
        }
        Ok(events)
    }

    async fn create_heartbeat_watchdog_decision(
        &self,
        input: NewHeartbeatWatchdogDecision,
    ) -> Result<HeartbeatWatchdogDecisionRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO heartbeat_run_watchdog_decisions (id, company_id, run_id,
                                                           evaluation_issue_id, decision,
                                                           snoozed_until, reason,
                                                           created_by_agent_id,
                                                           created_by_user_id,
                                                           created_by_run_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.run_id,
                input.evaluation_issue_id,
                input.decision,
                input.snoozed_until,
                input.reason,
                input.created_by_agent_id,
                input.created_by_user_id,
                input.created_by_run_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, run_id, evaluation_issue_id, decision, snoozed_until,
                        reason, created_by_agent_id, created_by_user_id, created_by_run_id,
                        created_at
                 FROM heartbeat_run_watchdog_decisions WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("decision was just inserted");
        Ok(row_to_heartbeat_watchdog_decision(&row)?)
    }

    async fn list_heartbeat_watchdog_decisions(
        &self,
        company_id: &str,
        run_id: &str,
    ) -> Result<Vec<HeartbeatWatchdogDecisionRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, run_id, evaluation_issue_id, decision, snoozed_until,
                        reason, created_by_agent_id, created_by_user_id, created_by_run_id,
                        created_at
                 FROM heartbeat_run_watchdog_decisions
                 WHERE company_id = ?1 AND run_id = ?2 ORDER BY created_at",
                libsql::params![company_id, run_id],
            )
            .await?;
        let mut decisions = Vec::new();
        while let Some(row) = rows.next().await? {
            decisions.push(row_to_heartbeat_watchdog_decision(&row)?);
        }
        Ok(decisions)
    }

    async fn create_env_image_template(
        &self,
        input: NewEnvImageTemplate,
    ) -> Result<EnvImageTemplateRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO environment_custom_image_templates (id, environment_id, provider,
                                                                 template_kind, template_ref,
                                                                 source_template_ref,
                                                                 source_environment_config_fingerprint,
                                                                 status, created_by_user_id,
                                                                 created_by_agent_id, metadata,
                                                                 created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.environment_id,
                    input.provider,
                    input.template_kind,
                    input.template_ref,
                    input.source_template_ref,
                    input.source_environment_config_fingerprint,
                    input.status,
                    input.created_by_user_id,
                    input.created_by_agent_id,
                    input.metadata.map(|v| v.to_string())
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, environment_id, provider, template_kind, template_ref,
                                source_template_ref, source_environment_config_fingerprint,
                                status, created_by_user_id, created_by_agent_id, captured_at,
                                last_used_at, superseded_by_template_id, metadata, created_at,
                                updated_at
                         FROM environment_custom_image_templates WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("template was just inserted");
                Ok(row_to_env_image_template(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_env_image_templates(
        &self,
        environment_id: &str,
    ) -> Result<Vec<EnvImageTemplateRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, environment_id, provider, template_kind, template_ref,
                        source_template_ref, source_environment_config_fingerprint, status,
                        created_by_user_id, created_by_agent_id, captured_at, last_used_at,
                        superseded_by_template_id, metadata, created_at, updated_at
                 FROM environment_custom_image_templates WHERE environment_id = ?1
                 ORDER BY created_at",
                libsql::params![environment_id],
            )
            .await?;
        let mut templates = Vec::new();
        while let Some(row) = rows.next().await? {
            templates.push(row_to_env_image_template(&row)?);
        }
        Ok(templates)
    }

    async fn create_env_lease(
        &self,
        input: NewEnvLease,
    ) -> Result<EnvLeaseRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(InfrastructureError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO environment_leases (id, company_id, environment_id,
                                             execution_workspace_id, issue_id,
                                             heartbeat_run_id, status, lease_policy, provider,
                                             provider_lease_id, acquired_at, last_used_at,
                                             expires_at, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?11, ?12,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.environment_id,
                input.execution_workspace_id,
                input.issue_id,
                input.heartbeat_run_id,
                input.status,
                input.lease_policy,
                input.provider,
                input.provider_lease_id,
                input.expires_at,
                input.metadata.map(|v| v.to_string())
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, environment_id, execution_workspace_id, issue_id,
                        heartbeat_run_id, status, lease_policy, provider, provider_lease_id,
                        acquired_at, last_used_at, expires_at, released_at, failure_reason,
                        cleanup_status, metadata, created_at, updated_at
                 FROM environment_leases WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("lease was just inserted");
        Ok(row_to_env_lease(&row)?)
    }

    async fn list_env_leases(
        &self,
        company_id: &str,
        environment_id: &str,
    ) -> Result<Vec<EnvLeaseRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, environment_id, execution_workspace_id, issue_id,
                        heartbeat_run_id, status, lease_policy, provider, provider_lease_id,
                        acquired_at, last_used_at, expires_at, released_at, failure_reason,
                        cleanup_status, metadata, created_at, updated_at
                 FROM environment_leases
                 WHERE company_id = ?1 AND environment_id = ?2 ORDER BY acquired_at",
                libsql::params![company_id, environment_id],
            )
            .await?;
        let mut leases = Vec::new();
        while let Some(row) = rows.next().await? {
            leases.push(row_to_env_lease(&row)?);
        }
        Ok(leases)
    }

    async fn release_env_lease(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
        released_at: Option<&str>,
        failure_reason: Option<&str>,
    ) -> Result<Option<EnvLeaseRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "environment_leases", id, company_id).await? {
            return Ok(None);
        }
        conn.execute(
            "UPDATE environment_leases
             SET status = ?1, released_at = ?2, failure_reason = ?3,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?4 AND company_id = ?5",
            libsql::params![status, released_at, failure_reason, id, company_id],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, environment_id, execution_workspace_id, issue_id,
                        heartbeat_run_id, status, lease_policy, provider, provider_lease_id,
                        acquired_at, last_used_at, expires_at, released_at, failure_reason,
                        cleanup_status, metadata, created_at, updated_at
                 FROM environment_leases WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("lease exists");
        Ok(Some(row_to_env_lease(&row)?))
    }

    async fn create_env_setup_session(
        &self,
        input: NewEnvSetupSession,
    ) -> Result<EnvSetupSessionRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO environment_custom_image_setup_sessions (id, environment_id,
                                                                      template_id,
                                                                      promoted_template_id,
                                                                      provider,
                                                                      provider_lease_id,
                                                                      environment_lease_id,
                                                                      status,
                                                                      started_by_user_id,
                                                                      started_by_agent_id,
                                                                      base_template_ref,
                                                                      expires_at, metadata,
                                                                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.environment_id,
                    input.template_id,
                    input.promoted_template_id,
                    input.provider,
                    input.provider_lease_id,
                    input.environment_lease_id,
                    input.status,
                    input.started_by_user_id,
                    input.started_by_agent_id,
                    input.base_template_ref,
                    input.expires_at,
                    input.metadata.map(|v| v.to_string())
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, environment_id, template_id, promoted_template_id, provider,
                                provider_lease_id, environment_lease_id, status,
                                started_by_user_id, started_by_agent_id, base_template_ref,
                                expires_at, finished_at, failure_reason, connection_summary,
                                connection_secret_ref, metadata, created_at, updated_at
                         FROM environment_custom_image_setup_sessions WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("session was just inserted");
                Ok(row_to_env_setup_session(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InfrastructureError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_env_setup_sessions(
        &self,
        environment_id: &str,
    ) -> Result<Vec<EnvSetupSessionRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, environment_id, template_id, promoted_template_id, provider,
                        provider_lease_id, environment_lease_id, status, started_by_user_id,
                        started_by_agent_id, base_template_ref, expires_at, finished_at,
                        failure_reason, connection_summary, connection_secret_ref, metadata,
                        created_at, updated_at
                 FROM environment_custom_image_setup_sessions WHERE environment_id = ?1
                 ORDER BY created_at",
                libsql::params![environment_id],
            )
            .await?;
        let mut sessions = Vec::new();
        while let Some(row) = rows.next().await? {
            sessions.push(row_to_env_setup_session(&row)?);
        }
        Ok(sessions)
    }

    async fn set_inbox_agent_policy(
        &self,
        input: NewInboxAgentPolicy,
    ) -> Result<InboxAgentPolicyRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(InfrastructureError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_user_id = input.user_id.clone();
        conn.execute(
            &format!(
                "INSERT INTO user_inbox_agent_policies (id, company_id, user_id, mode,
                                                        allowed_agent_ids, created_at,
                                                        updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, {now}, {now})
                 ON CONFLICT (company_id, user_id) DO UPDATE SET
                   mode = excluded.mode,
                   allowed_agent_ids = excluded.allowed_agent_ids,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id.clone(),
                input.company_id,
                input.user_id,
                input.mode,
                input.allowed_agent_ids.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, user_id, mode, allowed_agent_ids, created_at, updated_at
                 FROM user_inbox_agent_policies WHERE company_id = ?1 AND user_id = ?2",
                libsql::params![key_company_id, key_user_id],
            )
            .await?;
        let row = rows.next().await?.expect("policy was just upserted");
        Ok(row_to_inbox_agent_policy(&row)?)
    }

    async fn get_inbox_agent_policy(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Option<InboxAgentPolicyRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, user_id, mode, allowed_agent_ids, created_at, updated_at
                 FROM user_inbox_agent_policies WHERE company_id = ?1 AND user_id = ?2",
                libsql::params![company_id, user_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_inbox_agent_policy(&row)?)),
            None => Ok(None),
        }
    }

    async fn set_user_sidebar_preference(
        &self,
        user_id: &str,
        company_order: serde_json::Value,
    ) -> Result<UserSidebarPreferenceRecord, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        conn.execute(
            &format!(
                "INSERT INTO user_sidebar_preferences (id, user_id, company_order, created_at,
                                                       updated_at)
                 VALUES (?1, ?2, ?3, {now}, {now})
                 ON CONFLICT (user_id) DO UPDATE SET
                   company_order = excluded.company_order,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![id.clone(), user_id, company_order.to_string()],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, user_id, company_order, created_at, updated_at
                 FROM user_sidebar_preferences WHERE user_id = ?1",
                libsql::params![user_id],
            )
            .await?;
        let row = rows.next().await?.expect("preference was just upserted");
        Ok(row_to_user_sidebar_preference(&row)?)
    }

    async fn get_user_sidebar_preference(
        &self,
        user_id: &str,
    ) -> Result<Option<UserSidebarPreferenceRecord>, InfrastructureError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, user_id, company_order, created_at, updated_at
                 FROM user_sidebar_preferences WHERE user_id = ?1",
                libsql::params![user_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_user_sidebar_preference(&row)?)),
            None => Ok(None),
        }
    }
}

fn row_to_user(row: &libsql::Row) -> Result<UserRecord, libsql::Error> {
    Ok(UserRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        name: helpers::row_text(row, 1)?.expect("name"),
        email: helpers::row_text(row, 2)?.expect("email"),
        email_verified: helpers::row_i64(row, 3)? != 0,
        image: helpers::row_text(row, 4)?,
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
        updated_at: helpers::row_text(row, 6)?.expect("updated_at"),
    })
}

fn row_to_session(row: &libsql::Row) -> Result<SessionRecord, libsql::Error> {
    Ok(SessionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        expires_at: helpers::row_text(row, 1)?.expect("expires_at"),
        token: helpers::row_text(row, 2)?.expect("token"),
        created_at: helpers::row_text(row, 3)?.expect("created_at"),
        updated_at: helpers::row_text(row, 4)?.expect("updated_at"),
        ip_address: helpers::row_text(row, 5)?,
        user_agent: helpers::row_text(row, 6)?,
        user_id: helpers::row_text(row, 7)?.expect("user_id"),
    })
}

fn row_to_instance_settings(row: &libsql::Row) -> Result<InstanceSettingsRecord, libsql::Error> {
    Ok(InstanceSettingsRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        singleton_key: helpers::row_text(row, 1)?.expect("singleton_key"),
        default_environment_id: helpers::row_text(row, 2)?,
        general: json_or_default(helpers::row_text(row, 3)?),
        experimental: json_or_default(helpers::row_text(row, 4)?),
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
        updated_at: helpers::row_text(row, 6)?.expect("updated_at"),
    })
}

fn row_to_folder(row: &libsql::Row) -> Result<FolderRecord, libsql::Error> {
    Ok(FolderRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        kind: helpers::row_text(row, 2)?.expect("kind"),
        parent_id: helpers::row_text(row, 3)?,
        name: helpers::row_text(row, 4)?.expect("name"),
        slug: helpers::row_text(row, 5)?.expect("slug"),
        system_key: helpers::row_text(row, 6)?,
        color: helpers::row_text(row, 7)?,
        position: helpers::row_i64(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
        updated_at: helpers::row_text(row, 10)?.expect("updated_at"),
    })
}

fn row_to_agent_config_revision(
    row: &libsql::Row,
) -> Result<AgentConfigRevisionRecord, libsql::Error> {
    Ok(AgentConfigRevisionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        agent_id: helpers::row_text(row, 2)?.expect("agent_id"),
        created_by_agent_id: helpers::row_text(row, 3)?,
        created_by_user_id: helpers::row_text(row, 4)?,
        source: helpers::row_text(row, 5)?.expect("source"),
        rolled_back_from_revision_id: helpers::row_text(row, 6)?,
        changed_keys: json_or_default(helpers::row_text(row, 7)?),
        before_config: json_or_default(helpers::row_text(row, 8)?),
        after_config: json_or_default(helpers::row_text(row, 9)?),
        created_at: helpers::row_text(row, 10)?.expect("created_at"),
    })
}

fn row_to_inbox_dismissal(row: &libsql::Row) -> Result<InboxDismissalRecord, libsql::Error> {
    Ok(InboxDismissalRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        user_id: helpers::row_text(row, 2)?.expect("user_id"),
        item_key: helpers::row_text(row, 3)?.expect("item_key"),
        kind: helpers::row_text(row, 4)?.expect("kind"),
        dismissed_at: helpers::row_text(row, 5)?.expect("dismissed_at"),
        snoozed_until: helpers::row_text(row, 6)?,
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
        updated_at: helpers::row_text(row, 8)?.expect("updated_at"),
    })
}

fn row_to_document_membership(
    row: &libsql::Row,
) -> Result<DocumentMembershipRecord, libsql::Error> {
    Ok(DocumentMembershipRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        document_id: helpers::row_text(row, 2)?.expect("document_id"),
        user_id: helpers::row_text(row, 3)?.expect("user_id"),
        starred_at: helpers::row_text(row, 4)?,
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
        updated_at: helpers::row_text(row, 6)?.expect("updated_at"),
    })
}

fn row_to_routine_document(row: &libsql::Row) -> Result<RoutineDocumentRecord, libsql::Error> {
    Ok(RoutineDocumentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        routine_id: helpers::row_text(row, 2)?.expect("routine_id"),
        document_id: helpers::row_text(row, 3)?.expect("document_id"),
        key: helpers::row_text(row, 4)?.expect("key"),
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
        updated_at: helpers::row_text(row, 6)?.expect("updated_at"),
    })
}

fn row_to_approval_comment(row: &libsql::Row) -> Result<ApprovalCommentRecord, libsql::Error> {
    Ok(ApprovalCommentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        approval_id: helpers::row_text(row, 2)?.expect("approval_id"),
        author_agent_id: helpers::row_text(row, 3)?,
        author_user_id: helpers::row_text(row, 4)?,
        body: helpers::row_text(row, 5)?.expect("body"),
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
        updated_at: helpers::row_text(row, 7)?.expect("updated_at"),
    })
}

fn row_to_built_in_resource(row: &libsql::Row) -> Result<BuiltInResourceRecord, libsql::Error> {
    Ok(BuiltInResourceRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        bundle_key: helpers::row_text(row, 2)?.expect("bundle_key"),
        resource_kind: helpers::row_text(row, 3)?.expect("resource_kind"),
        resource_key: helpers::row_text(row, 4)?.expect("resource_key"),
        resource_id: helpers::row_text(row, 5)?.expect("resource_id"),
        stock_version: helpers::row_text(row, 6)?.expect("stock_version"),
        stock_hash: helpers::row_text(row, 7)?.expect("stock_hash"),
        defaults_json: json_or_default(helpers::row_text(row, 8)?),
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
        updated_at: helpers::row_text(row, 10)?.expect("updated_at"),
    })
}

fn row_to_issue_idempotency(row: &libsql::Row) -> Result<IssueIdempotencyRecord, libsql::Error> {
    Ok(IssueIdempotencyRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        idempotency_key: helpers::row_text(row, 2)?.expect("idempotency_key"),
        issue_id: helpers::row_text(row, 3)?.expect("issue_id"),
        created_at: helpers::row_text(row, 4)?.expect("created_at"),
    })
}

fn row_to_issue_inbox_archive(row: &libsql::Row) -> Result<IssueInboxArchiveRecord, libsql::Error> {
    Ok(IssueInboxArchiveRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id"),
        user_id: helpers::row_text(row, 3)?.expect("user_id"),
        archived_by_actor_type: helpers::row_text(row, 4)?.expect("archived_by_actor_type"),
        archived_by_agent_id: helpers::row_text(row, 5)?,
        archived_by_run_id: helpers::row_text(row, 6)?,
        archived_at: helpers::row_text(row, 7)?.expect("archived_at"),
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
        updated_at: helpers::row_text(row, 9)?.expect("updated_at"),
    })
}

fn row_to_plan_decomposition(
    row: &libsql::Row,
) -> Result<IssuePlanDecompositionRecord, libsql::Error> {
    Ok(IssuePlanDecompositionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        source_issue_id: helpers::row_text(row, 2)?.expect("source_issue_id"),
        accepted_plan_revision_id: helpers::row_text(row, 3)?.expect("accepted_plan_revision_id"),
        accepted_interaction_id: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status"),
        request_fingerprint: helpers::row_text(row, 6)?.expect("request_fingerprint"),
        requested_child_count: helpers::row_i64(row, 7)?,
        requested_children: json_or_default(helpers::row_text(row, 8)?),
        child_issue_ids: json_or_default(helpers::row_text(row, 9)?),
        owner_agent_id: helpers::row_text(row, 10)?,
        owner_user_id: helpers::row_text(row, 11)?,
        owner_run_id: helpers::row_text(row, 12)?,
        completed_at: helpers::row_text(row, 13)?,
        created_at: helpers::row_text(row, 14)?.expect("created_at"),
        updated_at: helpers::row_text(row, 15)?.expect("updated_at"),
    })
}

fn row_to_reference_mention(
    row: &libsql::Row,
) -> Result<IssueReferenceMentionRecord, libsql::Error> {
    Ok(IssueReferenceMentionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        source_issue_id: helpers::row_text(row, 2)?.expect("source_issue_id"),
        target_issue_id: helpers::row_text(row, 3)?.expect("target_issue_id"),
        source_kind: helpers::row_text(row, 4)?.expect("source_kind"),
        source_record_id: helpers::row_text(row, 5)?,
        document_key: helpers::row_text(row, 6)?,
        matched_text: helpers::row_text(row, 7)?,
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
        updated_at: helpers::row_text(row, 9)?.expect("updated_at"),
    })
}

fn row_to_tree_hold(row: &libsql::Row) -> Result<IssueTreeHoldRecord, libsql::Error> {
    Ok(IssueTreeHoldRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        root_issue_id: helpers::row_text(row, 2)?.expect("root_issue_id"),
        mode: helpers::row_text(row, 3)?.expect("mode"),
        status: helpers::row_text(row, 4)?.expect("status"),
        reason: helpers::row_text(row, 5)?,
        release_policy: helpers::row_text(row, 6)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_by_actor_type: helpers::row_text(row, 7)?.expect("created_by_actor_type"),
        created_by_agent_id: helpers::row_text(row, 8)?,
        created_by_user_id: helpers::row_text(row, 9)?,
        created_by_run_id: helpers::row_text(row, 10)?,
        released_at: helpers::row_text(row, 11)?,
        released_by_actor_type: helpers::row_text(row, 12)?,
        released_by_agent_id: helpers::row_text(row, 13)?,
        released_by_user_id: helpers::row_text(row, 14)?,
        released_by_run_id: helpers::row_text(row, 15)?,
        release_reason: helpers::row_text(row, 16)?,
        release_metadata: helpers::row_text(row, 17)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: helpers::row_text(row, 18)?.expect("created_at"),
        updated_at: helpers::row_text(row, 19)?.expect("updated_at"),
    })
}

fn row_to_tree_hold_member(row: &libsql::Row) -> Result<IssueTreeHoldMemberRecord, libsql::Error> {
    Ok(IssueTreeHoldMemberRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        hold_id: helpers::row_text(row, 2)?.expect("hold_id"),
        issue_id: helpers::row_text(row, 3)?.expect("issue_id"),
        parent_issue_id: helpers::row_text(row, 4)?,
        depth: helpers::row_i64(row, 5)?,
        issue_identifier: helpers::row_text(row, 6)?,
        issue_title: helpers::row_text(row, 7)?.expect("issue_title"),
        issue_status: helpers::row_text(row, 8)?.expect("issue_status"),
        assignee_agent_id: helpers::row_text(row, 9)?,
        assignee_user_id: helpers::row_text(row, 10)?,
        active_run_id: helpers::row_text(row, 11)?,
        active_run_status: helpers::row_text(row, 12)?,
        skipped: helpers::row_i64(row, 13)? != 0,
        skip_reason: helpers::row_text(row, 14)?,
        created_at: helpers::row_text(row, 15)?.expect("created_at"),
    })
}

fn row_to_watchdog(row: &libsql::Row) -> Result<IssueWatchdogRecord, libsql::Error> {
    Ok(IssueWatchdogRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id"),
        watchdog_agent_id: helpers::row_text(row, 3)?.expect("watchdog_agent_id"),
        instructions: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status"),
        watchdog_issue_id: helpers::row_text(row, 6)?,
        last_observed_fingerprint: helpers::row_text(row, 7)?,
        last_reviewed_fingerprint: helpers::row_text(row, 8)?,
        last_observed_stop_snapshot: helpers::row_text(row, 9)?
            .and_then(|v| serde_json::from_str(&v).ok()),
        last_reviewed_stop_snapshot: helpers::row_text(row, 10)?
            .and_then(|v| serde_json::from_str(&v).ok()),
        last_triggered_at: helpers::row_text(row, 11)?,
        last_completed_at: helpers::row_text(row, 12)?,
        trigger_count: helpers::row_i64(row, 13)?,
        created_by_agent_id: helpers::row_text(row, 14)?,
        created_by_user_id: helpers::row_text(row, 15)?,
        created_by_run_id: helpers::row_text(row, 16)?,
        updated_by_agent_id: helpers::row_text(row, 17)?,
        updated_by_user_id: helpers::row_text(row, 18)?,
        updated_by_run_id: helpers::row_text(row, 19)?,
        created_at: helpers::row_text(row, 20)?.expect("created_at"),
        updated_at: helpers::row_text(row, 21)?.expect("updated_at"),
    })
}

fn row_to_heartbeat_event(row: &libsql::Row) -> Result<HeartbeatRunEventRecord, libsql::Error> {
    Ok(HeartbeatRunEventRecord {
        id: helpers::row_i64(row, 0)?,
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        run_id: helpers::row_text(row, 2)?.expect("run_id"),
        agent_id: helpers::row_text(row, 3)?.expect("agent_id"),
        seq: helpers::row_i64(row, 4)?,
        event_type: helpers::row_text(row, 5)?.expect("event_type"),
        stream: helpers::row_text(row, 6)?,
        level: helpers::row_text(row, 7)?,
        color: helpers::row_text(row, 8)?,
        message: helpers::row_text(row, 9)?,
        payload: helpers::row_text(row, 10)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
    })
}

fn row_to_heartbeat_watchdog_decision(
    row: &libsql::Row,
) -> Result<HeartbeatWatchdogDecisionRecord, libsql::Error> {
    Ok(HeartbeatWatchdogDecisionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        run_id: helpers::row_text(row, 2)?.expect("run_id"),
        evaluation_issue_id: helpers::row_text(row, 3)?,
        decision: helpers::row_text(row, 4)?.expect("decision"),
        snoozed_until: helpers::row_text(row, 5)?,
        reason: helpers::row_text(row, 6)?,
        created_by_agent_id: helpers::row_text(row, 7)?,
        created_by_user_id: helpers::row_text(row, 8)?,
        created_by_run_id: helpers::row_text(row, 9)?,
        created_at: helpers::row_text(row, 10)?.expect("created_at"),
    })
}

fn row_to_env_image_template(row: &libsql::Row) -> Result<EnvImageTemplateRecord, libsql::Error> {
    Ok(EnvImageTemplateRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        environment_id: helpers::row_text(row, 1)?.expect("environment_id"),
        provider: helpers::row_text(row, 2)?.expect("provider"),
        template_kind: helpers::row_text(row, 3)?.expect("template_kind"),
        template_ref: helpers::row_text(row, 4)?.expect("template_ref"),
        source_template_ref: helpers::row_text(row, 5)?,
        source_environment_config_fingerprint: helpers::row_text(row, 6)?,
        status: helpers::row_text(row, 7)?.expect("status"),
        created_by_user_id: helpers::row_text(row, 8)?,
        created_by_agent_id: helpers::row_text(row, 9)?,
        captured_at: helpers::row_text(row, 10)?,
        last_used_at: helpers::row_text(row, 11)?,
        superseded_by_template_id: helpers::row_text(row, 12)?,
        metadata: helpers::row_text(row, 13)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: helpers::row_text(row, 14)?.expect("created_at"),
        updated_at: helpers::row_text(row, 15)?.expect("updated_at"),
    })
}

fn row_to_env_lease(row: &libsql::Row) -> Result<EnvLeaseRecord, libsql::Error> {
    Ok(EnvLeaseRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        environment_id: helpers::row_text(row, 2)?.expect("environment_id"),
        execution_workspace_id: helpers::row_text(row, 3)?,
        issue_id: helpers::row_text(row, 4)?,
        heartbeat_run_id: helpers::row_text(row, 5)?,
        status: helpers::row_text(row, 6)?.expect("status"),
        lease_policy: helpers::row_text(row, 7)?.expect("lease_policy"),
        provider: helpers::row_text(row, 8)?,
        provider_lease_id: helpers::row_text(row, 9)?,
        acquired_at: helpers::row_text(row, 10)?.expect("acquired_at"),
        last_used_at: helpers::row_text(row, 11)?.expect("last_used_at"),
        expires_at: helpers::row_text(row, 12)?,
        released_at: helpers::row_text(row, 13)?,
        failure_reason: helpers::row_text(row, 14)?,
        cleanup_status: helpers::row_text(row, 15)?,
        metadata: helpers::row_text(row, 16)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: helpers::row_text(row, 17)?.expect("created_at"),
        updated_at: helpers::row_text(row, 18)?.expect("updated_at"),
    })
}

fn row_to_env_setup_session(row: &libsql::Row) -> Result<EnvSetupSessionRecord, libsql::Error> {
    Ok(EnvSetupSessionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        environment_id: helpers::row_text(row, 1)?.expect("environment_id"),
        template_id: helpers::row_text(row, 2)?,
        promoted_template_id: helpers::row_text(row, 3)?,
        provider: helpers::row_text(row, 4)?.expect("provider"),
        provider_lease_id: helpers::row_text(row, 5)?,
        environment_lease_id: helpers::row_text(row, 6)?,
        status: helpers::row_text(row, 7)?.expect("status"),
        started_by_user_id: helpers::row_text(row, 8)?,
        started_by_agent_id: helpers::row_text(row, 9)?,
        base_template_ref: helpers::row_text(row, 10)?,
        expires_at: helpers::row_text(row, 11)?,
        finished_at: helpers::row_text(row, 12)?,
        failure_reason: helpers::row_text(row, 13)?,
        connection_summary: helpers::row_text(row, 14)?.and_then(|v| serde_json::from_str(&v).ok()),
        connection_secret_ref: helpers::row_text(row, 15)?,
        metadata: helpers::row_text(row, 16)?.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: helpers::row_text(row, 17)?.expect("created_at"),
        updated_at: helpers::row_text(row, 18)?.expect("updated_at"),
    })
}

fn row_to_inbox_agent_policy(row: &libsql::Row) -> Result<InboxAgentPolicyRecord, libsql::Error> {
    Ok(InboxAgentPolicyRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        user_id: helpers::row_text(row, 2)?.expect("user_id"),
        mode: helpers::row_text(row, 3)?.expect("mode"),
        allowed_agent_ids: json_or_default(helpers::row_text(row, 4)?),
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
        updated_at: helpers::row_text(row, 6)?.expect("updated_at"),
    })
}

fn row_to_user_sidebar_preference(
    row: &libsql::Row,
) -> Result<UserSidebarPreferenceRecord, libsql::Error> {
    Ok(UserSidebarPreferenceRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        user_id: helpers::row_text(row, 1)?.expect("user_id"),
        company_order: json_or_default(helpers::row_text(row, 2)?),
        created_at: helpers::row_text(row, 3)?.expect("created_at"),
        updated_at: helpers::row_text(row, 4)?.expect("updated_at"),
    })
}

fn json_or_default(value: Option<String>) -> serde_json::Value {
    value
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoInfrastructureRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'Agent', 'engineer', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'Issue 1', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i2', 'c1', 'Issue 2', 2, 'ALPHA-2')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO heartbeat_runs (id, company_id, agent_id, invocation_source)
             VALUES ('r1', 'c1', 'a1', 'manual')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO environments (id, name, driver) VALUES ('e1', 'dev', 'local')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO documents (id, company_id, title) VALUES ('d1', 'c1', 'Doc')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO document_revisions (id, company_id, document_id, revision_number, body)
             VALUES ('dr1', 'c1', 'd1', 1, 'v1')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoInfrastructureRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn auth_settings_folders_and_events() {
        let (_dir, repo) = repo().await;

        // Auth users + sessions.
        let user = repo
            .create_user(NewUser {
                id: "u1".to_owned(),
                name: "Alice".to_owned(),
                email: "alice@example.com".to_owned(),
                email_verified: true,
                image: None,
            })
            .await
            .unwrap();
        assert_eq!(user.name, "Alice");
        assert!(user.email_verified);
        let session = repo
            .create_session(NewSession {
                id: "s1".to_owned(),
                expires_at: "2026-12-31T00:00:00.000Z".to_owned(),
                token: "tok".to_owned(),
                ip_address: None,
                user_agent: Some("curl".to_owned()),
                user_id: "u1".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(session.user_id, "u1");
        assert_eq!(repo.list_sessions("u1").await.unwrap().len(), 1);

        // Instance settings.
        let settings = repo.get_instance_settings().await.unwrap();
        assert_eq!(settings.singleton_key, "default");
        let updated = repo
            .update_instance_settings(
                Some(Some("e1".to_owned())),
                Some(serde_json::json!({ "theme": "dark" })),
                None,
            )
            .await
            .unwrap();
        assert_eq!(updated.default_environment_id.as_deref(), Some("e1"));
        assert_eq!(updated.general["theme"], "dark");

        // Folders.
        let folder = repo
            .create_folder(NewFolder {
                company_id: "c1".to_owned(),
                kind: "issues".to_owned(),
                parent_id: None,
                name: "Inbox".to_owned(),
                slug: "inbox".to_owned(),
                system_key: Some("inbox".to_owned()),
                color: None,
                position: 0,
            })
            .await
            .unwrap();
        assert_eq!(folder.slug, "inbox");
        assert_eq!(
            repo.list_folders("c1", Some("issues")).await.unwrap().len(),
            1
        );
        // Duplicate root slug rejected.
        assert!(matches!(
            repo.create_folder(NewFolder {
                company_id: "c1".to_owned(),
                kind: "issues".to_owned(),
                parent_id: None,
                name: "Inbox 2".to_owned(),
                slug: "inbox".to_owned(),
                system_key: None,
                color: None,
                position: 1,
            })
            .await
            .unwrap_err(),
            InfrastructureError::AlreadyExists
        ));
        assert!(repo.delete_folder("c1", &folder.id).await.unwrap());

        // Heartbeat events + watchdog decision.
        let event = repo
            .append_heartbeat_event(NewHeartbeatRunEvent {
                company_id: "c1".to_owned(),
                run_id: "r1".to_owned(),
                agent_id: "a1".to_owned(),
                seq: 1,
                event_type: "info".to_owned(),
                stream: Some("stdout".to_owned()),
                level: Some("info".to_owned()),
                color: None,
                message: Some("hello".to_owned()),
                payload: Some(serde_json::json!({ "k": 1 })),
            })
            .await
            .unwrap();
        assert!(event.id > 0);
        assert_eq!(
            repo.list_heartbeat_events("c1", "r1").await.unwrap().len(),
            1
        );

        let decision = repo
            .create_heartbeat_watchdog_decision(NewHeartbeatWatchdogDecision {
                company_id: "c1".to_owned(),
                run_id: "r1".to_owned(),
                evaluation_issue_id: Some("i1".to_owned()),
                decision: "continue".to_owned(),
                snoozed_until: None,
                reason: Some("ok".to_owned()),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
                created_by_run_id: Some("r1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(decision.decision, "continue");
        assert_eq!(
            repo.list_heartbeat_watchdog_decisions("c1", "r1")
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn watchdogs_holds_decompositions_and_mentions() {
        let (_dir, repo) = repo().await;

        // Watchdog.
        let watchdog = repo
            .create_watchdog(NewIssueWatchdog {
                company_id: "c1".to_owned(),
                issue_id: "i1".to_owned(),
                watchdog_agent_id: "a1".to_owned(),
                instructions: Some("watch".to_owned()),
                status: "active".to_owned(),
                watchdog_issue_id: None,
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
                created_by_run_id: Some("r1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(watchdog.status, "active");
        // One watchdog per issue.
        assert!(matches!(
            repo.create_watchdog(NewIssueWatchdog {
                company_id: "c1".to_owned(),
                issue_id: "i1".to_owned(),
                watchdog_agent_id: "a1".to_owned(),
                instructions: None,
                status: "active".to_owned(),
                watchdog_issue_id: None,
                created_by_agent_id: None,
                created_by_user_id: None,
                created_by_run_id: None,
            })
            .await
            .unwrap_err(),
            InfrastructureError::AlreadyExists
        ));
        let updated = repo
            .update_watchdog_status(
                "c1",
                &watchdog.id,
                "paused",
                Some("2026-08-04T00:00:00.000Z"),
                None,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "paused");

        // Tree holds + members.
        let hold = repo
            .create_tree_hold(NewIssueTreeHold {
                company_id: "c1".to_owned(),
                root_issue_id: "i1".to_owned(),
                mode: "freeze".to_owned(),
                status: "active".to_owned(),
                reason: Some("release freeze".to_owned()),
                release_policy: None,
                created_by_actor_type: "user".to_owned(),
                created_by_agent_id: None,
                created_by_user_id: Some("u1".to_owned()),
                created_by_run_id: None,
            })
            .await
            .unwrap();
        let member = repo
            .add_tree_hold_member(NewIssueTreeHoldMember {
                company_id: "c1".to_owned(),
                hold_id: hold.id.clone(),
                issue_id: "i2".to_owned(),
                parent_issue_id: Some("i1".to_owned()),
                depth: 1,
                issue_identifier: Some("ALPHA-2".to_owned()),
                issue_title: "Issue 2".to_owned(),
                issue_status: "todo".to_owned(),
                assignee_agent_id: None,
                assignee_user_id: None,
                active_run_id: None,
                active_run_status: None,
                skipped: false,
                skip_reason: None,
            })
            .await
            .unwrap();
        assert_eq!(member.issue_title, "Issue 2");
        assert_eq!(
            repo.list_tree_hold_members("c1", &hold.id)
                .await
                .unwrap()
                .len(),
            1
        );
        let released = repo
            .release_tree_hold(ReleaseTreeHold {
                company_id: "c1".to_owned(),
                hold_id: hold.id.clone(),
                released_by_actor_type: Some("user".to_owned()),
                released_by_agent_id: None,
                released_by_user_id: Some("u1".to_owned()),
                released_by_run_id: None,
                release_reason: Some("done".to_owned()),
                release_metadata: Some(serde_json::json!({ "ok": true })),
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(released.status, "released");

        // Plan decomposition.
        let plan = repo
            .create_plan_decomposition(NewIssuePlanDecomposition {
                company_id: "c1".to_owned(),
                source_issue_id: "i1".to_owned(),
                accepted_plan_revision_id: "dr1".to_owned(),
                accepted_interaction_id: None,
                status: "in_flight".to_owned(),
                request_fingerprint: "fp".to_owned(),
                requested_child_count: 2,
                requested_children: serde_json::json!([]),
                child_issue_ids: serde_json::json!(["i2"]),
                owner_agent_id: Some("a1".to_owned()),
                owner_user_id: None,
                owner_run_id: Some("r1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(plan.requested_child_count, 2);
        assert_eq!(
            repo.list_plan_decompositions("c1", "i1")
                .await
                .unwrap()
                .len(),
            1
        );

        // Reference mention + dedupe.
        let mention = repo
            .create_reference_mention(NewIssueReferenceMention {
                company_id: "c1".to_owned(),
                source_issue_id: "i1".to_owned(),
                target_issue_id: "i2".to_owned(),
                source_kind: "description".to_owned(),
                source_record_id: None,
                document_key: None,
                matched_text: Some("i2".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(mention.source_kind, "description");
        assert!(matches!(
            repo.create_reference_mention(NewIssueReferenceMention {
                company_id: "c1".to_owned(),
                source_issue_id: "i1".to_owned(),
                target_issue_id: "i2".to_owned(),
                source_kind: "description".to_owned(),
                source_record_id: None,
                document_key: None,
                matched_text: None,
            })
            .await
            .unwrap_err(),
            InfrastructureError::AlreadyExists
        ));
        assert_eq!(
            repo.list_reference_mentions("c1", "i1")
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn environment_images_leases_sessions_and_preferences() {
        let (_dir, repo) = repo().await;

        // Image template.
        let template = repo
            .create_env_image_template(NewEnvImageTemplate {
                environment_id: "e1".to_owned(),
                provider: "docker".to_owned(),
                template_kind: "base".to_owned(),
                template_ref: "ref-1".to_owned(),
                source_template_ref: None,
                source_environment_config_fingerprint: None,
                status: "active".to_owned(),
                created_by_user_id: Some("u1".to_owned()),
                created_by_agent_id: None,
                metadata: Some(serde_json::json!({ "size": 100 })),
            })
            .await
            .unwrap();
        assert_eq!(template.template_ref, "ref-1");
        assert_eq!(repo.list_env_image_templates("e1").await.unwrap().len(), 1);

        // Lease.
        let lease = repo
            .create_env_lease(NewEnvLease {
                company_id: "c1".to_owned(),
                environment_id: "e1".to_owned(),
                execution_workspace_id: None,
                issue_id: Some("i1".to_owned()),
                heartbeat_run_id: Some("r1".to_owned()),
                status: "active".to_owned(),
                lease_policy: "ephemeral".to_owned(),
                provider: Some("docker".to_owned()),
                provider_lease_id: Some("pl-1".to_owned()),
                expires_at: None,
                metadata: None,
            })
            .await
            .unwrap();
        assert_eq!(lease.status, "active");
        let released = repo
            .release_env_lease(
                "c1",
                &lease.id,
                "released",
                Some("2026-08-04T00:00:00.000Z"),
                None,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(released.status, "released");

        // Setup session.
        let session = repo
            .create_env_setup_session(NewEnvSetupSession {
                environment_id: "e1".to_owned(),
                template_id: Some(template.id.clone()),
                promoted_template_id: None,
                provider: "docker".to_owned(),
                provider_lease_id: None,
                environment_lease_id: Some(lease.id.clone()),
                status: "starting".to_owned(),
                started_by_user_id: Some("u1".to_owned()),
                started_by_agent_id: None,
                base_template_ref: Some("ref-1".to_owned()),
                expires_at: None,
                metadata: None,
            })
            .await
            .unwrap();
        assert_eq!(session.status, "starting");
        assert_eq!(repo.list_env_setup_sessions("e1").await.unwrap().len(), 1);

        // Inbox agent policy upsert.
        let policy = repo
            .set_inbox_agent_policy(NewInboxAgentPolicy {
                company_id: "c1".to_owned(),
                user_id: "u1".to_owned(),
                mode: "allowlist".to_owned(),
                allowed_agent_ids: serde_json::json!(["a1"]),
            })
            .await
            .unwrap();
        assert_eq!(policy.mode, "allowlist");
        let policy2 = repo
            .set_inbox_agent_policy(NewInboxAgentPolicy {
                company_id: "c1".to_owned(),
                user_id: "u1".to_owned(),
                mode: "open".to_owned(),
                allowed_agent_ids: serde_json::json!([]),
            })
            .await
            .unwrap();
        assert_eq!(policy2.id, policy.id);
        assert_eq!(policy2.mode, "open");

        // User sidebar preference upsert.
        let pref = repo
            .set_user_sidebar_preference("u1", serde_json::json!(["c1"]))
            .await
            .unwrap();
        assert_eq!(pref.company_order[0], "c1");
        assert!(
            repo.get_user_sidebar_preference("u1")
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn dismissals_memberships_routine_docs_comments_resources_and_cross_company() {
        let (_dir, repo) = repo().await;

        // Inbox dismissals.
        let dismissal = repo
            .set_inbox_dismissal(NewInboxDismissal {
                company_id: "c1".to_owned(),
                user_id: "u1".to_owned(),
                item_key: "k1".to_owned(),
                kind: "snooze".to_owned(),
                snoozed_until: Some("2026-09-01T00:00:00.000Z".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(dismissal.kind, "snooze");
        assert_eq!(
            repo.list_inbox_dismissals("c1", "u1").await.unwrap().len(),
            1
        );
        assert!(repo.remove_inbox_dismissal("c1", "u1", "k1").await.unwrap());

        // Document membership.
        let membership = repo
            .set_document_membership(NewDocumentMembership {
                company_id: "c1".to_owned(),
                document_id: "d1".to_owned(),
                user_id: "u1".to_owned(),
                starred_at: Some("2026-08-04T00:00:00.000Z".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(membership.user_id, "u1");
        assert_eq!(
            repo.list_document_memberships("c1", "u1")
                .await
                .unwrap()
                .len(),
            1
        );

        // Cross-company folders are not visible.
        assert!(repo.list_folders("c2", None).await.unwrap().is_empty());
    }
}

#[cfg(test)]
mod approval_comment_tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    #[tokio::test]
    async fn approval_comment_roundtrip() {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'Agent', 'engineer', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO approvals (id, company_id, type, requested_by_agent_id)
             VALUES ('ap1', 'c1', 'hire_agent', 'a1')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoInfrastructureRepository::new(db);
        let comment = repo
            .create_approval_comment(NewApprovalComment {
                company_id: "c1".to_owned(),
                approval_id: "ap1".to_owned(),
                author_agent_id: Some("a1".to_owned()),
                author_user_id: None,
                body: "looks good".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(comment.body, "looks good");
        assert_eq!(
            repo.list_approval_comments("c1", "ap1")
                .await
                .unwrap()
                .len(),
            1
        );

        // Cross-company approval rejected.
        assert!(matches!(
            repo.create_approval_comment(NewApprovalComment {
                company_id: "c2".to_owned(),
                approval_id: "ap1".to_owned(),
                author_agent_id: None,
                author_user_id: Some("u1".to_owned()),
                body: "nope".to_owned(),
            })
            .await
            .unwrap_err(),
            InfrastructureError::ReferenceNotFound
        ));
    }
}
