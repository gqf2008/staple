//! Issue routes.

use serde::{Deserialize, Serialize};
use serde_json::json;
use staple_data::{IssuePatch, NewIssue};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::enforce_company_scope,
    dto::IssueDto,
    error::ApiError,
    permissions::{authorize_assignment, authorize_inbox_manage},
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/issues`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueRequest {
    /// Title (required, non-empty).
    #[serde(default)]
    pub title: Option<String>,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Linked project id.
    #[serde(default)]
    pub project_id: Option<String>,
    /// Linked goal id.
    #[serde(default)]
    pub goal_id: Option<String>,
    /// Parent issue id.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Status (defaults: `todo` when assigned, else `backlog`).
    #[serde(default)]
    pub status: Option<String>,
    /// Priority (default `medium`).
    #[serde(default)]
    pub priority: Option<String>,
    /// Assignee agent id.
    #[serde(default)]
    pub assignee_agent_id: Option<String>,
    /// Assignee user id.
    #[serde(default)]
    pub assignee_user_id: Option<String>,
    /// Work mode (default `standard`).
    #[serde(default)]
    pub work_mode: Option<String>,
    /// Billing code.
    #[serde(default)]
    pub billing_code: Option<String>,
    /// Execution workspace settings JSON.
    #[serde(default)]
    pub execution_workspace_settings: Option<serde_json::Value>,
}

/// Body for `PATCH /api/issues/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueRequest {
    /// New title.
    #[serde(default)]
    pub title: Option<String>,
    /// New description (`null` clears).
    #[serde(default)]
    pub description: Option<Option<String>>,
    /// New status (validated against the §8.2 state machine).
    #[serde(default)]
    pub status: Option<String>,
    /// New priority.
    #[serde(default)]
    pub priority: Option<String>,
    /// New assignee agent (`null` clears).
    #[serde(default)]
    pub assignee_agent_id: Option<Option<String>>,
    /// New billing code (`null` clears).
    #[serde(default)]
    pub billing_code: Option<Option<String>>,
    /// New execution workspace settings JSON (`null` clears).
    #[serde(default, deserialize_with = "crate::routes::deserialize_optional_json")]
    pub execution_workspace_settings: Option<Option<serde_json::Value>>,
}

/// A single validation issue, mirroring the upstream Zod error shape.
#[derive(Debug, Serialize)]
pub struct IssueValidationIssue {
    path: Vec<String>,
    message: String,
}

fn issue(path: &str, message: &str) -> IssueValidationIssue {
    IssueValidationIssue {
        path: vec![path.to_owned()],
        message: message.to_owned(),
    }
}

fn validate_create(body: &CreateIssueRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.title.as_deref().unwrap_or_default().trim().is_empty() {
        issues.push(issue(
            "title",
            "String must contain at least 1 character(s)",
        ));
    }
    if let Some(status) = &body.status
        && !matches!(
            status.as_str(),
            "backlog" | "todo" | "in_progress" | "in_review" | "done" | "blocked" | "cancelled"
        )
    {
        issues.push(issue(
            "status",
            "Invalid enum value. Expected 'backlog' | 'todo' | 'in_progress' | 'in_review' | 'done' | 'blocked' | 'cancelled'",
        ));
    }
    if let Some(priority) = &body.priority
        && !matches!(priority.as_str(), "critical" | "high" | "medium" | "low")
    {
        issues.push(issue(
            "priority",
            "Invalid enum value. Expected 'critical' | 'high' | 'medium' | 'low'",
        ));
    }
    if let Some(work_mode) = &body.work_mode
        && !matches!(work_mode.as_str(), "standard" | "ask" | "planning")
    {
        issues.push(issue(
            "workMode",
            "Invalid enum value. Expected 'standard' | 'ask' | 'planning'",
        ));
    }
    for (path, value) in [
        ("projectId", body.project_id.as_deref()),
        ("goalId", body.goal_id.as_deref()),
        ("parentId", body.parent_id.as_deref()),
        ("assigneeAgentId", body.assignee_agent_id.as_deref()),
    ] {
        if let Some(value) = value
            && !is_uuid(value)
        {
            issues.push(issue(path, "Invalid uuid"));
        }
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn validate_update(body: &UpdateIssueRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if let Some(title) = &body.title
        && title.trim().is_empty()
    {
        issues.push(issue(
            "title",
            "String must contain at least 1 character(s)",
        ));
    }
    if let Some(status) = &body.status
        && !matches!(
            status.as_str(),
            "backlog" | "todo" | "in_progress" | "in_review" | "done" | "blocked" | "cancelled"
        )
    {
        issues.push(issue(
            "status",
            "Invalid enum value. Expected 'backlog' | 'todo' | 'in_progress' | 'in_review' | 'done' | 'blocked' | 'cancelled'",
        ));
    }
    if let Some(priority) = &body.priority
        && !matches!(priority.as_str(), "critical" | "high" | "medium" | "low")
    {
        issues.push(issue(
            "priority",
            "Invalid enum value. Expected 'critical' | 'high' | 'medium' | 'low'",
        ));
    }
    if let Some(Some(assignee)) = &body.assignee_agent_id
        && !is_uuid(assignee)
    {
        issues.push(issue("assigneeAgentId", "Invalid uuid"));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/issues` — lists a company's issues.
#[route(GET "/api/companies/{company_id}/issues")]
pub async fn list_issues(cx: &Cx) -> Result<Json<Vec<IssueDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let issues = state
        .issues
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(issues.into_iter().map(IssueDto::from).collect()))
}

/// `POST /api/companies/{companyId}/issues` — creates an issue, returns 201.
#[route(POST "/api/companies/{company_id}/issues")]
pub async fn create_issue(
    cx: &Cx,
    Json(body): Json<CreateIssueRequest>,
) -> Result<(StatusCode, Json<IssueDto>), ApiError> {
    validate_create(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    if let Some(assignee_agent_id) = &body.assignee_agent_id {
        authorize_assignment(
            state,
            cx,
            &company_id,
            json!({
                "projectId": body.project_id.as_deref(),
                "assigneeAgentId": assignee_agent_id,
            }),
        )
        .await?;
    }
    let issue = state
        .issues
        .create(NewIssue {
            company_id,
            project_id: body.project_id,
            goal_id: body.goal_id,
            parent_id: body.parent_id,
            title: body.title.unwrap_or_default().trim().to_owned(),
            description: body.description,
            status: body.status,
            priority: body.priority,
            assignee_agent_id: body.assignee_agent_id,
            assignee_user_id: body.assignee_user_id,
            created_by_user_id: None,
            work_mode: body.work_mode,
            billing_code: body.billing_code,
            execution_workspace_settings: body
                .execution_workspace_settings
                .map(|value| value.to_string()),
        })
        .await
        .map_err(issue_error_to_api)?;
    log_activity(
        &state.activity,
        &issue.company_id,
        "issue.created",
        "issue",
        &issue.id,
        Some(json!({ "identifier": issue.identifier, "title": issue.title })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(issue.into())))
}

/// `GET /api/issues/{id}` — fetches one issue.
#[route(GET "/api/issues/{id}")]
pub async fn get_issue(cx: &Cx) -> Result<Json<IssueDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .issues
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(issue) => Ok(Json(issue.into())),
        None => Err(ApiError::not_found("Issue not found")),
    }
}

/// `PATCH /api/issues/{id}` — partially updates an issue (state machine
/// enforced by the repository).
#[route(PATCH "/api/issues/{id}")]
pub async fn update_issue(
    cx: &Cx,
    Json(body): Json<UpdateIssueRequest>,
) -> Result<Json<IssueDto>, ApiError> {
    validate_update(&body)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(Some(assignee_agent_id)) = &body.assignee_agent_id {
        let existing = state
            .issues
            .get(&id)
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
            .ok_or_else(|| ApiError::not_found("Issue not found"))?;
        authorize_assignment(
            state,
            cx,
            &existing.company_id,
            json!({
                "projectId": existing.project_id,
                "assigneeAgentId": assignee_agent_id,
            }),
        )
        .await?;
    }
    let patch = IssuePatch {
        title: body.title.map(|value| value.trim().to_owned()),
        description: body.description,
        status: body.status,
        priority: body.priority,
        assignee_agent_id: body.assignee_agent_id,
        billing_code: body.billing_code,
        execution_workspace_settings: body
            .execution_workspace_settings
            .map(|value| value.map(|settings| settings.to_string())),
    };
    match state
        .issues
        .update(&id, patch)
        .await
        .map_err(issue_error_to_api)?
    {
        Some(issue) => {
            log_activity(
                &state.activity,
                &issue.company_id,
                "issue.updated",
                "issue",
                &issue.id,
                Some(json!({ "status": issue.status })),
            )
            .await?;
            Ok(Json(issue.into()))
        }
        None => Err(ApiError::not_found("Issue not found")),
    }
}

/// `DELETE /api/issues/{id}` — deletes an issue.
#[route(DELETE "/api/issues/{id}")]
pub async fn delete_issue(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state.issues.delete(&id).await.map_err(issue_error_to_api)? {
        Some(issue) => {
            log_activity(
                &state.activity,
                &issue.company_id,
                "issue.deleted",
                "issue",
                &issue.id,
                None,
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Issue not found")),
    }
}

/// `GET /api/companies/{companyId}/inbox` — unarchived issues, newest first.
#[route(GET "/api/companies/{company_id}/inbox")]
pub async fn list_inbox(cx: &Cx) -> Result<Json<Vec<IssueDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let issues = state
        .issues
        .list_inbox(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(issues.into_iter().map(IssueDto::from).collect()))
}

/// Body for `POST /api/issues/{id}/archive` / `unarchive`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxActionRequest {
    /// Target user whose inbox is managed (required when an agent acts on
    /// another user's inbox; checked against the `inbox:manage` grant).
    #[serde(default)]
    pub user_id: Option<String>,
}

/// `POST /api/issues/{id}/archive` — archives an issue (hidden from inbox).
#[route(POST "/api/issues/{id}/archive")]
pub async fn archive_issue(
    cx: &Cx,
    Json(body): Json<InboxActionRequest>,
) -> Result<Json<IssueDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issue = state
        .issues
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Issue not found"))?;
    if let Some(user_id) = &body.user_id {
        authorize_inbox_manage(state, cx, &issue.company_id, user_id).await?;
    }
    match state
        .issues
        .set_hidden(&id, true)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(issue) => Ok(Json(issue.into())),
        None => Err(ApiError::not_found("Issue not found")),
    }
}

/// `POST /api/issues/{id}/unarchive` — restores an archived issue.
#[route(POST "/api/issues/{id}/unarchive")]
pub async fn unarchive_issue(
    cx: &Cx,
    Json(body): Json<InboxActionRequest>,
) -> Result<Json<IssueDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let issue = state
        .issues
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Issue not found"))?;
    if let Some(user_id) = &body.user_id {
        authorize_inbox_manage(state, cx, &issue.company_id, user_id).await?;
    }
    match state
        .issues
        .set_hidden(&id, false)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(issue) => Ok(Json(issue.into())),
        None => Err(ApiError::not_found("Issue not found")),
    }
}

fn issue_error_to_api(error: staple_data::IssueError) -> ApiError {
    use staple_data::IssueError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ReferenceNotFound(reference) => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": [reference], "message": "Referenced record not found" }]),
        ),
        E::ReferenceInDifferentCompany(reference) => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": [reference], "message": "Referenced record belongs to a different company" }]),
        ),
        E::InvalidStatusTransition { from, to } => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["status"], "message": format!("Invalid status transition: {from} -> {to}") }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
