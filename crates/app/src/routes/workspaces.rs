//! Environment and workspace routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewEnvironment, NewExecutionWorkspace, NewProjectWorkspace, NewRuntimeService,
    NewWorkspaceOperation,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    auth::{enforce_company_scope, require_board},
    error::ApiError,
    routes::{CompanyId, is_uuid},
    state::AppState,
};

/// Body for `POST /api/environments`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnvironmentRequest {
    /// Environment name (unique).
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Driver (`local` only once).
    #[serde(default)]
    pub driver: Option<String>,
    /// Config JSON.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// Body for `POST /api/companies/{companyId}/project-workspaces`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectWorkspaceRequest {
    /// Project id.
    pub project_id: String,
    /// Name.
    pub name: String,
    /// Working directory.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Repo URL.
    #[serde(default)]
    pub repo_url: Option<String>,
    /// Primary flag.
    #[serde(default)]
    pub is_primary: Option<bool>,
}

/// Body for `POST /api/companies/{companyId}/execution-workspaces`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExecutionWorkspaceRequest {
    /// Project id.
    pub project_id: String,
    /// Project workspace id.
    #[serde(default)]
    pub project_workspace_id: Option<String>,
    /// Source issue id.
    #[serde(default)]
    pub source_issue_id: Option<String>,
    /// Mode.
    pub mode: String,
    /// Strategy type.
    pub strategy_type: String,
    /// Name.
    pub name: String,
    /// Working directory.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Repo URL.
    #[serde(default)]
    pub repo_url: Option<String>,
}

/// Body for `POST /api/companies/{companyId}/runtime-services`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuntimeServiceRequest {
    /// Execution workspace id.
    #[serde(default)]
    pub execution_workspace_id: Option<String>,
    /// Issue id.
    #[serde(default)]
    pub issue_id: Option<String>,
    /// Scope type.
    pub scope_type: String,
    /// Scope id.
    #[serde(default)]
    pub scope_id: Option<String>,
    /// Service name.
    pub service_name: String,
    /// Lifecycle.
    pub lifecycle: String,
    /// Command.
    #[serde(default)]
    pub command: Option<String>,
    /// Port.
    #[serde(default)]
    pub port: Option<i64>,
    /// URL.
    #[serde(default)]
    pub url: Option<String>,
    /// Provider.
    pub provider: String,
}

/// Body for `POST /api/companies/{companyId}/workspace-operations`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOperationRequest {
    /// Execution workspace id.
    #[serde(default)]
    pub execution_workspace_id: Option<String>,
    /// Heartbeat run id.
    #[serde(default)]
    pub heartbeat_run_id: Option<String>,
    /// Issue id.
    #[serde(default)]
    pub issue_id: Option<String>,
    /// Phase.
    pub phase: String,
    /// Command.
    #[serde(default)]
    pub command: Option<String>,
    /// Log ref.
    #[serde(default)]
    pub log_ref: Option<String>,
}

fn validate_uuid_opt(value: &Option<String>, path: &str, issues: &mut Vec<serde_json::Value>) {
    if let Some(value) = value
        && !is_uuid(value)
    {
        issues.push(json!({ "path": [path], "message": "Invalid uuid" }));
    }
}

/// `GET /api/environments` — lists environments (board).
#[route(GET "/api/environments")]
pub async fn list_environments(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let environments = state
        .environments
        .list()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        serde_json::to_value(&environments).unwrap_or_default(),
    ))
}

/// `POST /api/environments` — creates an environment (board).
#[route(POST "/api/environments")]
pub async fn create_environment(
    cx: &Cx,
    Json(body): Json<CreateEnvironmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    require_board(cx)?;
    if body.name.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["name"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let environment = state
        .environments
        .create(NewEnvironment {
            name: body.name.trim().to_owned(),
            description: body.description,
            driver: body.driver.unwrap_or_else(|| "local".to_owned()),
            config: body.config.map(|value| value.to_string()),
        })
        .await
        .map_err(environment_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&environment).unwrap_or_default()),
    ))
}

/// `POST /api/environments/ensure-local` — ensures the default local
/// environment (board).
#[route(POST "/api/environments/ensure-local")]
pub async fn ensure_local_environment(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let environment = state
        .environments
        .ensure_local()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&environment).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/project-workspaces`.
#[route(POST "/api/companies/{company_id}/project-workspaces")]
pub async fn create_project_workspace(
    cx: &Cx,
    Json(body): Json<CreateProjectWorkspaceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if !is_uuid(&body.project_id) {
        issues.push(json!({ "path": ["projectId"], "message": "Invalid uuid" }));
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
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let workspace = state
        .workspaces
        .create_project_workspace(NewProjectWorkspace {
            company_id,
            project_id: body.project_id,
            name: body.name,
            cwd: body.cwd,
            repo_url: body.repo_url,
            is_primary: body.is_primary.unwrap_or(false),
        })
        .await
        .map_err(workspace_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&workspace).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/project-workspaces?projectId=...`.
#[route(GET "/api/companies/{company_id}/project-workspaces")]
pub async fn list_project_workspaces(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let workspaces = state
        .workspaces
        .list_project_workspaces(&company_id, None)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&workspaces).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/execution-workspaces`.
#[route(POST "/api/companies/{company_id}/execution-workspaces")]
pub async fn create_execution_workspace(
    cx: &Cx,
    Json(body): Json<CreateExecutionWorkspaceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if !is_uuid(&body.project_id) {
        issues.push(json!({ "path": ["projectId"], "message": "Invalid uuid" }));
    }
    validate_uuid_opt(
        &body.project_workspace_id,
        "projectWorkspaceId",
        &mut issues,
    );
    validate_uuid_opt(&body.source_issue_id, "sourceIssueId", &mut issues);
    if body.mode.trim().is_empty() || body.strategy_type.trim().is_empty() {
        issues.push(json!({ "path": ["mode"], "message": "mode and strategyType are required" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let workspace = state
        .workspaces
        .create_execution_workspace(NewExecutionWorkspace {
            company_id,
            project_id: body.project_id,
            project_workspace_id: body.project_workspace_id,
            source_issue_id: body.source_issue_id,
            mode: body.mode,
            strategy_type: body.strategy_type,
            name: body.name,
            cwd: body.cwd,
            repo_url: body.repo_url,
        })
        .await
        .map_err(workspace_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&workspace).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/execution-workspaces?projectId=...`.
#[route(GET "/api/companies/{company_id}/execution-workspaces")]
pub async fn list_execution_workspaces(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let workspaces = state
        .workspaces
        .list_execution_workspaces(&company_id, None)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&workspaces).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/runtime-services`.
#[route(POST "/api/companies/{company_id}/runtime-services")]
pub async fn create_runtime_service(
    cx: &Cx,
    Json(body): Json<CreateRuntimeServiceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    validate_uuid_opt(
        &body.execution_workspace_id,
        "executionWorkspaceId",
        &mut issues,
    );
    validate_uuid_opt(&body.issue_id, "issueId", &mut issues);
    if body.service_name.trim().is_empty() {
        issues.push(json!({ "path": ["serviceName"], "message": "String must contain at least 1 character(s)" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let service = state
        .workspaces
        .create_runtime_service(NewRuntimeService {
            company_id,
            execution_workspace_id: body.execution_workspace_id,
            issue_id: body.issue_id,
            scope_type: body.scope_type,
            scope_id: body.scope_id,
            service_name: body.service_name,
            lifecycle: body.lifecycle,
            command: body.command,
            port: body.port,
            url: body.url,
            provider: body.provider,
        })
        .await
        .map_err(workspace_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&service).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/runtime-services`.
#[route(GET "/api/companies/{company_id}/runtime-services")]
pub async fn list_runtime_services(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let services = state
        .workspaces
        .list_runtime_services(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&services).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/workspace-operations`.
#[route(POST "/api/companies/{company_id}/workspace-operations")]
pub async fn create_workspace_operation(
    cx: &Cx,
    Json(body): Json<CreateOperationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let operation = state
        .workspaces
        .create_operation(NewWorkspaceOperation {
            company_id,
            execution_workspace_id: body.execution_workspace_id,
            heartbeat_run_id: body.heartbeat_run_id,
            issue_id: body.issue_id,
            phase: body.phase,
            command: body.command,
            log_ref: body.log_ref,
        })
        .await
        .map_err(workspace_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&operation).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/workspace-operations`.
#[route(GET "/api/companies/{company_id}/workspace-operations")]
pub async fn list_workspace_operations(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let operations = state
        .workspaces
        .list_operations(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&operations).unwrap_or_default()))
}

fn environment_error_to_api(error: staple_data::EnvironmentError) -> ApiError {
    use staple_data::EnvironmentError as E;
    match error {
        E::AlreadyExists => ApiError::conflict("Environment already exists"),
        E::LocalAlreadyExists => ApiError::conflict("A local environment already exists"),
        other => ApiError::internal(other.to_string()),
    }
}

fn workspace_error_to_api(error: staple_data::WorkspaceError) -> ApiError {
    use staple_data::WorkspaceError as E;
    match error {
        E::ReferenceNotFound(reference) => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": [reference], "message": "Referenced record not found" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
