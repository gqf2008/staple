//! External object routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::NewExternalObject;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{error::ApiError, routes::Id, state::AppState};

/// Body for `POST /api/issues/{issueId}/external-objects`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExternalObjectRequest {
    /// Kind.
    pub kind: String,
    /// External id.
    pub external_id: String,
    /// URL.
    #[serde(default)]
    pub url: Option<String>,
    /// Metadata.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// `GET /api/issues/{issueId}/external-objects` — lists links.
#[route(GET "/api/issues/{id}/external-objects")]
pub async fn list_external_objects(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let objects = state
        .external_objects
        .list_for_issue(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&objects).unwrap_or_default()))
}

/// `POST /api/issues/{issueId}/external-objects` — links an external object.
#[route(POST "/api/issues/{id}/external-objects")]
pub async fn create_external_object(
    cx: &Cx,
    Json(body): Json<CreateExternalObjectRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.kind.trim().is_empty() {
        issues.push(
            json!({ "path": ["kind"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.external_id.trim().is_empty() {
        issues.push(json!({ "path": ["externalId"], "message": "String must contain at least 1 character(s)" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let object = state
        .external_objects
        .create(NewExternalObject {
            issue_id,
            kind: body.kind,
            external_id: body.external_id,
            url: body.url,
            metadata: body.metadata.map(|value| value.to_string()),
        })
        .await
        .map_err(external_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&object).unwrap_or_default()),
    ))
}

/// `POST /api/external-objects/{id}/refresh` — refreshes status.
#[route(POST "/api/external-objects/{id}/refresh")]
pub async fn refresh_external_object(
    cx: &Cx,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .external_objects
        .refresh(&id, &body.status)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(object) => Ok(Json(serde_json::to_value(&object).unwrap_or_default())),
        None => Err(ApiError::not_found("External object not found")),
    }
}

/// Body for refresh.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    /// New status.
    pub status: String,
}

fn external_error_to_api(error: staple_data::ExternalObjectError) -> ApiError {
    use staple_data::ExternalObjectError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::AlreadyExists => ApiError::conflict("External object link already exists"),
        other => ApiError::internal(other.to_string()),
    }
}
