//! Cost events and budget routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::NewCostEvent;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::require_board,
    dto::{AgentCostRowDto, BudgetSummaryDto, CostEventDto},
    error::ApiError,
    routes::{CompanyId, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/cost-events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostEventRequest {
    /// Agent id.
    pub agent_id: String,
    /// Issue id.
    #[serde(default)]
    pub issue_id: Option<String>,
    /// Billing code.
    #[serde(default)]
    pub billing_code: Option<String>,
    /// Provider.
    pub provider: String,
    /// Model.
    pub model: String,
    /// Input tokens.
    #[serde(default)]
    pub input_tokens: Option<i64>,
    /// Output tokens.
    #[serde(default)]
    pub output_tokens: Option<i64>,
    /// Cost in cents.
    pub cost_cents: i64,
    /// ISO 8601 occurrence time.
    pub occurred_at: String,
}

/// Body for `POST /api/companies/{companyId}/budget`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBudgetRequest {
    /// Monthly budget in cents (0 = unlimited).
    pub budget_monthly_cents: i64,
}

fn validate_event(body: &CreateCostEventRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if !is_uuid(&body.agent_id) {
        issues.push(json!({ "path": ["agentId"], "message": "Invalid uuid" }));
    }
    if let Some(issue_id) = &body.issue_id
        && !is_uuid(issue_id)
    {
        issues.push(json!({ "path": ["issueId"], "message": "Invalid uuid" }));
    }
    if body.provider.trim().is_empty() {
        issues.push(json!({ "path": ["provider"], "message": "String must contain at least 1 character(s)" }));
    }
    if body.model.trim().is_empty() {
        issues.push(
            json!({ "path": ["model"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.cost_cents < 0 {
        issues.push(json!({ "path": ["costCents"], "message": "Number must be greater than or equal to 0" }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `POST /api/companies/{companyId}/cost-events` — records a cost event and
/// applies the hard-stop rule. Returns 201 with the event plus hard-stop info.
#[route(POST "/api/companies/{company_id}/cost-events")]
pub async fn create_cost_event(
    cx: &Cx,
    Json(body): Json<CreateCostEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    validate_event(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let company_id_for_log = company_id.clone();
    let state = app_context::<AppState>(cx);
    let outcome = state
        .costs
        .create_event(NewCostEvent {
            company_id,
            agent_id: body.agent_id,
            issue_id: body.issue_id,
            billing_code: body.billing_code,
            provider: body.provider,
            model: body.model,
            input_tokens: body.input_tokens.unwrap_or(0),
            output_tokens: body.output_tokens.unwrap_or(0),
            cost_cents: body.cost_cents,
            occurred_at: body.occurred_at,
        })
        .await
        .map_err(cost_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id_for_log,
        "cost_event.recorded",
        "cost_event",
        &outcome.event.id,
        Some(json!({
            "agentId": outcome.event.agent_id,
            "costCents": outcome.event.cost_cents,
            "hardStopTriggered": outcome.hard_stop_triggered,
        })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "event": CostEventDto::from(outcome.event),
            "hardStop": {
                "triggered": outcome.hard_stop_triggered,
                "pausedAgentIds": outcome.paused_agent_ids,
            },
        })),
    ))
}

/// `GET /api/companies/{companyId}/costs/summary` — budget summary.
#[route(GET "/api/companies/{company_id}/costs/summary")]
pub async fn cost_summary(cx: &Cx) -> Result<Json<BudgetSummaryDto>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .costs
        .summary(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(summary) => Ok(Json(summary.into())),
        None => Err(ApiError::not_found("Company not found")),
    }
}

/// `GET /api/companies/{companyId}/costs/by-agent` — per-agent spending.
#[route(GET "/api/companies/{company_id}/costs/by-agent")]
pub async fn costs_by_agent(cx: &Cx) -> Result<Json<Vec<AgentCostRowDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let rows = state
        .costs
        .by_agent(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows.into_iter().map(AgentCostRowDto::from).collect()))
}

/// `POST /api/companies/{companyId}/budget` — sets the monthly budget.
#[route(POST "/api/companies/{company_id}/budget")]
pub async fn set_budget(
    cx: &Cx,
    Json(body): Json<SetBudgetRequest>,
) -> Result<Json<BudgetSummaryDto>, ApiError> {
    require_board(cx)?;
    if body.budget_monthly_cents < 0 {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["budgetMonthlyCents"], "message": "Number must be greater than or equal to 0" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .costs
        .set_company_budget(&company_id, body.budget_monthly_cents)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(summary) => {
            log_activity(
                &state.activity,
                &company_id,
                "budget.set",
                "company",
                &company_id,
                Some(json!({ "budgetMonthlyCents": summary.budget_monthly_cents })),
            )
            .await?;
            Ok(Json(summary.into()))
        }
        None => Err(ApiError::not_found("Company not found")),
    }
}

/// `POST /api/companies/{companyId}/budget/reset` — resets spending and
/// resumes budget-paused agents.
#[route(POST "/api/companies/{company_id}/budget/reset")]
pub async fn reset_budget(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .costs
        .reset_company_spending(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(resumed) => Ok(Json(
            json!({ "spendingReset": true, "resumedAgents": resumed }),
        )),
        None => Err(ApiError::not_found("Company not found")),
    }
}

fn cost_error_to_api(error: staple_data::CostError) -> ApiError {
    use staple_data::CostError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::AgentNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["agentId"], "message": "Agent not found" }]),
        ),
        E::IssueInDifferentCompany => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["issueId"], "message": "Issue belongs to a different company" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
