//! Agent API key routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::NewAgentApiKey;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::require_board,
    dto::AgentApiKeyDto,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/agent-api-keys`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    /// Agent id.
    pub agent_id: String,
    /// Display name.
    pub name: String,
}

/// `POST /api/companies/{companyId}/agent-api-keys` — creates a key; the
/// plaintext is returned once.
#[route(POST "/api/companies/{company_id}/agent-api-keys")]
pub async fn create_api_key(
    cx: &Cx,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    require_board(cx)?;
    let mut issues = Vec::new();
    if !is_uuid(&body.agent_id) {
        issues.push(json!({ "path": ["agentId"], "message": "Invalid uuid" }));
    }
    if body.name.trim().is_empty() {
        issues.push(
            json!({ "path": ["name"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let (record, plaintext) = state
        .api_keys
        .create_key(NewAgentApiKey {
            company_id: company_id.clone(),
            agent_id: body.agent_id,
            name: body.name,
        })
        .await
        .map_err(api_key_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "agent_api_key.created",
        "agent_api_key",
        &record.id,
        Some(json!({ "agentId": record.agent_id })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "key": AgentApiKeyDto::from(record),
            "plaintext": plaintext,
        })),
    ))
}

/// `GET /api/companies/{companyId}/agent-api-keys` — lists keys (hash only).
#[route(GET "/api/companies/{company_id}/agent-api-keys")]
pub async fn list_api_keys(cx: &Cx) -> Result<Json<Vec<AgentApiKeyDto>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let keys = state
        .api_keys
        .list_keys(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(keys.into_iter().map(AgentApiKeyDto::from).collect()))
}

/// `POST /api/agent-api-keys/{keyId}/revoke` — revokes a key.
#[route(POST "/api/agent-api-keys/{id}/revoke")]
pub async fn revoke_api_key(cx: &Cx) -> Result<Json<AgentApiKeyDto>, ApiError> {
    require_board(cx)?;
    let key_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .api_keys
        .revoke_key(&key_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(key) => {
            log_activity(
                &state.activity,
                &key.company_id,
                "agent_api_key.revoked",
                "agent_api_key",
                &key.id,
                None,
            )
            .await?;
            Ok(Json(key.into()))
        }
        None => Err(ApiError::not_found("API key not found")),
    }
}

fn api_key_error_to_api(error: staple_data::ApiKeyError) -> ApiError {
    use staple_data::ApiKeyError as E;
    match error {
        E::AgentNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["agentId"], "message": "Agent not found" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
