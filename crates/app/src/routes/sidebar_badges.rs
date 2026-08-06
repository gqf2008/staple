//! Sidebar badge counts (upstream `routes/sidebar-badges.ts` parity).

use serde::Serialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, path_param, route},
};

use crate::{
    auth::{enforce_company_scope, require_board},
    error::ApiError,
    routes::CompanyId,
    state::AppState,
};

/// Sidebar badge counts (upstream `SidebarBadges`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarBadges {
    /// Combined inbox badge (approvals + failed runs + join requests).
    pub inbox: usize,
    /// Actionable approvals.
    pub approvals: usize,
    /// Latest failed/timed-out runs per agent.
    pub failed_runs: usize,
    /// Pending join requests.
    pub join_requests: usize,
}

const ACTIONABLE_APPROVAL_STATUSES: [&str; 2] = ["pending", "revision_requested"];
const FAILED_RUN_STATUSES: [&str; 2] = ["failed", "timed_out"];

/// Computes sidebar badge counts for a company.
///
/// # Errors
///
/// Returns [`ApiError`] on database failure.
pub async fn sidebar_badges_for(
    state: &AppState,
    company_id: &str,
) -> Result<SidebarBadges, ApiError> {
    // Actionable approvals.
    let approvals = state
        .approvals
        .list(company_id, None)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .into_iter()
        .filter(|approval| ACTIONABLE_APPROVAL_STATUSES.contains(&approval.status.as_str()))
        .count();

    // Latest run per agent; count failed/timed-out ones.
    let runs = state
        .heartbeat
        .list(company_id, None, 100_000)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let mut latest_by_agent: std::collections::HashMap<&str, &staple_data::HeartbeatRunRecord> =
        std::collections::HashMap::new();
    for run in runs.iter() {
        let current = latest_by_agent.get(run.agent_id.as_str()).copied();
        if current.is_none_or(|existing| run.created_at > existing.created_at) {
            latest_by_agent.insert(run.agent_id.as_str(), run);
        }
    }
    let failed_runs = latest_by_agent
        .values()
        .filter(|run| FAILED_RUN_STATUSES.contains(&run.status.as_str()))
        .count();

    // Pending join requests.
    let join_requests = state
        .invites
        .list_join_requests(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .into_iter()
        .filter(|request| request.status == "pending_approval")
        .count();

    let inbox = approvals + failed_runs + join_requests;
    Ok(SidebarBadges {
        inbox,
        approvals,
        failed_runs,
        join_requests,
    })
}

/// `GET /api/companies/{companyId}/sidebar-badges` — sidebar badge counts.
#[route(GET "/api/companies/{company_id}/sidebar-badges")]
pub async fn get_sidebar_badges(cx: &Cx) -> Result<Json<SidebarBadges>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    Ok(Json(sidebar_badges_for(state, &company_id).await?))
}
