//! Decision desk routes.

use serde::Deserialize;
use serde_json::json;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{error::ApiError, routes::CompanyId, state::AppState};

/// Body for `POST /api/companies/{companyId}/decision-queues`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQueueRequest {
    /// Queue name.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Retention override in days.
    #[serde(default)]
    pub retention_days: Option<i64>,
}

/// Body for `POST /api/companies/{companyId}/decision-queues/{queueId}/items`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddItemRequest {
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Payload (arbitrary JSON).
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Body for `POST /api/companies/{companyId}/decision-triage`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTriageRequest {
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Decide-by time.
    #[serde(default)]
    pub decide_by: Option<String>,
    /// Snoozed until time.
    #[serde(default)]
    pub snoozed_until: Option<String>,
    /// Decision.
    #[serde(default)]
    pub decision: Option<String>,
    /// Deciding user.
    #[serde(default)]
    pub decided_by_user_id: Option<String>,
}

/// `POST /api/companies/{companyId}/decision-queues` — creates a queue.
#[route(POST "/api/companies/{company_id}/decision-queues")]
pub async fn create_queue(
    cx: &Cx,
    Json(body): Json<CreateQueueRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["name"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let queue = state
        .decisions
        .create_queue(
            &company_id,
            &body.name,
            body.description,
            body.retention_days,
        )
        .await
        .map_err(decision_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&queue).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/decision-queues` — lists queues.
#[route(GET "/api/companies/{company_id}/decision-queues")]
pub async fn list_queues(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let queues = state
        .decisions
        .list_queues(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&queues).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/decision-queues/{queueId}/items`.
#[route(POST "/api/companies/{company_id}/decision-queues/{queue_id}/items")]
pub async fn add_item(
    cx: &Cx,
    Json(body): Json<AddItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let queue_id = path_param::<QueueId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let item = state
        .decisions
        .add_item(
            &company_id,
            &queue_id,
            &body.source_kind,
            &body.source_id,
            body.payload.map(|value| value.to_string()),
        )
        .await
        .map_err(decision_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&item).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/decision-queues/{queueId}/items`.
#[route(GET "/api/companies/{company_id}/decision-queues/{queue_id}/items")]
pub async fn list_items(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let queue_id = path_param::<QueueId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let items = state
        .decisions
        .list_items(&company_id, &queue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&items).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/decision-triage` — upserts triage state.
#[route(POST "/api/companies/{company_id}/decision-triage")]
pub async fn set_triage(
    cx: &Cx,
    Json(body): Json<SetTriageRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let triage = state
        .decisions
        .set_triage(
            &company_id,
            &body.source_kind,
            &body.source_id,
            staple_data::TriageInput {
                decide_by: body.decide_by,
                snoozed_until: body.snoozed_until,
                decision: body.decision,
                decided_by_user_id: body.decided_by_user_id,
            },
        )
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&triage).unwrap_or_default()))
}

/// `GET /api/companies/{companyId}/decision-triage`.
#[route(GET "/api/companies/{company_id}/decision-triage")]
pub async fn list_triage(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let triage = state
        .decisions
        .list_triage(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&triage).unwrap_or_default()))
}

/// Shared `{queue_id}` path parameter.
#[path_param(error = bad_request("Invalid queue id"))]
pub(crate) struct QueueId(String);

fn decision_error_to_api(error: staple_data::DecisionError) -> ApiError {
    use staple_data::DecisionError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::QueueNotFound => ApiError::not_found("Queue not found"),
        E::QueueExists => ApiError::conflict("Queue already exists"),
        E::ItemExists => ApiError::conflict("Queue item already exists"),
        other => ApiError::internal(other.to_string()),
    }
}
