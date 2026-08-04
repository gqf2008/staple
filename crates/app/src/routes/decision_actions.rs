//! Decision action routes: bundles, decisions, target issues, effect
//! executions, and training examples (upstream decisions domain).

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewDecision, NewDecisionBundle, NewDecisionEffectExecution, NewDecisionTrainingExample,
    ResolveDecision,
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

/// Body for `POST /api/companies/{companyId}/decision-bundles`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBundleRequest {
    pub title: String,
    pub summary: String,
    pub origin_agent_id: String,
    pub origin_issue_id: String,
    pub origin_run_id: String,
}

/// Body for `POST /api/companies/{companyId}/decisions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDecisionRequest {
    #[serde(default)]
    pub bundle_id: Option<String>,
    pub origin_agent_id: String,
    pub origin_issue_id: String,
    pub origin_run_id: String,
    #[serde(default)]
    pub rule_key: Option<String>,
    pub title: String,
    pub body: String,
    #[serde(default = "default_array")]
    pub options: serde_json::Value,
    #[serde(default)]
    pub inputs: Option<serde_json::Value>,
    #[serde(default = "default_open")]
    pub status: String,
    #[serde(default)]
    pub execution_status: Option<String>,
    #[serde(default)]
    pub chosen_option_id: Option<String>,
    #[serde(default)]
    pub input_values: Option<serde_json::Value>,
    #[serde(default)]
    pub decided_by_user_id: Option<String>,
    #[serde(default)]
    pub decided_at: Option<String>,
    pub expires_at: String,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    pub signed_spec: String,
    #[serde(default = "default_object")]
    pub target_snapshots: serde_json::Value,
    #[serde(default = "default_none")]
    pub continuation_policy: String,
    #[serde(default = "default_object")]
    pub metadata: serde_json::Value,
}

fn default_array() -> serde_json::Value {
    serde_json::json!([])
}
fn default_object() -> serde_json::Value {
    serde_json::json!({})
}
fn default_open() -> String {
    "open".to_owned()
}
fn default_none() -> String {
    "none".to_owned()
}

/// Body for `POST .../decisions/{id}/resolve`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveDecisionRequest {
    pub status: String,
    #[serde(default)]
    pub execution_status: Option<String>,
    #[serde(default)]
    pub chosen_option_id: Option<String>,
    #[serde(default)]
    pub decided_by_user_id: Option<String>,
    #[serde(default)]
    pub decided_at: Option<String>,
    #[serde(default)]
    pub input_values: Option<serde_json::Value>,
}

/// Body for `POST .../decisions/{id}/target-issues`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTargetIssueRequest {
    pub issue_id: String,
}

/// Body for `POST .../decisions/{id}/effect-executions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEffectExecutionRequest {
    pub effect_index: i64,
    pub effect_type: String,
    pub target_issue_id: String,
    #[serde(default = "default_claimed")]
    pub status: String,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub activity_log_id: Option<String>,
    #[serde(default)]
    pub executed_at: Option<String>,
}

fn default_claimed() -> String {
    "claimed".to_owned()
}

/// Body for `PATCH .../effect-executions/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEffectExecutionRequest {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub executed_at: Option<String>,
}

/// Body for `POST .../decision-training-examples`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTrainingExampleRequest {
    pub source_kind: String,
    pub source_id: String,
    pub issue_id: String,
    pub cutoff_at: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_array")]
    pub notes_history: serde_json::Value,
    #[serde(default)]
    pub decision_outcome: Option<String>,
    #[serde(default = "default_retention")]
    pub retention_policy: String,
    pub snapshot: serde_json::Value,
    pub created_by_user_id: String,
}

fn default_retention() -> String {
    "scrub_deleted_comments_v1".to_owned()
}

fn decision_error_to_api(error: staple_data::DecisionActionError) -> ApiError {
    use staple_data::DecisionActionError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ReferenceNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "Referenced record not found" }]),
        ),
        E::AlreadyExists => ApiError::conflict("Record already exists"),
        E::DecisionNotFound => ApiError::not_found("Decision not found"),
        other => ApiError::internal(other.to_string()),
    }
}

/// `POST /api/companies/{companyId}/decision-bundles`.
#[route(POST "/api/companies/{company_id}/decision-bundles")]
pub async fn create_bundle(
    cx: &Cx,
    Json(body): Json<CreateBundleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .decision_actions
        .create_bundle(NewDecisionBundle {
            company_id,
            title: body.title,
            summary: body.summary,
            origin_agent_id: body.origin_agent_id,
            origin_issue_id: body.origin_issue_id,
            origin_run_id: body.origin_run_id,
        })
        .await
        .map_err(decision_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/decision-bundles`.
#[route(GET "/api/companies/{company_id}/decision-bundles")]
pub async fn list_bundles(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .decision_actions
        .list_bundles(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/decisions`.
#[route(POST "/api/companies/{company_id}/decisions")]
pub async fn create_decision(
    cx: &Cx,
    Json(body): Json<CreateDecisionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.title.trim().is_empty() {
        issues.push(
            json!({ "path": ["title"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.expires_at.trim().is_empty() {
        issues.push(json!({ "path": ["expiresAt"], "message": "String must contain at least 1 character(s)" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .decision_actions
        .create_decision(NewDecision {
            company_id,
            bundle_id: body.bundle_id,
            origin_agent_id: body.origin_agent_id,
            origin_issue_id: body.origin_issue_id,
            origin_run_id: body.origin_run_id,
            rule_key: body.rule_key,
            title: body.title,
            body: body.body,
            options: body.options,
            inputs: body.inputs,
            status: body.status,
            execution_status: body.execution_status,
            chosen_option_id: body.chosen_option_id,
            input_values: body.input_values,
            decided_by_user_id: body.decided_by_user_id,
            decided_at: body.decided_at,
            expires_at: body.expires_at,
            idempotency_key: body.idempotency_key,
            signed_spec: body.signed_spec,
            target_snapshots: body.target_snapshots,
            continuation_policy: body.continuation_policy,
            metadata: body.metadata,
        })
        .await
        .map_err(decision_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/decisions?status=`.
#[route(GET "/api/companies/{company_id}/decisions")]
pub async fn list_decisions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let status = query_params::<DecisionsQuery>(cx)
        .ok()
        .and_then(|q| q.status.clone());
    let state = app_context::<AppState>(cx);
    let records = state
        .decision_actions
        .list_decisions(&company_id, status.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

#[query_params]
struct DecisionsQuery {
    #[serde(rename = "status")]
    status: Option<String>,
}

/// `GET /api/companies/{companyId}/decisions/{id}`.
#[route(GET "/api/companies/{company_id}/decisions/{id}")]
pub async fn get_decision(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .decision_actions
        .get_decision(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Decision not found")),
    }
}

/// `POST /api/companies/{companyId}/decisions/{id}/resolve`.
#[route(POST "/api/companies/{company_id}/decisions/{id}/resolve")]
pub async fn resolve_decision(
    cx: &Cx,
    Json(body): Json<ResolveDecisionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .decision_actions
        .resolve_decision(ResolveDecision {
            company_id,
            decision_id: id,
            status: body.status,
            execution_status: body.execution_status,
            chosen_option_id: body.chosen_option_id,
            decided_by_user_id: body.decided_by_user_id,
            decided_at: body.decided_at,
            input_values: body.input_values,
        })
        .await
        .map_err(decision_error_to_api)?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Decision not found")),
    }
}

/// `GET /api/companies/{companyId}/decisions/{id}/target-issues`.
#[route(GET "/api/companies/{company_id}/decisions/{id}/target-issues")]
pub async fn list_target_issues(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .decision_actions
        .list_target_issues(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/decisions/{id}/target-issues`.
#[route(POST "/api/companies/{company_id}/decisions/{id}/target-issues")]
pub async fn add_target_issue(
    cx: &Cx,
    Json(body): Json<AddTargetIssueRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .decision_actions
        .add_target_issue(&company_id, &id, &body.issue_id)
        .await
        .map_err(decision_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `DELETE /api/companies/{companyId}/decisions/{id}/target-issues/{issue_id}`.
#[route(DELETE "/api/companies/{company_id}/decisions/{id}/target-issues/{issue_id}")]
pub async fn remove_target_issue(cx: &Cx) -> Result<StatusCode, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let removed = state
        .decision_actions
        .remove_target_issue(&company_id, &id, &issue_id)
        .await
        .map_err(decision_error_to_api)?;
    if !removed {
        return Err(ApiError::not_found("Target issue link not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/companies/{companyId}/decisions/{id}/effect-executions`.
#[route(GET "/api/companies/{company_id}/decisions/{id}/effect-executions")]
pub async fn list_effect_executions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .decision_actions
        .list_effect_executions(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/decisions/{id}/effect-executions`.
#[route(POST "/api/companies/{company_id}/decisions/{id}/effect-executions")]
pub async fn create_effect_execution(
    cx: &Cx,
    Json(body): Json<CreateEffectExecutionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .decision_actions
        .create_effect_execution(NewDecisionEffectExecution {
            company_id,
            decision_id: id,
            effect_index: body.effect_index,
            effect_type: body.effect_type,
            target_issue_id: body.target_issue_id,
            status: body.status,
            result: body.result,
            error: body.error,
            activity_log_id: body.activity_log_id,
            executed_at: body.executed_at,
        })
        .await
        .map_err(decision_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `PATCH /api/companies/{companyId}/effect-executions/{id}`.
#[route(PATCH "/api/companies/{company_id}/effect-executions/{id}")]
pub async fn update_effect_execution(
    cx: &Cx,
    Json(body): Json<UpdateEffectExecutionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .decision_actions
        .update_effect_execution(
            &company_id,
            &id,
            body.status.as_deref(),
            body.result,
            body.error,
            body.executed_at,
        )
        .await
        .map_err(decision_error_to_api)?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Effect execution not found")),
    }
}

/// `POST /api/companies/{companyId}/decision-training-examples`.
#[route(POST "/api/companies/{company_id}/decision-training-examples")]
pub async fn create_training_example(
    cx: &Cx,
    Json(body): Json<CreateTrainingExampleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .decision_actions
        .create_training_example(NewDecisionTrainingExample {
            company_id,
            source_kind: body.source_kind,
            source_id: body.source_id,
            issue_id: body.issue_id,
            cutoff_at: body.cutoff_at,
            notes: body.notes,
            notes_history: body.notes_history,
            decision_outcome: body.decision_outcome,
            retention_policy: body.retention_policy,
            snapshot: body.snapshot,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(decision_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/decision-training-examples?issueId=`.
#[route(GET "/api/companies/{company_id}/decision-training-examples")]
pub async fn list_training_examples(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let issue_id = query_params::<TrainingExamplesQuery>(cx)
        .ok()
        .and_then(|q| q.issue_id.clone());
    let state = app_context::<AppState>(cx);
    let records = state
        .decision_actions
        .list_training_examples(&company_id, issue_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

#[query_params]
struct TrainingExamplesQuery {
    #[serde(rename = "issueId")]
    issue_id: Option<String>,
}
