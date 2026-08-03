//! Goal routes.

use serde::{Deserialize, Serialize};
use serde_json::json;
use staple_data::{GoalPatch, NewGoal};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    dto::GoalDto,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/goals`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalRequest {
    /// Title (required, non-empty).
    #[serde(default)]
    pub title: Option<String>,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// `company | team | agent | task` (default `task`).
    #[serde(default)]
    pub level: Option<String>,
    /// `planned | active | achieved | cancelled` (default `planned`).
    #[serde(default)]
    pub status: Option<String>,
    /// Parent goal id.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Owning agent id.
    #[serde(default)]
    pub owner_agent_id: Option<String>,
}

/// Body for `PATCH /api/goals/{id}`. `null` clears nullable fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalRequest {
    /// New title.
    #[serde(default)]
    pub title: Option<String>,
    /// New description (`null` clears).
    #[serde(default)]
    pub description: Option<Option<String>>,
    /// New level.
    #[serde(default)]
    pub level: Option<String>,
    /// New status.
    #[serde(default)]
    pub status: Option<String>,
    /// New parent goal id (`null` clears).
    #[serde(default)]
    pub parent_id: Option<Option<String>>,
    /// New owner agent id (`null` clears).
    #[serde(default)]
    pub owner_agent_id: Option<Option<String>>,
}

/// A single validation issue, mirroring the upstream Zod error shape.
#[derive(Debug, Serialize)]
pub struct GoalValidationIssue {
    path: Vec<String>,
    message: String,
}

fn issue(path: &str, message: &str) -> GoalValidationIssue {
    GoalValidationIssue {
        path: vec![path.to_owned()],
        message: message.to_owned(),
    }
}

fn validate_create(body: &CreateGoalRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.title.as_deref().unwrap_or_default().trim().is_empty() {
        issues.push(issue(
            "title",
            "String must contain at least 1 character(s)",
        ));
    }
    if let Some(level) = &body.level
        && !matches!(level.as_str(), "company" | "team" | "agent" | "task")
    {
        issues.push(issue(
            "level",
            "Invalid enum value. Expected 'company' | 'team' | 'agent' | 'task'",
        ));
    }
    if let Some(status) = &body.status
        && !matches!(
            status.as_str(),
            "planned" | "active" | "achieved" | "cancelled"
        )
    {
        issues.push(issue(
            "status",
            "Invalid enum value. Expected 'planned' | 'active' | 'achieved' | 'cancelled'",
        ));
    }
    if let Some(parent_id) = &body.parent_id
        && !is_uuid(parent_id)
    {
        issues.push(issue("parentId", "Invalid uuid"));
    }
    if let Some(owner_agent_id) = &body.owner_agent_id
        && !is_uuid(owner_agent_id)
    {
        issues.push(issue("ownerAgentId", "Invalid uuid"));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn validate_update(body: &UpdateGoalRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if let Some(title) = &body.title
        && title.trim().is_empty()
    {
        issues.push(issue(
            "title",
            "String must contain at least 1 character(s)",
        ));
    }
    if let Some(level) = &body.level
        && !matches!(level.as_str(), "company" | "team" | "agent" | "task")
    {
        issues.push(issue(
            "level",
            "Invalid enum value. Expected 'company' | 'team' | 'agent' | 'task'",
        ));
    }
    if let Some(status) = &body.status
        && !matches!(
            status.as_str(),
            "planned" | "active" | "achieved" | "cancelled"
        )
    {
        issues.push(issue(
            "status",
            "Invalid enum value. Expected 'planned' | 'active' | 'achieved' | 'cancelled'",
        ));
    }
    if let Some(Some(parent_id)) = &body.parent_id
        && !is_uuid(parent_id)
    {
        issues.push(issue("parentId", "Invalid uuid"));
    }
    if let Some(Some(owner_agent_id)) = &body.owner_agent_id
        && !is_uuid(owner_agent_id)
    {
        issues.push(issue("ownerAgentId", "Invalid uuid"));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/goals` — lists a company's goals.
#[route(GET "/api/companies/{company_id}/goals")]
pub async fn list_goals(cx: &Cx) -> Result<Json<Vec<GoalDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let goals = state
        .goals
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(goals.into_iter().map(GoalDto::from).collect()))
}

/// `POST /api/companies/{companyId}/goals` — creates a goal, returns 201.
#[route(POST "/api/companies/{company_id}/goals")]
pub async fn create_goal(
    cx: &Cx,
    Json(body): Json<CreateGoalRequest>,
) -> Result<(StatusCode, Json<GoalDto>), ApiError> {
    validate_create(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let goal = state
        .goals
        .create(NewGoal {
            company_id,
            title: body.title.unwrap_or_default().trim().to_owned(),
            description: body.description,
            level: body.level.unwrap_or_else(|| "task".to_owned()),
            parent_id: body.parent_id,
            owner_agent_id: body.owner_agent_id,
            status: body.status.unwrap_or_else(|| "planned".to_owned()),
        })
        .await
        .map_err(goal_error_to_api)?;
    log_activity(
        &state.activity,
        &goal.company_id,
        "goal.created",
        "goal",
        &goal.id,
        Some(json!({ "title": goal.title })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(goal.into())))
}

/// `GET /api/goals/{id}` — fetches one goal.
#[route(GET "/api/goals/{id}")]
pub async fn get_goal(cx: &Cx) -> Result<Json<GoalDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .goals
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(goal) => Ok(Json(goal.into())),
        None => Err(ApiError::not_found("Goal not found")),
    }
}

/// `PATCH /api/goals/{id}` — partially updates a goal.
#[route(PATCH "/api/goals/{id}")]
pub async fn update_goal(
    cx: &Cx,
    Json(body): Json<UpdateGoalRequest>,
) -> Result<Json<GoalDto>, ApiError> {
    validate_update(&body)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let patch = GoalPatch {
        title: body.title.map(|value| value.trim().to_owned()),
        description: body.description,
        level: body.level,
        parent_id: body.parent_id,
        owner_agent_id: body.owner_agent_id,
        status: body.status,
    };
    match state
        .goals
        .update(&id, patch)
        .await
        .map_err(goal_error_to_api)?
    {
        Some(goal) => {
            log_activity(
                &state.activity,
                &goal.company_id,
                "goal.updated",
                "goal",
                &goal.id,
                None,
            )
            .await?;
            Ok(Json(goal.into()))
        }
        None => Err(ApiError::not_found("Goal not found")),
    }
}

/// `DELETE /api/goals/{id}` — deletes a goal.
#[route(DELETE "/api/goals/{id}")]
pub async fn delete_goal(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state.goals.delete(&id).await.map_err(goal_error_to_api)? {
        Some(goal) => {
            log_activity(
                &state.activity,
                &goal.company_id,
                "goal.deleted",
                "goal",
                &goal.id,
                None,
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Goal not found")),
    }
}

fn goal_error_to_api(error: staple_data::GoalError) -> ApiError {
    use staple_data::GoalError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ParentNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["parentId"], "message": "Parent goal not found" }]),
        ),
        E::ParentInDifferentCompany => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["parentId"], "message": "Parent goal belongs to a different company" }]),
        ),
        E::OwnerAgentNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["ownerAgentId"], "message": "Owner agent not found" }]),
        ),
        E::OwnerAgentInDifferentCompany => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["ownerAgentId"], "message": "Owner agent belongs to a different company" }]),
        ),
        E::InUse => ApiError::conflict("Goal is referenced by other records"),
        other => ApiError::internal(other.to_string()),
    }
}
