//! Toolchain domain: tool applications, connections, grants, catalog entries,
//! profiles, MCP gateways, invocations, and audit/telemetry records
//! (upstream tool_access.ts).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// Toolchain repository error.
#[derive(Debug, Error)]
pub enum ToolchainError {
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    #[error("company not found")]
    CompanyNotFound,
    #[error("referenced row not found or belongs to another company")]
    ReferenceNotFound,
    #[error("record already exists")]
    AlreadyExists,
    #[error("invalid input")]
    InvalidInput,
    #[error("record not found")]
    NotFound,
}

/// Maps a libSQL insert/update error to a [`ToolchainError`].
fn map_error(error: libsql::Error) -> ToolchainError {
    let message = error.to_string();
    if message.contains("UNIQUE constraint failed") {
        ToolchainError::AlreadyExists
    } else if message.contains("FOREIGN KEY constraint failed") {
        ToolchainError::ReferenceNotFound
    } else if message.contains("CHECK constraint failed") {
        ToolchainError::InvalidInput
    } else {
        ToolchainError::Db(error)
    }
}

/// A tool application (upstream `tool_applications`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolApplicationRecord {
    pub id: String,
    pub company_id: String,
    pub application_key: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub r#type: String,
    pub status: String,
    pub plugin_id: Option<String>,
    pub owner_agent_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub metadata: serde_json::Value,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool application.
#[derive(Debug, Clone)]
pub struct NewToolApplication {
    pub company_id: String,
    pub application_key: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub r#type: String,
    pub status: String,
    pub plugin_id: Option<String>,
    pub owner_agent_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub metadata: serde_json::Value,
}

/// A tool connection (upstream `tool_connections`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConnectionRecord {
    pub id: String,
    pub company_id: String,
    pub application_id: String,
    pub name: String,
    pub uid: String,
    pub connection_kind: String,
    pub ownership: String,
    pub transport: String,
    pub auth_kind: String,
    pub status: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub transport_config: serde_json::Value,
    pub credential_refs: serde_json::Value,
    pub credential_secret_refs: serde_json::Value,
    pub health_status: String,
    pub health_message: Option<String>,
    pub health_checked_at: Option<String>,
    pub last_health_at: Option<String>,
    pub last_catalog_refresh_at: Option<String>,
    pub last_error: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool connection.
#[derive(Debug, Clone)]
pub struct NewToolConnection {
    pub company_id: String,
    pub application_id: String,
    pub name: String,
    pub uid: String,
    pub connection_kind: String,
    pub ownership: String,
    pub transport: String,
    pub auth_kind: String,
    pub status: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub transport_config: serde_json::Value,
    pub credential_refs: serde_json::Value,
    pub credential_secret_refs: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// Partial update for a tool connection.
#[derive(Debug, Clone)]
pub struct UpdateToolConnection {
    pub company_id: String,
    pub id: String,
    pub name: Option<String>,
    pub status: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
    pub transport_config: Option<serde_json::Value>,
    pub health_status: Option<String>,
    pub health_message: Option<String>,
    pub health_checked_at: Option<String>,
    pub last_error: Option<String>,
}

/// A connection grant (upstream `connection_grants`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionGrantRecord {
    pub id: String,
    pub company_id: String,
    pub connection_id: String,
    pub kind: String,
    pub subject_user_id: Option<String>,
    pub provider_tenant: Option<serde_json::Value>,
    pub credential_secret_refs: serde_json::Value,
    pub status: String,
    pub is_default: bool,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub revoked_at: Option<String>,
    pub revoked_by_agent_id: Option<String>,
    pub revoked_by_user_id: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a connection grant.
#[derive(Debug, Clone)]
pub struct NewConnectionGrant {
    pub company_id: String,
    pub connection_id: String,
    pub kind: String,
    pub subject_user_id: Option<String>,
    pub provider_tenant: Option<serde_json::Value>,
    pub credential_secret_refs: serde_json::Value,
    pub status: String,
    pub is_default: bool,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A tool connection install (upstream `tool_connection_installs`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConnectionInstallRecord {
    pub id: String,
    pub company_id: String,
    pub connection_id: String,
    pub target_type: String,
    pub target_id: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_at: String,
}

/// Input for creating a tool connection install.
#[derive(Debug, Clone)]
pub struct NewToolConnectionInstall {
    pub company_id: String,
    pub connection_id: String,
    pub target_type: String,
    pub target_id: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A tool OAuth state (upstream `tool_oauth_states`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOauthStateRecord {
    pub state: String,
    pub company_id: String,
    pub connection_id: String,
    pub code_verifier: String,
    pub created_by_actor_type: Option<String>,
    pub created_by_actor_id: Option<String>,
    pub created_by_session_id: Option<String>,
    pub subject_user_id: Option<String>,
    pub requested_scopes: Option<serde_json::Value>,
    pub return_to: Option<String>,
    pub issue_id: Option<String>,
    pub interaction_id: Option<String>,
    pub expires_at: String,
    pub created_at: String,
}

/// Input for creating a tool OAuth state.
#[derive(Debug, Clone)]
pub struct NewToolOauthState {
    pub state: String,
    pub company_id: String,
    pub connection_id: String,
    pub code_verifier: String,
    pub created_by_actor_type: Option<String>,
    pub created_by_actor_id: Option<String>,
    pub created_by_session_id: Option<String>,
    pub subject_user_id: Option<String>,
    pub requested_scopes: Option<serde_json::Value>,
    pub return_to: Option<String>,
    pub issue_id: Option<String>,
    pub interaction_id: Option<String>,
    pub expires_at: String,
}

/// A tool catalog entry (upstream `tool_catalog_entries`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogEntryRecord {
    pub id: String,
    pub company_id: String,
    pub application_id: Option<String>,
    pub connection_id: String,
    pub entry_kind: String,
    pub name: String,
    pub tool_name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
    pub annotations: serde_json::Value,
    pub risk_level: String,
    pub is_read_only: bool,
    pub is_write: bool,
    pub is_destructive: bool,
    pub status: String,
    pub version: Option<String>,
    pub version_hash: String,
    pub schema_hash: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by_agent_id: Option<String>,
    pub reviewed_by_user_id: Option<String>,
    pub quarantined_at: Option<String>,
    pub quarantine_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool catalog entry.
#[derive(Debug, Clone)]
pub struct NewToolCatalogEntry {
    pub company_id: String,
    pub application_id: Option<String>,
    pub connection_id: String,
    pub entry_kind: String,
    pub name: String,
    pub tool_name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
    pub annotations: serde_json::Value,
    pub risk_level: String,
    pub is_read_only: bool,
    pub is_write: bool,
    pub is_destructive: bool,
    pub status: String,
    pub version: Option<String>,
    pub version_hash: String,
    pub schema_hash: Option<String>,
    pub reviewed_by_agent_id: Option<String>,
    pub reviewed_by_user_id: Option<String>,
}

/// A tool profile (upstream `tool_profiles`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolProfileRecord {
    pub id: String,
    pub company_id: String,
    pub profile_key: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub default_action: String,
    pub new_tools_reviewed_at: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool profile.
#[derive(Debug, Clone)]
pub struct NewToolProfile {
    pub company_id: String,
    pub profile_key: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub default_action: String,
    pub new_tools_reviewed_at: Option<String>,
    pub metadata: serde_json::Value,
}

/// Partial update for a tool profile.
#[derive(Debug, Clone)]
pub struct UpdateToolProfile {
    pub company_id: String,
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub default_action: Option<String>,
    pub new_tools_reviewed_at: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// A tool profile entry (upstream `tool_profile_entries`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolProfileEntryRecord {
    pub id: String,
    pub company_id: String,
    pub profile_id: String,
    pub selector_type: String,
    pub effect: String,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub tool_name: Option<String>,
    pub risk_level: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool profile entry.
#[derive(Debug, Clone)]
pub struct NewToolProfileEntry {
    pub company_id: String,
    pub profile_id: String,
    pub selector_type: String,
    pub effect: String,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub tool_name: Option<String>,
    pub risk_level: Option<String>,
    pub conditions: Option<serde_json::Value>,
}

/// A tool profile binding (upstream `tool_profile_bindings`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolProfileBindingRecord {
    pub id: String,
    pub company_id: String,
    pub profile_id: String,
    pub target_type: String,
    pub target_id: String,
    pub priority: i64,
    pub metadata: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool profile binding.
#[derive(Debug, Clone)]
pub struct NewToolProfileBinding {
    pub company_id: String,
    pub profile_id: String,
    pub target_type: String,
    pub target_id: String,
    pub priority: i64,
    pub metadata: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A tool MCP gateway (upstream `tool_mcp_gateways`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMcpGatewayRecord {
    pub id: String,
    pub company_id: String,
    pub gateway_public_id: String,
    pub name: String,
    pub slug: String,
    pub display_slug: String,
    pub description: Option<String>,
    pub status: String,
    pub profile_id: String,
    pub default_profile_mode: String,
    pub context_scope_type: String,
    pub context_scope_id: Option<String>,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub issue_id: Option<String>,
    pub approval_issue_id: Option<String>,
    pub auth_config: serde_json::Value,
    pub header_policy: serde_json::Value,
    pub metadata_policy: serde_json::Value,
    pub on_demand_tools_config: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool MCP gateway.
#[derive(Debug, Clone)]
pub struct NewToolMcpGateway {
    pub company_id: String,
    pub name: String,
    pub slug: String,
    pub display_slug: String,
    pub description: Option<String>,
    pub status: String,
    pub profile_id: String,
    pub default_profile_mode: String,
    pub context_scope_type: String,
    pub context_scope_id: Option<String>,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub issue_id: Option<String>,
    pub approval_issue_id: Option<String>,
    pub auth_config: serde_json::Value,
    pub header_policy: serde_json::Value,
    pub metadata_policy: serde_json::Value,
    pub on_demand_tools_config: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A tool MCP gateway token (upstream `tool_mcp_gateway_tokens`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMcpGatewayTokenRecord {
    pub id: String,
    pub company_id: String,
    pub gateway_id: String,
    pub name: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub subject_type: String,
    pub subject_id: Option<String>,
    pub client_label: String,
    pub owner_note: String,
    pub allowed_actions: serde_json::Value,
    pub expires_at: Option<String>,
    pub expiry_override_reason: Option<String>,
    pub expiry_override_by_user_id: Option<String>,
    pub expiry_override_by_agent_id: Option<String>,
    pub expiry_override_at: Option<String>,
    pub last_used_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool MCP gateway token.
#[derive(Debug, Clone)]
pub struct NewToolMcpGatewayToken {
    pub company_id: String,
    pub gateway_id: String,
    pub name: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub subject_type: String,
    pub subject_id: Option<String>,
    pub client_label: String,
    pub owner_note: String,
    pub allowed_actions: serde_json::Value,
    pub expires_at: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A tool policy (upstream `tool_policies`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPolicyRecord {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
    pub policy_type: String,
    pub priority: i64,
    pub enabled: bool,
    pub selectors: serde_json::Value,
    pub conditions: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool policy.
#[derive(Debug, Clone)]
pub struct NewToolPolicy {
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
    pub policy_type: String,
    pub priority: i64,
    pub enabled: bool,
    pub selectors: serde_json::Value,
    pub conditions: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A tool runtime slot (upstream `tool_runtime_slots`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRuntimeSlotRecord {
    pub id: String,
    pub company_id: String,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub project_workspace_id: Option<String>,
    pub execution_workspace_id: Option<String>,
    pub issue_id: Option<String>,
    pub owner_scope_type: String,
    pub owner_scope_id: Option<String>,
    pub runtime_kind: String,
    pub slot_key: String,
    pub status: String,
    pub reuse_key: Option<String>,
    pub workspace_scope: Option<String>,
    pub credential_scope_hash: Option<String>,
    pub provider: Option<String>,
    pub provider_ref: Option<String>,
    pub process_id: Option<i64>,
    pub command_template_key: Option<String>,
    pub health_status: String,
    pub health_message: Option<String>,
    pub last_health_check_at: Option<String>,
    pub last_started_at: Option<String>,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub last_used_at: Option<String>,
    pub idle_expires_at: Option<String>,
    pub idle_deadline_at: Option<String>,
    pub last_error: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool runtime slot.
#[derive(Debug, Clone)]
pub struct NewToolRuntimeSlot {
    pub company_id: String,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub project_workspace_id: Option<String>,
    pub execution_workspace_id: Option<String>,
    pub issue_id: Option<String>,
    pub owner_scope_type: String,
    pub owner_scope_id: Option<String>,
    pub runtime_kind: String,
    pub slot_key: String,
    pub status: String,
    pub reuse_key: Option<String>,
    pub workspace_scope: Option<String>,
    pub credential_scope_hash: Option<String>,
    pub provider: Option<String>,
    pub provider_ref: Option<String>,
    pub process_id: Option<i64>,
    pub command_template_key: Option<String>,
    pub health_status: String,
    pub health_message: Option<String>,
    pub metadata: serde_json::Value,
}

/// A tool stdio command template (upstream `tool_stdio_command_templates`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStdioCommandTemplateRecord {
    pub id: String,
    pub company_id: String,
    pub template_key: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub command: String,
    pub args: serde_json::Value,
    pub env_keys: serde_json::Value,
    pub tools: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub disabled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool stdio command template.
#[derive(Debug, Clone)]
pub struct NewToolStdioCommandTemplate {
    pub company_id: String,
    pub template_key: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub command: String,
    pub args: serde_json::Value,
    pub env_keys: serde_json::Value,
    pub tools: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A tool gateway session (upstream `tool_gateway_sessions`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolGatewaySessionRecord {
    pub id: String,
    pub company_id: String,
    pub agent_id: String,
    pub run_id: String,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_subject_type: Option<String>,
    pub client_subject_id: Option<String>,
    pub client_name: Option<String>,
    pub mcp_session_id: Option<String>,
    pub correlation_id: Option<String>,
    pub token_hash: String,
    pub expires_at: String,
    pub last_used_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool gateway session.
#[derive(Debug, Clone)]
pub struct NewToolGatewaySession {
    pub company_id: String,
    pub agent_id: String,
    pub run_id: String,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_subject_type: Option<String>,
    pub client_subject_id: Option<String>,
    pub client_name: Option<String>,
    pub mcp_session_id: Option<String>,
    pub correlation_id: Option<String>,
    pub token_hash: String,
    pub expires_at: String,
}

/// A tool gateway rate-limit counter (upstream `tool_gateway_rate_limit_counters`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolGatewayRateLimitCounterRecord {
    pub id: String,
    pub company_id: String,
    pub counter_key: String,
    pub window_start_at: String,
    pub window_ms: i64,
    pub limit: i64,
    pub count: i64,
    pub reset_at: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting a tool gateway rate-limit counter.
#[derive(Debug, Clone)]
pub struct NewToolGatewayRateLimitCounter {
    pub company_id: String,
    pub counter_key: String,
    pub window_start_at: String,
    pub window_ms: i64,
    pub limit: i64,
    pub count: i64,
    pub reset_at: String,
}

/// A tool invocation (upstream `tool_invocations`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInvocationRecord {
    pub id: String,
    pub company_id: String,
    pub idempotency_key: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub agent_id: Option<String>,
    pub issue_id: Option<String>,
    pub run_id: Option<String>,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_subject_type: Option<String>,
    pub client_subject_id: Option<String>,
    pub client_name: Option<String>,
    pub mcp_session_id: Option<String>,
    pub correlation_id: Option<String>,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub catalog_version_hash: Option<String>,
    pub catalog_schema_hash: Option<String>,
    pub provider_type: Option<String>,
    pub application_key: Option<String>,
    pub upstream_tool_name: Option<String>,
    pub risk_level: Option<String>,
    pub tool_name: String,
    pub arguments_hash: Option<String>,
    pub arguments_summary: Option<serde_json::Value>,
    pub policy_decision: Option<String>,
    pub matched_policy_ids: serde_json::Value,
    pub policy_explanation: Option<serde_json::Value>,
    pub credential_scope_summary: Option<serde_json::Value>,
    pub header_policy_summary: Option<serde_json::Value>,
    pub approval_state: String,
    pub status: String,
    pub upstream_request_id: Option<String>,
    pub result_hash: Option<String>,
    pub result_summary: Option<serde_json::Value>,
    pub result_size_bytes: Option<i64>,
    pub result_artifact_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool invocation.
#[derive(Debug, Clone)]
pub struct NewToolInvocation {
    pub company_id: String,
    pub idempotency_key: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub agent_id: Option<String>,
    pub issue_id: Option<String>,
    pub run_id: Option<String>,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_subject_type: Option<String>,
    pub client_subject_id: Option<String>,
    pub client_name: Option<String>,
    pub mcp_session_id: Option<String>,
    pub correlation_id: Option<String>,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub catalog_version_hash: Option<String>,
    pub catalog_schema_hash: Option<String>,
    pub provider_type: Option<String>,
    pub application_key: Option<String>,
    pub upstream_tool_name: Option<String>,
    pub risk_level: Option<String>,
    pub tool_name: String,
    pub arguments_hash: Option<String>,
    pub arguments_summary: Option<serde_json::Value>,
    pub policy_decision: Option<String>,
    pub matched_policy_ids: serde_json::Value,
    pub policy_explanation: Option<serde_json::Value>,
    pub credential_scope_summary: Option<serde_json::Value>,
    pub header_policy_summary: Option<serde_json::Value>,
    pub approval_state: String,
    pub status: String,
    pub upstream_request_id: Option<String>,
    pub result_hash: Option<String>,
    pub result_summary: Option<serde_json::Value>,
    pub result_size_bytes: Option<i64>,
    pub result_artifact_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// A tool action request (upstream `tool_action_requests`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolActionRequestRecord {
    pub id: String,
    pub company_id: String,
    pub invocation_id: String,
    pub issue_id: Option<String>,
    pub interaction_id: Option<String>,
    pub approval_id: Option<String>,
    pub status: String,
    pub canonical_arguments_hash: String,
    pub canonical_arguments_summary: serde_json::Value,
    pub signed_arguments: Option<String>,
    pub preview_markdown: Option<String>,
    pub requested_by_agent_id: Option<String>,
    pub requested_by_user_id: Option<String>,
    pub resolved_by_agent_id: Option<String>,
    pub resolved_by_user_id: Option<String>,
    pub decided_by_agent_id: Option<String>,
    pub decided_by_user_id: Option<String>,
    pub decided_at: Option<String>,
    pub expires_at: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a tool action request.
#[derive(Debug, Clone)]
pub struct NewToolActionRequest {
    pub company_id: String,
    pub invocation_id: String,
    pub issue_id: Option<String>,
    pub interaction_id: Option<String>,
    pub approval_id: Option<String>,
    pub status: String,
    pub canonical_arguments_hash: String,
    pub canonical_arguments_summary: serde_json::Value,
    pub signed_arguments: Option<String>,
    pub preview_markdown: Option<String>,
    pub requested_by_agent_id: Option<String>,
    pub requested_by_user_id: Option<String>,
    pub expires_at: Option<String>,
}

/// A tool call event (upstream `tool_call_events`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallEventRecord {
    pub id: String,
    pub company_id: String,
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub issue_id: Option<String>,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_subject_type: Option<String>,
    pub client_subject_id: Option<String>,
    pub client_name: Option<String>,
    pub mcp_session_id: Option<String>,
    pub correlation_id: Option<String>,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub invocation_id: Option<String>,
    pub action_request_id: Option<String>,
    pub runtime_slot_id: Option<String>,
    pub tool_name: Option<String>,
    pub decision: Option<String>,
    pub matched_policy_ids: serde_json::Value,
    pub reason_code: Option<String>,
    pub policy_explanation: Option<serde_json::Value>,
    pub credential_scope_summary: Option<serde_json::Value>,
    pub header_policy_summary: Option<serde_json::Value>,
    pub outcome: String,
    pub latency_ms: Option<i64>,
    pub arguments_summary: Option<serde_json::Value>,
    pub request_hash: Option<String>,
    pub request_summary: Option<serde_json::Value>,
    pub result_hash: Option<String>,
    pub result_summary: Option<serde_json::Value>,
    pub result_size_bytes: Option<i64>,
    pub redaction_plan: Option<serde_json::Value>,
    pub rate_limit_state: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

/// Input for creating a tool call event.
#[derive(Debug, Clone)]
pub struct NewToolCallEvent {
    pub company_id: String,
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub issue_id: Option<String>,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_subject_type: Option<String>,
    pub client_subject_id: Option<String>,
    pub client_name: Option<String>,
    pub mcp_session_id: Option<String>,
    pub correlation_id: Option<String>,
    pub application_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub invocation_id: Option<String>,
    pub action_request_id: Option<String>,
    pub runtime_slot_id: Option<String>,
    pub tool_name: Option<String>,
    pub decision: Option<String>,
    pub matched_policy_ids: serde_json::Value,
    pub reason_code: Option<String>,
    pub policy_explanation: Option<serde_json::Value>,
    pub credential_scope_summary: Option<serde_json::Value>,
    pub header_policy_summary: Option<serde_json::Value>,
    pub outcome: String,
    pub latency_ms: Option<i64>,
    pub arguments_summary: Option<serde_json::Value>,
    pub request_hash: Option<String>,
    pub request_summary: Option<serde_json::Value>,
    pub result_hash: Option<String>,
    pub result_summary: Option<serde_json::Value>,
    pub result_size_bytes: Option<i64>,
    pub redaction_plan: Option<serde_json::Value>,
    pub rate_limit_state: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

/// A tool access audit event (upstream `tool_access_audit_events`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAccessAuditEventRecord {
    pub id: String,
    pub company_id: String,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_name: Option<String>,
    pub correlation_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub outcome: String,
    pub reason_code: Option<String>,
    pub details: serde_json::Value,
    pub created_at: String,
}

/// Input for creating a tool access audit event.
#[derive(Debug, Clone)]
pub struct NewToolAccessAuditEvent {
    pub company_id: String,
    pub gateway_id: Option<String>,
    pub gateway_token_id: Option<String>,
    pub gateway_public_id: Option<String>,
    pub client_name: Option<String>,
    pub correlation_id: Option<String>,
    pub connection_id: Option<String>,
    pub catalog_entry_id: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub outcome: String,
    pub reason_code: Option<String>,
    pub details: serde_json::Value,
}

/// A connection token issuance (upstream `connection_token_issuances`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTokenIssuanceRecord {
    pub id: String,
    pub company_id: String,
    pub application_id: Option<String>,
    pub connection_id: String,
    pub agent_id: String,
    pub run_id: Option<String>,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub responsible_user_id: Option<String>,
    pub path: String,
    pub requested_scope: serde_json::Value,
    pub issued_scope: serde_json::Value,
    pub ttl_seconds: Option<i64>,
    pub expires_at: Option<String>,
    pub token_hash: Option<String>,
    pub outcome: String,
    pub error_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

/// Input for creating a connection token issuance.
#[derive(Debug, Clone)]
pub struct NewConnectionTokenIssuance {
    pub company_id: String,
    pub application_id: Option<String>,
    pub connection_id: String,
    pub agent_id: String,
    pub run_id: Option<String>,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub responsible_user_id: Option<String>,
    pub path: String,
    pub requested_scope: serde_json::Value,
    pub issued_scope: serde_json::Value,
    pub ttl_seconds: Option<i64>,
    pub expires_at: Option<String>,
    pub token_hash: Option<String>,
    pub outcome: String,
    pub error_code: Option<String>,
    pub metadata: serde_json::Value,
}

/// A tool rate-limit counter (upstream `tool_rate_limit_counters`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRateLimitCounterRecord {
    pub id: String,
    pub company_id: String,
    pub policy_id: String,
    pub counter_key: String,
    pub scope_type: String,
    pub scope_id: String,
    pub window_kind: String,
    pub window_start_at: String,
    pub limit: i64,
    pub remaining: i64,
    pub reset_at: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting a tool rate-limit counter.
#[derive(Debug, Clone)]
pub struct NewToolRateLimitCounter {
    pub company_id: String,
    pub policy_id: String,
    pub counter_key: String,
    pub scope_type: String,
    pub scope_id: String,
    pub window_kind: String,
    pub window_start_at: String,
    pub limit: i64,
    pub remaining: i64,
    pub reset_at: String,
}

/// A tool runtime metric counter (upstream `tool_runtime_metric_counters`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRuntimeMetricCounterRecord {
    pub id: String,
    pub company_id: String,
    pub metric: String,
    pub bucket_start_at: String,
    pub count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting a tool runtime metric counter.
#[derive(Debug, Clone)]
pub struct NewToolRuntimeMetricCounter {
    pub company_id: String,
    pub metric: String,
    pub bucket_start_at: String,
    pub count: i64,
}

/// Tool catalog persistence contract: applications, catalog entries, and
/// profiles (profile entries/bindings).
#[async_trait]
pub trait ToolCatalogRepository: Send + Sync {
    /// Creates a tool application.
    async fn create_application(
        &self,
        input: NewToolApplication,
    ) -> Result<ToolApplicationRecord, ToolchainError>;

    /// Lists tool applications for a company.
    async fn list_applications(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolApplicationRecord>, ToolchainError>;

    /// Fetches one tool application (company-scoped).
    async fn get_application(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolApplicationRecord>, ToolchainError>;

    /// Creates a tool catalog entry.
    async fn create_catalog_entry(
        &self,
        input: NewToolCatalogEntry,
    ) -> Result<ToolCatalogEntryRecord, ToolchainError>;

    /// Lists catalog entries for a company, optionally filtered by connection.
    async fn list_catalog_entries(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ToolCatalogEntryRecord>, ToolchainError>;

    /// Creates a tool profile.
    async fn create_profile(
        &self,
        input: NewToolProfile,
    ) -> Result<ToolProfileRecord, ToolchainError>;

    /// Fetches one tool profile (company-scoped).
    async fn get_profile(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolProfileRecord>, ToolchainError>;

    /// Lists tool profiles for a company.
    async fn list_profiles(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolProfileRecord>, ToolchainError>;

    /// Updates a tool profile.
    async fn update_profile(
        &self,
        input: UpdateToolProfile,
    ) -> Result<Option<ToolProfileRecord>, ToolchainError>;

    /// Deletes a tool profile (company-scoped).
    async fn delete_profile(&self, company_id: &str, id: &str) -> Result<bool, ToolchainError>;

    /// Creates a tool profile entry.
    async fn create_profile_entry(
        &self,
        input: NewToolProfileEntry,
    ) -> Result<ToolProfileEntryRecord, ToolchainError>;

    /// Lists profile entries for a company, optionally filtered by profile.
    async fn list_profile_entries(
        &self,
        company_id: &str,
        profile_id: Option<&str>,
    ) -> Result<Vec<ToolProfileEntryRecord>, ToolchainError>;

    /// Creates a tool profile binding.
    async fn create_profile_binding(
        &self,
        input: NewToolProfileBinding,
    ) -> Result<ToolProfileBindingRecord, ToolchainError>;

    /// Lists profile bindings for a company, optionally filtered by profile.
    async fn list_profile_bindings(
        &self,
        company_id: &str,
        profile_id: Option<&str>,
    ) -> Result<Vec<ToolProfileBindingRecord>, ToolchainError>;
}

/// Tool connection persistence contract: connections, grants, installs,
/// OAuth states, and token issuances.
#[async_trait]
pub trait ToolConnectionRepository: Send + Sync {
    /// Creates a tool connection.
    async fn create_connection(
        &self,
        input: NewToolConnection,
    ) -> Result<ToolConnectionRecord, ToolchainError>;

    /// Fetches one tool connection (company-scoped).
    async fn get_connection(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolConnectionRecord>, ToolchainError>;

    /// Lists tool connections for a company.
    async fn list_connections(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolConnectionRecord>, ToolchainError>;

    /// Updates a tool connection.
    async fn update_connection(
        &self,
        input: UpdateToolConnection,
    ) -> Result<Option<ToolConnectionRecord>, ToolchainError>;

    /// Creates a connection grant.
    async fn create_grant(
        &self,
        input: NewConnectionGrant,
    ) -> Result<ConnectionGrantRecord, ToolchainError>;

    /// Lists connection grants for a company, optionally filtered by connection.
    async fn list_grants(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ConnectionGrantRecord>, ToolchainError>;

    /// Revokes a connection grant.
    async fn revoke_grant(
        &self,
        company_id: &str,
        id: &str,
        revoked_by_user_id: Option<&str>,
    ) -> Result<Option<ConnectionGrantRecord>, ToolchainError>;

    /// Creates a tool connection install.
    async fn create_install(
        &self,
        input: NewToolConnectionInstall,
    ) -> Result<ToolConnectionInstallRecord, ToolchainError>;

    /// Lists connection installs for a company, optionally filtered by connection.
    async fn list_installs(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ToolConnectionInstallRecord>, ToolchainError>;

    /// Creates a tool OAuth state.
    async fn create_oauth_state(
        &self,
        input: NewToolOauthState,
    ) -> Result<ToolOauthStateRecord, ToolchainError>;

    /// Creates a connection token issuance.
    async fn create_token_issuance(
        &self,
        input: NewConnectionTokenIssuance,
    ) -> Result<ConnectionTokenIssuanceRecord, ToolchainError>;

    /// Lists connection token issuances for a company, optionally filtered by
    /// connection.
    async fn list_token_issuances(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ConnectionTokenIssuanceRecord>, ToolchainError>;
}

/// Tool gateway persistence contract: MCP gateways, tokens, policies, runtime
/// slots, stdio templates, gateway sessions, invocations, action requests,
/// call events, audit events, and counters.
#[async_trait]
pub trait ToolGatewayRepository: Send + Sync {
    /// Creates a tool MCP gateway.
    async fn create_gateway(
        &self,
        input: NewToolMcpGateway,
    ) -> Result<ToolMcpGatewayRecord, ToolchainError>;

    /// Fetches one gateway (company-scoped).
    async fn get_gateway(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolMcpGatewayRecord>, ToolchainError>;

    /// Lists gateways for a company.
    async fn list_gateways(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolMcpGatewayRecord>, ToolchainError>;

    /// Creates a gateway token.
    async fn create_gateway_token(
        &self,
        input: NewToolMcpGatewayToken,
    ) -> Result<ToolMcpGatewayTokenRecord, ToolchainError>;

    /// Lists gateway tokens for a company, optionally filtered by gateway.
    async fn list_gateway_tokens(
        &self,
        company_id: &str,
        gateway_id: Option<&str>,
    ) -> Result<Vec<ToolMcpGatewayTokenRecord>, ToolchainError>;

    /// Creates a tool policy.
    async fn create_policy(&self, input: NewToolPolicy)
    -> Result<ToolPolicyRecord, ToolchainError>;

    /// Lists tool policies for a company.
    async fn list_policies(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolPolicyRecord>, ToolchainError>;

    /// Creates a tool runtime slot.
    async fn create_runtime_slot(
        &self,
        input: NewToolRuntimeSlot,
    ) -> Result<ToolRuntimeSlotRecord, ToolchainError>;

    /// Lists runtime slots for a company.
    async fn list_runtime_slots(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolRuntimeSlotRecord>, ToolchainError>;

    /// Creates a stdio command template.
    async fn create_stdio_template(
        &self,
        input: NewToolStdioCommandTemplate,
    ) -> Result<ToolStdioCommandTemplateRecord, ToolchainError>;

    /// Lists stdio command templates for a company.
    async fn list_stdio_templates(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolStdioCommandTemplateRecord>, ToolchainError>;

    /// Creates a gateway session.
    async fn create_gateway_session(
        &self,
        input: NewToolGatewaySession,
    ) -> Result<ToolGatewaySessionRecord, ToolchainError>;

    /// Lists gateway sessions for a company, optionally filtered by gateway.
    async fn list_gateway_sessions(
        &self,
        company_id: &str,
        gateway_id: Option<&str>,
    ) -> Result<Vec<ToolGatewaySessionRecord>, ToolchainError>;

    /// Creates a tool invocation.
    async fn create_invocation(
        &self,
        input: NewToolInvocation,
    ) -> Result<ToolInvocationRecord, ToolchainError>;

    /// Fetches one tool invocation (company-scoped).
    async fn get_invocation(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolInvocationRecord>, ToolchainError>;

    /// Lists tool invocations for a company.
    async fn list_invocations(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolInvocationRecord>, ToolchainError>;

    /// Creates a tool action request.
    async fn create_action_request(
        &self,
        input: NewToolActionRequest,
    ) -> Result<ToolActionRequestRecord, ToolchainError>;

    /// Lists action requests for a company, optionally filtered by invocation.
    async fn list_action_requests(
        &self,
        company_id: &str,
        invocation_id: Option<&str>,
    ) -> Result<Vec<ToolActionRequestRecord>, ToolchainError>;

    /// Creates a tool call event.
    async fn create_call_event(
        &self,
        input: NewToolCallEvent,
    ) -> Result<ToolCallEventRecord, ToolchainError>;

    /// Lists call events for a company, optionally filtered by invocation.
    async fn list_call_events(
        &self,
        company_id: &str,
        invocation_id: Option<&str>,
    ) -> Result<Vec<ToolCallEventRecord>, ToolchainError>;

    /// Creates a tool access audit event.
    async fn create_audit_event(
        &self,
        input: NewToolAccessAuditEvent,
    ) -> Result<ToolAccessAuditEventRecord, ToolchainError>;

    /// Lists access audit events for a company.
    async fn list_audit_events(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolAccessAuditEventRecord>, ToolchainError>;

    /// Upserts a gateway rate-limit counter.
    async fn upsert_gateway_rate_limit_counter(
        &self,
        input: NewToolGatewayRateLimitCounter,
    ) -> Result<ToolGatewayRateLimitCounterRecord, ToolchainError>;

    /// Upserts a tool rate-limit counter.
    async fn upsert_rate_limit_counter(
        &self,
        input: NewToolRateLimitCounter,
    ) -> Result<ToolRateLimitCounterRecord, ToolchainError>;

    /// Upserts a tool runtime metric counter.
    async fn upsert_runtime_metric_counter(
        &self,
        input: NewToolRuntimeMetricCounter,
    ) -> Result<ToolRuntimeMetricCounterRecord, ToolchainError>;
}

/// Turso/libSQL implementation of [`ToolCatalogRepository`].
#[derive(Debug)]
pub struct TursoToolCatalogRepository {
    db: Database,
}

impl TursoToolCatalogRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const APPLICATION_COLUMNS: &str = "id, company_id, application_key, name, description, type,
                                   status, plugin_id, owner_agent_id, owner_user_id, metadata,
                                   archived_at, created_at, updated_at";
const CONNECTION_COLUMNS: &str = "id, company_id, application_id, name, uid, connection_kind,
                                  ownership, transport, auth_kind, status, enabled, config,
                                  transport_config, credential_refs, credential_secret_refs,
                                  health_status, health_message, health_checked_at,
                                  last_health_at, last_catalog_refresh_at, last_error,
                                  created_by_agent_id, created_by_user_id, created_at,
                                  updated_at";
const GRANT_COLUMNS: &str = "id, company_id, connection_id, kind, subject_user_id,
                             provider_tenant, credential_secret_refs, status, is_default,
                             created_by_agent_id, created_by_user_id, revoked_at,
                             revoked_by_agent_id, revoked_by_user_id, last_used_at, created_at,
                             updated_at";
const INSTALL_COLUMNS: &str = "id, company_id, connection_id, target_type, target_id,
                               created_by_agent_id, created_by_user_id, created_at";
const OAUTH_STATE_COLUMNS: &str = "state, company_id, connection_id, code_verifier,
                                   created_by_actor_type, created_by_actor_id,
                                   created_by_session_id, subject_user_id, requested_scopes,
                                   return_to, issue_id, interaction_id, expires_at, created_at";
const CATALOG_ENTRY_COLUMNS: &str = "id, company_id, application_id, connection_id, entry_kind,
                                     name, tool_name, title, description, input_schema,
                                     output_schema, annotations, risk_level, is_read_only,
                                     is_write, is_destructive, status, version, version_hash,
                                     schema_hash, first_seen_at, last_seen_at, reviewed_at,
                                     reviewed_by_agent_id, reviewed_by_user_id, quarantined_at,
                                     quarantine_reason, created_at, updated_at";
const PROFILE_COLUMNS: &str = "id, company_id, profile_key, name, description, status,
                               default_action, new_tools_reviewed_at, metadata, created_at,
                               updated_at";
const PROFILE_ENTRY_COLUMNS: &str = "id, company_id, profile_id, selector_type, effect,
                                     application_id, connection_id, catalog_entry_id, tool_name,
                                     risk_level, conditions, created_at, updated_at";
const PROFILE_BINDING_COLUMNS: &str = "id, company_id, profile_id, target_type, target_id,
                                       priority, metadata, created_by_agent_id,
                                       created_by_user_id, created_at, updated_at";
const GATEWAY_COLUMNS: &str = "id, company_id, gateway_public_id, name, slug, display_slug,
                               description, status, profile_id, default_profile_mode,
                               context_scope_type, context_scope_id, agent_id, project_id,
                               issue_id, approval_issue_id, auth_config, header_policy,
                               metadata_policy, on_demand_tools_config, metadata,
                               created_by_agent_id, created_by_user_id, archived_at, created_at,
                               updated_at";
const GATEWAY_TOKEN_COLUMNS: &str = "id, company_id, gateway_id, name, token_hash, token_prefix,
                                     subject_type, subject_id, client_label, owner_note,
                                     allowed_actions, expires_at, expiry_override_reason,
                                     expiry_override_by_user_id, expiry_override_by_agent_id,
                                     expiry_override_at, last_used_at, revoked_at,
                                     created_by_agent_id, created_by_user_id, created_at,
                                     updated_at";
const POLICY_COLUMNS: &str = "id, company_id, name, description, policy_type, priority, enabled,
                              selectors, conditions, config, created_by_agent_id,
                              created_by_user_id, created_at, updated_at";
const RUNTIME_SLOT_COLUMNS: &str = "id, company_id, application_id, connection_id,
                                    project_workspace_id, execution_workspace_id, issue_id,
                                    owner_scope_type, owner_scope_id, runtime_kind, slot_key,
                                    status, reuse_key, workspace_scope, credential_scope_hash,
                                    provider, provider_ref, process_id, command_template_key,
                                    health_status, health_message, last_health_check_at,
                                    last_started_at, started_at, stopped_at, last_used_at,
                                    idle_expires_at, idle_deadline_at, last_error, metadata,
                                    created_at, updated_at";
const STDIO_TEMPLATE_COLUMNS: &str = "id, company_id, template_key, name, description, status,
                                      command, args, env_keys, tools, created_by_agent_id,
                                      created_by_user_id, disabled_at, created_at, updated_at";
const GATEWAY_SESSION_COLUMNS: &str = "id, company_id, agent_id, run_id, issue_id, project_id,
                                       gateway_id, gateway_token_id, gateway_public_id,
                                       client_subject_type, client_subject_id, client_name,
                                       mcp_session_id, correlation_id, token_hash, expires_at,
                                       last_used_at, revoked_at, created_at, updated_at";
const GATEWAY_RATE_LIMIT_COLUMNS: &str = "id, company_id, counter_key, window_start_at,
                                          window_ms, \"limit\", count, reset_at, created_at,
                                          updated_at";
const INVOCATION_COLUMNS: &str = "id, company_id, idempotency_key, actor_type, actor_id,
                                  agent_id, issue_id, run_id, gateway_id, gateway_token_id,
                                  gateway_public_id, client_subject_type, client_subject_id,
                                  client_name, mcp_session_id, correlation_id, application_id,
                                  connection_id, catalog_entry_id, catalog_version_hash,
                                  catalog_schema_hash, provider_type, application_key,
                                  upstream_tool_name, risk_level, tool_name, arguments_hash,
                                  arguments_summary, policy_decision, matched_policy_ids,
                                  policy_explanation, credential_scope_summary,
                                  header_policy_summary, approval_state, status,
                                  upstream_request_id, result_hash, result_summary,
                                  result_size_bytes, result_artifact_id, error_code,
                                  error_message, started_at, completed_at, created_at,
                                  updated_at";
const ACTION_REQUEST_COLUMNS: &str = "id, company_id, invocation_id, issue_id, interaction_id,
                                      approval_id, status, canonical_arguments_hash,
                                      canonical_arguments_summary, signed_arguments,
                                      preview_markdown, requested_by_agent_id,
                                      requested_by_user_id, resolved_by_agent_id,
                                      resolved_by_user_id, decided_by_agent_id,
                                      decided_by_user_id, decided_at, expires_at, resolved_at,
                                      created_at, updated_at";
const CALL_EVENT_COLUMNS: &str = "id, company_id, event_type, actor_type, actor_id, agent_id,
                                  run_id, issue_id, gateway_id, gateway_token_id,
                                  gateway_public_id, client_subject_type, client_subject_id,
                                  client_name, mcp_session_id, correlation_id, application_id,
                                  connection_id, catalog_entry_id, invocation_id,
                                  action_request_id, runtime_slot_id, tool_name, decision,
                                  matched_policy_ids, reason_code, policy_explanation,
                                  credential_scope_summary, header_policy_summary, outcome,
                                  latency_ms, arguments_summary, request_hash, request_summary,
                                  result_hash, result_summary, result_size_bytes, redaction_plan,
                                  rate_limit_state, metadata, error_code, error_message,
                                  created_at";
const AUDIT_EVENT_COLUMNS: &str = "id, company_id, gateway_id, gateway_token_id,
                                   gateway_public_id, client_name, correlation_id,
                                   connection_id, catalog_entry_id, actor_type, actor_id,
                                   action, outcome, reason_code, details, created_at";
const TOKEN_ISSUANCE_COLUMNS: &str = "id, company_id, application_id, connection_id, agent_id,
                                      run_id, issue_id, project_id, responsible_user_id, path,
                                      requested_scope, issued_scope, ttl_seconds, expires_at,
                                      token_hash, outcome, error_code, metadata, created_at";
const RATE_LIMIT_COLUMNS: &str = "id, company_id, policy_id, counter_key, scope_type, scope_id,
                                  window_kind, window_start_at, \"limit\", remaining, reset_at,
                                  created_at, updated_at";
const METRIC_COUNTER_COLUMNS: &str = "id, company_id, metric, bucket_start_at, count,
                                      created_at, updated_at";

fn json_string(value: &serde_json::Value) -> String {
    value.to_string()
}

fn bool_int(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

#[async_trait]
impl ToolCatalogRepository for TursoToolCatalogRepository {
    async fn create_application(
        &self,
        input: NewToolApplication,
    ) -> Result<ToolApplicationRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        if let Some(plugin_id) = &input.plugin_id
            && !helpers::find_row(&conn, "plugins", plugin_id).await?
        {
            return Err(ToolchainError::ReferenceNotFound);
        }
        if let Some(agent_id) = &input.owner_agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", agent_id, &input.company_id)
                .await?
        {
            return Err(ToolchainError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_applications (id, company_id, application_key, name,
                                                description, type, status, plugin_id,
                                                owner_agent_id, owner_user_id, metadata,
                                                created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.application_key,
                    input.name,
                    input.description,
                    input.r#type,
                    input.status,
                    input.plugin_id,
                    input.owner_agent_id,
                    input.owner_user_id,
                    json_string(&input.metadata)
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_application(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_applications(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolApplicationRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {APPLICATION_COLUMNS} FROM tool_applications
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_application(&row)?);
        }
        Ok(records)
    }

    async fn get_application(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolApplicationRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(select_application_scoped(&conn, company_id, id).await?)
    }

    async fn create_catalog_entry(
        &self,
        input: NewToolCatalogEntry,
    ) -> Result<ToolCatalogEntryRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_catalog_entries (id, company_id, application_id,
                                                   connection_id, entry_kind, name, tool_name,
                                                   title, description, input_schema,
                                                   output_schema, annotations, risk_level,
                                                   is_read_only, is_write, is_destructive,
                                                   status, version, version_hash, schema_hash,
                                                   first_seen_at, last_seen_at,
                                                   reviewed_by_agent_id, reviewed_by_user_id,
                                                   created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?21, ?22,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.application_id,
                    input.connection_id,
                    input.entry_kind,
                    input.name,
                    input.tool_name,
                    input.title,
                    input.description,
                    json_string(&input.input_schema),
                    input.output_schema.as_ref().map(json_string),
                    json_string(&input.annotations),
                    input.risk_level,
                    bool_int(input.is_read_only),
                    bool_int(input.is_write),
                    bool_int(input.is_destructive),
                    input.status,
                    input.version,
                    input.version_hash,
                    input.schema_hash,
                    input.reviewed_by_agent_id,
                    input.reviewed_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_catalog_entry(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_catalog_entries(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ToolCatalogEntryRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = connection_id
            .map(|_| "AND connection_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = connection_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CATALOG_ENTRY_COLUMNS} FROM tool_catalog_entries
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_catalog_entry(&row)?);
        }
        Ok(records)
    }

    async fn create_profile(
        &self,
        input: NewToolProfile,
    ) -> Result<ToolProfileRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_profiles (id, company_id, profile_key, name, description,
                                            status, default_action, new_tools_reviewed_at,
                                            metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.profile_key,
                    input.name,
                    input.description,
                    input.status,
                    input.default_action,
                    input.new_tools_reviewed_at,
                    json_string(&input.metadata)
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_profile(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn get_profile(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolProfileRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(select_profile_scoped(&conn, company_id, id).await?)
    }

    async fn list_profiles(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolProfileRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PROFILE_COLUMNS} FROM tool_profiles
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_profile(&row)?);
        }
        Ok(records)
    }

    async fn update_profile(
        &self,
        input: UpdateToolProfile,
    ) -> Result<Option<ToolProfileRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut push = |column: &str, value: Option<libsql::Value>| {
            if let Some(value) = value {
                sets.push(format!("{column} = ?{}", values.len() + 1));
                values.push(value);
            }
        };
        push("name", input.name.map(libsql::Value::from));
        push("description", input.description.map(libsql::Value::from));
        push("status", input.status.map(libsql::Value::from));
        push(
            "default_action",
            input.default_action.map(libsql::Value::from),
        );
        push(
            "new_tools_reviewed_at",
            input.new_tools_reviewed_at.map(libsql::Value::from),
        );
        push(
            "metadata",
            input.metadata.map(|v| libsql::Value::from(json_string(&v))),
        );
        if sets.is_empty() {
            return select_profile_scoped(&conn, &input.company_id, &input.id).await;
        }
        values.push(input.id.clone().into());
        let sql = format!(
            "UPDATE tool_profiles SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?{} AND company_id = ?{}",
            sets.join(", "),
            values.len(),
            values.len() + 1
        );
        values.push(input.company_id.clone().into());
        conn.execute(&sql, values).await?;
        Ok(select_profile_scoped(&conn, &input.company_id, &input.id).await?)
    }

    async fn delete_profile(&self, company_id: &str, id: &str) -> Result<bool, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let deleted = conn
            .execute(
                "DELETE FROM tool_profiles WHERE id = ?1 AND company_id = ?2",
                libsql::params![id, company_id],
            )
            .await?;
        Ok(deleted > 0)
    }

    async fn create_profile_entry(
        &self,
        input: NewToolProfileEntry,
    ) -> Result<ToolProfileEntryRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_profile_entries (id, company_id, profile_id, selector_type,
                                                   effect, application_id, connection_id,
                                                   catalog_entry_id, tool_name, risk_level,
                                                   conditions, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.profile_id,
                    input.selector_type,
                    input.effect,
                    input.application_id,
                    input.connection_id,
                    input.catalog_entry_id,
                    input.tool_name,
                    input.risk_level,
                    input.conditions.as_ref().map(json_string)
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_profile_entry(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_profile_entries(
        &self,
        company_id: &str,
        profile_id: Option<&str>,
    ) -> Result<Vec<ToolProfileEntryRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = profile_id
            .map(|_| "AND profile_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = profile_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PROFILE_ENTRY_COLUMNS} FROM tool_profile_entries
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_profile_entry(&row)?);
        }
        Ok(records)
    }

    async fn create_profile_binding(
        &self,
        input: NewToolProfileBinding,
    ) -> Result<ToolProfileBindingRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_profile_bindings (id, company_id, profile_id, target_type,
                                                    target_id, priority, metadata,
                                                    created_by_agent_id, created_by_user_id,
                                                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.profile_id,
                    input.target_type,
                    input.target_id,
                    input.priority,
                    json_string(&input.metadata),
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_profile_binding(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_profile_bindings(
        &self,
        company_id: &str,
        profile_id: Option<&str>,
    ) -> Result<Vec<ToolProfileBindingRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = profile_id
            .map(|_| "AND profile_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = profile_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PROFILE_BINDING_COLUMNS} FROM tool_profile_bindings
                     WHERE company_id = ?1 {filter} ORDER BY priority, created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_profile_binding(&row)?);
        }
        Ok(records)
    }
}

/// Turso/libSQL implementation of [`ToolConnectionRepository`].
#[derive(Debug)]
pub struct TursoToolConnectionRepository {
    db: Database,
}

impl TursoToolConnectionRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ToolConnectionRepository for TursoToolConnectionRepository {
    async fn create_connection(
        &self,
        input: NewToolConnection,
    ) -> Result<ToolConnectionRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        if let Some(agent_id) = &input.created_by_agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", agent_id, &input.company_id)
                .await?
        {
            return Err(ToolchainError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_connections (id, company_id, application_id, name, uid,
                                               connection_kind, ownership, transport, auth_kind,
                                               status, enabled, config, transport_config,
                                               credential_refs, credential_secret_refs,
                                               health_status, created_by_agent_id,
                                               created_by_user_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         'unchecked', ?16, ?17,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.application_id,
                    input.name,
                    input.uid,
                    input.connection_kind,
                    input.ownership,
                    input.transport,
                    input.auth_kind,
                    input.status,
                    bool_int(input.enabled),
                    json_string(&input.config),
                    json_string(&input.transport_config),
                    json_string(&input.credential_refs),
                    json_string(&input.credential_secret_refs),
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_connection(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn get_connection(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolConnectionRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(select_connection_scoped(&conn, company_id, id).await?)
    }

    async fn list_connections(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolConnectionRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CONNECTION_COLUMNS} FROM tool_connections
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_connection(&row)?);
        }
        Ok(records)
    }

    async fn update_connection(
        &self,
        input: UpdateToolConnection,
    ) -> Result<Option<ToolConnectionRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut push = |column: &str, value: Option<libsql::Value>| {
            if let Some(value) = value {
                sets.push(format!("{column} = ?{}", values.len() + 1));
                values.push(value);
            }
        };
        push("name", input.name.map(libsql::Value::from));
        push("status", input.status.map(libsql::Value::from));
        push(
            "enabled",
            input.enabled.map(|v| libsql::Value::from(bool_int(v))),
        );
        push(
            "config",
            input.config.map(|v| libsql::Value::from(json_string(&v))),
        );
        push(
            "transport_config",
            input
                .transport_config
                .map(|v| libsql::Value::from(json_string(&v))),
        );
        push(
            "health_status",
            input.health_status.map(libsql::Value::from),
        );
        push(
            "health_message",
            input.health_message.map(libsql::Value::from),
        );
        push(
            "health_checked_at",
            input.health_checked_at.map(libsql::Value::from),
        );
        push("last_error", input.last_error.map(libsql::Value::from));
        if sets.is_empty() {
            return select_connection_scoped(&conn, &input.company_id, &input.id).await;
        }
        values.push(input.id.clone().into());
        let sql = format!(
            "UPDATE tool_connections SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?{} AND company_id = ?{}",
            sets.join(", "),
            values.len(),
            values.len() + 1
        );
        values.push(input.company_id.clone().into());
        let updated = conn.execute(&sql, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        Ok(select_connection_scoped(&conn, &input.company_id, &input.id).await?)
    }

    async fn create_grant(
        &self,
        input: NewConnectionGrant,
    ) -> Result<ConnectionGrantRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO connection_grants (id, company_id, connection_id, kind,
                                                subject_user_id, provider_tenant,
                                                credential_secret_refs, status, is_default,
                                                created_by_agent_id, created_by_user_id,
                                                created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.connection_id,
                    input.kind,
                    input.subject_user_id,
                    input.provider_tenant.as_ref().map(json_string),
                    json_string(&input.credential_secret_refs),
                    input.status,
                    bool_int(input.is_default),
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_grant(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_grants(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ConnectionGrantRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = connection_id
            .map(|_| "AND connection_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = connection_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {GRANT_COLUMNS} FROM connection_grants
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_grant(&row)?);
        }
        Ok(records)
    }

    async fn revoke_grant(
        &self,
        company_id: &str,
        id: &str,
        revoked_by_user_id: Option<&str>,
    ) -> Result<Option<ConnectionGrantRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE connection_grants
                 SET status = 'revoked', revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     revoked_by_user_id = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?2 AND company_id = ?3 AND status = 'active'",
                libsql::params![revoked_by_user_id, id, company_id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {GRANT_COLUMNS} FROM connection_grants WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("grant exists");
        Ok(Some(row_to_grant(&row)?))
    }

    async fn create_install(
        &self,
        input: NewToolConnectionInstall,
    ) -> Result<ToolConnectionInstallRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_connection_installs (id, company_id, connection_id,
                                                       target_type, target_id,
                                                       created_by_agent_id, created_by_user_id,
                                                       created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.connection_id,
                    input.target_type,
                    input.target_id,
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_install(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_installs(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ToolConnectionInstallRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = connection_id
            .map(|_| "AND connection_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = connection_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {INSTALL_COLUMNS} FROM tool_connection_installs
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_install(&row)?);
        }
        Ok(records)
    }

    async fn create_oauth_state(
        &self,
        input: NewToolOauthState,
    ) -> Result<ToolOauthStateRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let result = conn
            .execute(
                "INSERT INTO tool_oauth_states (state, company_id, connection_id, code_verifier,
                                                created_by_actor_type, created_by_actor_id,
                                                created_by_session_id, subject_user_id,
                                                requested_scopes, return_to, issue_id,
                                                interaction_id, expires_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    input.state.clone(),
                    input.company_id,
                    input.connection_id,
                    input.code_verifier,
                    input.created_by_actor_type,
                    input.created_by_actor_id,
                    input.created_by_session_id,
                    input.subject_user_id,
                    input.requested_scopes.as_ref().map(json_string),
                    input.return_to,
                    input.issue_id,
                    input.interaction_id,
                    input.expires_at
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_oauth_state(&conn, &input.state)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn create_token_issuance(
        &self,
        input: NewConnectionTokenIssuance,
    ) -> Result<ConnectionTokenIssuanceRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO connection_token_issuances (id, company_id, application_id,
                                                         connection_id, agent_id, run_id,
                                                         issue_id, project_id,
                                                         responsible_user_id, path,
                                                         requested_scope, issued_scope,
                                                         ttl_seconds, expires_at, token_hash,
                                                         outcome, error_code, metadata,
                                                         created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.application_id,
                    input.connection_id,
                    input.agent_id,
                    input.run_id,
                    input.issue_id,
                    input.project_id,
                    input.responsible_user_id,
                    input.path,
                    json_string(&input.requested_scope),
                    json_string(&input.issued_scope),
                    input.ttl_seconds,
                    input.expires_at,
                    input.token_hash,
                    input.outcome,
                    input.error_code,
                    json_string(&input.metadata)
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_token_issuance(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_token_issuances(
        &self,
        company_id: &str,
        connection_id: Option<&str>,
    ) -> Result<Vec<ConnectionTokenIssuanceRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = connection_id
            .map(|_| "AND connection_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = connection_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {TOKEN_ISSUANCE_COLUMNS} FROM connection_token_issuances
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_token_issuance(&row)?);
        }
        Ok(records)
    }
}

/// Turso/libSQL implementation of [`ToolGatewayRepository`].
#[derive(Debug)]
pub struct TursoToolGatewayRepository {
    db: Database,
}

impl TursoToolGatewayRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ToolGatewayRepository for TursoToolGatewayRepository {
    async fn create_gateway(
        &self,
        input: NewToolMcpGateway,
    ) -> Result<ToolMcpGatewayRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let gateway_public_id = format!("gw_{}", Uuid::new_v4().simple());
        let result = conn
            .execute(
                "INSERT INTO tool_mcp_gateways (id, company_id, gateway_public_id, name, slug,
                                                display_slug, description, status, profile_id,
                                                default_profile_mode, context_scope_type,
                                                context_scope_id, agent_id, project_id, issue_id,
                                                approval_issue_id, auth_config, header_policy,
                                                metadata_policy, on_demand_tools_config,
                                                metadata, created_by_agent_id, created_by_user_id,
                                                created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    gateway_public_id,
                    input.name,
                    input.slug,
                    input.display_slug,
                    input.description,
                    input.status,
                    input.profile_id,
                    input.default_profile_mode,
                    input.context_scope_type,
                    input.context_scope_id,
                    input.agent_id,
                    input.project_id,
                    input.issue_id,
                    input.approval_issue_id,
                    json_string(&input.auth_config),
                    json_string(&input.header_policy),
                    json_string(&input.metadata_policy),
                    json_string(&input.on_demand_tools_config),
                    json_string(&input.metadata),
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_gateway(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn get_gateway(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolMcpGatewayRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(select_gateway_scoped(&conn, company_id, id).await?)
    }

    async fn list_gateways(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolMcpGatewayRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {GATEWAY_COLUMNS} FROM tool_mcp_gateways
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_gateway(&row)?);
        }
        Ok(records)
    }

    async fn create_gateway_token(
        &self,
        input: NewToolMcpGatewayToken,
    ) -> Result<ToolMcpGatewayTokenRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_mcp_gateway_tokens (id, company_id, gateway_id, name,
                                                      token_hash, token_prefix, subject_type,
                                                      subject_id, client_label, owner_note,
                                                      allowed_actions, expires_at,
                                                      created_by_agent_id, created_by_user_id,
                                                      created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.gateway_id,
                    input.name,
                    input.token_hash,
                    input.token_prefix,
                    input.subject_type,
                    input.subject_id,
                    input.client_label,
                    input.owner_note,
                    json_string(&input.allowed_actions),
                    input.expires_at,
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_gateway_token(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_gateway_tokens(
        &self,
        company_id: &str,
        gateway_id: Option<&str>,
    ) -> Result<Vec<ToolMcpGatewayTokenRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = gateway_id
            .map(|_| "AND gateway_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = gateway_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {GATEWAY_TOKEN_COLUMNS} FROM tool_mcp_gateway_tokens
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_gateway_token(&row)?);
        }
        Ok(records)
    }

    async fn create_policy(
        &self,
        input: NewToolPolicy,
    ) -> Result<ToolPolicyRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_policies (id, company_id, name, description, policy_type,
                                            priority, enabled, selectors, conditions, config,
                                            created_by_agent_id, created_by_user_id, created_at,
                                            updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.name,
                    input.description,
                    input.policy_type,
                    input.priority,
                    bool_int(input.enabled),
                    json_string(&input.selectors),
                    input.conditions.as_ref().map(json_string),
                    input.config.as_ref().map(json_string),
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_policy(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_policies(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolPolicyRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {POLICY_COLUMNS} FROM tool_policies
                     WHERE company_id = ?1 ORDER BY priority, created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_policy(&row)?);
        }
        Ok(records)
    }

    async fn create_runtime_slot(
        &self,
        input: NewToolRuntimeSlot,
    ) -> Result<ToolRuntimeSlotRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_runtime_slots (id, company_id, application_id, connection_id,
                                                 project_workspace_id, execution_workspace_id,
                                                 issue_id, owner_scope_type, owner_scope_id,
                                                 runtime_kind, slot_key, status, reuse_key,
                                                 workspace_scope, credential_scope_hash,
                                                 provider, provider_ref, process_id,
                                                 command_template_key, health_status,
                                                 health_message, metadata, created_at,
                                                 updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, 'unchecked', ?20, ?21,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.application_id,
                    input.connection_id,
                    input.project_workspace_id,
                    input.execution_workspace_id,
                    input.issue_id,
                    input.owner_scope_type,
                    input.owner_scope_id,
                    input.runtime_kind,
                    input.slot_key,
                    input.status,
                    input.reuse_key,
                    input.workspace_scope,
                    input.credential_scope_hash,
                    input.provider,
                    input.provider_ref,
                    input.process_id,
                    input.command_template_key,
                    input.health_message,
                    json_string(&input.metadata)
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_runtime_slot(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_runtime_slots(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolRuntimeSlotRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RUNTIME_SLOT_COLUMNS} FROM tool_runtime_slots
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_runtime_slot(&row)?);
        }
        Ok(records)
    }

    async fn create_stdio_template(
        &self,
        input: NewToolStdioCommandTemplate,
    ) -> Result<ToolStdioCommandTemplateRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_stdio_command_templates (id, company_id, template_key, name,
                                                           description, status, command, args,
                                                           env_keys, tools, created_by_agent_id,
                                                           created_by_user_id, created_at,
                                                           updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.template_key,
                    input.name,
                    input.description,
                    input.status,
                    input.command,
                    json_string(&input.args),
                    json_string(&input.env_keys),
                    json_string(&input.tools),
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_stdio_template(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_stdio_templates(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolStdioCommandTemplateRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STDIO_TEMPLATE_COLUMNS} FROM tool_stdio_command_templates
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_stdio_template(&row)?);
        }
        Ok(records)
    }

    async fn create_gateway_session(
        &self,
        input: NewToolGatewaySession,
    ) -> Result<ToolGatewaySessionRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_gateway_sessions (id, company_id, agent_id, run_id, issue_id,
                                                    project_id, gateway_id, gateway_token_id,
                                                    gateway_public_id, client_subject_type,
                                                    client_subject_id, client_name,
                                                    mcp_session_id, correlation_id, token_hash,
                                                    expires_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.agent_id,
                    input.run_id,
                    input.issue_id,
                    input.project_id,
                    input.gateway_id,
                    input.gateway_token_id,
                    input.gateway_public_id,
                    input.client_subject_type,
                    input.client_subject_id,
                    input.client_name,
                    input.mcp_session_id,
                    input.correlation_id,
                    input.token_hash,
                    input.expires_at
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_gateway_session(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_gateway_sessions(
        &self,
        company_id: &str,
        gateway_id: Option<&str>,
    ) -> Result<Vec<ToolGatewaySessionRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = gateway_id
            .map(|_| "AND gateway_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = gateway_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {GATEWAY_SESSION_COLUMNS} FROM tool_gateway_sessions
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_gateway_session(&row)?);
        }
        Ok(records)
    }

    async fn create_invocation(
        &self,
        input: NewToolInvocation,
    ) -> Result<ToolInvocationRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_invocations (id, company_id, idempotency_key, actor_type,
                                               actor_id, agent_id, issue_id, run_id, gateway_id,
                                               gateway_token_id, gateway_public_id,
                                               client_subject_type, client_subject_id,
                                               client_name, mcp_session_id, correlation_id,
                                               application_id, connection_id, catalog_entry_id,
                                               catalog_version_hash, catalog_schema_hash,
                                               provider_type, application_key, upstream_tool_name,
                                               risk_level, tool_name, arguments_hash,
                                               arguments_summary, policy_decision,
                                               matched_policy_ids, policy_explanation,
                                               credential_scope_summary, header_policy_summary,
                                               approval_state, status, upstream_request_id,
                                               result_hash, result_summary, result_size_bytes,
                                               result_artifact_id, error_code, error_message,
                                               started_at, completed_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29,
                         ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43,
                         ?44,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.idempotency_key,
                    input.actor_type,
                    input.actor_id,
                    input.agent_id,
                    input.issue_id,
                    input.run_id,
                    input.gateway_id,
                    input.gateway_token_id,
                    input.gateway_public_id,
                    input.client_subject_type,
                    input.client_subject_id,
                    input.client_name,
                    input.mcp_session_id,
                    input.correlation_id,
                    input.application_id,
                    input.connection_id,
                    input.catalog_entry_id,
                    input.catalog_version_hash,
                    input.catalog_schema_hash,
                    input.provider_type,
                    input.application_key,
                    input.upstream_tool_name,
                    input.risk_level,
                    input.tool_name,
                    input.arguments_hash,
                    input.arguments_summary.as_ref().map(json_string),
                    input.policy_decision,
                    json_string(&input.matched_policy_ids),
                    input.policy_explanation.as_ref().map(json_string),
                    input.credential_scope_summary.as_ref().map(json_string),
                    input.header_policy_summary.as_ref().map(json_string),
                    input.approval_state,
                    input.status,
                    input.upstream_request_id,
                    input.result_hash,
                    input.result_summary.as_ref().map(json_string),
                    input.result_size_bytes,
                    input.result_artifact_id,
                    input.error_code,
                    input.error_message,
                    input.started_at,
                    input.completed_at
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_invocation(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn get_invocation(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ToolInvocationRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(select_invocation_scoped(&conn, company_id, id).await?)
    }

    async fn list_invocations(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolInvocationRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {INVOCATION_COLUMNS} FROM tool_invocations
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_invocation(&row)?);
        }
        Ok(records)
    }

    async fn create_action_request(
        &self,
        input: NewToolActionRequest,
    ) -> Result<ToolActionRequestRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_action_requests (id, company_id, invocation_id, issue_id,
                                                   interaction_id, approval_id, status,
                                                   canonical_arguments_hash,
                                                   canonical_arguments_summary, signed_arguments,
                                                   preview_markdown, requested_by_agent_id,
                                                   requested_by_user_id, expires_at, created_at,
                                                   updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.invocation_id,
                    input.issue_id,
                    input.interaction_id,
                    input.approval_id,
                    input.status,
                    input.canonical_arguments_hash,
                    json_string(&input.canonical_arguments_summary),
                    input.signed_arguments,
                    input.preview_markdown,
                    input.requested_by_agent_id,
                    input.requested_by_user_id,
                    input.expires_at
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_action_request(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_action_requests(
        &self,
        company_id: &str,
        invocation_id: Option<&str>,
    ) -> Result<Vec<ToolActionRequestRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = invocation_id
            .map(|_| "AND invocation_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = invocation_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {ACTION_REQUEST_COLUMNS} FROM tool_action_requests
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_action_request(&row)?);
        }
        Ok(records)
    }

    async fn create_call_event(
        &self,
        input: NewToolCallEvent,
    ) -> Result<ToolCallEventRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_call_events (id, company_id, event_type, actor_type, actor_id,
                                               agent_id, run_id, issue_id, gateway_id,
                                               gateway_token_id, gateway_public_id,
                                               client_subject_type, client_subject_id,
                                               client_name, mcp_session_id, correlation_id,
                                               application_id, connection_id, catalog_entry_id,
                                               invocation_id, action_request_id, runtime_slot_id,
                                               tool_name, decision, matched_policy_ids,
                                               reason_code, policy_explanation,
                                               credential_scope_summary, header_policy_summary,
                                               outcome, latency_ms, arguments_summary,
                                               request_hash, request_summary, result_hash,
                                               result_summary, result_size_bytes, redaction_plan,
                                               rate_limit_state, metadata, error_code,
                                               error_message, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29,
                         ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.event_type,
                    input.actor_type,
                    input.actor_id,
                    input.agent_id,
                    input.run_id,
                    input.issue_id,
                    input.gateway_id,
                    input.gateway_token_id,
                    input.gateway_public_id,
                    input.client_subject_type,
                    input.client_subject_id,
                    input.client_name,
                    input.mcp_session_id,
                    input.correlation_id,
                    input.application_id,
                    input.connection_id,
                    input.catalog_entry_id,
                    input.invocation_id,
                    input.action_request_id,
                    input.runtime_slot_id,
                    input.tool_name,
                    input.decision,
                    json_string(&input.matched_policy_ids),
                    input.reason_code,
                    input.policy_explanation.as_ref().map(json_string),
                    input.credential_scope_summary.as_ref().map(json_string),
                    input.header_policy_summary.as_ref().map(json_string),
                    input.outcome,
                    input.latency_ms,
                    input.arguments_summary.as_ref().map(json_string),
                    input.request_hash,
                    input.request_summary.as_ref().map(json_string),
                    input.result_hash,
                    input.result_summary.as_ref().map(json_string),
                    input.result_size_bytes,
                    input.redaction_plan.as_ref().map(json_string),
                    input.rate_limit_state.as_ref().map(json_string),
                    input.metadata.as_ref().map(json_string),
                    input.error_code,
                    input.error_message
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_call_event(&conn, &id).await?.expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_call_events(
        &self,
        company_id: &str,
        invocation_id: Option<&str>,
    ) -> Result<Vec<ToolCallEventRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let filter = invocation_id
            .map(|_| "AND invocation_id = ?2")
            .unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = invocation_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CALL_EVENT_COLUMNS} FROM tool_call_events
                     WHERE company_id = ?1 {filter} ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_call_event(&row)?);
        }
        Ok(records)
    }

    async fn create_audit_event(
        &self,
        input: NewToolAccessAuditEvent,
    ) -> Result<ToolAccessAuditEventRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO tool_access_audit_events (id, company_id, gateway_id,
                                                       gateway_token_id, gateway_public_id,
                                                       client_name, correlation_id,
                                                       connection_id, catalog_entry_id,
                                                       actor_type, actor_id, action, outcome,
                                                       reason_code, details, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.gateway_id,
                    input.gateway_token_id,
                    input.gateway_public_id,
                    input.client_name,
                    input.correlation_id,
                    input.connection_id,
                    input.catalog_entry_id,
                    input.actor_type,
                    input.actor_id,
                    input.action,
                    input.outcome,
                    input.reason_code,
                    json_string(&input.details)
                ],
            )
            .await;
        match result {
            Ok(_) => Ok(select_audit_event(&conn, &id)
                .await?
                .expect("just inserted")),
            Err(error) => Err(map_error(error)),
        }
    }

    async fn list_audit_events(
        &self,
        company_id: &str,
    ) -> Result<Vec<ToolAccessAuditEventRecord>, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {AUDIT_EVENT_COLUMNS} FROM tool_access_audit_events
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().await? {
            records.push(row_to_audit_event(&row)?);
        }
        Ok(records)
    }

    async fn upsert_gateway_rate_limit_counter(
        &self,
        input: NewToolGatewayRateLimitCounter,
    ) -> Result<ToolGatewayRateLimitCounterRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let result = conn
            .execute(
                "INSERT INTO tool_gateway_rate_limit_counters (id, company_id, counter_key,
                                                               window_start_at, window_ms,
                                                               \"limit\", count, reset_at,
                                                               created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT (company_id, counter_key, window_start_at) DO UPDATE SET
                   count = excluded.count, reset_at = excluded.reset_at,
                   updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![
                    Uuid::new_v4().to_string(),
                    input.company_id.clone(),
                    input.counter_key.clone(),
                    input.window_start_at.clone(),
                    input.window_ms,
                    input.limit,
                    input.count,
                    input.reset_at
                ],
            )
            .await;
        if let Err(error) = result {
            return Err(map_error(error));
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {GATEWAY_RATE_LIMIT_COLUMNS} FROM tool_gateway_rate_limit_counters
                     WHERE company_id = ?1 AND counter_key = ?2 AND window_start_at = ?3"
                ),
                libsql::params![input.company_id, input.counter_key, input.window_start_at],
            )
            .await?;
        let row = rows.next().await?.expect("counter exists");
        Ok(row_to_gateway_rate_limit_counter(&row)?)
    }

    async fn upsert_rate_limit_counter(
        &self,
        input: NewToolRateLimitCounter,
    ) -> Result<ToolRateLimitCounterRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let result = conn
            .execute(
                "INSERT INTO tool_rate_limit_counters (id, company_id, policy_id, counter_key,
                                                       scope_type, scope_id, window_kind,
                                                       window_start_at, \"limit\", remaining,
                                                       reset_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT (company_id, policy_id, counter_key, window_kind, window_start_at)
                 DO UPDATE SET remaining = excluded.remaining, reset_at = excluded.reset_at,
                   updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![
                    Uuid::new_v4().to_string(),
                    input.company_id.clone(),
                    input.policy_id.clone(),
                    input.counter_key.clone(),
                    input.scope_type,
                    input.scope_id,
                    input.window_kind.clone(),
                    input.window_start_at.clone(),
                    input.limit,
                    input.remaining,
                    input.reset_at
                ],
            )
            .await;
        if let Err(error) = result {
            return Err(map_error(error));
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RATE_LIMIT_COLUMNS} FROM tool_rate_limit_counters
                     WHERE company_id = ?1 AND policy_id = ?2 AND counter_key = ?3
                       AND window_kind = ?4 AND window_start_at = ?5"
                ),
                libsql::params![
                    input.company_id,
                    input.policy_id,
                    input.counter_key,
                    input.window_kind,
                    input.window_start_at
                ],
            )
            .await?;
        let row = rows.next().await?.expect("counter exists");
        Ok(row_to_rate_limit_counter(&row)?)
    }

    async fn upsert_runtime_metric_counter(
        &self,
        input: NewToolRuntimeMetricCounter,
    ) -> Result<ToolRuntimeMetricCounterRecord, ToolchainError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ToolchainError::CompanyNotFound);
        }
        let result = conn
            .execute(
                "INSERT INTO tool_runtime_metric_counters (id, company_id, metric,
                                                           bucket_start_at, count, created_at,
                                                           updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT (company_id, metric, bucket_start_at) DO UPDATE SET
                   count = count + excluded.count,
                   updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![
                    Uuid::new_v4().to_string(),
                    input.company_id.clone(),
                    input.metric.clone(),
                    input.bucket_start_at.clone(),
                    input.count
                ],
            )
            .await;
        if let Err(error) = result {
            return Err(map_error(error));
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {METRIC_COUNTER_COLUMNS} FROM tool_runtime_metric_counters
                     WHERE company_id = ?1 AND metric = ?2 AND bucket_start_at = ?3"
                ),
                libsql::params![input.company_id, input.metric, input.bucket_start_at],
            )
            .await?;
        let row = rows.next().await?.expect("counter exists");
        Ok(row_to_metric_counter(&row)?)
    }
}

fn json_or_default(value: Option<String>) -> serde_json::Value {
    value
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

fn json_opt(value: Option<String>) -> Option<serde_json::Value> {
    value.and_then(|v| serde_json::from_str(&v).ok())
}

fn bool_col(value: i64) -> bool {
    value != 0
}

async fn select_application(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolApplicationRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {APPLICATION_COLUMNS} FROM tool_applications WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_application(&row)?)),
        None => Ok(None),
    }
}

async fn select_application_scoped(
    conn: &libsql::Connection,
    company_id: &str,
    id: &str,
) -> Result<Option<ToolApplicationRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {APPLICATION_COLUMNS} FROM tool_applications
                 WHERE id = ?1 AND company_id = ?2"
            ),
            libsql::params![id, company_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_application(&row)?)),
        None => Ok(None),
    }
}

fn row_to_application(row: &libsql::Row) -> Result<ToolApplicationRecord, libsql::Error> {
    Ok(ToolApplicationRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        application_key: helpers::row_text(row, 2)?,
        name: helpers::row_text(row, 3)?.expect("name"),
        description: helpers::row_text(row, 4)?,
        r#type: helpers::row_text(row, 5)?.expect("type"),
        status: helpers::row_text(row, 6)?.expect("status"),
        plugin_id: helpers::row_text(row, 7)?,
        owner_agent_id: helpers::row_text(row, 8)?,
        owner_user_id: helpers::row_text(row, 9)?,
        metadata: json_or_default(helpers::row_text(row, 10)?),
        archived_at: helpers::row_text(row, 11)?,
        created_at: helpers::row_text(row, 12)?.expect("created_at"),
        updated_at: helpers::row_text(row, 13)?.expect("updated_at"),
    })
}

async fn select_connection(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolConnectionRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {CONNECTION_COLUMNS} FROM tool_connections WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_connection(&row)?)),
        None => Ok(None),
    }
}

async fn select_connection_scoped(
    conn: &libsql::Connection,
    company_id: &str,
    id: &str,
) -> Result<Option<ToolConnectionRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {CONNECTION_COLUMNS} FROM tool_connections
                 WHERE id = ?1 AND company_id = ?2"
            ),
            libsql::params![id, company_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_connection(&row)?)),
        None => Ok(None),
    }
}

fn row_to_connection(row: &libsql::Row) -> Result<ToolConnectionRecord, libsql::Error> {
    Ok(ToolConnectionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        application_id: helpers::row_text(row, 2)?.expect("application_id"),
        name: helpers::row_text(row, 3)?.expect("name"),
        uid: helpers::row_text(row, 4)?.expect("uid"),
        connection_kind: helpers::row_text(row, 5)?.expect("connection_kind"),
        ownership: helpers::row_text(row, 6)?.expect("ownership"),
        transport: helpers::row_text(row, 7)?.expect("transport"),
        auth_kind: helpers::row_text(row, 8)?.expect("auth_kind"),
        status: helpers::row_text(row, 9)?.expect("status"),
        enabled: bool_col(helpers::row_i64(row, 10)?),
        config: json_or_default(helpers::row_text(row, 11)?),
        transport_config: json_or_default(helpers::row_text(row, 12)?),
        credential_refs: json_or_default(helpers::row_text(row, 13)?),
        credential_secret_refs: json_or_default(helpers::row_text(row, 14)?),
        health_status: helpers::row_text(row, 15)?.expect("health_status"),
        health_message: helpers::row_text(row, 16)?,
        health_checked_at: helpers::row_text(row, 17)?,
        last_health_at: helpers::row_text(row, 18)?,
        last_catalog_refresh_at: helpers::row_text(row, 19)?,
        last_error: helpers::row_text(row, 20)?,
        created_by_agent_id: helpers::row_text(row, 21)?,
        created_by_user_id: helpers::row_text(row, 22)?,
        created_at: helpers::row_text(row, 23)?.expect("created_at"),
        updated_at: helpers::row_text(row, 24)?.expect("updated_at"),
    })
}

async fn select_grant(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ConnectionGrantRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {GRANT_COLUMNS} FROM connection_grants WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_grant(&row)?)),
        None => Ok(None),
    }
}

fn row_to_grant(row: &libsql::Row) -> Result<ConnectionGrantRecord, libsql::Error> {
    Ok(ConnectionGrantRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        connection_id: helpers::row_text(row, 2)?.expect("connection_id"),
        kind: helpers::row_text(row, 3)?.expect("kind"),
        subject_user_id: helpers::row_text(row, 4)?,
        provider_tenant: json_opt(helpers::row_text(row, 5)?),
        credential_secret_refs: json_or_default(helpers::row_text(row, 6)?),
        status: helpers::row_text(row, 7)?.expect("status"),
        is_default: bool_col(helpers::row_i64(row, 8)?),
        created_by_agent_id: helpers::row_text(row, 9)?,
        created_by_user_id: helpers::row_text(row, 10)?,
        revoked_at: helpers::row_text(row, 11)?,
        revoked_by_agent_id: helpers::row_text(row, 12)?,
        revoked_by_user_id: helpers::row_text(row, 13)?,
        last_used_at: helpers::row_text(row, 14)?,
        created_at: helpers::row_text(row, 15)?.expect("created_at"),
        updated_at: helpers::row_text(row, 16)?.expect("updated_at"),
    })
}

async fn select_install(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolConnectionInstallRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {INSTALL_COLUMNS} FROM tool_connection_installs WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_install(&row)?)),
        None => Ok(None),
    }
}

fn row_to_install(row: &libsql::Row) -> Result<ToolConnectionInstallRecord, libsql::Error> {
    Ok(ToolConnectionInstallRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        connection_id: helpers::row_text(row, 2)?.expect("connection_id"),
        target_type: helpers::row_text(row, 3)?.expect("target_type"),
        target_id: helpers::row_text(row, 4)?.expect("target_id"),
        created_by_agent_id: helpers::row_text(row, 5)?,
        created_by_user_id: helpers::row_text(row, 6)?,
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
    })
}

async fn select_oauth_state(
    conn: &libsql::Connection,
    state: &str,
) -> Result<Option<ToolOauthStateRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {OAUTH_STATE_COLUMNS} FROM tool_oauth_states WHERE state = ?1"),
            libsql::params![state],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_oauth_state(&row)?)),
        None => Ok(None),
    }
}

fn row_to_oauth_state(row: &libsql::Row) -> Result<ToolOauthStateRecord, libsql::Error> {
    Ok(ToolOauthStateRecord {
        state: helpers::row_text(row, 0)?.expect("state"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        connection_id: helpers::row_text(row, 2)?.expect("connection_id"),
        code_verifier: helpers::row_text(row, 3)?.expect("code_verifier"),
        created_by_actor_type: helpers::row_text(row, 4)?,
        created_by_actor_id: helpers::row_text(row, 5)?,
        created_by_session_id: helpers::row_text(row, 6)?,
        subject_user_id: helpers::row_text(row, 7)?,
        requested_scopes: json_opt(helpers::row_text(row, 8)?),
        return_to: helpers::row_text(row, 9)?,
        issue_id: helpers::row_text(row, 10)?,
        interaction_id: helpers::row_text(row, 11)?,
        expires_at: helpers::row_text(row, 12)?.expect("expires_at"),
        created_at: helpers::row_text(row, 13)?.expect("created_at"),
    })
}

async fn select_catalog_entry(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolCatalogEntryRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {CATALOG_ENTRY_COLUMNS} FROM tool_catalog_entries WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_catalog_entry(&row)?)),
        None => Ok(None),
    }
}

fn row_to_catalog_entry(row: &libsql::Row) -> Result<ToolCatalogEntryRecord, libsql::Error> {
    Ok(ToolCatalogEntryRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        application_id: helpers::row_text(row, 2)?,
        connection_id: helpers::row_text(row, 3)?.expect("connection_id"),
        entry_kind: helpers::row_text(row, 4)?.expect("entry_kind"),
        name: helpers::row_text(row, 5)?.expect("name"),
        tool_name: helpers::row_text(row, 6)?.expect("tool_name"),
        title: helpers::row_text(row, 7)?,
        description: helpers::row_text(row, 8)?,
        input_schema: json_or_default(helpers::row_text(row, 9)?),
        output_schema: json_opt(helpers::row_text(row, 10)?),
        annotations: json_or_default(helpers::row_text(row, 11)?),
        risk_level: helpers::row_text(row, 12)?.expect("risk_level"),
        is_read_only: bool_col(helpers::row_i64(row, 13)?),
        is_write: bool_col(helpers::row_i64(row, 14)?),
        is_destructive: bool_col(helpers::row_i64(row, 15)?),
        status: helpers::row_text(row, 16)?.expect("status"),
        version: helpers::row_text(row, 17)?,
        version_hash: helpers::row_text(row, 18)?.expect("version_hash"),
        schema_hash: helpers::row_text(row, 19)?,
        first_seen_at: helpers::row_text(row, 20)?.expect("first_seen_at"),
        last_seen_at: helpers::row_text(row, 21)?.expect("last_seen_at"),
        reviewed_at: helpers::row_text(row, 22)?,
        reviewed_by_agent_id: helpers::row_text(row, 23)?,
        reviewed_by_user_id: helpers::row_text(row, 24)?,
        quarantined_at: helpers::row_text(row, 25)?,
        quarantine_reason: helpers::row_text(row, 26)?,
        created_at: helpers::row_text(row, 27)?.expect("created_at"),
        updated_at: helpers::row_text(row, 28)?.expect("updated_at"),
    })
}

async fn select_profile(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolProfileRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {PROFILE_COLUMNS} FROM tool_profiles WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_profile(&row)?)),
        None => Ok(None),
    }
}

async fn select_profile_scoped(
    conn: &libsql::Connection,
    company_id: &str,
    id: &str,
) -> Result<Option<ToolProfileRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {PROFILE_COLUMNS} FROM tool_profiles WHERE id = ?1 AND company_id = ?2"
            ),
            libsql::params![id, company_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_profile(&row)?)),
        None => Ok(None),
    }
}

fn row_to_profile(row: &libsql::Row) -> Result<ToolProfileRecord, libsql::Error> {
    Ok(ToolProfileRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        profile_key: helpers::row_text(row, 2)?.expect("profile_key"),
        name: helpers::row_text(row, 3)?.expect("name"),
        description: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status"),
        default_action: helpers::row_text(row, 6)?.expect("default_action"),
        new_tools_reviewed_at: helpers::row_text(row, 7)?,
        metadata: json_or_default(helpers::row_text(row, 8)?),
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
        updated_at: helpers::row_text(row, 10)?.expect("updated_at"),
    })
}

async fn select_profile_entry(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolProfileEntryRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {PROFILE_ENTRY_COLUMNS} FROM tool_profile_entries WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_profile_entry(&row)?)),
        None => Ok(None),
    }
}

fn row_to_profile_entry(row: &libsql::Row) -> Result<ToolProfileEntryRecord, libsql::Error> {
    Ok(ToolProfileEntryRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        profile_id: helpers::row_text(row, 2)?.expect("profile_id"),
        selector_type: helpers::row_text(row, 3)?.expect("selector_type"),
        effect: helpers::row_text(row, 4)?.expect("effect"),
        application_id: helpers::row_text(row, 5)?,
        connection_id: helpers::row_text(row, 6)?,
        catalog_entry_id: helpers::row_text(row, 7)?,
        tool_name: helpers::row_text(row, 8)?,
        risk_level: helpers::row_text(row, 9)?,
        conditions: json_opt(helpers::row_text(row, 10)?),
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
        updated_at: helpers::row_text(row, 12)?.expect("updated_at"),
    })
}

async fn select_profile_binding(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolProfileBindingRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {PROFILE_BINDING_COLUMNS} FROM tool_profile_bindings WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_profile_binding(&row)?)),
        None => Ok(None),
    }
}

fn row_to_profile_binding(row: &libsql::Row) -> Result<ToolProfileBindingRecord, libsql::Error> {
    Ok(ToolProfileBindingRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        profile_id: helpers::row_text(row, 2)?.expect("profile_id"),
        target_type: helpers::row_text(row, 3)?.expect("target_type"),
        target_id: helpers::row_text(row, 4)?.expect("target_id"),
        priority: helpers::row_i64(row, 5)?,
        metadata: json_or_default(helpers::row_text(row, 6)?),
        created_by_agent_id: helpers::row_text(row, 7)?,
        created_by_user_id: helpers::row_text(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
        updated_at: helpers::row_text(row, 10)?.expect("updated_at"),
    })
}

async fn select_gateway(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolMcpGatewayRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {GATEWAY_COLUMNS} FROM tool_mcp_gateways WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_gateway(&row)?)),
        None => Ok(None),
    }
}

async fn select_gateway_scoped(
    conn: &libsql::Connection,
    company_id: &str,
    id: &str,
) -> Result<Option<ToolMcpGatewayRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {GATEWAY_COLUMNS} FROM tool_mcp_gateways WHERE id = ?1 AND company_id = ?2"
            ),
            libsql::params![id, company_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_gateway(&row)?)),
        None => Ok(None),
    }
}

fn row_to_gateway(row: &libsql::Row) -> Result<ToolMcpGatewayRecord, libsql::Error> {
    Ok(ToolMcpGatewayRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        gateway_public_id: helpers::row_text(row, 2)?.expect("gateway_public_id"),
        name: helpers::row_text(row, 3)?.expect("name"),
        slug: helpers::row_text(row, 4)?.expect("slug"),
        display_slug: helpers::row_text(row, 5)?.expect("display_slug"),
        description: helpers::row_text(row, 6)?,
        status: helpers::row_text(row, 7)?.expect("status"),
        profile_id: helpers::row_text(row, 8)?.expect("profile_id"),
        default_profile_mode: helpers::row_text(row, 9)?.expect("default_profile_mode"),
        context_scope_type: helpers::row_text(row, 10)?.expect("context_scope_type"),
        context_scope_id: helpers::row_text(row, 11)?,
        agent_id: helpers::row_text(row, 12)?,
        project_id: helpers::row_text(row, 13)?,
        issue_id: helpers::row_text(row, 14)?,
        approval_issue_id: helpers::row_text(row, 15)?,
        auth_config: json_or_default(helpers::row_text(row, 16)?),
        header_policy: json_or_default(helpers::row_text(row, 17)?),
        metadata_policy: json_or_default(helpers::row_text(row, 18)?),
        on_demand_tools_config: json_or_default(helpers::row_text(row, 19)?),
        metadata: json_or_default(helpers::row_text(row, 20)?),
        created_by_agent_id: helpers::row_text(row, 21)?,
        created_by_user_id: helpers::row_text(row, 22)?,
        archived_at: helpers::row_text(row, 23)?,
        created_at: helpers::row_text(row, 24)?.expect("created_at"),
        updated_at: helpers::row_text(row, 25)?.expect("updated_at"),
    })
}

async fn select_gateway_token(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolMcpGatewayTokenRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {GATEWAY_TOKEN_COLUMNS} FROM tool_mcp_gateway_tokens WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_gateway_token(&row)?)),
        None => Ok(None),
    }
}

fn row_to_gateway_token(row: &libsql::Row) -> Result<ToolMcpGatewayTokenRecord, libsql::Error> {
    Ok(ToolMcpGatewayTokenRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        gateway_id: helpers::row_text(row, 2)?.expect("gateway_id"),
        name: helpers::row_text(row, 3)?.expect("name"),
        token_hash: helpers::row_text(row, 4)?.expect("token_hash"),
        token_prefix: helpers::row_text(row, 5)?.expect("token_prefix"),
        subject_type: helpers::row_text(row, 6)?.expect("subject_type"),
        subject_id: helpers::row_text(row, 7)?,
        client_label: helpers::row_text(row, 8)?.expect("client_label"),
        owner_note: helpers::row_text(row, 9)?.expect("owner_note"),
        allowed_actions: json_or_default(helpers::row_text(row, 10)?),
        expires_at: helpers::row_text(row, 11)?,
        expiry_override_reason: helpers::row_text(row, 12)?,
        expiry_override_by_user_id: helpers::row_text(row, 13)?,
        expiry_override_by_agent_id: helpers::row_text(row, 14)?,
        expiry_override_at: helpers::row_text(row, 15)?,
        last_used_at: helpers::row_text(row, 16)?,
        revoked_at: helpers::row_text(row, 17)?,
        created_by_agent_id: helpers::row_text(row, 18)?,
        created_by_user_id: helpers::row_text(row, 19)?,
        created_at: helpers::row_text(row, 20)?.expect("created_at"),
        updated_at: helpers::row_text(row, 21)?.expect("updated_at"),
    })
}

async fn select_policy(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolPolicyRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {POLICY_COLUMNS} FROM tool_policies WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_policy(&row)?)),
        None => Ok(None),
    }
}

fn row_to_policy(row: &libsql::Row) -> Result<ToolPolicyRecord, libsql::Error> {
    Ok(ToolPolicyRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        name: helpers::row_text(row, 2)?.expect("name"),
        description: helpers::row_text(row, 3)?,
        policy_type: helpers::row_text(row, 4)?.expect("policy_type"),
        priority: helpers::row_i64(row, 5)?,
        enabled: bool_col(helpers::row_i64(row, 6)?),
        selectors: json_or_default(helpers::row_text(row, 7)?),
        conditions: json_opt(helpers::row_text(row, 8)?),
        config: json_opt(helpers::row_text(row, 9)?),
        created_by_agent_id: helpers::row_text(row, 10)?,
        created_by_user_id: helpers::row_text(row, 11)?,
        created_at: helpers::row_text(row, 12)?.expect("created_at"),
        updated_at: helpers::row_text(row, 13)?.expect("updated_at"),
    })
}

async fn select_runtime_slot(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolRuntimeSlotRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {RUNTIME_SLOT_COLUMNS} FROM tool_runtime_slots WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_runtime_slot(&row)?)),
        None => Ok(None),
    }
}

fn row_to_runtime_slot(row: &libsql::Row) -> Result<ToolRuntimeSlotRecord, libsql::Error> {
    Ok(ToolRuntimeSlotRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        application_id: helpers::row_text(row, 2)?,
        connection_id: helpers::row_text(row, 3)?,
        project_workspace_id: helpers::row_text(row, 4)?,
        execution_workspace_id: helpers::row_text(row, 5)?,
        issue_id: helpers::row_text(row, 6)?,
        owner_scope_type: helpers::row_text(row, 7)?.expect("owner_scope_type"),
        owner_scope_id: helpers::row_text(row, 8)?,
        runtime_kind: helpers::row_text(row, 9)?.expect("runtime_kind"),
        slot_key: helpers::row_text(row, 10)?.expect("slot_key"),
        status: helpers::row_text(row, 11)?.expect("status"),
        reuse_key: helpers::row_text(row, 12)?,
        workspace_scope: helpers::row_text(row, 13)?,
        credential_scope_hash: helpers::row_text(row, 14)?,
        provider: helpers::row_text(row, 15)?,
        provider_ref: helpers::row_text(row, 16)?,
        process_id: helpers::row_i64_opt(row, 17)?,
        command_template_key: helpers::row_text(row, 18)?,
        health_status: helpers::row_text(row, 19)?.expect("health_status"),
        health_message: helpers::row_text(row, 20)?,
        last_health_check_at: helpers::row_text(row, 21)?,
        last_started_at: helpers::row_text(row, 22)?,
        started_at: helpers::row_text(row, 23)?,
        stopped_at: helpers::row_text(row, 24)?,
        last_used_at: helpers::row_text(row, 25)?,
        idle_expires_at: helpers::row_text(row, 26)?,
        idle_deadline_at: helpers::row_text(row, 27)?,
        last_error: helpers::row_text(row, 28)?,
        metadata: json_or_default(helpers::row_text(row, 29)?),
        created_at: helpers::row_text(row, 30)?.expect("created_at"),
        updated_at: helpers::row_text(row, 31)?.expect("updated_at"),
    })
}

async fn select_stdio_template(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolStdioCommandTemplateRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {STDIO_TEMPLATE_COLUMNS} FROM tool_stdio_command_templates WHERE id = ?1"
            ),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_stdio_template(&row)?)),
        None => Ok(None),
    }
}

fn row_to_stdio_template(
    row: &libsql::Row,
) -> Result<ToolStdioCommandTemplateRecord, libsql::Error> {
    Ok(ToolStdioCommandTemplateRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        template_key: helpers::row_text(row, 2)?.expect("template_key"),
        name: helpers::row_text(row, 3)?.expect("name"),
        description: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status"),
        command: helpers::row_text(row, 6)?.expect("command"),
        args: json_or_default(helpers::row_text(row, 7)?),
        env_keys: json_or_default(helpers::row_text(row, 8)?),
        tools: json_or_default(helpers::row_text(row, 9)?),
        created_by_agent_id: helpers::row_text(row, 10)?,
        created_by_user_id: helpers::row_text(row, 11)?,
        disabled_at: helpers::row_text(row, 12)?,
        created_at: helpers::row_text(row, 13)?.expect("created_at"),
        updated_at: helpers::row_text(row, 14)?.expect("updated_at"),
    })
}

async fn select_gateway_session(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolGatewaySessionRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {GATEWAY_SESSION_COLUMNS} FROM tool_gateway_sessions WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_gateway_session(&row)?)),
        None => Ok(None),
    }
}

fn row_to_gateway_session(row: &libsql::Row) -> Result<ToolGatewaySessionRecord, libsql::Error> {
    Ok(ToolGatewaySessionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        agent_id: helpers::row_text(row, 2)?.expect("agent_id"),
        run_id: helpers::row_text(row, 3)?.expect("run_id"),
        issue_id: helpers::row_text(row, 4)?,
        project_id: helpers::row_text(row, 5)?,
        gateway_id: helpers::row_text(row, 6)?,
        gateway_token_id: helpers::row_text(row, 7)?,
        gateway_public_id: helpers::row_text(row, 8)?,
        client_subject_type: helpers::row_text(row, 9)?,
        client_subject_id: helpers::row_text(row, 10)?,
        client_name: helpers::row_text(row, 11)?,
        mcp_session_id: helpers::row_text(row, 12)?,
        correlation_id: helpers::row_text(row, 13)?,
        token_hash: helpers::row_text(row, 14)?.expect("token_hash"),
        expires_at: helpers::row_text(row, 15)?.expect("expires_at"),
        last_used_at: helpers::row_text(row, 16)?,
        revoked_at: helpers::row_text(row, 17)?,
        created_at: helpers::row_text(row, 18)?.expect("created_at"),
        updated_at: helpers::row_text(row, 19)?.expect("updated_at"),
    })
}

async fn select_invocation(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolInvocationRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {INVOCATION_COLUMNS} FROM tool_invocations WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_invocation(&row)?)),
        None => Ok(None),
    }
}

async fn select_invocation_scoped(
    conn: &libsql::Connection,
    company_id: &str,
    id: &str,
) -> Result<Option<ToolInvocationRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {INVOCATION_COLUMNS} FROM tool_invocations
                 WHERE id = ?1 AND company_id = ?2"
            ),
            libsql::params![id, company_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_invocation(&row)?)),
        None => Ok(None),
    }
}

fn row_to_invocation(row: &libsql::Row) -> Result<ToolInvocationRecord, libsql::Error> {
    Ok(ToolInvocationRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        idempotency_key: helpers::row_text(row, 2)?,
        actor_type: helpers::row_text(row, 3)?.expect("actor_type"),
        actor_id: helpers::row_text(row, 4)?,
        agent_id: helpers::row_text(row, 5)?,
        issue_id: helpers::row_text(row, 6)?,
        run_id: helpers::row_text(row, 7)?,
        gateway_id: helpers::row_text(row, 8)?,
        gateway_token_id: helpers::row_text(row, 9)?,
        gateway_public_id: helpers::row_text(row, 10)?,
        client_subject_type: helpers::row_text(row, 11)?,
        client_subject_id: helpers::row_text(row, 12)?,
        client_name: helpers::row_text(row, 13)?,
        mcp_session_id: helpers::row_text(row, 14)?,
        correlation_id: helpers::row_text(row, 15)?,
        application_id: helpers::row_text(row, 16)?,
        connection_id: helpers::row_text(row, 17)?,
        catalog_entry_id: helpers::row_text(row, 18)?,
        catalog_version_hash: helpers::row_text(row, 19)?,
        catalog_schema_hash: helpers::row_text(row, 20)?,
        provider_type: helpers::row_text(row, 21)?,
        application_key: helpers::row_text(row, 22)?,
        upstream_tool_name: helpers::row_text(row, 23)?,
        risk_level: helpers::row_text(row, 24)?,
        tool_name: helpers::row_text(row, 25)?.expect("tool_name"),
        arguments_hash: helpers::row_text(row, 26)?,
        arguments_summary: json_opt(helpers::row_text(row, 27)?),
        policy_decision: helpers::row_text(row, 28)?,
        matched_policy_ids: json_or_default(helpers::row_text(row, 29)?),
        policy_explanation: json_opt(helpers::row_text(row, 30)?),
        credential_scope_summary: json_opt(helpers::row_text(row, 31)?),
        header_policy_summary: json_opt(helpers::row_text(row, 32)?),
        approval_state: helpers::row_text(row, 33)?.expect("approval_state"),
        status: helpers::row_text(row, 34)?.expect("status"),
        upstream_request_id: helpers::row_text(row, 35)?,
        result_hash: helpers::row_text(row, 36)?,
        result_summary: json_opt(helpers::row_text(row, 37)?),
        result_size_bytes: helpers::row_i64_opt(row, 38)?,
        result_artifact_id: helpers::row_text(row, 39)?,
        error_code: helpers::row_text(row, 40)?,
        error_message: helpers::row_text(row, 41)?,
        started_at: helpers::row_text(row, 42)?,
        completed_at: helpers::row_text(row, 43)?,
        created_at: helpers::row_text(row, 44)?.expect("created_at"),
        updated_at: helpers::row_text(row, 45)?.expect("updated_at"),
    })
}

async fn select_action_request(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolActionRequestRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {ACTION_REQUEST_COLUMNS} FROM tool_action_requests WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_action_request(&row)?)),
        None => Ok(None),
    }
}

fn row_to_action_request(row: &libsql::Row) -> Result<ToolActionRequestRecord, libsql::Error> {
    Ok(ToolActionRequestRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        invocation_id: helpers::row_text(row, 2)?.expect("invocation_id"),
        issue_id: helpers::row_text(row, 3)?,
        interaction_id: helpers::row_text(row, 4)?,
        approval_id: helpers::row_text(row, 5)?,
        status: helpers::row_text(row, 6)?.expect("status"),
        canonical_arguments_hash: helpers::row_text(row, 7)?.expect("canonical_arguments_hash"),
        canonical_arguments_summary: json_or_default(helpers::row_text(row, 8)?),
        signed_arguments: helpers::row_text(row, 9)?,
        preview_markdown: helpers::row_text(row, 10)?,
        requested_by_agent_id: helpers::row_text(row, 11)?,
        requested_by_user_id: helpers::row_text(row, 12)?,
        resolved_by_agent_id: helpers::row_text(row, 13)?,
        resolved_by_user_id: helpers::row_text(row, 14)?,
        decided_by_agent_id: helpers::row_text(row, 15)?,
        decided_by_user_id: helpers::row_text(row, 16)?,
        decided_at: helpers::row_text(row, 17)?,
        expires_at: helpers::row_text(row, 18)?,
        resolved_at: helpers::row_text(row, 19)?,
        created_at: helpers::row_text(row, 20)?.expect("created_at"),
        updated_at: helpers::row_text(row, 21)?.expect("updated_at"),
    })
}

async fn select_call_event(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolCallEventRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {CALL_EVENT_COLUMNS} FROM tool_call_events WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_call_event(&row)?)),
        None => Ok(None),
    }
}

fn row_to_call_event(row: &libsql::Row) -> Result<ToolCallEventRecord, libsql::Error> {
    Ok(ToolCallEventRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        event_type: helpers::row_text(row, 2)?.expect("event_type"),
        actor_type: helpers::row_text(row, 3)?.expect("actor_type"),
        actor_id: helpers::row_text(row, 4)?,
        agent_id: helpers::row_text(row, 5)?,
        run_id: helpers::row_text(row, 6)?,
        issue_id: helpers::row_text(row, 7)?,
        gateway_id: helpers::row_text(row, 8)?,
        gateway_token_id: helpers::row_text(row, 9)?,
        gateway_public_id: helpers::row_text(row, 10)?,
        client_subject_type: helpers::row_text(row, 11)?,
        client_subject_id: helpers::row_text(row, 12)?,
        client_name: helpers::row_text(row, 13)?,
        mcp_session_id: helpers::row_text(row, 14)?,
        correlation_id: helpers::row_text(row, 15)?,
        application_id: helpers::row_text(row, 16)?,
        connection_id: helpers::row_text(row, 17)?,
        catalog_entry_id: helpers::row_text(row, 18)?,
        invocation_id: helpers::row_text(row, 19)?,
        action_request_id: helpers::row_text(row, 20)?,
        runtime_slot_id: helpers::row_text(row, 21)?,
        tool_name: helpers::row_text(row, 22)?,
        decision: helpers::row_text(row, 23)?,
        matched_policy_ids: json_or_default(helpers::row_text(row, 24)?),
        reason_code: helpers::row_text(row, 25)?,
        policy_explanation: json_opt(helpers::row_text(row, 26)?),
        credential_scope_summary: json_opt(helpers::row_text(row, 27)?),
        header_policy_summary: json_opt(helpers::row_text(row, 28)?),
        outcome: helpers::row_text(row, 29)?.expect("outcome"),
        latency_ms: helpers::row_i64_opt(row, 30)?,
        arguments_summary: json_opt(helpers::row_text(row, 31)?),
        request_hash: helpers::row_text(row, 32)?,
        request_summary: json_opt(helpers::row_text(row, 33)?),
        result_hash: helpers::row_text(row, 34)?,
        result_summary: json_opt(helpers::row_text(row, 35)?),
        result_size_bytes: helpers::row_i64_opt(row, 36)?,
        redaction_plan: json_opt(helpers::row_text(row, 37)?),
        rate_limit_state: json_opt(helpers::row_text(row, 38)?),
        metadata: json_opt(helpers::row_text(row, 39)?),
        error_code: helpers::row_text(row, 40)?,
        error_message: helpers::row_text(row, 41)?,
        created_at: helpers::row_text(row, 42)?.expect("created_at"),
    })
}

async fn select_audit_event(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ToolAccessAuditEventRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!("SELECT {AUDIT_EVENT_COLUMNS} FROM tool_access_audit_events WHERE id = ?1"),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_audit_event(&row)?)),
        None => Ok(None),
    }
}

fn row_to_audit_event(row: &libsql::Row) -> Result<ToolAccessAuditEventRecord, libsql::Error> {
    Ok(ToolAccessAuditEventRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        gateway_id: helpers::row_text(row, 2)?,
        gateway_token_id: helpers::row_text(row, 3)?,
        gateway_public_id: helpers::row_text(row, 4)?,
        client_name: helpers::row_text(row, 5)?,
        correlation_id: helpers::row_text(row, 6)?,
        connection_id: helpers::row_text(row, 7)?,
        catalog_entry_id: helpers::row_text(row, 8)?,
        actor_type: helpers::row_text(row, 9)?.expect("actor_type"),
        actor_id: helpers::row_text(row, 10)?,
        action: helpers::row_text(row, 11)?.expect("action"),
        outcome: helpers::row_text(row, 12)?.expect("outcome"),
        reason_code: helpers::row_text(row, 13)?,
        details: json_or_default(helpers::row_text(row, 14)?),
        created_at: helpers::row_text(row, 15)?.expect("created_at"),
    })
}

async fn select_token_issuance(
    conn: &libsql::Connection,
    id: &str,
) -> Result<Option<ConnectionTokenIssuanceRecord>, ToolchainError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {TOKEN_ISSUANCE_COLUMNS} FROM connection_token_issuances WHERE id = ?1"
            ),
            libsql::params![id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some(row_to_token_issuance(&row)?)),
        None => Ok(None),
    }
}

fn row_to_token_issuance(
    row: &libsql::Row,
) -> Result<ConnectionTokenIssuanceRecord, libsql::Error> {
    Ok(ConnectionTokenIssuanceRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        application_id: helpers::row_text(row, 2)?,
        connection_id: helpers::row_text(row, 3)?.expect("connection_id"),
        agent_id: helpers::row_text(row, 4)?.expect("agent_id"),
        run_id: helpers::row_text(row, 5)?,
        issue_id: helpers::row_text(row, 6)?,
        project_id: helpers::row_text(row, 7)?,
        responsible_user_id: helpers::row_text(row, 8)?,
        path: helpers::row_text(row, 9)?.expect("path"),
        requested_scope: json_or_default(helpers::row_text(row, 10)?),
        issued_scope: json_or_default(helpers::row_text(row, 11)?),
        ttl_seconds: helpers::row_i64_opt(row, 12)?,
        expires_at: helpers::row_text(row, 13)?,
        token_hash: helpers::row_text(row, 14)?,
        outcome: helpers::row_text(row, 15)?.expect("outcome"),
        error_code: helpers::row_text(row, 16)?,
        metadata: json_or_default(helpers::row_text(row, 17)?),
        created_at: helpers::row_text(row, 18)?.expect("created_at"),
    })
}

fn row_to_gateway_rate_limit_counter(
    row: &libsql::Row,
) -> Result<ToolGatewayRateLimitCounterRecord, libsql::Error> {
    Ok(ToolGatewayRateLimitCounterRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        counter_key: helpers::row_text(row, 2)?.expect("counter_key"),
        window_start_at: helpers::row_text(row, 3)?.expect("window_start_at"),
        window_ms: helpers::row_i64(row, 4)?,
        limit: helpers::row_i64(row, 5)?,
        count: helpers::row_i64(row, 6)?,
        reset_at: helpers::row_text(row, 7)?.expect("reset_at"),
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
        updated_at: helpers::row_text(row, 9)?.expect("updated_at"),
    })
}

fn row_to_rate_limit_counter(
    row: &libsql::Row,
) -> Result<ToolRateLimitCounterRecord, libsql::Error> {
    Ok(ToolRateLimitCounterRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        policy_id: helpers::row_text(row, 2)?.expect("policy_id"),
        counter_key: helpers::row_text(row, 3)?.expect("counter_key"),
        scope_type: helpers::row_text(row, 4)?.expect("scope_type"),
        scope_id: helpers::row_text(row, 5)?.expect("scope_id"),
        window_kind: helpers::row_text(row, 6)?.expect("window_kind"),
        window_start_at: helpers::row_text(row, 7)?.expect("window_start_at"),
        limit: helpers::row_i64(row, 8)?,
        remaining: helpers::row_i64(row, 9)?,
        reset_at: helpers::row_text(row, 10)?.expect("reset_at"),
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
        updated_at: helpers::row_text(row, 12)?.expect("updated_at"),
    })
}

fn row_to_metric_counter(
    row: &libsql::Row,
) -> Result<ToolRuntimeMetricCounterRecord, libsql::Error> {
    Ok(ToolRuntimeMetricCounterRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        metric: helpers::row_text(row, 2)?.expect("metric"),
        bucket_start_at: helpers::row_text(row, 3)?.expect("bucket_start_at"),
        count: helpers::row_i64(row, 4)?,
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
        updated_at: helpers::row_text(row, 6)?.expect("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn seed(db: &Database) {
        migrate(db).await.unwrap();
        let conn = crate::connect(db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'Agent 1', 'engineer', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO plugins (id, plugin_key, package_name, version, manifest_json)
             VALUES ('p1', 'test-plugin', 'test-plugin', '1.0.0', '{}')",
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
            "INSERT INTO heartbeat_runs (id, company_id, agent_id, invocation_source)
             VALUES ('r1', 'c1', 'a1', 'manual')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO projects (id, company_id, name) VALUES ('pr1', 'c1', 'Project 1')",
            (),
        )
        .await
        .unwrap();
    }

    async fn repos() -> (
        TempDir,
        TursoToolCatalogRepository,
        TursoToolConnectionRepository,
        TursoToolGatewayRepository,
    ) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        seed(&db).await;
        // Each repository owns its own Database handle to the same file.
        let catalog = TursoToolCatalogRepository::new(
            open(&crate::DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        let connections = TursoToolConnectionRepository::new(
            open(&crate::DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        let gateway = TursoToolGatewayRepository::new(
            open(&crate::DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        (dir, catalog, connections, gateway)
    }

    fn app_input() -> NewToolApplication {
        NewToolApplication {
            company_id: "c1".to_owned(),
            application_key: Some("app-key-1".to_owned()),
            name: "App 1".to_owned(),
            description: Some("desc".to_owned()),
            r#type: "internal".to_owned(),
            status: "active".to_owned(),
            plugin_id: Some("p1".to_owned()),
            owner_agent_id: Some("a1".to_owned()),
            owner_user_id: Some("u1".to_owned()),
            metadata: serde_json::json!({ "origin": "test" }),
        }
    }

    fn profile_input() -> NewToolProfile {
        NewToolProfile {
            company_id: "c1".to_owned(),
            profile_key: "default".to_owned(),
            name: "Default".to_owned(),
            description: Some("base profile".to_owned()),
            status: "active".to_owned(),
            default_action: "deny".to_owned(),
            new_tools_reviewed_at: None,
            metadata: serde_json::json!({}),
        }
    }

    fn connection_input(application_id: &str) -> NewToolConnection {
        NewToolConnection {
            company_id: "c1".to_owned(),
            application_id: application_id.to_owned(),
            name: "Conn 1".to_owned(),
            uid: "conn-uid-1".to_owned(),
            connection_kind: "managed".to_owned(),
            ownership: "customer".to_owned(),
            transport: "mcp_remote".to_owned(),
            auth_kind: "none".to_owned(),
            status: "draft".to_owned(),
            enabled: false,
            config: serde_json::json!({}),
            transport_config: serde_json::json!({}),
            credential_refs: serde_json::json!([]),
            credential_secret_refs: serde_json::json!([]),
            created_by_agent_id: Some("a1".to_owned()),
            created_by_user_id: None,
        }
    }

    fn invocation_input() -> NewToolInvocation {
        NewToolInvocation {
            company_id: "c1".to_owned(),
            idempotency_key: Some("inv-1".to_owned()),
            actor_type: "agent".to_owned(),
            actor_id: Some("a1".to_owned()),
            agent_id: Some("a1".to_owned()),
            issue_id: Some("i1".to_owned()),
            run_id: Some("r1".to_owned()),
            gateway_id: None,
            gateway_token_id: None,
            gateway_public_id: None,
            client_subject_type: None,
            client_subject_id: None,
            client_name: None,
            mcp_session_id: None,
            correlation_id: Some("corr-1".to_owned()),
            application_id: None,
            connection_id: None,
            catalog_entry_id: None,
            catalog_version_hash: None,
            catalog_schema_hash: None,
            provider_type: None,
            application_key: None,
            upstream_tool_name: None,
            risk_level: Some("write".to_owned()),
            tool_name: "create_file".to_owned(),
            arguments_hash: Some("abc123".to_owned()),
            arguments_summary: Some(serde_json::json!({ "path": "/tmp/x" })),
            policy_decision: Some("allow".to_owned()),
            matched_policy_ids: serde_json::json!([]),
            policy_explanation: None,
            credential_scope_summary: None,
            header_policy_summary: None,
            approval_state: "not_required".to_owned(),
            status: "succeeded".to_owned(),
            upstream_request_id: None,
            result_hash: Some("def456".to_owned()),
            result_summary: Some(serde_json::json!({ "ok": true })),
            result_size_bytes: Some(12),
            result_artifact_id: None,
            error_code: None,
            error_message: None,
            started_at: Some("2026-08-04T00:00:00.000Z".to_owned()),
            completed_at: Some("2026-08-04T00:00:01.000Z".to_owned()),
        }
    }

    #[tokio::test]
    async fn catalog_application_profile_lifecycle() {
        let (_dir, catalog, _, _) = repos().await;
        let app = catalog.create_application(app_input()).await.unwrap();
        assert_eq!(app.name, "App 1");
        assert_eq!(catalog.list_applications("c1").await.unwrap().len(), 1);
        assert_eq!(
            catalog
                .get_application("c1", &app.id)
                .await
                .unwrap()
                .unwrap()
                .name,
            "App 1"
        );
        assert!(
            catalog
                .get_application("c2", &app.id)
                .await
                .unwrap()
                .is_none()
        );

        // Duplicate application name rejected.
        assert!(matches!(
            catalog.create_application(app_input()).await.unwrap_err(),
            ToolchainError::AlreadyExists
        ));

        let profile = catalog.create_profile(profile_input()).await.unwrap();
        assert_eq!(profile.default_action, "deny");
        assert!(matches!(
            catalog.create_profile(profile_input()).await.unwrap_err(),
            ToolchainError::AlreadyExists
        ));
        let updated = catalog
            .update_profile(UpdateToolProfile {
                company_id: "c1".to_owned(),
                id: profile.id.clone(),
                name: Some("Renamed".to_owned()),
                description: None,
                status: Some("disabled".to_owned()),
                default_action: Some("allow".to_owned()),
                new_tools_reviewed_at: None,
                metadata: None,
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.status, "disabled");
        assert_eq!(catalog.list_profiles("c1").await.unwrap().len(), 1);
        assert!(catalog.delete_profile("c1", &profile.id).await.unwrap());
        assert!(!catalog.delete_profile("c1", &profile.id).await.unwrap());

        // Profile entries and bindings.
        let profile = catalog.create_profile(profile_input()).await.unwrap();
        let entry = catalog
            .create_profile_entry(NewToolProfileEntry {
                company_id: "c1".to_owned(),
                profile_id: profile.id.clone(),
                selector_type: "tool_name".to_owned(),
                effect: "include".to_owned(),
                application_id: Some(app.id.clone()),
                connection_id: None,
                catalog_entry_id: None,
                tool_name: Some("create_file".to_owned()),
                risk_level: Some("write".to_owned()),
                conditions: None,
            })
            .await
            .unwrap();
        assert_eq!(entry.tool_name.as_deref(), Some("create_file"));
        assert_eq!(
            catalog
                .list_profile_entries("c1", Some(&profile.id))
                .await
                .unwrap()
                .len(),
            1
        );
        let binding = catalog
            .create_profile_binding(NewToolProfileBinding {
                company_id: "c1".to_owned(),
                profile_id: profile.id.clone(),
                target_type: "company".to_owned(),
                target_id: "c1".to_owned(),
                priority: 100,
                metadata: serde_json::json!({}),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(binding.priority, 100);
        assert!(matches!(
            catalog
                .create_profile_binding(NewToolProfileBinding {
                    company_id: "c1".to_owned(),
                    profile_id: profile.id.clone(),
                    target_type: "company".to_owned(),
                    target_id: "c1".to_owned(),
                    priority: 200,
                    metadata: serde_json::json!({}),
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));
    }

    #[tokio::test]
    async fn catalog_cross_company_rejection() {
        let (_dir, catalog, _, _) = repos().await;
        let app = catalog.create_application(app_input()).await.unwrap();

        // Cross-company application owner agent rejected.
        assert!(matches!(
            catalog
                .create_application(NewToolApplication {
                    company_id: "c2".to_owned(),
                    application_key: None,
                    name: "Other".to_owned(),
                    description: None,
                    r#type: "internal".to_owned(),
                    status: "active".to_owned(),
                    plugin_id: None,
                    owner_agent_id: Some("a1".to_owned()),
                    owner_user_id: None,
                    metadata: serde_json::json!({}),
                })
                .await
                .unwrap_err(),
            ToolchainError::ReferenceNotFound
        ));
        assert_eq!(app.company_id, "c1");
        assert!(catalog.list_applications("c2").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn connection_grant_install_lifecycle() {
        let (_dir, catalog, connections, _) = repos().await;
        let app = catalog.create_application(app_input()).await.unwrap();
        let conn = connections
            .create_connection(connection_input(&app.id))
            .await
            .unwrap();
        assert_eq!(conn.transport, "mcp_remote");
        assert!(!conn.enabled);
        assert_eq!(connections.list_connections("c1").await.unwrap().len(), 1);
        assert!(
            connections
                .get_connection("c2", &conn.id)
                .await
                .unwrap()
                .is_none()
        );

        let updated = connections
            .update_connection(UpdateToolConnection {
                company_id: "c1".to_owned(),
                id: conn.id.clone(),
                name: None,
                status: Some("active".to_owned()),
                enabled: Some(true),
                config: Some(serde_json::json!({ "baseUrl": "https://example.com" })),
                transport_config: None,
                health_status: Some("healthy".to_owned()),
                health_message: Some("ok".to_owned()),
                health_checked_at: None,
                last_error: None,
            })
            .await
            .unwrap()
            .unwrap();
        assert!(updated.enabled);
        assert_eq!(updated.health_status, "healthy");

        // Grants: one default workspace grant per connection.
        let grant = connections
            .create_grant(NewConnectionGrant {
                company_id: "c1".to_owned(),
                connection_id: conn.id.clone(),
                kind: "workspace".to_owned(),
                subject_user_id: None,
                provider_tenant: None,
                credential_secret_refs: serde_json::json!([]),
                status: "active".to_owned(),
                is_default: true,
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert!(grant.is_default);
        assert!(matches!(
            connections
                .create_grant(NewConnectionGrant {
                    company_id: "c1".to_owned(),
                    connection_id: conn.id.clone(),
                    kind: "workspace".to_owned(),
                    subject_user_id: None,
                    provider_tenant: None,
                    credential_secret_refs: serde_json::json!([]),
                    status: "active".to_owned(),
                    is_default: true,
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));
        let revoked = connections
            .revoke_grant("c1", &grant.id, Some("u1"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(revoked.status, "revoked");

        // Installs: unique (connection, target).
        let install = connections
            .create_install(NewToolConnectionInstall {
                company_id: "c1".to_owned(),
                connection_id: conn.id.clone(),
                target_type: "agent".to_owned(),
                target_id: "a1".to_owned(),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(install.target_type, "agent");
        assert!(matches!(
            connections
                .create_install(NewToolConnectionInstall {
                    company_id: "c1".to_owned(),
                    connection_id: conn.id.clone(),
                    target_type: "agent".to_owned(),
                    target_id: "a1".to_owned(),
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));

        // OAuth state.
        let state = connections
            .create_oauth_state(NewToolOauthState {
                state: "state-1".to_owned(),
                company_id: "c1".to_owned(),
                connection_id: conn.id.clone(),
                code_verifier: "verifier".to_owned(),
                created_by_actor_type: Some("user".to_owned()),
                created_by_actor_id: Some("u1".to_owned()),
                created_by_session_id: None,
                subject_user_id: None,
                requested_scopes: Some(serde_json::json!(["tools:read"])),
                return_to: Some("/tools".to_owned()),
                issue_id: None,
                interaction_id: None,
                expires_at: "2026-12-31T00:00:00.000Z".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(state.state, "state-1");

        // Token issuance.
        let issuance = connections
            .create_token_issuance(NewConnectionTokenIssuance {
                company_id: "c1".to_owned(),
                application_id: Some(app.id.clone()),
                connection_id: conn.id.clone(),
                agent_id: "a1".to_owned(),
                run_id: Some("r1".to_owned()),
                issue_id: Some("i1".to_owned()),
                project_id: Some("pr1".to_owned()),
                responsible_user_id: Some("u1".to_owned()),
                path: "exchange".to_owned(),
                requested_scope: serde_json::json!(["read"]),
                issued_scope: serde_json::json!(["read"]),
                ttl_seconds: Some(300),
                expires_at: Some("2026-08-04T00:05:00.000Z".to_owned()),
                token_hash: Some("a".repeat(64)),
                outcome: "success".to_owned(),
                error_code: None,
                metadata: serde_json::json!({}),
            })
            .await
            .unwrap();
        assert_eq!(issuance.path, "exchange");
        assert_eq!(
            connections
                .list_token_issuances("c1", Some(&conn.id))
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn connection_dedupe_and_cross_company() {
        let (_dir, catalog, connections, _) = repos().await;
        let app = catalog.create_application(app_input()).await.unwrap();
        let conn = connections
            .create_connection(connection_input(&app.id))
            .await
            .unwrap();
        // Duplicate name rejected.
        assert!(matches!(
            connections
                .create_connection(NewToolConnection {
                    company_id: "c1".to_owned(),
                    application_id: app.id.clone(),
                    name: "Conn 1".to_owned(),
                    uid: "other-uid".to_owned(),
                    connection_kind: "managed".to_owned(),
                    ownership: "customer".to_owned(),
                    transport: "rest_api".to_owned(),
                    auth_kind: "none".to_owned(),
                    status: "draft".to_owned(),
                    enabled: false,
                    config: serde_json::json!({}),
                    transport_config: serde_json::json!({}),
                    credential_refs: serde_json::json!([]),
                    credential_secret_refs: serde_json::json!([]),
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));
        // Cross-company connection (application belongs to c1) rejected.
        assert!(matches!(
            connections
                .create_connection(NewToolConnection {
                    company_id: "c2".to_owned(),
                    application_id: app.id.clone(),
                    name: "Other".to_owned(),
                    uid: "other".to_owned(),
                    connection_kind: "managed".to_owned(),
                    ownership: "customer".to_owned(),
                    transport: "rest_api".to_owned(),
                    auth_kind: "none".to_owned(),
                    status: "draft".to_owned(),
                    enabled: false,
                    config: serde_json::json!({}),
                    transport_config: serde_json::json!({}),
                    credential_refs: serde_json::json!([]),
                    credential_secret_refs: serde_json::json!([]),
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::ReferenceNotFound
        ));
        // Cross-company grant (connection belongs to c1) rejected.
        assert!(matches!(
            connections
                .create_grant(NewConnectionGrant {
                    company_id: "c2".to_owned(),
                    connection_id: conn.id.clone(),
                    kind: "workspace".to_owned(),
                    subject_user_id: None,
                    provider_tenant: None,
                    credential_secret_refs: serde_json::json!([]),
                    status: "active".to_owned(),
                    is_default: false,
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::ReferenceNotFound
        ));
    }

    #[tokio::test]
    async fn gateway_token_invocation_lifecycle() {
        let (_dir, catalog, _, gateway) = repos().await;
        let profile = catalog.create_profile(profile_input()).await.unwrap();
        let gw = gateway
            .create_gateway(NewToolMcpGateway {
                company_id: "c1".to_owned(),
                name: "Gateway 1".to_owned(),
                slug: "gateway-1".to_owned(),
                display_slug: "Gateway 1".to_owned(),
                description: None,
                status: "active".to_owned(),
                profile_id: profile.id.clone(),
                default_profile_mode: "gateway_only".to_owned(),
                context_scope_type: "none".to_owned(),
                context_scope_id: None,
                agent_id: Some("a1".to_owned()),
                project_id: Some("pr1".to_owned()),
                issue_id: Some("i1".to_owned()),
                approval_issue_id: None,
                auth_config: serde_json::json!({ "version": 1 }),
                header_policy: serde_json::json!({ "version": 1 }),
                metadata_policy: serde_json::json!({ "version": 1 }),
                on_demand_tools_config: serde_json::json!({ "enabled": false }),
                metadata: serde_json::json!({}),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert!(gw.gateway_public_id.starts_with("gw_"));
        assert_eq!(gateway.list_gateways("c1").await.unwrap().len(), 1);

        let token = gateway
            .create_gateway_token(NewToolMcpGatewayToken {
                company_id: "c1".to_owned(),
                gateway_id: gw.id.clone(),
                name: "token-1".to_owned(),
                token_hash: "hash-1".to_owned(),
                token_prefix: "pcgw".to_owned(),
                subject_type: "gateway_client".to_owned(),
                subject_id: None,
                client_label: "client".to_owned(),
                owner_note: "".to_owned(),
                allowed_actions: serde_json::json!(["tools/list", "tools/call"]),
                expires_at: Some("2026-12-31T00:00:00.000Z".to_owned()),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(token.token_prefix, "pcgw");
        assert!(matches!(
            gateway
                .create_gateway_token(NewToolMcpGatewayToken {
                    company_id: "c1".to_owned(),
                    gateway_id: gw.id.clone(),
                    name: "token-2".to_owned(),
                    token_hash: "hash-1".to_owned(),
                    token_prefix: "pcgw".to_owned(),
                    subject_type: "gateway_client".to_owned(),
                    subject_id: None,
                    client_label: "client".to_owned(),
                    owner_note: "".to_owned(),
                    allowed_actions: serde_json::json!([]),
                    expires_at: None,
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));

        let invocation = gateway.create_invocation(invocation_input()).await.unwrap();
        assert_eq!(invocation.status, "succeeded");
        assert_eq!(
            invocation.result_summary.as_ref().unwrap()["ok"],
            serde_json::json!(true)
        );
        // Duplicate idempotency key rejected.
        assert!(matches!(
            gateway
                .create_invocation(invocation_input())
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));
        assert_eq!(gateway.list_invocations("c1").await.unwrap().len(), 1);

        // Action request + call event + audit event.
        let action = gateway
            .create_action_request(NewToolActionRequest {
                company_id: "c1".to_owned(),
                invocation_id: invocation.id.clone(),
                issue_id: Some("i1".to_owned()),
                interaction_id: None,
                approval_id: None,
                status: "pending".to_owned(),
                canonical_arguments_hash: "hash".to_owned(),
                canonical_arguments_summary: serde_json::json!({ "path": "/tmp/x" }),
                signed_arguments: None,
                preview_markdown: Some("Create /tmp/x".to_owned()),
                requested_by_agent_id: Some("a1".to_owned()),
                requested_by_user_id: None,
                expires_at: Some("2026-08-04T00:10:00.000Z".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(action.status, "pending");
        let call = gateway
            .create_call_event(NewToolCallEvent {
                company_id: "c1".to_owned(),
                event_type: "call_started".to_owned(),
                actor_type: "agent".to_owned(),
                actor_id: Some("a1".to_owned()),
                agent_id: Some("a1".to_owned()),
                run_id: Some("r1".to_owned()),
                issue_id: Some("i1".to_owned()),
                gateway_id: Some(gw.id.clone()),
                gateway_token_id: Some(token.id.clone()),
                gateway_public_id: Some(gw.gateway_public_id.clone()),
                client_subject_type: None,
                client_subject_id: None,
                client_name: Some("client".to_owned()),
                mcp_session_id: None,
                correlation_id: Some("corr-1".to_owned()),
                application_id: None,
                connection_id: None,
                catalog_entry_id: None,
                invocation_id: Some(invocation.id.clone()),
                action_request_id: Some(action.id.clone()),
                runtime_slot_id: None,
                tool_name: Some("create_file".to_owned()),
                decision: Some("allow".to_owned()),
                matched_policy_ids: serde_json::json!([]),
                reason_code: None,
                policy_explanation: None,
                credential_scope_summary: None,
                header_policy_summary: None,
                outcome: "succeeded".to_owned(),
                latency_ms: Some(5),
                arguments_summary: None,
                request_hash: None,
                request_summary: None,
                result_hash: None,
                result_summary: Some(serde_json::json!({ "ok": true })),
                result_size_bytes: Some(2),
                redaction_plan: None,
                rate_limit_state: None,
                metadata: None,
                error_code: None,
                error_message: None,
            })
            .await
            .unwrap();
        assert_eq!(call.event_type, "call_started");
        let audit = gateway
            .create_audit_event(NewToolAccessAuditEvent {
                company_id: "c1".to_owned(),
                gateway_id: Some(gw.id.clone()),
                gateway_token_id: Some(token.id.clone()),
                gateway_public_id: Some(gw.gateway_public_id.clone()),
                client_name: Some("client".to_owned()),
                correlation_id: Some("corr-1".to_owned()),
                connection_id: None,
                catalog_entry_id: None,
                actor_type: "system".to_owned(),
                actor_id: None,
                action: "tools/call".to_owned(),
                outcome: "success".to_owned(),
                reason_code: None,
                details: serde_json::json!({}),
            })
            .await
            .unwrap();
        assert_eq!(audit.action, "tools/call");
        assert_eq!(
            gateway
                .list_call_events("c1", Some(&invocation.id))
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            gateway
                .list_action_requests("c1", Some(&invocation.id))
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn gateway_dedupe_and_cross_company() {
        let (_dir, catalog, _, gateway) = repos().await;
        let profile = catalog.create_profile(profile_input()).await.unwrap();
        let _ = gateway
            .create_gateway(NewToolMcpGateway {
                company_id: "c1".to_owned(),
                name: "Gateway 1".to_owned(),
                slug: "gateway-1".to_owned(),
                display_slug: "Gateway 1".to_owned(),
                description: None,
                status: "active".to_owned(),
                profile_id: profile.id.clone(),
                default_profile_mode: "gateway_only".to_owned(),
                context_scope_type: "none".to_owned(),
                context_scope_id: None,
                agent_id: None,
                project_id: None,
                issue_id: None,
                approval_issue_id: None,
                auth_config: serde_json::json!({ "version": 1 }),
                header_policy: serde_json::json!({ "version": 1 }),
                metadata_policy: serde_json::json!({ "version": 1 }),
                on_demand_tools_config: serde_json::json!({ "enabled": false }),
                metadata: serde_json::json!({}),
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        // Duplicate slug rejected.
        assert!(matches!(
            gateway
                .create_gateway(NewToolMcpGateway {
                    company_id: "c1".to_owned(),
                    name: "Other".to_owned(),
                    slug: "gateway-1".to_owned(),
                    display_slug: "Other".to_owned(),
                    description: None,
                    status: "active".to_owned(),
                    profile_id: profile.id.clone(),
                    default_profile_mode: "gateway_only".to_owned(),
                    context_scope_type: "none".to_owned(),
                    context_scope_id: None,
                    agent_id: None,
                    project_id: None,
                    issue_id: None,
                    approval_issue_id: None,
                    auth_config: serde_json::json!({ "version": 1 }),
                    header_policy: serde_json::json!({ "version": 1 }),
                    metadata_policy: serde_json::json!({ "version": 1 }),
                    on_demand_tools_config: serde_json::json!({ "enabled": false }),
                    metadata: serde_json::json!({}),
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));
        // Cross-company gateway (profile belongs to c1) rejected.
        assert!(matches!(
            gateway
                .create_gateway(NewToolMcpGateway {
                    company_id: "c2".to_owned(),
                    name: "Other".to_owned(),
                    slug: "gateway-2".to_owned(),
                    display_slug: "Other".to_owned(),
                    description: None,
                    status: "active".to_owned(),
                    profile_id: profile.id.clone(),
                    default_profile_mode: "gateway_only".to_owned(),
                    context_scope_type: "none".to_owned(),
                    context_scope_id: None,
                    agent_id: None,
                    project_id: None,
                    issue_id: None,
                    approval_issue_id: None,
                    auth_config: serde_json::json!({ "version": 1 }),
                    header_policy: serde_json::json!({ "version": 1 }),
                    metadata_policy: serde_json::json!({ "version": 1 }),
                    on_demand_tools_config: serde_json::json!({ "enabled": false }),
                    metadata: serde_json::json!({}),
                    created_by_agent_id: None,
                    created_by_user_id: None,
                })
                .await
                .unwrap_err(),
            ToolchainError::ReferenceNotFound
        ));
    }

    #[tokio::test]
    async fn gateway_sessions_runtime_and_counters() {
        let (_dir, catalog, _, gateway) = repos().await;
        let profile = catalog.create_profile(profile_input()).await.unwrap();
        let gw = gateway
            .create_gateway(NewToolMcpGateway {
                company_id: "c1".to_owned(),
                name: "Gateway 1".to_owned(),
                slug: "gateway-1".to_owned(),
                display_slug: "Gateway 1".to_owned(),
                description: None,
                status: "active".to_owned(),
                profile_id: profile.id.clone(),
                default_profile_mode: "gateway_only".to_owned(),
                context_scope_type: "none".to_owned(),
                context_scope_id: None,
                agent_id: Some("a1".to_owned()),
                project_id: None,
                issue_id: None,
                approval_issue_id: None,
                auth_config: serde_json::json!({ "version": 1 }),
                header_policy: serde_json::json!({ "version": 1 }),
                metadata_policy: serde_json::json!({ "version": 1 }),
                on_demand_tools_config: serde_json::json!({ "enabled": false }),
                metadata: serde_json::json!({}),
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();

        let session = gateway
            .create_gateway_session(NewToolGatewaySession {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                run_id: "r1".to_owned(),
                issue_id: Some("i1".to_owned()),
                project_id: Some("pr1".to_owned()),
                gateway_id: Some(gw.id.clone()),
                gateway_token_id: None,
                gateway_public_id: Some(gw.gateway_public_id.clone()),
                client_subject_type: None,
                client_subject_id: None,
                client_name: Some("client".to_owned()),
                mcp_session_id: Some("mcp-1".to_owned()),
                correlation_id: Some("corr-1".to_owned()),
                token_hash: "session-hash".to_owned(),
                expires_at: "2026-08-04T00:30:00.000Z".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(session.mcp_session_id.as_deref(), Some("mcp-1"));
        assert!(matches!(
            gateway
                .create_gateway_session(NewToolGatewaySession {
                    company_id: "c1".to_owned(),
                    agent_id: "a1".to_owned(),
                    run_id: "r1".to_owned(),
                    issue_id: None,
                    project_id: None,
                    gateway_id: None,
                    gateway_token_id: None,
                    gateway_public_id: None,
                    client_subject_type: None,
                    client_subject_id: None,
                    client_name: None,
                    mcp_session_id: None,
                    correlation_id: None,
                    token_hash: "session-hash".to_owned(),
                    expires_at: "2026-08-04T00:30:00.000Z".to_owned(),
                })
                .await
                .unwrap_err(),
            ToolchainError::AlreadyExists
        ));

        let slot = gateway
            .create_runtime_slot(NewToolRuntimeSlot {
                company_id: "c1".to_owned(),
                application_id: None,
                connection_id: None,
                project_workspace_id: None,
                execution_workspace_id: None,
                issue_id: None,
                owner_scope_type: "connection".to_owned(),
                owner_scope_id: None,
                runtime_kind: "local_stdio".to_owned(),
                slot_key: "slot-1".to_owned(),
                status: "stopped".to_owned(),
                reuse_key: None,
                workspace_scope: None,
                credential_scope_hash: None,
                provider: None,
                provider_ref: None,
                process_id: None,
                command_template_key: None,
                health_status: "unchecked".to_owned(),
                health_message: None,
                metadata: serde_json::json!({}),
            })
            .await
            .unwrap();
        assert_eq!(slot.slot_key, "slot-1");

        let template = gateway
            .create_stdio_template(NewToolStdioCommandTemplate {
                company_id: "c1".to_owned(),
                template_key: "tpl-1".to_owned(),
                name: "Template 1".to_owned(),
                description: None,
                status: "active".to_owned(),
                command: "node server.js".to_owned(),
                args: serde_json::json!([]),
                env_keys: serde_json::json!([]),
                tools: serde_json::json!([]),
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(template.command, "node server.js");

        let policy = gateway
            .create_policy(NewToolPolicy {
                company_id: "c1".to_owned(),
                name: "Policy 1".to_owned(),
                description: None,
                policy_type: "allow".to_owned(),
                priority: 100,
                enabled: true,
                selectors: serde_json::json!({}),
                conditions: None,
                config: None,
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(policy.policy_type, "allow");

        // Counter upserts.
        let counter = gateway
            .upsert_gateway_rate_limit_counter(NewToolGatewayRateLimitCounter {
                company_id: "c1".to_owned(),
                counter_key: "gw:default".to_owned(),
                window_start_at: "2026-08-04T00:00:00.000Z".to_owned(),
                window_ms: 60_000,
                limit: 100,
                count: 1,
                reset_at: "2026-08-04T00:01:00.000Z".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(counter.count, 1);
        let counter = gateway
            .upsert_gateway_rate_limit_counter(NewToolGatewayRateLimitCounter {
                company_id: "c1".to_owned(),
                counter_key: "gw:default".to_owned(),
                window_start_at: "2026-08-04T00:00:00.000Z".to_owned(),
                window_ms: 60_000,
                limit: 100,
                count: 5,
                reset_at: "2026-08-04T00:01:00.000Z".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(counter.count, 5);

        let rate = gateway
            .upsert_rate_limit_counter(NewToolRateLimitCounter {
                company_id: "c1".to_owned(),
                policy_id: policy.id.clone(),
                counter_key: "user:u1".to_owned(),
                scope_type: "user".to_owned(),
                scope_id: "u1".to_owned(),
                window_kind: "minute".to_owned(),
                window_start_at: "2026-08-04T00:00:00.000Z".to_owned(),
                limit: 10,
                remaining: 9,
                reset_at: "2026-08-04T00:01:00.000Z".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(rate.remaining, 9);

        let metric = gateway
            .upsert_runtime_metric_counter(NewToolRuntimeMetricCounter {
                company_id: "c1".to_owned(),
                metric: "invocations".to_owned(),
                bucket_start_at: "2026-08-04T00:00:00.000Z".to_owned(),
                count: 3,
            })
            .await
            .unwrap();
        assert_eq!(metric.count, 3);
        let metric = gateway
            .upsert_runtime_metric_counter(NewToolRuntimeMetricCounter {
                company_id: "c1".to_owned(),
                metric: "invocations".to_owned(),
                bucket_start_at: "2026-08-04T00:00:00.000Z".to_owned(),
                count: 2,
            })
            .await
            .unwrap();
        assert_eq!(metric.count, 5);
    }
}
