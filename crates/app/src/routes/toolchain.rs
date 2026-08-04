//! Toolchain routes: tool applications/catalog/profiles, connections, and
//! MCP gateway/invocation surfaces (upstream tool_access.ts domain).

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewConnectionGrant, NewConnectionTokenIssuance, NewToolAccessAuditEvent, NewToolActionRequest,
    NewToolApplication, NewToolCallEvent, NewToolCatalogEntry, NewToolConnection,
    NewToolConnectionInstall, NewToolGatewaySession, NewToolInvocation, NewToolMcpGateway,
    NewToolMcpGatewayToken, NewToolPolicy, NewToolProfile, NewToolProfileBinding,
    NewToolProfileEntry, NewToolRuntimeSlot, NewToolStdioCommandTemplate, ToolchainError,
    UpdateToolConnection, UpdateToolProfile,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, query_params, route},
};

use crate::{
    error::ApiError,
    routes::{CompanyId, Id},
    state::AppState,
};

fn default_object() -> serde_json::Value {
    serde_json::json!({})
}

fn default_array() -> serde_json::Value {
    serde_json::json!([])
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_priority() -> i64 {
    100
}

fn default_active() -> String {
    "active".to_owned()
}

fn default_draft() -> String {
    "draft".to_owned()
}

fn default_deny() -> String {
    "deny".to_owned()
}

fn default_system() -> String {
    "system".to_owned()
}

fn default_pending() -> String {
    "pending".to_owned()
}

fn default_not_required() -> String {
    "not_required".to_owned()
}

fn default_managed() -> String {
    "managed".to_owned()
}

fn default_customer() -> String {
    "customer".to_owned()
}

fn default_none_auth() -> String {
    "none".to_owned()
}

fn default_unchecked() -> String {
    "unchecked".to_owned()
}

fn default_connection_scope() -> String {
    "connection".to_owned()
}

fn default_local_stdio() -> String {
    "local_stdio".to_owned()
}

fn default_stopped() -> String {
    "stopped".to_owned()
}

fn default_tool() -> String {
    "tool".to_owned()
}

fn default_read() -> String {
    "read".to_owned()
}

fn default_gateway_only() -> String {
    "gateway_only".to_owned()
}

fn default_context_none() -> String {
    "none".to_owned()
}

fn default_gateway_client() -> String {
    "gateway_client".to_owned()
}

fn default_exchange() -> String {
    "exchange".to_owned()
}

fn default_success() -> String {
    "success".to_owned()
}

fn default_include() -> String {
    "include".to_owned()
}

fn toolchain_error_to_api(error: ToolchainError) -> ApiError {
    use ToolchainError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ReferenceNotFound => ApiError::not_found("Referenced record not found or out of scope"),
        E::AlreadyExists => ApiError::conflict("Record already exists"),
        E::InvalidInput => ApiError::bad_request("Invalid input"),
        E::NotFound => ApiError::not_found("Record not found"),
        E::Db(message) => ApiError::internal(message.to_string()),
        E::Data(message) => ApiError::internal(message.to_string()),
    }
}

/// Body for `POST /api/companies/{companyId}/tools/applications`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationRequest {
    #[serde(default)]
    pub application_key: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub r#type: String,
    #[serde(default = "default_active")]
    pub status: String,
    #[serde(default)]
    pub plugin_id: Option<String>,
    #[serde(default)]
    pub owner_agent_id: Option<String>,
    #[serde(default)]
    pub owner_user_id: Option<String>,
    #[serde(default = "default_object")]
    pub metadata: serde_json::Value,
}

/// Body for `POST /api/companies/{companyId}/connections`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub application_id: String,
    pub name: String,
    pub uid: String,
    #[serde(default = "default_managed")]
    pub connection_kind: String,
    #[serde(default = "default_customer")]
    pub ownership: String,
    pub transport: String,
    #[serde(default = "default_none_auth")]
    pub auth_kind: String,
    #[serde(default = "default_draft")]
    pub status: String,
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_object")]
    pub config: serde_json::Value,
    #[serde(default = "default_object")]
    pub transport_config: serde_json::Value,
    #[serde(default = "default_array")]
    pub credential_refs: serde_json::Value,
    #[serde(default = "default_array")]
    pub credential_secret_refs: serde_json::Value,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `PATCH /api/companies/{companyId}/connections/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnectionRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub transport_config: Option<serde_json::Value>,
    #[serde(default)]
    pub health_status: Option<String>,
    #[serde(default)]
    pub health_message: Option<String>,
    #[serde(default)]
    pub health_checked_at: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

/// Body for `POST .../connections/{id}/grants`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGrantRequest {
    pub kind: String,
    #[serde(default)]
    pub subject_user_id: Option<String>,
    #[serde(default)]
    pub provider_tenant: Option<serde_json::Value>,
    #[serde(default = "default_array")]
    pub credential_secret_refs: serde_json::Value,
    #[serde(default = "default_active")]
    pub status: String,
    #[serde(default = "default_false")]
    pub is_default: bool,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `POST .../connections/{id}/installs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstallRequest {
    pub target_type: String,
    pub target_id: String,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `POST .../connections/{id}/token-issuances`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenIssuanceRequest {
    #[serde(default)]
    pub application_id: Option<String>,
    pub agent_id: String,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub responsible_user_id: Option<String>,
    #[serde(default = "default_exchange")]
    pub path: String,
    #[serde(default = "default_array")]
    pub requested_scope: serde_json::Value,
    #[serde(default = "default_array")]
    pub issued_scope: serde_json::Value,
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub token_hash: Option<String>,
    #[serde(default = "default_success")]
    pub outcome: String,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default = "default_object")]
    pub metadata: serde_json::Value,
}

/// Body for `POST .../tools/catalog-entries`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCatalogEntryRequest {
    #[serde(default)]
    pub application_id: Option<String>,
    pub connection_id: String,
    #[serde(default = "default_tool")]
    pub entry_kind: String,
    pub name: String,
    pub tool_name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_object")]
    pub input_schema: serde_json::Value,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default = "default_object")]
    pub annotations: serde_json::Value,
    #[serde(default = "default_read")]
    pub risk_level: String,
    #[serde(default = "default_true")]
    pub is_read_only: bool,
    #[serde(default = "default_false")]
    pub is_write: bool,
    #[serde(default = "default_false")]
    pub is_destructive: bool,
    #[serde(default = "default_active")]
    pub status: String,
    #[serde(default)]
    pub version: Option<String>,
    pub version_hash: String,
    #[serde(default)]
    pub schema_hash: Option<String>,
    #[serde(default)]
    pub reviewed_by_agent_id: Option<String>,
    #[serde(default)]
    pub reviewed_by_user_id: Option<String>,
}

/// Body for `POST .../tools/profiles`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileRequest {
    pub profile_key: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_active")]
    pub status: String,
    #[serde(default = "default_deny")]
    pub default_action: String,
    #[serde(default)]
    pub new_tools_reviewed_at: Option<String>,
    #[serde(default = "default_object")]
    pub metadata: serde_json::Value,
}

/// Body for `PATCH .../tools/profiles/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub default_action: Option<String>,
    #[serde(default)]
    pub new_tools_reviewed_at: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Body for `POST .../tools/profiles/{id}/entries`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileEntryRequest {
    pub selector_type: String,
    #[serde(default = "default_include")]
    pub effect: String,
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub catalog_entry_id: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub risk_level: Option<String>,
    #[serde(default)]
    pub conditions: Option<serde_json::Value>,
}

/// Body for `POST .../tools/profiles/{id}/bindings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileBindingRequest {
    pub target_type: String,
    pub target_id: String,
    #[serde(default = "default_priority")]
    pub priority: i64,
    #[serde(default = "default_object")]
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `POST .../tools/gateways`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGatewayRequest {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub display_slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_active")]
    pub status: String,
    pub profile_id: String,
    #[serde(default = "default_gateway_only")]
    pub default_profile_mode: String,
    #[serde(default = "default_context_none")]
    pub context_scope_type: String,
    #[serde(default)]
    pub context_scope_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub approval_issue_id: Option<String>,
    #[serde(default = "default_object")]
    pub auth_config: serde_json::Value,
    #[serde(default = "default_object")]
    pub header_policy: serde_json::Value,
    #[serde(default = "default_object")]
    pub metadata_policy: serde_json::Value,
    #[serde(default = "default_object")]
    pub on_demand_tools_config: serde_json::Value,
    #[serde(default = "default_object")]
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `POST .../tools/gateways/{id}/tokens`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGatewayTokenRequest {
    pub name: String,
    pub token_hash: String,
    #[serde(default)]
    pub token_prefix: String,
    #[serde(default = "default_gateway_client")]
    pub subject_type: String,
    #[serde(default)]
    pub subject_id: Option<String>,
    #[serde(default)]
    pub client_label: String,
    #[serde(default)]
    pub owner_note: String,
    #[serde(default = "default_array")]
    pub allowed_actions: serde_json::Value,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `POST .../tools/gateways/{id}/sessions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGatewaySessionRequest {
    pub agent_id: String,
    pub run_id: String,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub gateway_token_id: Option<String>,
    #[serde(default)]
    pub gateway_public_id: Option<String>,
    #[serde(default)]
    pub client_subject_type: Option<String>,
    #[serde(default)]
    pub client_subject_id: Option<String>,
    #[serde(default)]
    pub client_name: Option<String>,
    #[serde(default)]
    pub mcp_session_id: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    pub token_hash: String,
    pub expires_at: String,
}

/// Body for `POST .../tools/invocations`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvocationRequest {
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default = "default_system")]
    pub actor_type: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub gateway_id: Option<String>,
    #[serde(default)]
    pub gateway_token_id: Option<String>,
    #[serde(default)]
    pub gateway_public_id: Option<String>,
    #[serde(default)]
    pub client_subject_type: Option<String>,
    #[serde(default)]
    pub client_subject_id: Option<String>,
    #[serde(default)]
    pub client_name: Option<String>,
    #[serde(default)]
    pub mcp_session_id: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub catalog_entry_id: Option<String>,
    #[serde(default)]
    pub catalog_version_hash: Option<String>,
    #[serde(default)]
    pub catalog_schema_hash: Option<String>,
    #[serde(default)]
    pub provider_type: Option<String>,
    #[serde(default)]
    pub application_key: Option<String>,
    #[serde(default)]
    pub upstream_tool_name: Option<String>,
    #[serde(default)]
    pub risk_level: Option<String>,
    pub tool_name: String,
    #[serde(default)]
    pub arguments_hash: Option<String>,
    #[serde(default)]
    pub arguments_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub policy_decision: Option<String>,
    #[serde(default = "default_array")]
    pub matched_policy_ids: serde_json::Value,
    #[serde(default)]
    pub policy_explanation: Option<serde_json::Value>,
    #[serde(default)]
    pub credential_scope_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub header_policy_summary: Option<serde_json::Value>,
    #[serde(default = "default_not_required")]
    pub approval_state: String,
    #[serde(default = "default_pending")]
    pub status: String,
    #[serde(default)]
    pub upstream_request_id: Option<String>,
    #[serde(default)]
    pub result_hash: Option<String>,
    #[serde(default)]
    pub result_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub result_size_bytes: Option<i64>,
    #[serde(default)]
    pub result_artifact_id: Option<String>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// Body for `POST .../tools/invocations/{id}/action-requests`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActionRequestRequest {
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub interaction_id: Option<String>,
    #[serde(default)]
    pub approval_id: Option<String>,
    #[serde(default = "default_pending")]
    pub status: String,
    pub canonical_arguments_hash: String,
    pub canonical_arguments_summary: serde_json::Value,
    #[serde(default)]
    pub signed_arguments: Option<String>,
    #[serde(default)]
    pub preview_markdown: Option<String>,
    #[serde(default)]
    pub requested_by_agent_id: Option<String>,
    #[serde(default)]
    pub requested_by_user_id: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Body for `POST .../tools/invocations/{id}/call-events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCallEventRequest {
    pub event_type: String,
    #[serde(default = "default_system")]
    pub actor_type: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub gateway_id: Option<String>,
    #[serde(default)]
    pub gateway_token_id: Option<String>,
    #[serde(default)]
    pub gateway_public_id: Option<String>,
    #[serde(default)]
    pub client_subject_type: Option<String>,
    #[serde(default)]
    pub client_subject_id: Option<String>,
    #[serde(default)]
    pub client_name: Option<String>,
    #[serde(default)]
    pub mcp_session_id: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub catalog_entry_id: Option<String>,
    #[serde(default)]
    pub invocation_id: Option<String>,
    #[serde(default)]
    pub action_request_id: Option<String>,
    #[serde(default)]
    pub runtime_slot_id: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default = "default_array")]
    pub matched_policy_ids: serde_json::Value,
    #[serde(default)]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub policy_explanation: Option<serde_json::Value>,
    #[serde(default)]
    pub credential_scope_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub header_policy_summary: Option<serde_json::Value>,
    #[serde(default = "default_pending")]
    pub outcome: String,
    #[serde(default)]
    pub latency_ms: Option<i64>,
    #[serde(default)]
    pub arguments_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub request_hash: Option<String>,
    #[serde(default)]
    pub request_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub result_hash: Option<String>,
    #[serde(default)]
    pub result_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub result_size_bytes: Option<i64>,
    #[serde(default)]
    pub redaction_plan: Option<serde_json::Value>,
    #[serde(default)]
    pub rate_limit_state: Option<serde_json::Value>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

/// Body for `POST .../tools/audit-events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAuditEventRequest {
    #[serde(default)]
    pub gateway_id: Option<String>,
    #[serde(default)]
    pub gateway_token_id: Option<String>,
    #[serde(default)]
    pub gateway_public_id: Option<String>,
    #[serde(default)]
    pub client_name: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub catalog_entry_id: Option<String>,
    #[serde(default = "default_system")]
    pub actor_type: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub action: String,
    pub outcome: String,
    #[serde(default)]
    pub reason_code: Option<String>,
    #[serde(default = "default_object")]
    pub details: serde_json::Value,
}

/// Body for `POST .../tools/policies`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePolicyRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub policy_type: String,
    #[serde(default = "default_priority")]
    pub priority: i64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_object")]
    pub selectors: serde_json::Value,
    #[serde(default)]
    pub conditions: Option<serde_json::Value>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `POST .../tools/runtime-slots`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuntimeSlotRequest {
    #[serde(default)]
    pub application_id: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub project_workspace_id: Option<String>,
    #[serde(default)]
    pub execution_workspace_id: Option<String>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default = "default_connection_scope")]
    pub owner_scope_type: String,
    #[serde(default)]
    pub owner_scope_id: Option<String>,
    #[serde(default = "default_local_stdio")]
    pub runtime_kind: String,
    pub slot_key: String,
    #[serde(default = "default_stopped")]
    pub status: String,
    #[serde(default)]
    pub reuse_key: Option<String>,
    #[serde(default)]
    pub workspace_scope: Option<String>,
    #[serde(default)]
    pub credential_scope_hash: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub provider_ref: Option<String>,
    #[serde(default)]
    pub process_id: Option<i64>,
    #[serde(default)]
    pub command_template_key: Option<String>,
    #[serde(default = "default_unchecked")]
    pub health_status: String,
    #[serde(default)]
    pub health_message: Option<String>,
    #[serde(default = "default_object")]
    pub metadata: serde_json::Value,
}

/// Body for `POST .../tools/stdio-templates`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStdioTemplateRequest {
    pub template_key: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_active")]
    pub status: String,
    pub command: String,
    #[serde(default = "default_array")]
    pub args: serde_json::Value,
    #[serde(default = "default_array")]
    pub env_keys: serde_json::Value,
    #[serde(default = "default_array")]
    pub tools: serde_json::Value,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

#[query_params]
struct ConnectionQuery {
    #[serde(rename = "connectionId")]
    connection_id: Option<String>,
}

#[query_params]
struct RevokeQuery {
    #[serde(rename = "revokedByUserId")]
    revoked_by_user_id: Option<String>,
}

/// `GET /api/companies/{companyId}/tools/applications`.
#[route(GET "/api/companies/{company_id}/tools/applications")]
pub async fn list_applications(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_catalog
        .list_applications(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/applications`.
#[route(POST "/api/companies/{company_id}/tools/applications")]
pub async fn create_application(
    cx: &Cx,
    Json(body): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_catalog
        .create_application(NewToolApplication {
            company_id,
            application_key: body.application_key,
            name: body.name,
            description: body.description,
            r#type: body.r#type,
            status: body.status,
            plugin_id: body.plugin_id,
            owner_agent_id: body.owner_agent_id,
            owner_user_id: body.owner_user_id,
            metadata: body.metadata,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/applications/{id}`.
#[route(GET "/api/companies/{company_id}/tools/applications/{id}")]
pub async fn get_application(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .tool_catalog
        .get_application(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Tool application not found")),
    }
}

/// `GET /api/companies/{companyId}/tools/catalog-entries`.
#[route(GET "/api/companies/{company_id}/tools/catalog-entries")]
pub async fn list_catalog_entries(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let connection_id = query_params::<ConnectionQuery>(cx)
        .ok()
        .and_then(|q| q.connection_id.clone());
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_catalog
        .list_catalog_entries(&company_id, connection_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/catalog-entries`.
#[route(POST "/api/companies/{company_id}/tools/catalog-entries")]
pub async fn create_catalog_entry(
    cx: &Cx,
    Json(body): Json<CreateCatalogEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_catalog
        .create_catalog_entry(NewToolCatalogEntry {
            company_id,
            application_id: body.application_id,
            connection_id: body.connection_id,
            entry_kind: body.entry_kind,
            name: body.name,
            tool_name: body.tool_name,
            title: body.title,
            description: body.description,
            input_schema: body.input_schema,
            output_schema: body.output_schema,
            annotations: body.annotations,
            risk_level: body.risk_level,
            is_read_only: body.is_read_only,
            is_write: body.is_write,
            is_destructive: body.is_destructive,
            status: body.status,
            version: body.version,
            version_hash: body.version_hash,
            schema_hash: body.schema_hash,
            reviewed_by_agent_id: body.reviewed_by_agent_id,
            reviewed_by_user_id: body.reviewed_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/profiles`.
#[route(GET "/api/companies/{company_id}/tools/profiles")]
pub async fn list_profiles(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_catalog
        .list_profiles(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/profiles`.
#[route(POST "/api/companies/{company_id}/tools/profiles")]
pub async fn create_profile(
    cx: &Cx,
    Json(body): Json<CreateProfileRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_catalog
        .create_profile(NewToolProfile {
            company_id,
            profile_key: body.profile_key,
            name: body.name,
            description: body.description,
            status: body.status,
            default_action: body.default_action,
            new_tools_reviewed_at: body.new_tools_reviewed_at,
            metadata: body.metadata,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/profiles/{id}`.
#[route(GET "/api/companies/{company_id}/tools/profiles/{id}")]
pub async fn get_profile(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .tool_catalog
        .get_profile(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Tool profile not found")),
    }
}

/// `PATCH /api/companies/{companyId}/tools/profiles/{id}`.
#[route(PATCH "/api/companies/{company_id}/tools/profiles/{id}")]
pub async fn update_profile(
    cx: &Cx,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .tool_catalog
        .update_profile(UpdateToolProfile {
            company_id,
            id,
            name: body.name,
            description: body.description,
            status: body.status,
            default_action: body.default_action,
            new_tools_reviewed_at: body.new_tools_reviewed_at,
            metadata: body.metadata,
        })
        .await
        .map_err(toolchain_error_to_api)?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Tool profile not found")),
    }
}

/// `DELETE /api/companies/{companyId}/tools/profiles/{id}`.
#[route(DELETE "/api/companies/{company_id}/tools/profiles/{id}")]
pub async fn delete_profile(cx: &Cx) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let deleted = state
        .tool_catalog
        .delete_profile(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    if deleted {
        Ok((StatusCode::NO_CONTENT, Json(json!({}))))
    } else {
        Err(ApiError::not_found("Tool profile not found"))
    }
}

/// `GET /api/companies/{companyId}/tools/profiles/{id}/entries`.
#[route(GET "/api/companies/{company_id}/tools/profiles/{id}/entries")]
pub async fn list_profile_entries(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_catalog
        .list_profile_entries(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/profiles/{id}/entries`.
#[route(POST "/api/companies/{company_id}/tools/profiles/{id}/entries")]
pub async fn create_profile_entry(
    cx: &Cx,
    Json(body): Json<CreateProfileEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_catalog
        .create_profile_entry(NewToolProfileEntry {
            company_id,
            profile_id: id,
            selector_type: body.selector_type,
            effect: body.effect,
            application_id: body.application_id,
            connection_id: body.connection_id,
            catalog_entry_id: body.catalog_entry_id,
            tool_name: body.tool_name,
            risk_level: body.risk_level,
            conditions: body.conditions,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/profiles/{id}/bindings`.
#[route(GET "/api/companies/{company_id}/tools/profiles/{id}/bindings")]
pub async fn list_profile_bindings(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_catalog
        .list_profile_bindings(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/profiles/{id}/bindings`.
#[route(POST "/api/companies/{company_id}/tools/profiles/{id}/bindings")]
pub async fn create_profile_binding(
    cx: &Cx,
    Json(body): Json<CreateProfileBindingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_catalog
        .create_profile_binding(NewToolProfileBinding {
            company_id,
            profile_id: id,
            target_type: body.target_type,
            target_id: body.target_id,
            priority: body.priority,
            metadata: body.metadata,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/connections`.
#[route(GET "/api/companies/{company_id}/connections")]
pub async fn list_connections(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_connections
        .list_connections(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/connections`.
#[route(POST "/api/companies/{company_id}/connections")]
pub async fn create_connection(
    cx: &Cx,
    Json(body): Json<CreateConnectionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_connections
        .create_connection(NewToolConnection {
            company_id,
            application_id: body.application_id,
            name: body.name,
            uid: body.uid,
            connection_kind: body.connection_kind,
            ownership: body.ownership,
            transport: body.transport,
            auth_kind: body.auth_kind,
            status: body.status,
            enabled: body.enabled,
            config: body.config,
            transport_config: body.transport_config,
            credential_refs: body.credential_refs,
            credential_secret_refs: body.credential_secret_refs,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/connections/{id}`.
#[route(GET "/api/companies/{company_id}/connections/{id}")]
pub async fn get_connection(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .tool_connections
        .get_connection(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Tool connection not found")),
    }
}

/// `PATCH /api/companies/{companyId}/connections/{id}`.
#[route(PATCH "/api/companies/{company_id}/connections/{id}")]
pub async fn update_connection(
    cx: &Cx,
    Json(body): Json<UpdateConnectionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .tool_connections
        .update_connection(UpdateToolConnection {
            company_id,
            id,
            name: body.name,
            status: body.status,
            enabled: body.enabled,
            config: body.config,
            transport_config: body.transport_config,
            health_status: body.health_status,
            health_message: body.health_message,
            health_checked_at: body.health_checked_at,
            last_error: body.last_error,
        })
        .await
        .map_err(toolchain_error_to_api)?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Tool connection not found")),
    }
}

/// `GET /api/companies/{companyId}/connections/{id}/grants`.
#[route(GET "/api/companies/{company_id}/connections/{id}/grants")]
pub async fn list_grants(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_connections
        .list_grants(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/connections/{id}/grants`.
#[route(POST "/api/companies/{company_id}/connections/{id}/grants")]
pub async fn create_grant(
    cx: &Cx,
    Json(body): Json<CreateGrantRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_connections
        .create_grant(NewConnectionGrant {
            company_id,
            connection_id: id,
            kind: body.kind,
            subject_user_id: body.subject_user_id,
            provider_tenant: body.provider_tenant,
            credential_secret_refs: body.credential_secret_refs,
            status: body.status,
            is_default: body.is_default,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `POST /api/companies/{companyId}/connections/{id}/grants/{grantId}/revoke`.
#[route(POST "/api/companies/{company_id}/connections/{id}/grants/{grant_id}/revoke")]
pub async fn revoke_grant(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let grant_id = path_param::<Id>(cx)?.to_string();
    let revoked_by = query_params::<RevokeQuery>(cx)
        .ok()
        .and_then(|q| q.revoked_by_user_id.clone());
    let state = app_context::<AppState>(cx);
    match state
        .tool_connections
        .revoke_grant(&company_id, &grant_id, revoked_by.as_deref())
        .await
        .map_err(toolchain_error_to_api)?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found(
            "Connection grant not found or already revoked",
        )),
    }
}

/// `GET /api/companies/{companyId}/connections/{id}/installs`.
#[route(GET "/api/companies/{company_id}/connections/{id}/installs")]
pub async fn list_installs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_connections
        .list_installs(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/connections/{id}/installs`.
#[route(POST "/api/companies/{company_id}/connections/{id}/installs")]
pub async fn create_install(
    cx: &Cx,
    Json(body): Json<CreateInstallRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_connections
        .create_install(NewToolConnectionInstall {
            company_id,
            connection_id: id,
            target_type: body.target_type,
            target_id: body.target_id,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/connections/{id}/token-issuances`.
#[route(GET "/api/companies/{company_id}/connections/{id}/token-issuances")]
pub async fn list_token_issuances(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_connections
        .list_token_issuances(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/connections/{id}/token-issuances`.
#[route(POST "/api/companies/{company_id}/connections/{id}/token-issuances")]
pub async fn create_token_issuance(
    cx: &Cx,
    Json(body): Json<CreateTokenIssuanceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_connections
        .create_token_issuance(NewConnectionTokenIssuance {
            company_id,
            application_id: body.application_id,
            connection_id: id,
            agent_id: body.agent_id,
            run_id: body.run_id,
            issue_id: body.issue_id,
            project_id: body.project_id,
            responsible_user_id: body.responsible_user_id,
            path: body.path,
            requested_scope: body.requested_scope,
            issued_scope: body.issued_scope,
            ttl_seconds: body.ttl_seconds,
            expires_at: body.expires_at,
            token_hash: body.token_hash,
            outcome: body.outcome,
            error_code: body.error_code,
            metadata: body.metadata,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/gateways`.
#[route(GET "/api/companies/{company_id}/tools/gateways")]
pub async fn list_gateways(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_gateways(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/gateways`.
#[route(POST "/api/companies/{company_id}/tools/gateways")]
pub async fn create_gateway(
    cx: &Cx,
    Json(body): Json<CreateGatewayRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_gateway(NewToolMcpGateway {
            company_id,
            name: body.name,
            slug: body.slug,
            display_slug: body.display_slug,
            description: body.description,
            status: body.status,
            profile_id: body.profile_id,
            default_profile_mode: body.default_profile_mode,
            context_scope_type: body.context_scope_type,
            context_scope_id: body.context_scope_id,
            agent_id: body.agent_id,
            project_id: body.project_id,
            issue_id: body.issue_id,
            approval_issue_id: body.approval_issue_id,
            auth_config: body.auth_config,
            header_policy: body.header_policy,
            metadata_policy: body.metadata_policy,
            on_demand_tools_config: body.on_demand_tools_config,
            metadata: body.metadata,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/gateways/{id}`.
#[route(GET "/api/companies/{company_id}/tools/gateways/{id}")]
pub async fn get_gateway(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .tool_gateway
        .get_gateway(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Tool gateway not found")),
    }
}

/// `GET /api/companies/{companyId}/tools/gateways/{id}/tokens`.
#[route(GET "/api/companies/{company_id}/tools/gateways/{id}/tokens")]
pub async fn list_gateway_tokens(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_gateway_tokens(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/gateways/{id}/tokens`.
#[route(POST "/api/companies/{company_id}/tools/gateways/{id}/tokens")]
pub async fn create_gateway_token(
    cx: &Cx,
    Json(body): Json<CreateGatewayTokenRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_gateway_token(NewToolMcpGatewayToken {
            company_id,
            gateway_id: id,
            name: body.name,
            token_hash: body.token_hash,
            token_prefix: body.token_prefix,
            subject_type: body.subject_type,
            subject_id: body.subject_id,
            client_label: body.client_label,
            owner_note: body.owner_note,
            allowed_actions: body.allowed_actions,
            expires_at: body.expires_at,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/gateways/{id}/sessions`.
#[route(GET "/api/companies/{company_id}/tools/gateways/{id}/sessions")]
pub async fn list_gateway_sessions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_gateway_sessions(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/gateways/{id}/sessions`.
#[route(POST "/api/companies/{company_id}/tools/gateways/{id}/sessions")]
pub async fn create_gateway_session(
    cx: &Cx,
    Json(body): Json<CreateGatewaySessionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_gateway_session(NewToolGatewaySession {
            company_id,
            agent_id: body.agent_id,
            run_id: body.run_id,
            issue_id: body.issue_id,
            project_id: body.project_id,
            gateway_id: Some(id),
            gateway_token_id: body.gateway_token_id,
            gateway_public_id: body.gateway_public_id,
            client_subject_type: body.client_subject_type,
            client_subject_id: body.client_subject_id,
            client_name: body.client_name,
            mcp_session_id: body.mcp_session_id,
            correlation_id: body.correlation_id,
            token_hash: body.token_hash,
            expires_at: body.expires_at,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/invocations`.
#[route(GET "/api/companies/{company_id}/tools/invocations")]
pub async fn list_invocations(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_invocations(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/invocations`.
#[route(POST "/api/companies/{company_id}/tools/invocations")]
pub async fn create_invocation(
    cx: &Cx,
    Json(body): Json<CreateInvocationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_invocation(NewToolInvocation {
            company_id,
            idempotency_key: body.idempotency_key,
            actor_type: body.actor_type,
            actor_id: body.actor_id,
            agent_id: body.agent_id,
            issue_id: body.issue_id,
            run_id: body.run_id,
            gateway_id: body.gateway_id,
            gateway_token_id: body.gateway_token_id,
            gateway_public_id: body.gateway_public_id,
            client_subject_type: body.client_subject_type,
            client_subject_id: body.client_subject_id,
            client_name: body.client_name,
            mcp_session_id: body.mcp_session_id,
            correlation_id: body.correlation_id,
            application_id: body.application_id,
            connection_id: body.connection_id,
            catalog_entry_id: body.catalog_entry_id,
            catalog_version_hash: body.catalog_version_hash,
            catalog_schema_hash: body.catalog_schema_hash,
            provider_type: body.provider_type,
            application_key: body.application_key,
            upstream_tool_name: body.upstream_tool_name,
            risk_level: body.risk_level,
            tool_name: body.tool_name,
            arguments_hash: body.arguments_hash,
            arguments_summary: body.arguments_summary,
            policy_decision: body.policy_decision,
            matched_policy_ids: body.matched_policy_ids,
            policy_explanation: body.policy_explanation,
            credential_scope_summary: body.credential_scope_summary,
            header_policy_summary: body.header_policy_summary,
            approval_state: body.approval_state,
            status: body.status,
            upstream_request_id: body.upstream_request_id,
            result_hash: body.result_hash,
            result_summary: body.result_summary,
            result_size_bytes: body.result_size_bytes,
            result_artifact_id: body.result_artifact_id,
            error_code: body.error_code,
            error_message: body.error_message,
            started_at: body.started_at,
            completed_at: body.completed_at,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/invocations/{id}`.
#[route(GET "/api/companies/{company_id}/tools/invocations/{id}")]
pub async fn get_invocation(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .tool_gateway
        .get_invocation(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Tool invocation not found")),
    }
}

/// `GET /api/companies/{companyId}/tools/invocations/{id}/action-requests`.
#[route(GET "/api/companies/{company_id}/tools/invocations/{id}/action-requests")]
pub async fn list_action_requests(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_action_requests(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/invocations/{id}/action-requests`.
#[route(POST "/api/companies/{company_id}/tools/invocations/{id}/action-requests")]
pub async fn create_action_request(
    cx: &Cx,
    Json(body): Json<CreateActionRequestRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_action_request(NewToolActionRequest {
            company_id,
            invocation_id: id,
            issue_id: body.issue_id,
            interaction_id: body.interaction_id,
            approval_id: body.approval_id,
            status: body.status,
            canonical_arguments_hash: body.canonical_arguments_hash,
            canonical_arguments_summary: body.canonical_arguments_summary,
            signed_arguments: body.signed_arguments,
            preview_markdown: body.preview_markdown,
            requested_by_agent_id: body.requested_by_agent_id,
            requested_by_user_id: body.requested_by_user_id,
            expires_at: body.expires_at,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/invocations/{id}/call-events`.
#[route(GET "/api/companies/{company_id}/tools/invocations/{id}/call-events")]
pub async fn list_call_events(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_call_events(&company_id, Some(&id))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/invocations/{id}/call-events`.
#[route(POST "/api/companies/{company_id}/tools/invocations/{id}/call-events")]
pub async fn create_call_event(
    cx: &Cx,
    Json(body): Json<CreateCallEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_call_event(NewToolCallEvent {
            company_id,
            event_type: body.event_type,
            actor_type: body.actor_type,
            actor_id: body.actor_id,
            agent_id: body.agent_id,
            run_id: body.run_id,
            issue_id: body.issue_id,
            gateway_id: body.gateway_id,
            gateway_token_id: body.gateway_token_id,
            gateway_public_id: body.gateway_public_id,
            client_subject_type: body.client_subject_type,
            client_subject_id: body.client_subject_id,
            client_name: body.client_name,
            mcp_session_id: body.mcp_session_id,
            correlation_id: body.correlation_id,
            application_id: body.application_id,
            connection_id: body.connection_id,
            catalog_entry_id: body.catalog_entry_id,
            invocation_id: Some(id),
            action_request_id: body.action_request_id,
            runtime_slot_id: body.runtime_slot_id,
            tool_name: body.tool_name,
            decision: body.decision,
            matched_policy_ids: body.matched_policy_ids,
            reason_code: body.reason_code,
            policy_explanation: body.policy_explanation,
            credential_scope_summary: body.credential_scope_summary,
            header_policy_summary: body.header_policy_summary,
            outcome: body.outcome,
            latency_ms: body.latency_ms,
            arguments_summary: body.arguments_summary,
            request_hash: body.request_hash,
            request_summary: body.request_summary,
            result_hash: body.result_hash,
            result_summary: body.result_summary,
            result_size_bytes: body.result_size_bytes,
            redaction_plan: body.redaction_plan,
            rate_limit_state: body.rate_limit_state,
            metadata: body.metadata,
            error_code: body.error_code,
            error_message: body.error_message,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/audit-events`.
#[route(GET "/api/companies/{company_id}/tools/audit-events")]
pub async fn list_audit_events(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_audit_events(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/audit-events`.
#[route(POST "/api/companies/{company_id}/tools/audit-events")]
pub async fn create_audit_event(
    cx: &Cx,
    Json(body): Json<CreateAuditEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_audit_event(NewToolAccessAuditEvent {
            company_id,
            gateway_id: body.gateway_id,
            gateway_token_id: body.gateway_token_id,
            gateway_public_id: body.gateway_public_id,
            client_name: body.client_name,
            correlation_id: body.correlation_id,
            connection_id: body.connection_id,
            catalog_entry_id: body.catalog_entry_id,
            actor_type: body.actor_type,
            actor_id: body.actor_id,
            action: body.action,
            outcome: body.outcome,
            reason_code: body.reason_code,
            details: body.details,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/policies`.
#[route(GET "/api/companies/{company_id}/tools/policies")]
pub async fn list_policies(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_policies(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/policies`.
#[route(POST "/api/companies/{company_id}/tools/policies")]
pub async fn create_policy(
    cx: &Cx,
    Json(body): Json<CreatePolicyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_policy(NewToolPolicy {
            company_id,
            name: body.name,
            description: body.description,
            policy_type: body.policy_type,
            priority: body.priority,
            enabled: body.enabled,
            selectors: body.selectors,
            conditions: body.conditions,
            config: body.config,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/runtime-slots`.
#[route(GET "/api/companies/{company_id}/tools/runtime-slots")]
pub async fn list_runtime_slots(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_runtime_slots(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/runtime-slots`.
#[route(POST "/api/companies/{company_id}/tools/runtime-slots")]
pub async fn create_runtime_slot(
    cx: &Cx,
    Json(body): Json<CreateRuntimeSlotRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_runtime_slot(NewToolRuntimeSlot {
            company_id,
            application_id: body.application_id,
            connection_id: body.connection_id,
            project_workspace_id: body.project_workspace_id,
            execution_workspace_id: body.execution_workspace_id,
            issue_id: body.issue_id,
            owner_scope_type: body.owner_scope_type,
            owner_scope_id: body.owner_scope_id,
            runtime_kind: body.runtime_kind,
            slot_key: body.slot_key,
            status: body.status,
            reuse_key: body.reuse_key,
            workspace_scope: body.workspace_scope,
            credential_scope_hash: body.credential_scope_hash,
            provider: body.provider,
            provider_ref: body.provider_ref,
            process_id: body.process_id,
            command_template_key: body.command_template_key,
            health_status: body.health_status,
            health_message: body.health_message,
            metadata: body.metadata,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/tools/stdio-templates`.
#[route(GET "/api/companies/{company_id}/tools/stdio-templates")]
pub async fn list_stdio_templates(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .tool_gateway
        .list_stdio_templates(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/tools/stdio-templates`.
#[route(POST "/api/companies/{company_id}/tools/stdio-templates")]
pub async fn create_stdio_template(
    cx: &Cx,
    Json(body): Json<CreateStdioTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .tool_gateway
        .create_stdio_template(NewToolStdioCommandTemplate {
            company_id,
            template_key: body.template_key,
            name: body.name,
            description: body.description,
            status: body.status,
            command: body.command,
            args: body.args,
            env_keys: body.env_keys,
            tools: body.tools,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(toolchain_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}
