//! Issue document routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewIssueDocument, UpdateIssueDocument};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{audit::log_activity, dto::DocumentDto, error::ApiError, routes::Id, state::AppState};

/// Body for `POST /api/issues/{issueId}/documents`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentRequest {
    /// Stable workflow key (`plan`, `design`, `notes`, ...).
    pub key: String,
    /// Optional title.
    #[serde(default)]
    pub title: Option<String>,
    /// Initial body.
    pub body: String,
}

/// Body for `PATCH /api/issues/{issueId}/documents/{key}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentRequest {
    /// New body.
    pub body: String,
    /// Optional change summary.
    #[serde(default)]
    pub change_summary: Option<String>,
}

fn validate_key(key: &str) -> Result<(), ApiError> {
    if key.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["key"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    Ok(())
}

/// `GET /api/issues/{issueId}/documents` — lists an issue's documents.
#[route(GET "/api/issues/{id}/documents")]
pub async fn list_documents(cx: &Cx) -> Result<Json<Vec<DocumentDto>>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let documents = state
        .documents
        .list_issue_documents(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(documents.into_iter().map(DocumentDto::from).collect()))
}

/// `POST /api/issues/{issueId}/documents` — creates a document, returns 201.
#[route(POST "/api/issues/{id}/documents")]
pub async fn create_document(
    cx: &Cx,
    Json(body): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<DocumentDto>), ApiError> {
    validate_key(&body.key)?;
    if body.body.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["body"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let issue_id_for_log = issue_id.clone();
    let state = app_context::<AppState>(cx);
    let document = state
        .documents
        .create_issue_document(NewIssueDocument {
            issue_id,
            key: body.key.trim().to_lowercase(),
            title: body.title,
            body: body.body,
            created_by_user_id: None,
        })
        .await
        .map_err(document_error_to_api)?;
    log_activity(
        &state.activity,
        &document.company_id,
        "document.created",
        "document",
        &document.id,
        Some(json!({ "issueId": issue_id_for_log })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(document.into())))
}

/// `GET /api/issues/{issueId}/documents/{key}` — fetches one document.
#[route(GET "/api/issues/{id}/documents/{key}")]
pub async fn get_document(cx: &Cx) -> Result<Json<DocumentDto>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let key = path_param::<Key>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .documents
        .get_issue_document_by_key(&issue_id, &key)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(document) => Ok(Json(document.into())),
        None => Err(ApiError::not_found("Document not found")),
    }
}

/// `PATCH /api/issues/{issueId}/documents/{key}` — appends a revision.
#[route(PATCH "/api/issues/{id}/documents/{key}")]
pub async fn update_document(
    cx: &Cx,
    Json(body): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentDto>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let key = path_param::<Key>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let document = state
        .documents
        .update_issue_document(UpdateIssueDocument {
            issue_id,
            key,
            body: body.body,
            change_summary: body.change_summary,
            updated_by_user_id: None,
        })
        .await
        .map_err(document_error_to_api)?;
    log_activity(
        &state.activity,
        &document.company_id,
        "document.updated",
        "document",
        &document.id,
        Some(json!({ "revision": document.latest_revision_number })),
    )
    .await?;
    Ok(Json(document.into()))
}

/// Shared `{key}` path parameter for documents.
#[path_param(error = bad_request("Invalid document key"))]
pub(crate) struct Key(String);

fn document_error_to_api(error: staple_data::DocumentError) -> ApiError {
    use staple_data::DocumentError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::KeyExists => ApiError::conflict("Document key already exists on this issue"),
        E::DocumentNotFound => ApiError::not_found("Document not found"),
        other => ApiError::internal(other.to_string()),
    }
}
