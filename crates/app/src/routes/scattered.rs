//! Scattered domain routes: status cards, summary slots, smoke runs,
//! feedback, finance events, and document annotations.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewAnchorSnapshot, NewAnnotationComment, NewAnnotationThread, NewFeedbackExport,
    NewFeedbackVote, NewFinanceEvent, NewSmokeRun, NewSmokeRunStep, NewStatusCard,
    NewStatusCardUpdate, NewSummarySlot,
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

fn scattered_error_to_api(error: staple_data::ScatteredError) -> ApiError {
    use staple_data::ScatteredError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ReferenceNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "Referenced record not found" }]),
        ),
        E::AlreadyExists => ApiError::conflict("Record already exists"),
        E::NotFound => ApiError::not_found("Record not found"),
        other => ApiError::internal(other.to_string()),
    }
}

fn default_array() -> serde_json::Value {
    serde_json::json!([])
}
fn default_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Body for `POST /api/companies/{companyId}/status-cards`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStatusCardRequest {
    #[serde(default)]
    pub created_by_user_id: Option<String>,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_pinned: bool,
    pub interest_prompt: String,
    #[serde(default = "default_array")]
    pub queries: serde_json::Value,
    #[serde(default)]
    pub query_version: i64,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub refresh_policy: serde_json::Value,
    #[serde(default = "default_compiling")]
    pub state: String,
    #[serde(default)]
    pub document_id: Option<String>,
}

fn default_compiling() -> String {
    "compiling".to_owned()
}

/// `GET /api/companies/{companyId}/status-cards`.
#[route(GET "/api/companies/{company_id}/status-cards")]
pub async fn list_status_cards(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_status_cards(&company_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/status-cards`.
#[route(POST "/api/companies/{company_id}/status-cards")]
pub async fn create_status_card(
    cx: &Cx,
    Json(body): Json<CreateStatusCardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_status_card(NewStatusCard {
            company_id,
            created_by_user_id: body.created_by_user_id,
            created_by_agent_id: body.created_by_agent_id,
            title: body.title,
            title_pinned: body.title_pinned,
            interest_prompt: body.interest_prompt,
            queries: body.queries,
            query_version: body.query_version,
            agent_id: body.agent_id,
            refresh_policy: body.refresh_policy,
            state: body.state,
            document_id: body.document_id,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `POST /api/companies/{companyId}/status-cards/{id}/archive`.
#[route(POST "/api/companies/{company_id}/status-cards/{id}/archive")]
pub async fn archive_status_card(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .scattered
        .archive_status_card(&company_id, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Status card not found")),
    }
}

/// Body for `POST /api/companies/{companyId}/status-cards/{id}/updates`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStatusCardUpdateRequest {
    pub kind: String,
    pub trigger: String,
    #[serde(default)]
    pub generation_issue_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default = "default_array")]
    pub changes: serde_json::Value,
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub cost_cents: i64,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub query_version: Option<i64>,
    #[serde(default)]
    pub change_summary: Option<String>,
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
}

/// `GET /api/companies/{companyId}/status-cards/{id}/updates`.
#[route(GET "/api/companies/{company_id}/status-cards/{id}/updates")]
pub async fn list_status_card_updates(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_status_card_updates(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/status-cards/{id}/updates`.
#[route(POST "/api/companies/{company_id}/status-cards/{id}/updates")]
pub async fn create_status_card_update(
    cx: &Cx,
    Json(body): Json<CreateStatusCardUpdateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_status_card_update(NewStatusCardUpdate {
            card_id: id,
            kind: body.kind,
            trigger: body.trigger,
            generation_issue_id: body.generation_issue_id,
            run_id: body.run_id,
            changes: body.changes,
            input_tokens: body.input_tokens,
            output_tokens: body.output_tokens,
            cost_cents: body.cost_cents,
            model: body.model,
            query_version: body.query_version,
            change_summary: body.change_summary,
            status: body.status,
            error: body.error,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Summary slots --------------------------------------------------------

/// Body for `POST /api/companies/{companyId}/summary-slots`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSummarySlotRequest {
    pub scope_kind: String,
    #[serde(default)]
    pub scope_id: Option<String>,
    pub slot_key: String,
    #[serde(default)]
    pub document_id: Option<String>,
    #[serde(default = "default_idle")]
    pub status: String,
    #[serde(default)]
    pub failure_reason: Option<String>,
    #[serde(default)]
    pub generating_issue_id: Option<String>,
    #[serde(default)]
    pub last_generated_at: Option<String>,
    #[serde(default)]
    pub last_generated_by_agent_id: Option<String>,
    #[serde(default)]
    pub last_model: Option<String>,
}

fn default_idle() -> String {
    "idle".to_owned()
}

/// `GET /api/companies/{companyId}/summary-slots`.
#[route(GET "/api/companies/{company_id}/summary-slots")]
pub async fn list_summary_slots(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_summary_slots(&company_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/summary-slots`.
#[route(POST "/api/companies/{company_id}/summary-slots")]
pub async fn upsert_summary_slot(
    cx: &Cx,
    Json(body): Json<UpsertSummarySlotRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .upsert_summary_slot(NewSummarySlot {
            company_id,
            scope_kind: body.scope_kind,
            scope_id: body.scope_id,
            slot_key: body.slot_key,
            document_id: body.document_id,
            status: body.status,
            failure_reason: body.failure_reason,
            generating_issue_id: body.generating_issue_id,
            last_generated_at: body.last_generated_at,
            last_generated_by_agent_id: body.last_generated_by_agent_id,
            last_model: body.last_model,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Smoke runs -----------------------------------------------------------

/// Body for `POST /api/companies/{companyId}/smoke-runs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSmokeRunRequest {
    pub trigger: String,
    #[serde(default = "default_running")]
    pub status: String,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default = "default_object")]
    pub summary: serde_json::Value,
}

fn default_running() -> String {
    "running".to_owned()
}

/// `GET /api/companies/{companyId}/smoke-runs`.
#[route(GET "/api/companies/{company_id}/smoke-runs")]
pub async fn list_smoke_runs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_smoke_runs(&company_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/smoke-runs`.
#[route(POST "/api/companies/{company_id}/smoke-runs")]
pub async fn create_smoke_run(
    cx: &Cx,
    Json(body): Json<CreateSmokeRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_smoke_run(NewSmokeRun {
            company_id,
            trigger: body.trigger,
            status: body.status,
            finished_at: body.finished_at,
            summary: body.summary,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// Body for `POST /api/companies/{companyId}/smoke-runs/{runId}/steps`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSmokeStepRequest {
    pub path: String,
    pub scenario_step: String,
    pub status: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub screenshot_artifact_ref: Option<serde_json::Value>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
}

/// `GET /api/companies/{companyId}/smoke-runs/{runId}/steps`.
#[route(GET "/api/companies/{company_id}/smoke-runs/{run_id}/steps")]
pub async fn list_smoke_steps(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_smoke_steps(&company_id, &run_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/smoke-runs/{runId}/steps`.
#[route(POST "/api/companies/{company_id}/smoke-runs/{run_id}/steps")]
pub async fn add_smoke_step(
    cx: &Cx,
    Json(body): Json<AddSmokeStepRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .add_smoke_step(NewSmokeRunStep {
            company_id,
            run_id,
            path: body.path,
            scenario_step: body.scenario_step,
            status: body.status,
            detail: body.detail,
            screenshot_artifact_ref: body.screenshot_artifact_ref,
            duration_ms: body.duration_ms,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Feedback -------------------------------------------------------------

/// Body for `POST /api/companies/{companyId}/feedback-votes`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedbackVoteRequest {
    pub issue_id: String,
    pub target_type: String,
    pub target_id: String,
    pub author_user_id: String,
    pub vote: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub shared_with_labs: bool,
    #[serde(default)]
    pub shared_at: Option<String>,
    #[serde(default)]
    pub consent_version: Option<String>,
    #[serde(default)]
    pub redaction_summary: Option<serde_json::Value>,
}

/// `GET /api/companies/{companyId}/feedback-votes?issueId=`.
#[route(GET "/api/companies/{company_id}/feedback-votes")]
pub async fn list_feedback_votes(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let issue_id = query_params::<FeedbackVotesQuery>(cx)
        .ok()
        .and_then(|q| q.issue_id.clone())
        .unwrap_or_default();
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_feedback_votes(&company_id, &issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

#[query_params]
struct FeedbackVotesQuery {
    #[serde(rename = "issueId")]
    issue_id: Option<String>,
}

/// `POST /api/companies/{companyId}/feedback-votes`.
#[route(POST "/api/companies/{company_id}/feedback-votes")]
pub async fn create_feedback_vote(
    cx: &Cx,
    Json(body): Json<CreateFeedbackVoteRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_feedback_vote(NewFeedbackVote {
            company_id,
            issue_id: body.issue_id,
            target_type: body.target_type,
            target_id: body.target_id,
            author_user_id: body.author_user_id,
            vote: body.vote,
            reason: body.reason,
            shared_with_labs: body.shared_with_labs,
            shared_at: body.shared_at,
            consent_version: body.consent_version,
            redaction_summary: body.redaction_summary,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// Body for `POST /api/companies/{companyId}/feedback-exports`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedbackExportRequest {
    pub feedback_vote_id: String,
    pub issue_id: String,
    #[serde(default)]
    pub project_id: Option<String>,
    pub author_user_id: String,
    pub target_type: String,
    pub target_id: String,
    pub vote: String,
    #[serde(default = "default_local_only")]
    pub status: String,
    #[serde(default)]
    pub destination: Option<String>,
    #[serde(default)]
    pub export_id: Option<String>,
    #[serde(default)]
    pub consent_version: Option<String>,
    #[serde(default)]
    pub payload_digest: Option<String>,
    #[serde(default)]
    pub payload_snapshot: Option<serde_json::Value>,
    pub target_summary: serde_json::Value,
    #[serde(default)]
    pub redaction_summary: Option<serde_json::Value>,
    #[serde(default)]
    pub attempt_count: i64,
    #[serde(default)]
    pub exported_at: Option<String>,
    #[serde(default)]
    pub failure_reason: Option<String>,
}

fn default_local_only() -> String {
    "local_only".to_owned()
}

/// `GET /api/companies/{companyId}/feedback-exports`.
#[route(GET "/api/companies/{company_id}/feedback-exports")]
pub async fn list_feedback_exports(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_feedback_exports(&company_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/feedback-exports`.
#[route(POST "/api/companies/{company_id}/feedback-exports")]
pub async fn create_feedback_export(
    cx: &Cx,
    Json(body): Json<CreateFeedbackExportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_feedback_export(NewFeedbackExport {
            company_id,
            feedback_vote_id: body.feedback_vote_id,
            issue_id: body.issue_id,
            project_id: body.project_id,
            author_user_id: body.author_user_id,
            target_type: body.target_type,
            target_id: body.target_id,
            vote: body.vote,
            status: body.status,
            destination: body.destination,
            export_id: body.export_id,
            consent_version: body.consent_version,
            payload_digest: body.payload_digest,
            payload_snapshot: body.payload_snapshot,
            target_summary: body.target_summary,
            redaction_summary: body.redaction_summary,
            attempt_count: body.attempt_count,
            exported_at: body.exported_at,
            failure_reason: body.failure_reason,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Finance events -------------------------------------------------------

/// Body for `POST /api/companies/{companyId}/finance-events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFinanceEventRequest {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub goal_id: Option<String>,
    #[serde(default)]
    pub heartbeat_run_id: Option<String>,
    #[serde(default)]
    pub cost_event_id: Option<String>,
    #[serde(default)]
    pub billing_code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub event_kind: String,
    #[serde(default = "default_debit")]
    pub direction: String,
    pub biller: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub execution_adapter_type: Option<String>,
    #[serde(default)]
    pub pricing_tier: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub quantity: Option<i64>,
    #[serde(default)]
    pub unit: Option<String>,
    pub amount_cents: i64,
    #[serde(default = "default_usd")]
    pub currency: String,
    #[serde(default)]
    pub estimated: bool,
    #[serde(default)]
    pub external_invoice_id: Option<String>,
    #[serde(default)]
    pub metadata_json: Option<serde_json::Value>,
    pub occurred_at: String,
}

fn default_debit() -> String {
    "debit".to_owned()
}
fn default_usd() -> String {
    "USD".to_owned()
}

/// `GET /api/companies/{companyId}/finance-events`.
#[route(GET "/api/companies/{company_id}/finance-events")]
pub async fn list_finance_events(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_finance_events(&company_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/finance-events`.
#[route(POST "/api/companies/{company_id}/finance-events")]
pub async fn create_finance_event(
    cx: &Cx,
    Json(body): Json<CreateFinanceEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_finance_event(NewFinanceEvent {
            company_id,
            agent_id: body.agent_id,
            issue_id: body.issue_id,
            project_id: body.project_id,
            goal_id: body.goal_id,
            heartbeat_run_id: body.heartbeat_run_id,
            cost_event_id: body.cost_event_id,
            billing_code: body.billing_code,
            description: body.description,
            event_kind: body.event_kind,
            direction: body.direction,
            biller: body.biller,
            provider: body.provider,
            execution_adapter_type: body.execution_adapter_type,
            pricing_tier: body.pricing_tier,
            region: body.region,
            model: body.model,
            quantity: body.quantity,
            unit: body.unit,
            amount_cents: body.amount_cents,
            currency: body.currency,
            estimated: body.estimated,
            external_invoice_id: body.external_invoice_id,
            metadata_json: body.metadata_json,
            occurred_at: body.occurred_at,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Document annotations -------------------------------------------------

/// Body for `POST /api/companies/{companyId}/documents/{documentId}/annotation-threads`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAnnotationThreadRequest {
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub routine_id: Option<String>,
    #[serde(default)]
    pub case_id: Option<String>,
    pub document_key: String,
    #[serde(default = "default_open")]
    pub status: String,
    #[serde(default = "default_active")]
    pub anchor_state: String,
    #[serde(default)]
    pub original_revision_id: Option<String>,
    pub original_revision_number: i64,
    #[serde(default)]
    pub current_revision_id: Option<String>,
    pub current_revision_number: i64,
    pub selected_text: String,
    #[serde(default)]
    pub prefix_text: String,
    #[serde(default)]
    pub suffix_text: String,
    pub normalized_start: i64,
    pub normalized_end: i64,
    pub markdown_start: i64,
    pub markdown_end: i64,
    #[serde(default = "default_exact")]
    pub anchor_confidence: String,
    pub anchor_selector: serde_json::Value,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

fn default_open() -> String {
    "open".to_owned()
}
fn default_active() -> String {
    "active".to_owned()
}
fn default_exact() -> String {
    "exact".to_owned()
}

/// `GET /api/companies/{companyId}/documents/{documentId}/annotation-threads`.
#[route(GET "/api/companies/{company_id}/documents/{document_id}/annotation-threads")]
pub async fn list_annotation_threads(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let document_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_annotation_threads(&company_id, &document_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/documents/{documentId}/annotation-threads`.
#[route(POST "/api/companies/{company_id}/documents/{document_id}/annotation-threads")]
pub async fn create_annotation_thread(
    cx: &Cx,
    Json(body): Json<CreateAnnotationThreadRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let document_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_annotation_thread(NewAnnotationThread {
            company_id,
            issue_id: body.issue_id,
            routine_id: body.routine_id,
            case_id: body.case_id,
            document_id,
            document_key: body.document_key,
            status: body.status,
            anchor_state: body.anchor_state,
            original_revision_id: body.original_revision_id,
            original_revision_number: body.original_revision_number,
            current_revision_id: body.current_revision_id,
            current_revision_number: body.current_revision_number,
            selected_text: body.selected_text,
            prefix_text: body.prefix_text,
            suffix_text: body.suffix_text,
            normalized_start: body.normalized_start,
            normalized_end: body.normalized_end,
            markdown_start: body.markdown_start,
            markdown_end: body.markdown_end,
            anchor_confidence: body.anchor_confidence,
            anchor_selector: body.anchor_selector,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// Body for `POST /api/companies/{companyId}/annotation-threads/{threadId}/comments`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddAnnotationCommentRequest {
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub routine_id: Option<String>,
    #[serde(default)]
    pub case_id: Option<String>,
    pub document_id: String,
    pub body: String,
    pub author_type: String,
    #[serde(default)]
    pub author_agent_id: Option<String>,
    #[serde(default)]
    pub author_user_id: Option<String>,
    #[serde(default)]
    pub created_by_run_id: Option<String>,
    #[serde(default)]
    pub issue_comment_id: Option<String>,
    #[serde(default)]
    pub source_trust: Option<serde_json::Value>,
}

/// `GET /api/companies/{companyId}/annotation-threads/{threadId}/comments`.
#[route(GET "/api/companies/{company_id}/annotation-threads/{thread_id}/comments")]
pub async fn list_annotation_comments(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let thread_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_annotation_comments(&company_id, &thread_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/annotation-threads/{threadId}/comments`.
#[route(POST "/api/companies/{company_id}/annotation-threads/{thread_id}/comments")]
pub async fn add_annotation_comment(
    cx: &Cx,
    Json(body): Json<AddAnnotationCommentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let thread_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .add_annotation_comment(NewAnnotationComment {
            company_id,
            thread_id,
            issue_id: body.issue_id,
            routine_id: body.routine_id,
            case_id: body.case_id,
            document_id: body.document_id,
            body: body.body,
            author_type: body.author_type,
            author_agent_id: body.author_agent_id,
            author_user_id: body.author_user_id,
            created_by_run_id: body.created_by_run_id,
            issue_comment_id: body.issue_comment_id,
            source_trust: body.source_trust,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// Body for `POST /api/companies/{companyId}/annotation-threads/{threadId}/anchor-snapshots`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAnchorSnapshotRequest {
    pub document_id: String,
    #[serde(default)]
    pub from_revision_id: Option<String>,
    #[serde(default)]
    pub from_revision_number: Option<i64>,
    #[serde(default)]
    pub to_revision_id: Option<String>,
    pub to_revision_number: i64,
    pub previous_anchor: serde_json::Value,
    #[serde(default)]
    pub next_anchor: Option<serde_json::Value>,
    pub anchor_state: String,
    pub anchor_confidence: String,
    #[serde(default)]
    pub failure_reason: Option<String>,
}

/// `GET /api/companies/{companyId}/annotation-threads/{threadId}/anchor-snapshots`.
#[route(GET "/api/companies/{company_id}/annotation-threads/{thread_id}/anchor-snapshots")]
pub async fn list_anchor_snapshots(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let thread_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .scattered
        .list_anchor_snapshots(&company_id, &thread_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/annotation-threads/{threadId}/anchor-snapshots`.
#[route(POST "/api/companies/{company_id}/annotation-threads/{thread_id}/anchor-snapshots")]
pub async fn create_anchor_snapshot(
    cx: &Cx,
    Json(body): Json<CreateAnchorSnapshotRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let thread_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .scattered
        .create_anchor_snapshot(NewAnchorSnapshot {
            company_id,
            thread_id,
            document_id: body.document_id,
            from_revision_id: body.from_revision_id,
            from_revision_number: body.from_revision_number,
            to_revision_id: body.to_revision_id,
            to_revision_number: body.to_revision_number,
            previous_anchor: body.previous_anchor,
            next_anchor: body.next_anchor,
            anchor_state: body.anchor_state,
            anchor_confidence: body.anchor_confidence,
            failure_reason: body.failure_reason,
        })
        .await
        .map_err(scattered_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}
