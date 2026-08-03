//! Agent routes (minimal: subordinate budgets; CRUD arrives with #56).

use serde::Deserialize;
use serde_json::json;
use staple_data::AgentBudgetRecord;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    error::ApiError,
    permissions::authorize_subordinate_budget,
    routes::{AgentId, CompanyId},
    state::AppState,
};

/// Body for `PATCH /api/agents/{agentId}/budgets`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAgentBudgetRequest {
    /// Monthly budget in cents (0 = unlimited).
    pub budget_monthly_cents: i64,
}

/// `PATCH /api/agents/{agentId}/budgets` — sets an agent's monthly budget.
///
/// Board may set any agent budget; an agent principal may only set the budget
/// of an agent inside its own managed subtree (SPEC §9.3).
#[route(PATCH "/api/agents/{agent_id}/budgets")]
pub async fn set_agent_budget(
    cx: &Cx,
    Json(body): Json<SetAgentBudgetRequest>,
) -> Result<Json<AgentBudgetRecord>, ApiError> {
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    if body.budget_monthly_cents < 0 {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["budgetMonthlyCents"], "message": "Number must be greater than or equal to 0" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let company_id = state
        .agents
        .company_of(&agent_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Agent not found"))?;
    authorize_subordinate_budget(state, cx, &company_id, &agent_id).await?;
    let record = state
        .agents
        .set_budget(&company_id, &agent_id, body.budget_monthly_cents)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .expect("agent exists");
    log_activity(
        &state.activity,
        &company_id,
        "agent.budget_updated",
        "agent",
        &agent_id,
        Some(json!({ "budgetMonthlyCents": record.budget_monthly_cents })),
    )
    .await?;
    Ok(Json(record))
}

/// `GET /api/companies/{companyId}/agents` — lists agents.
#[route(GET "/api/companies/{company_id}/agents")]
pub async fn list_agents(cx: &Cx) -> Result<Json<Vec<staple_data::AgentRecord>>, ApiError> {
    crate::auth::enforce_company_scope(cx, &path_param::<CompanyId>(cx)?.to_string())?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let agents = state
        .agents
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(agents))
}

/// `GET /api/agents/{id}` — fetches one agent.
#[route(GET "/api/agents/{agent_id}")]
pub async fn get_agent(cx: &Cx) -> Result<Json<staple_data::AgentRecord>, ApiError> {
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = state
        .agents
        .company_of(&agent_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Agent not found"))?;
    crate::auth::enforce_company_scope(cx, &company_id)?;
    state
        .agents
        .get(&company_id, &agent_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Agent not found"))
}

/// Body for `PATCH /api/agents/{id}/status`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAgentStatusRequest {
    /// New status (`active` | `paused` | `terminated` | ...).
    pub status: String,
    /// Pause reason (`null` clears).
    #[serde(default)]
    pub pause_reason: Option<Option<String>>,
}

/// `PATCH /api/agents/{id}/status` — pauses/resumes/terminates an agent
/// (board-only; managers may only change their own subtree).
#[route(PATCH "/api/agents/{agent_id}/status")]
pub async fn set_agent_status(
    cx: &Cx,
    Json(body): Json<SetAgentStatusRequest>,
) -> Result<Json<staple_data::AgentRecord>, ApiError> {
    if !matches!(
        body.status.as_str(),
        "active" | "paused" | "terminated" | "idle" | "error"
    ) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{
                "path": ["status"],
                "message": "Invalid enum value. Expected 'active' | 'paused' | 'terminated' | 'idle' | 'error'",
            }]),
        ));
    }
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = state
        .agents
        .company_of(&agent_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Agent not found"))?;
    crate::auth::enforce_company_scope(cx, &company_id)?;
    if body.status != "active" {
        // Status changes other than resume are board-governed.
        crate::auth::require_board(cx)?;
    }
    let record = state
        .agents
        .update_status(&company_id, &agent_id, &body.status, body.pause_reason)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .expect("agent exists");
    log_activity(
        &state.activity,
        &company_id,
        "agent.status_updated",
        "agent",
        &agent_id,
        Some(json!({ "status": record.status })),
    )
    .await?;
    Ok(Json(record))
}
