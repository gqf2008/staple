//! Adapter registry routes: discovery plus invoke/observe/cancel passthrough.

use serde::Deserialize;
use serde_json::json;
use staple_adapters::InvocationInput;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{error::ApiError, routes::Id, state::AppState};

/// Body for `POST /api/adapters/{type}/invoke`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeRequest {
    /// Task instructions.
    pub task: String,
}

/// `GET /api/adapters` — lists registered adapter types.
#[route(GET "/api/adapters")]
pub async fn list_adapters(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let state = app_context::<AppState>(cx);
    Ok(Json(json!({ "adapters": state.adapters.names() })))
}

/// `POST /api/adapters/{type}/invoke` — invokes a run through an adapter.
#[route(POST "/api/adapters/{type}/invoke")]
pub async fn invoke_adapter(
    cx: &Cx,
    Json(body): Json<InvokeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let adapter_type = path_param::<Type>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let adapter = state
        .adapters
        .get(&adapter_type)
        .ok_or_else(|| ApiError::not_found("Adapter not found"))?;
    let handle = adapter
        .invoke(InvocationInput {
            task: body.task,
            cwd: None,
            env: vec![],
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&handle).unwrap_or_default()),
    ))
}

/// `GET /api/adapters/{type}/runs/{runId}` — observes a run.
#[route(GET "/api/adapters/{type}/runs/{id}")]
pub async fn observe_adapter_run(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let adapter_type = path_param::<Type>(cx)?.to_string();
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let adapter = state
        .adapters
        .get(&adapter_type)
        .ok_or_else(|| ApiError::not_found("Adapter not found"))?;
    let status = adapter
        .observe(&run_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&status).unwrap_or_default()))
}

/// `POST /api/adapters/{type}/runs/{runId}/cancel` — cancels a run.
#[route(POST "/api/adapters/{type}/runs/{id}/cancel")]
pub async fn cancel_adapter_run(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let adapter_type = path_param::<Type>(cx)?.to_string();
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let adapter = state
        .adapters
        .get(&adapter_type)
        .ok_or_else(|| ApiError::not_found("Adapter not found"))?;
    adapter
        .cancel(&run_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(json!({ "cancelled": true })))
}

/// Shared `{type}` path parameter for adapters.
#[path_param(error = bad_request("Invalid adapter type"))]
pub(crate) struct Type(String);
