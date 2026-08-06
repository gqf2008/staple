//! Heartbeat run routes (execution control plane).

use serde::Deserialize;
use serde_json::json;
use staple_data::{CompleteHeartbeatRun, NewHeartbeatRun};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    dto::HeartbeatRunDto,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/heartbeat-runs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRunRequest {
    /// Agent id.
    pub agent_id: String,
    /// `scheduler | manual | callback`.
    #[serde(default)]
    pub invocation_source: Option<String>,
    /// Issue to check out, if any.
    #[serde(default)]
    pub issue_id: Option<String>,
    /// Context snapshot (JSON object).
    #[serde(default)]
    pub context_snapshot: Option<serde_json::Value>,
    /// Trigger detail.
    #[serde(default)]
    pub trigger_detail: Option<String>,
}

/// Body for `POST /api/heartbeat-runs/{id}/complete`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRunRequest {
    /// Terminal status: `succeeded | failed | cancelled | timed_out`.
    pub status: String,
    /// Error message.
    #[serde(default)]
    pub error: Option<String>,
    /// Failure attribution: `infrastructure | agent`.
    #[serde(default)]
    pub error_kind: Option<String>,
}

/// Body for `POST /api/heartbeat-runs/{id}/watchdog-actions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchdogActionRequest {
    /// Target issue id.
    pub issue_id: String,
    /// Action name.
    pub action: String,
}

fn validate_start(body: &StartRunRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if !is_uuid(&body.agent_id) {
        issues.push(json!({ "path": ["agentId"], "message": "Invalid uuid" }));
    }
    if let Some(issue_id) = &body.issue_id
        && !is_uuid(issue_id)
    {
        issues.push(json!({ "path": ["issueId"], "message": "Invalid uuid" }));
    }
    if let Some(source) = &body.invocation_source
        && !matches!(source.as_str(), "scheduler" | "manual" | "callback")
    {
        issues.push(json!({
            "path": ["invocationSource"],
            "message": "Invalid enum value. Expected 'scheduler' | 'manual' | 'callback'"
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `POST /api/companies/{companyId}/heartbeat-runs` — starts a run, atomically
/// checking out the issue (409 when another run holds the execution lock).
#[route(POST "/api/companies/{company_id}/heartbeat-runs")]
pub async fn start_run(
    cx: &Cx,
    Json(body): Json<StartRunRequest>,
) -> Result<(StatusCode, Json<HeartbeatRunDto>), ApiError> {
    validate_start(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    // Shared-workspace concurrency gate: defer when the targeted shared
    // workspace is busy under `auto`/`serialize` (issue #206 phase B).
    if let Some(issue_id) = &body.issue_id {
        let busy = crate::workspace_policy::check_shared_workspace_busy(
            state,
            &company_id,
            issue_id,
            None,
        )
        .await?;
        if busy.busy {
            let _ = crate::workspace_policy::enqueue_workspace_busy_retry(
                state,
                &company_id,
                &body.agent_id,
                issue_id,
                1,
            )
            .await;
            return Err(ApiError::conflict("Shared workspace is busy; run deferred"));
        }
    }
    let run = state
        .heartbeat
        .start(NewHeartbeatRun {
            company_id,
            agent_id: body.agent_id,
            invocation_source: body
                .invocation_source
                .unwrap_or_else(|| "manual".to_owned()),
            issue_id: body.issue_id,
            context_snapshot: body.context_snapshot.map(|value| value.to_string()),
            trigger_detail: body.trigger_detail,
        })
        .await
        .map_err(heartbeat_error_to_api)?;
    log_activity(
        &state.activity,
        &run.company_id,
        "heartbeat_run.started",
        "heartbeat_run",
        &run.id,
        Some(json!({ "agentId": run.agent_id, "invocationSource": run.invocation_source })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(run.into())))
}

/// `GET /api/companies/{companyId}/heartbeat-runs` — lists runs.
#[route(GET "/api/companies/{company_id}/heartbeat-runs")]
pub async fn list_runs(cx: &Cx) -> Result<Json<Vec<HeartbeatRunDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let runs = state
        .heartbeat
        .list(&company_id, None, 200)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(runs.into_iter().map(HeartbeatRunDto::from).collect()))
}

/// `GET /api/heartbeat-runs/{id}` — observes one run.
#[route(GET "/api/heartbeat-runs/{id}")]
pub async fn get_run(cx: &Cx) -> Result<Json<HeartbeatRunDto>, ApiError> {
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .heartbeat
        .get(&run_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(run) => Ok(Json(run.into())),
        None => Err(ApiError::not_found("Heartbeat run not found")),
    }
}

/// `POST /api/heartbeat-runs/{id}/complete` — completes a run, releasing the
/// issue execution lock.
#[route(POST "/api/heartbeat-runs/{id}/complete")]
pub async fn complete_run(
    cx: &Cx,
    Json(body): Json<CompleteRunRequest>,
) -> Result<Json<HeartbeatRunDto>, ApiError> {
    if !matches!(
        body.status.as_str(),
        "succeeded" | "failed" | "cancelled" | "timed_out"
    ) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{
                "path": ["status"],
                "message": "Invalid enum value. Expected 'succeeded' | 'failed' | 'cancelled' | 'timed_out'"
            }]),
        ));
    }
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .heartbeat
        .complete(
            &run_id,
            CompleteHeartbeatRun {
                status: body.status,
                error: body.error,
                error_kind: body.error_kind,
            },
        )
        .await
        .map_err(heartbeat_error_to_api)?
    {
        Some(run) => {
            log_activity(
                &state.activity,
                &run.company_id,
                "heartbeat_run.completed",
                "heartbeat_run",
                &run.id,
                Some(json!({ "status": run.status, "errorKind": run.error_kind })),
            )
            .await?;
            Ok(Json(run.into()))
        }
        None => Err(ApiError::not_found("Heartbeat run not found")),
    }
}

/// `POST /api/heartbeat-runs/{id}/cancel` — cancels a run.
#[route(POST "/api/heartbeat-runs/{id}/cancel")]
pub async fn cancel_run(cx: &Cx) -> Result<Json<HeartbeatRunDto>, ApiError> {
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .heartbeat
        .cancel(&run_id)
        .await
        .map_err(heartbeat_error_to_api)?
    {
        Some(run) => {
            log_activity(
                &state.activity,
                &run.company_id,
                "heartbeat_run.cancelled",
                "heartbeat_run",
                &run.id,
                None,
            )
            .await?;
            Ok(Json(run.into()))
        }
        None => Err(ApiError::not_found("Heartbeat run not found")),
    }
}

/// `POST /api/heartbeat-runs/{id}/watchdog-actions` — authorizes a watchdog
/// action against the §9.9 contract; unauthorized targets get 403.
#[route(POST "/api/heartbeat-runs/{id}/watchdog-actions")]
pub async fn watchdog_action(
    cx: &Cx,
    Json(body): Json<WatchdogActionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !is_uuid(&body.issue_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["issueId"], "message": "Invalid uuid" }]),
        ));
    }
    if body.action.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["action"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let authorized = state
        .heartbeat
        .watchdog_authorized(&run_id, &body.issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    if !authorized {
        return Err(ApiError::forbidden(
            "Watchdog is not authorized for this issue",
        ));
    }
    Ok(Json(json!({ "allowed": true, "action": body.action })))
}

fn heartbeat_error_to_api(error: staple_data::HeartbeatError) -> ApiError {
    use staple_data::HeartbeatError as E;
    match error {
        E::AgentNotFound => ApiError::not_found("Agent not found"),
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::IssueExecutionLocked => {
            ApiError::conflict("Issue is already checked out by another run")
        }
        E::RunNotRunning => ApiError::conflict("Heartbeat run is not in a running state"),
        E::WatchdogNotAuthorized => {
            ApiError::forbidden("Watchdog is not authorized for this issue")
        }
        other => ApiError::internal(other.to_string()),
    }
}
