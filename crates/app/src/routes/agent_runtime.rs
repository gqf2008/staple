//! Agent runtime routes: task sessions, runtime state, wakeup requests, and
//! issue recovery actions.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    AgentRuntimeStateRecord, AgentTaskSessionRecord, AgentWakeupRequestRecord,
    IssueRecoveryActionRecord, NewRecoveryAction, NewRuntimeState, NewTaskSession,
    NewWakeupRequest,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::require_board,
    error::ApiError,
    routes::{AgentId, CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/agent-task-sessions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSessionRequest {
    /// Agent id.
    pub agent_id: String,
    /// Adapter type.
    pub adapter_type: String,
    /// Task key.
    pub task_key: String,
    /// Session params JSON.
    #[serde(default)]
    pub session_params: Option<serde_json::Value>,
    /// Display id.
    #[serde(default)]
    pub session_display_id: Option<String>,
    /// Last run id.
    #[serde(default)]
    pub last_run_id: Option<String>,
}

/// Body for `PUT /api/companies/{companyId}/agent-runtime-state/{agentId}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertRuntimeStateRequest {
    /// Adapter type.
    pub adapter_type: String,
    /// Session id.
    #[serde(default)]
    pub session_id: Option<String>,
    /// State JSON.
    #[serde(default)]
    pub state: Option<serde_json::Value>,
    /// Last run id.
    #[serde(default)]
    pub last_run_id: Option<String>,
    /// Last run status.
    #[serde(default)]
    pub last_run_status: Option<String>,
    /// Total input tokens.
    #[serde(default)]
    pub total_input_tokens: Option<i64>,
    /// Total output tokens.
    #[serde(default)]
    pub total_output_tokens: Option<i64>,
    /// Total cached input tokens.
    #[serde(default)]
    pub total_cached_input_tokens: Option<i64>,
    /// Total cost cents.
    #[serde(default)]
    pub total_cost_cents: Option<i64>,
    /// Last error.
    #[serde(default)]
    pub last_error: Option<String>,
}

/// Body for `POST /api/companies/{companyId}/agent-wakeup-requests`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueWakeupRequest {
    /// Agent id.
    pub agent_id: String,
    /// Source.
    pub source: String,
    /// Trigger detail.
    #[serde(default)]
    pub trigger_detail: Option<String>,
    /// Reason.
    #[serde(default)]
    pub reason: Option<String>,
    /// Payload JSON.
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    /// Idempotency key.
    #[serde(default)]
    pub idempotency_key: Option<String>,
}

/// Body for `POST /api/companies/{companyId}/agent-wakeup-requests/{id}/finish`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishWakeupRequest {
    /// Status (`finished` | `failed`).
    pub status: String,
    /// Error.
    #[serde(default)]
    pub error: Option<String>,
    /// Run id.
    #[serde(default)]
    pub run_id: Option<String>,
}

/// Body for `POST /api/companies/{companyId}/recovery-actions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRecoveryActionRequest {
    /// Source issue id.
    pub source_issue_id: String,
    /// Recovery issue id.
    #[serde(default)]
    pub recovery_issue_id: Option<String>,
    /// Kind.
    pub kind: String,
    /// Owner agent id.
    #[serde(default)]
    pub owner_agent_id: Option<String>,
    /// Cause.
    pub cause: String,
    /// Fingerprint.
    pub fingerprint: String,
    /// Evidence JSON.
    #[serde(default)]
    pub evidence: Option<serde_json::Value>,
    /// Next action.
    pub next_action: String,
    /// Wake policy JSON.
    #[serde(default)]
    pub wake_policy: Option<serde_json::Value>,
    /// Monitor policy JSON.
    #[serde(default)]
    pub monitor_policy: Option<serde_json::Value>,
    /// Max attempts.
    #[serde(default)]
    pub max_attempts: Option<i64>,
    /// Timeout.
    #[serde(default)]
    pub timeout_at: Option<String>,
}

/// Body for `POST /api/companies/{companyId}/recovery-actions/{id}/...`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryTransitionRequest {
    /// Outcome (resolve).
    #[serde(default)]
    pub outcome: Option<String>,
    /// Resolution note.
    #[serde(default)]
    pub resolution_note: Option<String>,
}

fn validate_session(body: &UpsertSessionRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if !is_uuid(&body.agent_id) {
        issues.push(json!({ "path": ["agentId"], "message": "Invalid uuid" }));
    }
    if body.task_key.trim().is_empty() {
        issues.push(json!({ "path": ["taskKey"], "message": "String must contain at least 1 character(s)" }));
    }
    if body.adapter_type.trim().is_empty() {
        issues.push(json!({ "path": ["adapterType"], "message": "String must contain at least 1 character(s)" }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/agent-task-sessions` — lists sessions.
#[route(GET "/api/companies/{company_id}/agent-task-sessions")]
pub async fn list_sessions(cx: &Cx) -> Result<Json<Vec<AgentTaskSessionRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let sessions = state
        .agent_runtime
        .session_list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(sessions))
}

/// `POST /api/companies/{companyId}/agent-task-sessions` — upserts a session.
#[route(POST "/api/companies/{company_id}/agent-task-sessions")]
pub async fn upsert_session(
    cx: &Cx,
    Json(body): Json<UpsertSessionRequest>,
) -> Result<(StatusCode, Json<AgentTaskSessionRecord>), ApiError> {
    require_board(cx)?;
    validate_session(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let session = state
        .agent_runtime
        .session_upsert(NewTaskSession {
            company_id: company_id.clone(),
            agent_id: body.agent_id,
            adapter_type: body.adapter_type,
            task_key: body.task_key,
            session_params_json: body.session_params,
            session_display_id: body.session_display_id,
            last_run_id: body.last_run_id,
        })
        .await
        .map_err(runtime_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "agent_task_session.upserted",
        "agent_task_session",
        &session.id,
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(session)))
}

/// `GET /api/companies/{companyId}/agent-runtime-state/{agentId}` — reads.
#[route(GET "/api/companies/{company_id}/agent-runtime-state/{agent_id}")]
pub async fn get_runtime_state(cx: &Cx) -> Result<Json<AgentRuntimeStateRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .agent_runtime
        .runtime_get(&company_id, &agent_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Runtime state not found"))
}

/// `PUT /api/companies/{companyId}/agent-runtime-state/{agentId}` — upserts.
#[route(PUT "/api/companies/{company_id}/agent-runtime-state/{agent_id}")]
pub async fn upsert_runtime_state(
    cx: &Cx,
    Json(body): Json<UpsertRuntimeStateRequest>,
) -> Result<Json<AgentRuntimeStateRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .agent_runtime
        .runtime_upsert(NewRuntimeState {
            company_id,
            agent_id,
            adapter_type: body.adapter_type,
            session_id: body.session_id,
            state_json: body.state.unwrap_or_else(|| json!({})),
            last_run_id: body.last_run_id,
            last_run_status: body.last_run_status,
            total_input_tokens: body.total_input_tokens.unwrap_or(0),
            total_output_tokens: body.total_output_tokens.unwrap_or(0),
            total_cached_input_tokens: body.total_cached_input_tokens.unwrap_or(0),
            total_cost_cents: body.total_cost_cents.unwrap_or(0),
            last_error: body.last_error,
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok(Json(record))
}

/// `GET /api/companies/{companyId}/agent-wakeup-requests` — lists.
#[route(GET "/api/companies/{company_id}/agent-wakeup-requests")]
pub async fn list_wakeups(cx: &Cx) -> Result<Json<Vec<AgentWakeupRequestRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let requests = state
        .agent_runtime
        .wakeup_list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(requests))
}

/// `POST /api/companies/{companyId}/agent-wakeup-requests` — enqueues.
#[route(POST "/api/companies/{company_id}/agent-wakeup-requests")]
pub async fn enqueue_wakeup(
    cx: &Cx,
    Json(body): Json<EnqueueWakeupRequest>,
) -> Result<(StatusCode, Json<AgentWakeupRequestRecord>), ApiError> {
    require_board(cx)?;
    if !is_uuid(&body.agent_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["agentId"], "message": "Invalid uuid" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let request = state
        .agent_runtime
        .wakeup_enqueue(NewWakeupRequest {
            company_id: company_id.clone(),
            agent_id: body.agent_id,
            source: body.source,
            trigger_detail: body.trigger_detail,
            reason: body.reason,
            payload: body.payload,
            requested_by_actor_type: Some("board".to_owned()),
            requested_by_actor_id: None,
            idempotency_key: body.idempotency_key,
        })
        .await
        .map_err(runtime_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "agent_wakeup.enqueued",
        "agent_wakeup_request",
        &request.id,
        Some(json!({ "agentId": request.agent_id, "source": request.source })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(request)))
}

/// `POST /api/companies/{companyId}/agent-wakeup-requests/{id}/claim` — claims.
#[route(POST "/api/companies/{company_id}/agent-wakeup-requests/{id}/claim")]
pub async fn claim_wakeup(cx: &Cx) -> Result<Json<AgentWakeupRequestRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .agent_runtime
        .wakeup_claim(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| {
            ApiError::unprocessable(
                "Validation error",
                json!([{ "path": ["id"], "message": "Wakeup request is not queued" }]),
            )
        })
}

/// `POST /api/companies/{companyId}/agent-wakeup-requests/{id}/finish`.
#[route(POST "/api/companies/{company_id}/agent-wakeup-requests/{id}/finish")]
pub async fn finish_wakeup(
    cx: &Cx,
    Json(body): Json<FinishWakeupRequest>,
) -> Result<Json<AgentWakeupRequestRecord>, ApiError> {
    require_board(cx)?;
    if !matches!(body.status.as_str(), "finished" | "failed") {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["status"], "message": "Invalid enum value. Expected 'finished' | 'failed'" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .agent_runtime
        .wakeup_finish(&company_id, &id, &body.status, body.error, body.run_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Wakeup request not claimed"))
}

/// `GET /api/companies/{companyId}/recovery-actions` — lists.
#[route(GET "/api/companies/{company_id}/recovery-actions")]
pub async fn list_recovery_actions(
    cx: &Cx,
) -> Result<Json<Vec<IssueRecoveryActionRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let actions = state
        .agent_runtime
        .recovery_list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(actions))
}

/// `POST /api/companies/{companyId}/recovery-actions` — creates.
#[route(POST "/api/companies/{company_id}/recovery-actions")]
pub async fn create_recovery_action(
    cx: &Cx,
    Json(body): Json<CreateRecoveryActionRequest>,
) -> Result<(StatusCode, Json<IssueRecoveryActionRecord>), ApiError> {
    require_board(cx)?;
    if !is_uuid(&body.source_issue_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["sourceIssueId"], "message": "Invalid uuid" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let action = state
        .agent_runtime
        .recovery_create(NewRecoveryAction {
            company_id: company_id.clone(),
            source_issue_id: body.source_issue_id,
            recovery_issue_id: body.recovery_issue_id,
            kind: body.kind,
            owner_agent_id: body.owner_agent_id,
            cause: body.cause,
            fingerprint: body.fingerprint,
            evidence: body.evidence.unwrap_or_else(|| json!({})),
            next_action: body.next_action,
            wake_policy: body.wake_policy,
            monitor_policy: body.monitor_policy,
            max_attempts: body.max_attempts,
            timeout_at: body.timeout_at,
        })
        .await
        .map_err(runtime_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "recovery_action.created",
        "issue_recovery_action",
        &action.id,
        Some(json!({ "sourceIssueId": action.source_issue_id, "kind": action.kind })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(action)))
}

/// `POST /api/companies/{companyId}/recovery-actions/{id}/resolve` — resolves.
#[route(POST "/api/companies/{company_id}/recovery-actions/{id}/resolve")]
pub async fn resolve_recovery_action(
    cx: &Cx,
    Json(body): Json<RecoveryTransitionRequest>,
) -> Result<Json<IssueRecoveryActionRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .agent_runtime
        .recovery_set_status(
            &company_id,
            &id,
            "resolved",
            body.outcome,
            body.resolution_note,
        )
        .await
        .map_err(runtime_error_to_api)?
        .map(Json)
        .ok_or_else(|| {
            ApiError::unprocessable(
                "Validation error",
                json!([{ "path": ["id"], "message": "Recovery action is not active" }]),
            )
        })
}

/// `POST /api/companies/{companyId}/recovery-actions/{id}/escalate`.
#[route(POST "/api/companies/{company_id}/recovery-actions/{id}/escalate")]
pub async fn escalate_recovery_action(
    cx: &Cx,
    Json(body): Json<RecoveryTransitionRequest>,
) -> Result<Json<IssueRecoveryActionRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .agent_runtime
        .recovery_set_status(&company_id, &id, "escalated", None, body.resolution_note)
        .await
        .map_err(runtime_error_to_api)?
        .map(Json)
        .ok_or_else(|| {
            ApiError::unprocessable(
                "Validation error",
                json!([{ "path": ["id"], "message": "Recovery action is not active" }]),
            )
        })
}

/// `POST /api/companies/{companyId}/recovery-actions/{id}/restore` — reactivates.
#[route(POST "/api/companies/{company_id}/recovery-actions/{id}/restore")]
pub async fn restore_recovery_action(cx: &Cx) -> Result<Json<IssueRecoveryActionRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .agent_runtime
        .recovery_set_status(&company_id, &id, "active", None, None)
        .await
        .map_err(runtime_error_to_api)?
        .map(Json)
        .ok_or_else(|| ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "Recovery action is not active or escalated" }]),
        ))
}

/// `POST /api/companies/{companyId}/recovery-actions/{id}/cancel` — cancels.
#[route(POST "/api/companies/{company_id}/recovery-actions/{id}/cancel")]
pub async fn cancel_recovery_action(
    cx: &Cx,
    Json(body): Json<RecoveryTransitionRequest>,
) -> Result<Json<IssueRecoveryActionRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .agent_runtime
        .recovery_set_status(&company_id, &id, "cancelled", None, body.resolution_note)
        .await
        .map_err(runtime_error_to_api)?
        .map(Json)
        .ok_or_else(|| {
            ApiError::unprocessable(
                "Validation error",
                json!([{ "path": ["id"], "message": "Recovery action is not active" }]),
            )
        })
}

fn runtime_error_to_api(error: staple_data::AgentRuntimeError) -> ApiError {
    use staple_data::AgentRuntimeError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::AgentNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["agentId"], "message": "Agent not found" }]),
        ),
        E::SourceIssueNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["sourceIssueId"], "message": "Source issue not found" }]),
        ),
        E::NotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "Referenced record not found" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
