//! Board API keys and CLI auth challenges routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{BoardApiKeyRecord, CliAuthChallengeRecord, NewBoardApiKey, NewCliAuthChallenge};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    auth::require_board,
    error::ApiError,
    routes::{Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/board-api-keys`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardKeyRequest {
    /// Owning user id.
    pub user_id: String,
    /// Display name.
    pub name: String,
    /// ISO 8601 expiry (optional).
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Body for `POST /api/cli-auth-challenges`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCliChallengeRequest {
    /// Command text.
    pub command: String,
    /// Client name.
    #[serde(default)]
    pub client_name: Option<String>,
    /// Requested access (`board`).
    #[serde(default)]
    pub requested_access: Option<String>,
    /// Requested company id.
    #[serde(default)]
    pub requested_company_id: Option<String>,
    /// Pending key name.
    pub pending_key_name: String,
    /// ISO 8601 expiry.
    #[serde(default)]
    pub expires_at: Option<String>,
}

fn default_expiry() -> String {
    "2999-01-01T00:00:00.000Z".to_owned()
}

/// `POST /api/board-api-keys` — creates a board API key (returns plaintext).
#[route(POST "/api/board-api-keys")]
pub async fn create_board_key(
    cx: &Cx,
    Json(body): Json<CreateBoardKeyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    require_board(cx)?;
    if body.name.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["name"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let (key, plaintext) = state
        .board_keys
        .create_key(NewBoardApiKey {
            user_id: body.user_id,
            name: body.name,
            expires_at: body.expires_at,
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "key": key, "plaintext": plaintext })),
    ))
}

/// `GET /api/board-api-keys` — lists board API keys.
#[route(GET "/api/board-api-keys")]
pub async fn list_board_keys(cx: &Cx) -> Result<Json<Vec<BoardApiKeyRecord>>, ApiError> {
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let keys = state
        .board_keys
        .list_keys()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(keys))
}

/// `POST /api/board-api-keys/{id}/revoke` — revokes a board API key.
#[route(POST "/api/board-api-keys/{id}/revoke")]
pub async fn revoke_board_key(cx: &Cx) -> Result<Json<BoardApiKeyRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    if !is_uuid(&id) {
        return Err(ApiError::bad_request("Invalid id"));
    }
    let state = app_context::<AppState>(cx);
    let key = state
        .board_keys
        .revoke_key(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Board API key not found"))?;
    Ok(Json(key))
}

/// `POST /api/cli-auth-challenges` — creates a CLI auth challenge.
#[route(POST "/api/cli-auth-challenges")]
pub async fn create_cli_challenge(
    cx: &Cx,
    Json(body): Json<CreateCliChallengeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    require_board(cx)?;
    if body.command.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["command"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let requested_access = body.requested_access.unwrap_or_else(|| "board".to_owned());
    if requested_access != "board" {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["requestedAccess"], "message": "Invalid enum value. Expected 'board'" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let (challenge, secret) = state
        .board_keys
        .create_challenge(NewCliAuthChallenge {
            command: body.command,
            client_name: body.client_name,
            requested_access,
            requested_company_id: body.requested_company_id,
            pending_key_name: body.pending_key_name,
            expires_at: body.expires_at.unwrap_or_else(default_expiry),
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "challenge": challenge, "secret": secret })),
    ))
}

/// `GET /api/cli-auth-challenges` — lists CLI auth challenges.
#[route(GET "/api/cli-auth-challenges")]
pub async fn list_cli_challenges(cx: &Cx) -> Result<Json<Vec<CliAuthChallengeRecord>>, ApiError> {
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let challenges = state
        .board_keys
        .list_challenges()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(challenges))
}

/// `POST /api/cli-auth-challenges/{id}/approve` — approves a challenge and
/// materializes the pending board API key.
#[route(POST "/api/cli-auth-challenges/{id}/approve")]
pub async fn approve_cli_challenge(cx: &Cx) -> Result<Json<CliAuthChallengeRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    if !is_uuid(&id) {
        return Err(ApiError::bad_request("Invalid id"));
    }
    let state = app_context::<AppState>(cx);
    let challenge = state
        .board_keys
        .approve_challenge(&id, None)
        .await
        .map_err(board_key_error_to_api)?;
    Ok(Json(challenge))
}

/// `POST /api/cli-auth-challenges/{id}/cancel` — cancels a challenge.
#[route(POST "/api/cli-auth-challenges/{id}/cancel")]
pub async fn cancel_cli_challenge(cx: &Cx) -> Result<Json<CliAuthChallengeRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    if !is_uuid(&id) {
        return Err(ApiError::bad_request("Invalid id"));
    }
    let state = app_context::<AppState>(cx);
    let challenge = state
        .board_keys
        .cancel_challenge(&id)
        .await
        .map_err(board_key_error_to_api)?;
    Ok(Json(challenge))
}

fn board_key_error_to_api(error: staple_data::BoardKeyError) -> ApiError {
    use staple_data::BoardKeyError as E;
    match error {
        E::ChallengeNotFound => ApiError::not_found("Challenge not found or not pending"),
        other => ApiError::internal(other.to_string()),
    }
}
