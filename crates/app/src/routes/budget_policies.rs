//! Budget policies and incidents routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{BudgetIncidentRecord, BudgetPolicyRecord, NewBudgetIncident, NewBudgetPolicy};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::require_board,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/budget-policies`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBudgetPolicyRequest {
    /// Scope type (`company` | `agent` | `project`).
    pub scope_type: String,
    /// Scope id.
    pub scope_id: String,
    /// Metric (`billed_cents`).
    #[serde(default)]
    pub metric: Option<String>,
    /// Window kind (`calendar_month_utc` | `rolling_30d`).
    pub window_kind: String,
    /// Amount in cents.
    pub amount: i64,
    /// Warn percent.
    #[serde(default)]
    pub warn_percent: Option<i64>,
    /// Hard stop enabled.
    #[serde(default)]
    pub hard_stop_enabled: Option<bool>,
    /// Notify enabled.
    #[serde(default)]
    pub notify_enabled: Option<bool>,
}

/// Body for `POST /api/companies/{companyId}/budget-incidents`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBudgetIncidentRequest {
    /// Policy id.
    pub policy_id: String,
    /// Scope type.
    pub scope_type: String,
    /// Scope id.
    pub scope_id: String,
    /// Metric.
    #[serde(default)]
    pub metric: Option<String>,
    /// Window kind.
    pub window_kind: String,
    /// Window start.
    pub window_start: String,
    /// Window end.
    pub window_end: String,
    /// Threshold type (`warn` | `hard_stop`).
    pub threshold_type: String,
    /// Amount limit.
    pub amount_limit: i64,
    /// Amount observed.
    pub amount_observed: i64,
}

fn validate_policy(body: &CreateBudgetPolicyRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if !matches!(body.scope_type.as_str(), "company" | "agent" | "project") {
        issues.push(json!({
            "path": ["scopeType"],
            "message": "Invalid enum value. Expected 'company' | 'agent' | 'project'",
        }));
    }
    if !matches!(
        body.window_kind.as_str(),
        "calendar_month_utc" | "rolling_30d"
    ) {
        issues.push(json!({
            "path": ["windowKind"],
            "message": "Invalid enum value. Expected 'calendar_month_utc' | 'rolling_30d'",
        }));
    }
    if body.amount < 0 {
        issues.push(json!({
            "path": ["amount"],
            "message": "Number must be greater than or equal to 0",
        }));
    }
    if let Some(warn) = body.warn_percent
        && !(0..=100).contains(&warn)
    {
        issues.push(json!({
            "path": ["warnPercent"],
            "message": "Number must be between 0 and 100",
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn validate_incident(body: &CreateBudgetIncidentRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if !is_uuid(&body.policy_id) {
        issues.push(json!({ "path": ["policyId"], "message": "Invalid uuid" }));
    }
    if !matches!(body.threshold_type.as_str(), "warn" | "hard_stop") {
        issues.push(json!({
            "path": ["thresholdType"],
            "message": "Invalid enum value. Expected 'warn' | 'hard_stop'",
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/budget-policies` — lists policies.
#[route(GET "/api/companies/{company_id}/budget-policies")]
pub async fn list_policies(cx: &Cx) -> Result<Json<Vec<BudgetPolicyRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let policies = state
        .budget_policies
        .list_policies(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(policies))
}

/// `POST /api/companies/{companyId}/budget-policies` — upserts a policy.
#[route(POST "/api/companies/{company_id}/budget-policies")]
pub async fn upsert_policy(
    cx: &Cx,
    Json(body): Json<CreateBudgetPolicyRequest>,
) -> Result<(StatusCode, Json<BudgetPolicyRecord>), ApiError> {
    require_board(cx)?;
    validate_policy(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .budget_policies
        .upsert_policy(NewBudgetPolicy {
            company_id: company_id.clone(),
            scope_type: body.scope_type,
            scope_id: body.scope_id,
            metric: body.metric.unwrap_or_else(|| "billed_cents".to_owned()),
            window_kind: body.window_kind,
            amount: body.amount,
            warn_percent: body.warn_percent.unwrap_or(80),
            hard_stop_enabled: body.hard_stop_enabled.unwrap_or(true),
            notify_enabled: body.notify_enabled.unwrap_or(true),
            created_by_user_id: None,
        })
        .await
        .map_err(policy_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "budget_policy.upserted",
        "budget_policy",
        &record.id,
        Some(json!({ "scopeType": record.scope_type, "scopeId": record.scope_id })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `DELETE /api/companies/{companyId}/budget-policies/{id}` — deletes a policy.
#[route(DELETE "/api/companies/{company_id}/budget-policies/{id}")]
pub async fn delete_policy(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .budget_policies
        .delete_policy(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => {
            log_activity(
                &state.activity,
                &company_id,
                "budget_policy.deleted",
                "budget_policy",
                &record.id,
                None,
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Budget policy not found")),
    }
}

/// `GET /api/companies/{companyId}/budget-incidents` — lists incidents.
#[route(GET "/api/companies/{company_id}/budget-incidents")]
pub async fn list_incidents(cx: &Cx) -> Result<Json<Vec<BudgetIncidentRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let incidents = state
        .budget_policies
        .list_incidents(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(incidents))
}

/// `POST /api/companies/{companyId}/budget-incidents` — records an incident.
#[route(POST "/api/companies/{company_id}/budget-incidents")]
pub async fn create_incident(
    cx: &Cx,
    Json(body): Json<CreateBudgetIncidentRequest>,
) -> Result<(StatusCode, Json<BudgetIncidentRecord>), ApiError> {
    require_board(cx)?;
    validate_incident(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .budget_policies
        .create_incident(NewBudgetIncident {
            company_id: company_id.clone(),
            policy_id: body.policy_id,
            scope_type: body.scope_type,
            scope_id: body.scope_id,
            metric: body.metric.unwrap_or_else(|| "billed_cents".to_owned()),
            window_kind: body.window_kind,
            window_start: body.window_start,
            window_end: body.window_end,
            threshold_type: body.threshold_type,
            amount_limit: body.amount_limit,
            amount_observed: body.amount_observed,
        })
        .await
        .map_err(policy_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "budget_incident.created",
        "budget_incident",
        &record.id,
        Some(json!({ "thresholdType": record.threshold_type, "amountObserved": record.amount_observed })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `POST /api/companies/{companyId}/budget-incidents/{id}/resolve` — resolves.
#[route(POST "/api/companies/{company_id}/budget-incidents/{id}/resolve")]
pub async fn resolve_incident(cx: &Cx) -> Result<Json<BudgetIncidentRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .budget_policies
        .set_incident_status(&company_id, &id, "resolved")
        .await
        .map_err(policy_error_to_api)?
        .ok_or_else(|| ApiError::not_found("Budget incident not found or not open"))?;
    Ok(Json(record))
}

/// `POST /api/companies/{companyId}/budget-incidents/{id}/dismiss` — dismisses.
#[route(POST "/api/companies/{company_id}/budget-incidents/{id}/dismiss")]
pub async fn dismiss_incident(cx: &Cx) -> Result<Json<BudgetIncidentRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .budget_policies
        .set_incident_status(&company_id, &id, "dismissed")
        .await
        .map_err(policy_error_to_api)?
        .ok_or_else(|| ApiError::not_found("Budget incident not found or not open"))?;
    Ok(Json(record))
}

fn policy_error_to_api(error: staple_data::BudgetPolicyError) -> ApiError {
    use staple_data::BudgetPolicyError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::PolicyNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["policyId"], "message": "Policy not found" }]),
        ),
        E::IncidentNotFound => ApiError::not_found("Budget incident not found"),
        other => ApiError::internal(other.to_string()),
    }
}
