//! Issue structure routes: labels, thread interactions, read states,
//! issue approvals, execution decisions.

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewExecutionDecision, NewLabel, NewThreadInteraction};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/labels`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLabelRequest {
    /// Label name.
    pub name: String,
    /// Color.
    pub color: String,
}

/// Body for `POST /api/issues/{issueId}/labels`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachLabelRequest {
    /// Label id.
    pub label_id: String,
}

/// Body for `POST /api/issues/{issueId}/thread-interactions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadInteractionRequest {
    /// Kind.
    pub kind: String,
    /// Payload.
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Body for `PUT /api/issues/{issueId}/read-state`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadStateRequest {
    /// User id.
    pub user_id: String,
    /// ISO 8601 last read time.
    pub last_read_at: String,
}

/// Body for `POST /api/issues/{issueId}/approvals`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkApprovalRequest {
    /// Approval id.
    pub approval_id: String,
}

/// Body for `POST /api/issues/{issueId}/execution-decisions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExecutionDecisionRequest {
    /// Stage id.
    pub stage_id: String,
    /// Stage type.
    pub stage_type: String,
    /// Actor agent id.
    #[serde(default)]
    pub actor_agent_id: Option<String>,
    /// Actor user id.
    #[serde(default)]
    pub actor_user_id: Option<String>,
    /// Outcome.
    pub outcome: String,
    /// Body.
    #[serde(default)]
    pub body: Option<String>,
}

/// `POST /api/companies/{companyId}/labels` — creates a label.
#[route(POST "/api/companies/{company_id}/labels")]
pub async fn create_label(
    cx: &Cx,
    Json(body): Json<CreateLabelRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.name.trim().is_empty() {
        issues.push(
            json!({ "path": ["name"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.color.trim().is_empty() {
        issues.push(
            json!({ "path": ["color"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let label = state
        .labels
        .create(NewLabel {
            company_id: company_id.clone(),
            name: body.name.trim().to_owned(),
            color: body.color,
        })
        .await
        .map_err(label_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "label.created",
        "label",
        &label.id,
        Some(json!({ "name": label.name })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&label).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/labels` — lists labels.
#[route(GET "/api/companies/{company_id}/labels")]
pub async fn list_labels(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let labels = state
        .labels
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&labels).unwrap_or_default()))
}

/// `POST /api/issues/{issueId}/labels` — attaches a label.
#[route(POST "/api/issues/{id}/labels")]
pub async fn attach_label(
    cx: &Cx,
    Json(body): Json<AttachLabelRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if !is_uuid(&body.label_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["labelId"], "message": "Invalid uuid" }]),
        ));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .labels
        .attach(&issue_id, &body.label_id)
        .await
        .map_err(label_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/issues/{issueId}/labels` — lists an issue's labels.
#[route(GET "/api/issues/{id}/labels")]
pub async fn list_issue_labels(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let labels = state
        .labels
        .list_for_issue(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&labels).unwrap_or_default()))
}

/// `DELETE /api/issues/{issueId}/labels/{labelId}` — detaches a label.
#[route(DELETE "/api/issues/{id}/labels/{label_id}")]
pub async fn detach_label(cx: &Cx) -> Result<StatusCode, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let label_id = path_param::<LabelId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .labels
        .detach(&issue_id, &label_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/issues/{issueId}/thread-interactions`.
#[route(POST "/api/issues/{id}/thread-interactions")]
pub async fn create_thread_interaction(
    cx: &Cx,
    Json(body): Json<CreateThreadInteractionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if body.kind.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["kind"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = resolve_issue_company(state, &issue_id).await?;
    let record = state
        .issue_structure
        .create_thread_interaction(NewThreadInteraction {
            company_id,
            issue_id,
            kind: body.kind,
            payload: body.payload.unwrap_or_else(|| json!({})).to_string(),
        })
        .await
        .map_err(structure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/issues/{issueId}/thread-interactions`.
#[route(GET "/api/issues/{id}/thread-interactions")]
pub async fn list_thread_interactions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .issue_structure
        .list_thread_interactions(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `PUT /api/issues/{issueId}/read-state` — upserts the read state.
#[route(PUT "/api/issues/{id}/read-state")]
pub async fn upsert_read_state(
    cx: &Cx,
    Json(body): Json<ReadStateRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.user_id.trim().is_empty() || body.last_read_at.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["user_id"], "message": "user_id and last_read_at are required" }]),
        ));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = resolve_issue_company(state, &issue_id).await?;
    let record = state
        .issue_structure
        .upsert_read_state(&company_id, &issue_id, &body.user_id, &body.last_read_at)
        .await
        .map_err(structure_error_to_api)?;
    Ok(Json(serde_json::to_value(&record).unwrap_or_default()))
}

/// `POST /api/issues/{issueId}/approvals` — links an approval.
#[route(POST "/api/issues/{id}/approvals")]
pub async fn link_approval(
    cx: &Cx,
    Json(body): Json<LinkApprovalRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if !is_uuid(&body.approval_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["approvalId"], "message": "Invalid uuid" }]),
        ));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = resolve_issue_company(state, &issue_id).await?;
    let record = state
        .issue_structure
        .link_approval(&company_id, &issue_id, &body.approval_id)
        .await
        .map_err(structure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/issues/{issueId}/approvals` — lists linked approvals.
#[route(GET "/api/issues/{id}/approvals")]
pub async fn list_issue_approvals(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .issue_structure
        .list_issue_approvals(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/issues/{issueId}/execution-decisions`.
#[route(POST "/api/issues/{id}/execution-decisions")]
pub async fn create_execution_decision(
    cx: &Cx,
    Json(body): Json<CreateExecutionDecisionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.stage_id.trim().is_empty() || body.stage_type.trim().is_empty() {
        issues
            .push(json!({ "path": ["stageId"], "message": "stageId and stageType are required" }));
    }
    if body.outcome.trim().is_empty() {
        issues.push(json!({ "path": ["outcome"], "message": "String must contain at least 1 character(s)" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = resolve_issue_company(state, &issue_id).await?;
    let record = state
        .issue_structure
        .create_execution_decision(NewExecutionDecision {
            company_id,
            issue_id,
            stage_id: body.stage_id,
            stage_type: body.stage_type,
            actor_agent_id: body.actor_agent_id,
            actor_user_id: body.actor_user_id,
            outcome: body.outcome,
            body: body.body.unwrap_or_default(),
        })
        .await
        .map_err(structure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/issues/{issueId}/execution-decisions`.
#[route(GET "/api/issues/{id}/execution-decisions")]
pub async fn list_execution_decisions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .issue_structure
        .list_execution_decisions(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// Resolves an issue's company id through the issues repository.
async fn resolve_issue_company(state: &AppState, issue_id: &str) -> Result<String, ApiError> {
    state
        .issues
        .get(issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(|issue| issue.company_id)
        .ok_or_else(|| ApiError::not_found("Issue not found"))
}

/// Shared `{label_id}` path parameter.
#[path_param(error = bad_request("Invalid label id"))]
pub(crate) struct LabelId(String);

fn label_error_to_api(error: staple_data::LabelError) -> ApiError {
    use staple_data::LabelError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::AlreadyExists => ApiError::conflict("Label already exists"),
        E::NotFound => ApiError::not_found("Label or issue not found"),
        E::AlreadyAttached => ApiError::conflict("Label already attached"),
        other => ApiError::internal(other.to_string()),
    }
}

fn structure_error_to_api(error: staple_data::IssueStructureError) -> ApiError {
    use staple_data::IssueStructureError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::ApprovalNotFound => ApiError::not_found("Approval not found"),
        E::AlreadyExists => ApiError::conflict("Link already exists"),
        other => ApiError::internal(other.to_string()),
    }
}
