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
    audit::log_activity, error::ApiError, permissions::authorize_subordinate_budget,
    routes::AgentId, state::AppState,
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
