//! Issue comment routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::NewIssueComment;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{dto::IssueCommentDto, error::ApiError, routes::Id, state::AppState};

/// Body for `POST /api/issues/{issueId}/comments`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCommentRequest {
    /// Comment body (required, non-empty).
    #[serde(default)]
    pub body: Option<String>,
    /// Author user id.
    #[serde(default)]
    pub author_user_id: Option<String>,
}

/// `GET /api/issues/{issueId}/comments` — lists an issue's comments.
#[route(GET "/api/issues/{id}/comments")]
pub async fn list_comments(cx: &Cx) -> Result<Json<Vec<IssueCommentDto>>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let comments = state
        .comments
        .list(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        comments.into_iter().map(IssueCommentDto::from).collect(),
    ))
}

/// `POST /api/issues/{issueId}/comments` — adds a comment, returns 201.
#[route(POST "/api/issues/{id}/comments")]
pub async fn add_comment(
    cx: &Cx,
    Json(body): Json<AddCommentRequest>,
) -> Result<(StatusCode, Json<IssueCommentDto>), ApiError> {
    let Some(body_text) = body.body.filter(|value| !value.trim().is_empty()) else {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["body"], "message": "String must contain at least 1 character(s)" }]),
        ));
    };
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let comment = state
        .comments
        .create(NewIssueComment {
            issue_id,
            author_agent_id: None,
            author_user_id: body.author_user_id,
            body: body_text.trim().to_owned(),
        })
        .await
        .map_err(comment_error_to_api)?;
    Ok((StatusCode::CREATED, Json(comment.into())))
}

/// `GET /api/issues/{issueId}/comments/{commentId}` — fetches one comment.
#[route(GET "/api/issues/{id}/comments/{comment_id}")]
pub async fn get_comment(cx: &Cx) -> Result<Json<IssueCommentDto>, ApiError> {
    let comment_id = path_param::<CommentId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .comments
        .get(&comment_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(comment) => Ok(Json(comment.into())),
        None => Err(ApiError::not_found("Comment not found")),
    }
}

/// `DELETE /api/issues/{issueId}/comments/{commentId}` — deletes a comment.
#[route(DELETE "/api/issues/{id}/comments/{comment_id}")]
pub async fn delete_comment(cx: &Cx) -> Result<StatusCode, ApiError> {
    let comment_id = path_param::<CommentId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .comments
        .delete(&comment_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Comment not found")),
    }
}

/// Shared `{comment_id}` path parameter.
#[path_param(error = bad_request("Invalid comment id"))]
pub(crate) struct CommentId(String);

fn comment_error_to_api(error: staple_data::IssueCommentError) -> ApiError {
    use staple_data::IssueCommentError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::AuthorInDifferentCompany => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["authorAgentId"], "message": "Author agent belongs to a different company" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
