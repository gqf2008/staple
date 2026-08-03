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

/// Body for `POST /api/companies/{companyId}/decision-triage-events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendTriageEventRequest {
    /// Triage id.
    pub triage_id: String,
    /// Event type.
    pub event_type: String,
    /// Decision.
    #[serde(default)]
    pub decision: Option<String>,
}

/// `GET /api/companies/{companyId}/decision-triage-events?triageId=...`.
#[route(GET "/api/companies/{company_id}/decision-triage-events")]
pub async fn list_triage_events(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::DecisionTriageEventRecord>>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let triage_id = topcoat::router::query_params::<TriageQuery>(cx)
        .ok()
        .and_then(|query| query.triage_id.clone());
    let events = state
        .decisions
        .list_triage_events(&company_id, triage_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(events))
}

/// `POST /api/companies/{companyId}/decision-triage-events` — appends an
/// immutable triage event.
#[route(POST "/api/companies/{company_id}/decision-triage-events")]
pub async fn append_triage_event(
    cx: &Cx,
    Json(body): Json<AppendTriageEventRequest>,
) -> Result<(StatusCode, Json<staple_data::DecisionTriageEventRecord>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let event = state
        .decisions
        .append_triage_event(
            &company_id,
            &body.triage_id,
            &body.event_type,
            body.decision,
            None,
        )
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok((StatusCode::CREATED, Json(event)))
}

/// `GET /api/companies/{companyId}/decision-retention`.
#[route(GET "/api/companies/{company_id}/decision-retention")]
pub async fn list_retention(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::DecisionRetentionRecord>>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let retention = state
        .decisions
        .list_retention(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(retention))
}

/// `POST /api/companies/{companyId}/decision-retention/{sourceKind}/{sourceId}/keep`.
#[route(POST "/api/companies/{company_id}/decision-retention/{source_kind}/{source_id}/keep")]
pub async fn set_keep(cx: &Cx) -> Result<Json<staple_data::DecisionRetentionRecord>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let source_kind = path_param::<SourceKind>(cx)?.to_string();
    let source_id = path_param::<SourceId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .decisions
        .retention_set_keep(&company_id, &source_kind, &source_id, true)
        .await
        .map_err(decision_error_to_api)?;
    Ok(Json(record))
}

/// `POST /api/companies/{companyId}/decision-retention/{sourceKind}/{sourceId}/archive`.
#[route(POST "/api/companies/{company_id}/decision-retention/{source_kind}/{source_id}/archive")]
pub async fn archive_retention(
    cx: &Cx,
) -> Result<Json<staple_data::DecisionRetentionRecord>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let source_kind = path_param::<SourceKind>(cx)?.to_string();
    let source_id = path_param::<SourceId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .decisions
        .retention_archive(
            &company_id,
            &source_kind,
            &source_id,
            Some("manual".to_owned()),
            None,
        )
        .await
        .map_err(decision_error_to_api)?;
    Ok(Json(record))
}

/// `POST /api/companies/{companyId}/decision-retention/{sourceKind}/{sourceId}/restore`.
#[route(POST "/api/companies/{company_id}/decision-retention/{source_kind}/{source_id}/restore")]
pub async fn restore_retention(
    cx: &Cx,
) -> Result<Json<staple_data::DecisionRetentionRecord>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let source_kind = path_param::<SourceKind>(cx)?.to_string();
    let source_id = path_param::<SourceId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .decisions
        .retention_restore(&company_id, &source_kind, &source_id)
        .await
        .map_err(decision_error_to_api)?;
    Ok(Json(record))
}

/// `GET /api/companies/{companyId}/decision-archive-notification-outbox`.
#[route(GET "/api/companies/{company_id}/decision-archive-notification-outbox")]
pub async fn list_outbox(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::DecisionOutboxRecord>>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let outbox = state
        .decisions
        .list_outbox(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(outbox))
}

/// `POST /api/companies/{companyId}/decision-archive-notification-outbox/{id}/sent`.
#[route(POST "/api/companies/{company_id}/decision-archive-notification-outbox/{id}/sent")]
pub async fn mark_outbox_sent(
    cx: &Cx,
) -> Result<Json<staple_data::DecisionOutboxRecord>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<crate::routes::Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .decisions
        .outbox_mark_sent(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Outbox row not found or already sent"))
}

/// `POST /api/companies/{companyId}/decision-desk/sweep` — runs the built-in
/// 90-day retention sweeper.
#[route(POST "/api/companies/{company_id}/decision-desk/sweep")]
pub async fn sweep_retention(cx: &Cx) -> Result<Json<staple_data::DecisionSweepResult>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let result = state
        .decisions
        .sweep(&company_id, 90)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(result))
}

/// `{source_kind}` path parameter.
#[path_param(error = bad_request("Invalid source kind"))]
pub(crate) struct SourceKind(String);

/// `{source_id}` path parameter.
#[path_param(error = bad_request("Invalid source id"))]
pub(crate) struct SourceId(String);

/// Query for triage events.
#[topcoat::router::query_params]
struct TriageQuery {
    /// Optional triage filter.
    #[serde(rename = "triageId")]
    triage_id: Option<String>,
}
