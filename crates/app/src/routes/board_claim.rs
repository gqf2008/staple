//! Board ownership claim routes (`GET/POST /api/board-claim/{token}`).

use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    auth::require_board,
    board_claim::{ClaimError, ClaimStatus, LOCAL_BOARD_USER_ID},
    error::ApiError,
    state::AppState,
};

/// `{token}` path parameter for board-claim routes.
#[path_param(error = bad_request("Invalid token"))]
struct Token(String);

/// Body for `POST /api/board-claim/{token}/claim`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimRequest {
    /// Claim code from the challenge URL.
    pub code: String,
}

fn query_param(cx: &Cx, key: &str) -> Option<String> {
    topcoat::context::try_request_context::<http::request::Parts>(cx).and_then(|parts| {
        parts.uri.query().and_then(|query| {
            query.split('&').find_map(|pair| {
                let (k, value) = pair.split_once('=')?;
                (k == key).then(|| value.to_owned())
            })
        })
    })
}

/// `GET /api/board-claim/{token}?code=` — challenge status.
#[route(GET "/api/board-claim/{token}")]
pub async fn board_claim_status(cx: &Cx) -> Result<Json<ClaimStatus>, ApiError> {
    let token = path_param::<Token>(cx)?.to_string();
    let code = query_param(cx, "code");
    let state = app_context::<AppState>(cx);
    let status = state.board_claim.inspect(&token, code.as_deref());
    if status.status == "invalid" {
        return Err(ApiError::not_found("Board claim challenge not found"));
    }
    Ok(Json(status))
}

/// `POST /api/board-claim/{token}/claim` — claims board ownership.
#[route(POST "/api/board-claim/{token}/claim")]
pub async fn board_claim_claim(
    cx: &Cx,
    Json(body): Json<ClaimRequest>,
) -> Result<(StatusCode, Json<ClaimStatus>), ApiError> {
    require_board(cx)?;
    let token = path_param::<Token>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let actor = crate::auth::current_actor(cx);
    let claimed = state
        .board_claim
        .claim(&token, &body.code, &actor)
        .map_err(|error| match error {
            ClaimError::Invalid => ApiError::not_found("Board claim challenge not found"),
            ClaimError::Expired => ApiError::conflict("Board claim challenge expired"),
            ClaimError::Claimed => {
                ApiError::conflict("Board claim challenge is no longer available")
            }
        })?;
    if claimed.status == "claimed" && claimed.claimed_by_user_id.as_deref() == Some(&actor) {
        // Promote the claiming user to instance admin (upstream claimBoardOwnership).
        let _ = state
            .memberships
            .upsert_role(staple_data::NewInstanceUserRole {
                user_id: actor.clone(),
                role: "instance_admin".to_owned(),
            })
            .await;
    }
    if claimed.status == "claimed" {
        return Ok((StatusCode::OK, Json(claimed)));
    }
    Err(ApiError::conflict(
        "Board claim challenge is no longer available",
    ))
}

/// `POST /board-claim/{token}/claim/ui` — UI form claim, redirects back.
#[route(POST "/board-claim/{token}/claim/ui")]
pub async fn board_claim_claim_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    require_board(cx)?;
    let token = path_param::<Token>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let actor = crate::auth::current_actor(cx);
    // The UI form posts without a body; use the seeded challenge code via
    // inspect (the code is embedded in the page URL query).
    let code = query_param(cx, "code").unwrap_or_default();
    let _ = state.board_claim.claim(&token, &code, &actor);
    let _ = state
        .memberships
        .upsert_role(staple_data::NewInstanceUserRole {
            user_id: if actor == "board" {
                LOCAL_BOARD_USER_ID.to_owned()
            } else {
                actor
            },
            role: "instance_admin".to_owned(),
        })
        .await;
    Ok(topcoat::router::error::see_other(&format!(
        "/board-claim/{token}?code={code}"
    )))
}
