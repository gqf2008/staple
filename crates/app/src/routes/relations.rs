//! Issue blocker (relation) routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::NewIssueRelation;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    dto::IssueRelationDto,
    error::ApiError,
    routes::{Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/issues/{issueId}/blockers`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBlockerRequest {
    /// The issue that blocks the path issue.
    pub blocker_issue_id: String,
}

/// `GET /api/issues/{issueId}/blockers` — lists the issues blocking it.
#[route(GET "/api/issues/{id}/blockers")]
pub async fn list_blockers(cx: &Cx) -> Result<Json<Vec<IssueRelationDto>>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let relations = state
        .relations
        .list_blockers(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        relations.into_iter().map(IssueRelationDto::from).collect(),
    ))
}

/// `POST /api/issues/{issueId}/blockers` — records that `blockerIssueId`
/// blocks this issue, returns 201.
#[route(POST "/api/issues/{id}/blockers")]
pub async fn add_blocker(
    cx: &Cx,
    Json(body): Json<AddBlockerRequest>,
) -> Result<(StatusCode, Json<IssueRelationDto>), ApiError> {
    if !is_uuid(&body.blocker_issue_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["blockerIssueId"], "message": "Invalid uuid" }]),
        ));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let relation = state
        .relations
        .add_blocker(NewIssueRelation {
            issue_id: body.blocker_issue_id,
            related_issue_id: issue_id,
        })
        .await
        .map_err(relation_error_to_api)?;
    log_activity(
        &state.activity,
        &relation.company_id,
        "blocker.added",
        "issue_relation",
        &relation.id,
        Some(json!({ "blockingIssueId": relation.issue_id, "blockedIssueId": relation.related_issue_id })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(relation.into())))
}

/// `DELETE /api/issue-relations/{relationId}` — removes a blocker relation.
#[route(DELETE "/api/issue-relations/{relation_id}")]
pub async fn remove_blocker(cx: &Cx) -> Result<StatusCode, ApiError> {
    let relation_id = path_param::<RelationId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .relations
        .remove_blocker(&relation_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(relation) => {
            log_activity(
                &state.activity,
                &relation.company_id,
                "blocker.removed",
                "issue_relation",
                &relation.id,
                None,
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Blocker relation not found")),
    }
}

/// Shared `{relation_id}` path parameter.
#[path_param(error = bad_request("Invalid relation id"))]
pub(crate) struct RelationId(String);

fn relation_error_to_api(error: staple_data::IssueRelationError) -> ApiError {
    use staple_data::IssueRelationError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::AlreadyExists => ApiError::conflict("Blocker relation already exists"),
        other => ApiError::internal(other.to_string()),
    }
}
