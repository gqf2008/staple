//! Project routes.

use serde::{Deserialize, Serialize};
use serde_json::json;
use staple_data::{NewProject, ProjectPatch};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    dto::ProjectDto,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/projects`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    /// Name (required, non-empty).
    #[serde(default)]
    pub name: Option<String>,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// `backlog | planned | in_progress | completed | cancelled` (default `backlog`).
    #[serde(default)]
    pub status: Option<String>,
    /// Linked goal id.
    #[serde(default)]
    pub goal_id: Option<String>,
    /// Lead agent id.
    #[serde(default)]
    pub lead_agent_id: Option<String>,
    /// Target date.
    #[serde(default)]
    pub target_date: Option<String>,
}

/// Body for `PATCH /api/projects/{id}`. `null` clears nullable fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    /// New name.
    #[serde(default)]
    pub name: Option<String>,
    /// New description (`null` clears).
    #[serde(default)]
    pub description: Option<Option<String>>,
    /// New status.
    #[serde(default)]
    pub status: Option<String>,
    /// New linked goal id (`null` clears).
    #[serde(default)]
    pub goal_id: Option<Option<String>>,
    /// New lead agent id (`null` clears).
    #[serde(default)]
    pub lead_agent_id: Option<Option<String>>,
    /// New target date (`null` clears).
    #[serde(default)]
    pub target_date: Option<Option<String>>,
}

/// A single validation issue, mirroring the upstream Zod error shape.
#[derive(Debug, Serialize)]
pub struct ProjectValidationIssue {
    path: Vec<String>,
    message: String,
}

fn issue(path: &str, message: &str) -> ProjectValidationIssue {
    ProjectValidationIssue {
        path: vec![path.to_owned()],
        message: message.to_owned(),
    }
}

fn validate_create(body: &CreateProjectRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.name.as_deref().unwrap_or_default().trim().is_empty() {
        issues.push(issue("name", "String must contain at least 1 character(s)"));
    }
    if let Some(status) = &body.status
        && !matches!(
            status.as_str(),
            "backlog" | "planned" | "in_progress" | "completed" | "cancelled"
        )
    {
        issues.push(issue(
            "status",
            "Invalid enum value. Expected 'backlog' | 'planned' | 'in_progress' | 'completed' | 'cancelled'",
        ));
    }
    if let Some(goal_id) = &body.goal_id
        && !is_uuid(goal_id)
    {
        issues.push(issue("goalId", "Invalid uuid"));
    }
    if let Some(lead_agent_id) = &body.lead_agent_id
        && !is_uuid(lead_agent_id)
    {
        issues.push(issue("leadAgentId", "Invalid uuid"));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn validate_update(body: &UpdateProjectRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if let Some(name) = &body.name
        && name.trim().is_empty()
    {
        issues.push(issue("name", "String must contain at least 1 character(s)"));
    }
    if let Some(status) = &body.status
        && !matches!(
            status.as_str(),
            "backlog" | "planned" | "in_progress" | "completed" | "cancelled"
        )
    {
        issues.push(issue(
            "status",
            "Invalid enum value. Expected 'backlog' | 'planned' | 'in_progress' | 'completed' | 'cancelled'",
        ));
    }
    if let Some(Some(goal_id)) = &body.goal_id
        && !is_uuid(goal_id)
    {
        issues.push(issue("goalId", "Invalid uuid"));
    }
    if let Some(Some(lead_agent_id)) = &body.lead_agent_id
        && !is_uuid(lead_agent_id)
    {
        issues.push(issue("leadAgentId", "Invalid uuid"));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/projects` — lists a company's projects.
#[route(GET "/api/companies/{company_id}/projects")]
pub async fn list_projects(cx: &Cx) -> Result<Json<Vec<ProjectDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let projects = state
        .projects
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(projects.into_iter().map(ProjectDto::from).collect()))
}

/// `POST /api/companies/{companyId}/projects` — creates a project, returns 201.
#[route(POST "/api/companies/{company_id}/projects")]
pub async fn create_project(
    cx: &Cx,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectDto>), ApiError> {
    validate_create(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let project = state
        .projects
        .create(NewProject {
            company_id,
            goal_id: body.goal_id,
            name: body.name.unwrap_or_default().trim().to_owned(),
            description: body.description,
            status: body.status.unwrap_or_else(|| "backlog".to_owned()),
            lead_agent_id: body.lead_agent_id,
            target_date: body.target_date,
            env: None,
        })
        .await
        .map_err(project_error_to_api)?;
    log_activity(
        &state.activity,
        &project.company_id,
        "project.created",
        "project",
        &project.id,
        Some(json!({ "name": project.name })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(project.into())))
}

/// `GET /api/projects/{id}` — fetches one project.
#[route(GET "/api/projects/{id}")]
pub async fn get_project(cx: &Cx) -> Result<Json<ProjectDto>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .projects
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(project) => Ok(Json(project.into())),
        None => Err(ApiError::not_found("Project not found")),
    }
}

/// `PATCH /api/projects/{id}` — partially updates a project.
#[route(PATCH "/api/projects/{id}")]
pub async fn update_project(
    cx: &Cx,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectDto>, ApiError> {
    validate_update(&body)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let patch = ProjectPatch {
        goal_id: body.goal_id,
        name: body.name.map(|value| value.trim().to_owned()),
        description: body.description,
        status: body.status,
        lead_agent_id: body.lead_agent_id,
        target_date: body.target_date,
    };
    match state
        .projects
        .update(&id, patch)
        .await
        .map_err(project_error_to_api)?
    {
        Some(project) => {
            log_activity(
                &state.activity,
                &project.company_id,
                "project.updated",
                "project",
                &project.id,
                None,
            )
            .await?;
            Ok(Json(project.into()))
        }
        None => Err(ApiError::not_found("Project not found")),
    }
}

/// `DELETE /api/projects/{id}` — deletes a project.
#[route(DELETE "/api/projects/{id}")]
pub async fn delete_project(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .projects
        .delete(&id)
        .await
        .map_err(project_error_to_api)?
    {
        Some(project) => {
            log_activity(
                &state.activity,
                &project.company_id,
                "project.deleted",
                "project",
                &project.id,
                None,
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Project not found")),
    }
}

fn project_error_to_api(error: staple_data::ProjectError) -> ApiError {
    use staple_data::ProjectError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::GoalNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["goalId"], "message": "Goal not found" }]),
        ),
        E::GoalInDifferentCompany => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["goalId"], "message": "Goal belongs to a different company" }]),
        ),
        E::LeadAgentNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["leadAgentId"], "message": "Lead agent not found" }]),
        ),
        E::LeadAgentInDifferentCompany => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["leadAgentId"], "message": "Lead agent belongs to a different company" }]),
        ),
        E::InUse => ApiError::conflict("Project is referenced by other records"),
        other => ApiError::internal(other.to_string()),
    }
}
