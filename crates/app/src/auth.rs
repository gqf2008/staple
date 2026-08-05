//! Request authentication and authorization.
//!
//! Identity model (aligned with SPEC §9):
//! - requests without credentials act as the board (local-implicit mode)
//! - `Authorization: Bearer sk-...` authenticates an agent via its API key
//! - invalid/revoked keys get 401
//! - agent principals are company-scoped: cross-company paths return 403

use staple_data::{AgentPrincipal, BoardApiKeyRecord};

use crate::error::ApiError;

/// The authenticated principal for the current request.
#[derive(Debug, Clone)]
pub enum Principal {
    /// Board operator (local-implicit).
    Board,
    /// Board operator authenticated with a board API key.
    BoardKey(BoardApiKeyRecord),
    /// Authenticated agent.
    Agent(AgentPrincipal),
}

impl Principal {
    /// The agent's company id, when the principal is an agent.
    #[must_use]
    pub fn agent_company_id(&self) -> Option<&str> {
        match self {
            Self::Board | Self::BoardKey(_) => None,
            Self::Agent(agent) => Some(&agent.company_id),
        }
    }

    /// Whether the principal is an agent.
    #[must_use]
    pub fn is_agent(&self) -> bool {
        matches!(self, Self::Agent(_))
    }

    /// Whether the principal carries board authority (local board or key).
    #[must_use]
    pub fn is_board(&self) -> bool {
        matches!(self, Self::Board | Self::BoardKey(_))
    }
}

/// Reads the principal from the request context (defaults to board).
#[must_use]
pub fn current_principal(cx: &topcoat::context::Cx) -> Principal {
    topcoat::context::try_request_context::<Principal>(cx)
        .cloned()
        .unwrap_or(Principal::Board)
}

/// Rejects agent principals that are not acting on their own company.
///
/// # Errors
///
/// Returns 403 when an agent targets another company.
pub fn enforce_company_scope(cx: &topcoat::context::Cx, company_id: &str) -> Result<(), ApiError> {
    match current_principal(cx) {
        Principal::Board | Principal::BoardKey(_) => Ok(()),
        Principal::Agent(agent) if agent.company_id == company_id => Ok(()),
        Principal::Agent(_) => Err(ApiError::forbidden(
            "Agent key cannot access another company",
        )),
    }
}

/// Rejects agent principals for board-only actions.
///
/// # Errors
///
/// Returns 403 when an agent performs a board-only action.
pub fn require_board(cx: &topcoat::context::Cx) -> Result<(), ApiError> {
    if current_principal(cx).is_board() {
        Ok(())
    } else {
        Err(ApiError::forbidden("Board access required"))
    }
}

/// Resolves the current board operator identity for UI form actions.
///
/// UI-initiated operations record this actor instead of a hardcoded
/// `"board"` string. Resolution order for board principals:
/// 1. `X-Board-User` request header;
/// 2. `?user=` query parameter;
/// 3. `"board"` fallback, preserving the previous behavior.
///
/// Agent principals always resolve to `"board"`: agent operations are
/// recorded through their own `*_agent_id` fields on API routes, and an
/// agent key must not be able to claim a user identity (the header/query
/// are ignored for agents).
#[must_use]
pub fn current_actor(cx: &topcoat::context::Cx) -> String {
    let Some(parts) = topcoat::context::try_request_context::<http::request::Parts>(cx) else {
        return "board".to_owned();
    };
    match current_principal(cx) {
        Principal::Agent(_) => "board".to_owned(),
        Principal::Board | Principal::BoardKey(_) => parts
            .headers
            .get("x-board-user")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                parts.uri.query().and_then(|query| {
                    query.split('&').find_map(|pair| {
                        let (key, value) = pair.split_once('=')?;
                        (key == "user").then_some(value)
                    })
                })
            })
            .unwrap_or("board")
            .to_owned(),
    }
}
