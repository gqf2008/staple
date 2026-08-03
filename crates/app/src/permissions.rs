//! Permission-matrix authorization helpers (upstream SPEC §9.3/§9.8).
//!
//! Board principals are always allowed. Agent principals are checked against
//! `principal_permission_grants` where the matrix requires a grant; the
//! default-open behaviors (simple company-wide assignment, own-inbox
//! management) remain so existing agent keys keep working.

use serde_json::json;
use topcoat::context::Cx;

use staple_domain::{agent_is_in_subtree, scope_allows};

use crate::{
    auth::{Principal, current_principal, enforce_company_scope},
    error::ApiError,
    state::AppState,
};

/// Authorizes a task assignment for the current principal.
///
/// Semantics (SPEC §9.8):
/// - board: always allowed;
/// - agent with a broad `tasks:assign` grant: allowed;
/// - agent with only `tasks:assign_scope`: allowed only when the requested
///   scope (project/assignee agent) matches every constraint family in the
///   grant scope; a missing structured scope is a denial;
/// - agent with no assignment grant: simple company-wide default (kept for
///   compatibility with upstream's standard agent default).
///
/// Denials return a generic explanation and never disclose which constraint
/// failed or anything about unrelated resources.
///
/// # Errors
///
/// Returns 403 when the grant does not cover the requested scope.
pub async fn authorize_assignment(
    state: &AppState,
    cx: &Cx,
    company_id: &str,
    requested_scope: serde_json::Value,
) -> Result<(), ApiError> {
    let Principal::Agent(agent) = current_principal(cx) else {
        return Ok(());
    };
    enforce_company_scope(cx, company_id)?;

    let broad = state
        .permission_grants
        .find(company_id, "agent", &agent.agent_id, "tasks:assign")
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    if broad.is_some() {
        return Ok(());
    }

    let scoped = state
        .permission_grants
        .find(company_id, "agent", &agent.agent_id, "tasks:assign_scope")
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let Some(scoped) = scoped else {
        // No assignment grant: standard agent default (company-wide).
        return Ok(());
    };

    let hierarchy = state
        .agents
        .hierarchy(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .into_iter()
        .map(|row| staple_domain::AgentHierarchyRow {
            id: row.id,
            reports_to: row.reports_to,
        })
        .collect::<Vec<_>>();
    if scope_allows(
        scoped.scope.as_ref(),
        Some(&requested_scope),
        true,
        Some(&hierarchy),
    ) {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "Permission tasks:assign_scope does not cover the requested scope.",
        ))
    }
}

/// Authorizes managing a user's inbox state for the current principal.
///
/// Board is always allowed. Agents use the default-open policy: no
/// `inbox:manage` grant means the request proceeds (own-inbox default); a
/// grant must cover the target user through `userId`/`userIds` scope.
///
/// # Errors
///
/// Returns 403 when an `inbox:manage` grant does not cover the target user.
pub async fn authorize_inbox_manage(
    state: &AppState,
    cx: &Cx,
    company_id: &str,
    target_user_id: &str,
) -> Result<(), ApiError> {
    let Principal::Agent(agent) = current_principal(cx) else {
        return Ok(());
    };
    enforce_company_scope(cx, company_id)?;
    let Some(grant) = state
        .permission_grants
        .find(company_id, "agent", &agent.agent_id, "inbox:manage")
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    else {
        return Ok(());
    };
    if scope_allows(
        grant.scope.as_ref(),
        Some(&json!({ "userId": target_user_id })),
        false,
        None,
    ) {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "Permission inbox:manage does not cover the requested user.",
        ))
    }
}

/// Authorizes setting a subordinate agent's budget (SPEC §9.3).
///
/// Board may set any agent budget. An agent may only set the budget of an
/// agent inside its own `reports_to` subtree (manager subtree only).
///
/// # Errors
///
/// Returns 403 when the target agent is outside the actor's managed subtree.
pub async fn authorize_subordinate_budget(
    state: &AppState,
    cx: &Cx,
    company_id: &str,
    target_agent_id: &str,
) -> Result<(), ApiError> {
    let Principal::Agent(agent) = current_principal(cx) else {
        return Ok(());
    };
    enforce_company_scope(cx, company_id)?;

    let hierarchy = state
        .agents
        .hierarchy(company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .into_iter()
        .map(|row| staple_domain::AgentHierarchyRow {
            id: row.id,
            reports_to: row.reports_to,
        })
        .collect::<Vec<_>>();
    if agent_is_in_subtree(&hierarchy, &agent.agent_id, target_agent_id) {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "Agent can only set budgets for its managed subtree.",
        ))
    }
}
