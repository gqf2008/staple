//! Routine routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewRoutine, NewTrigger, UpdateRoutine};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/routines`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoutineRequest {
    /// Title.
    pub title: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Project id.
    #[serde(default)]
    pub project_id: Option<String>,
    /// Goal id.
    #[serde(default)]
    pub goal_id: Option<String>,
    /// Parent issue id.
    #[serde(default)]
    pub parent_issue_id: Option<String>,
    /// Assignee agent id.
    #[serde(default)]
    pub assignee_agent_id: Option<String>,
    /// Priority.
    #[serde(default)]
    pub priority: Option<String>,
    /// Variables JSON.
    #[serde(default)]
    pub variables: Option<serde_json::Value>,
}

/// Body for `PATCH /api/routines/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoutineRequest {
    /// Title.
    pub title: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Variables JSON.
    #[serde(default)]
    pub variables: Option<serde_json::Value>,
}

/// Body for `POST /api/routines/{id}/triggers`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTriggerRequest {
    /// Schedule kind.
    pub schedule_kind: String,
    /// Schedule expression.
    #[serde(default)]
    pub schedule_expr: Option<String>,
}

/// `POST /api/companies/{companyId}/routines` — creates a routine.
#[route(POST "/api/companies/{company_id}/routines")]
pub async fn create_routine(
    cx: &Cx,
    Json(body): Json<CreateRoutineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.title.trim().is_empty() {
        issues.push(
            json!({ "path": ["title"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    for (path, value) in [
        ("projectId", body.project_id.as_deref()),
        ("goalId", body.goal_id.as_deref()),
        ("parentIssueId", body.parent_issue_id.as_deref()),
        ("assigneeAgentId", body.assignee_agent_id.as_deref()),
    ] {
        if let Some(value) = value
            && !is_uuid(value)
        {
            issues.push(json!({ "path": [path], "message": "Invalid uuid" }));
        }
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let routine = state
        .routines
        .create(NewRoutine {
            company_id: company_id.clone(),
            project_id: body.project_id,
            goal_id: body.goal_id,
            parent_issue_id: body.parent_issue_id,
            title: body.title.trim().to_owned(),
            description: body.description,
            assignee_agent_id: body.assignee_agent_id,
            priority: body.priority.unwrap_or_else(|| "medium".to_owned()),
            variables: body.variables.map(|value| value.to_string()),
        })
        .await
        .map_err(routine_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "routine.created",
        "routine",
        &routine.id,
        Some(json!({ "title": routine.title })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&routine).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/routines` — lists routines.
#[route(GET "/api/companies/{company_id}/routines")]
pub async fn list_routines(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let routines = state
        .routines
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&routines).unwrap_or_default()))
}

/// `GET /api/routines/{id}` — fetches one routine.
#[route(GET "/api/routines/{id}")]
pub async fn get_routine(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .routines
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(routine) => Ok(Json(serde_json::to_value(&routine).unwrap_or_default())),
        None => Err(ApiError::not_found("Routine not found")),
    }
}

/// `PATCH /api/routines/{id}` — updates a routine (appends a revision).
#[route(PATCH "/api/routines/{id}")]
pub async fn update_routine(
    cx: &Cx,
    Json(body): Json<UpdateRoutineRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.title.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["title"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(existing) = state
        .routines
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    else {
        return Err(ApiError::not_found("Routine not found"));
    };
    let routine = state
        .routines
        .update(UpdateRoutine {
            company_id: existing.company_id.clone(),
            routine_id: id,
            title: body.title.trim().to_owned(),
            description: body.description,
            variables: body.variables.map(|value| value.to_string()),
        })
        .await
        .map_err(routine_error_to_api)?;
    log_activity(
        &state.activity,
        &existing.company_id,
        "routine.updated",
        "routine",
        &routine.id,
        Some(json!({ "revision": routine.latest_revision_number })),
    )
    .await?;
    Ok(Json(serde_json::to_value(&routine).unwrap_or_default()))
}

/// `POST /api/routines/{id}/trigger` — triggers a routine (creates a run).
#[route(POST "/api/routines/{id}/trigger")]
pub async fn trigger_routine(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(existing) = state
        .routines
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    else {
        return Err(ApiError::not_found("Routine not found"));
    };
    let run = state
        .routines
        .trigger(&existing.company_id, &id)
        .await
        .map_err(routine_error_to_api)?;
    log_activity(
        &state.activity,
        &existing.company_id,
        "routine.triggered",
        "routine",
        &existing.id,
        Some(json!({ "runId": run.id })),
    )
    .await?;
    Ok(Json(serde_json::to_value(&run).unwrap_or_default()))
}

/// `GET /api/routines/{id}/runs` — lists runs.
#[route(GET "/api/routines/{id}/runs")]
pub async fn list_routine_runs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(existing) = state
        .routines
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    else {
        return Err(ApiError::not_found("Routine not found"));
    };
    let runs = state
        .routines
        .list_runs(&existing.company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&runs).unwrap_or_default()))
}

/// `POST /api/routines/{id}/triggers` — creates a trigger.
#[route(POST "/api/routines/{id}/triggers")]
pub async fn create_trigger(
    cx: &Cx,
    Json(body): Json<CreateTriggerRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if !matches!(body.schedule_kind.as_str(), "manual" | "cron" | "webhook") {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["scheduleKind"], "message": "Invalid enum value. Expected 'manual' | 'cron' | 'webhook'" }]),
        ));
    }
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(existing) = state
        .routines
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    else {
        return Err(ApiError::not_found("Routine not found"));
    };
    let trigger = state
        .routines
        .create_trigger(NewTrigger {
            company_id: existing.company_id.clone(),
            routine_id: id,
            schedule_kind: body.schedule_kind,
            schedule_expr: body.schedule_expr,
        })
        .await
        .map_err(routine_error_to_api)?;
    log_activity(
        &state.activity,
        &existing.company_id,
        "routine_trigger.created",
        "routine_trigger",
        trigger["id"].as_str().unwrap_or(""),
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(trigger)))
}

/// `GET /api/routines/{id}/triggers` — lists triggers.
#[route(GET "/api/routines/{id}/triggers")]
pub async fn list_triggers(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(existing) = state
        .routines
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    else {
        return Err(ApiError::not_found("Routine not found"));
    };
    let triggers = state
        .routines
        .list_triggers(&existing.company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&triggers).unwrap_or_default()))
}

fn routine_error_to_api(error: staple_data::RoutineError) -> ApiError {
    use staple_data::RoutineError as E;
    match error {
        E::RoutineNotFound => ApiError::not_found("Routine not found"),
        E::ReferenceNotFound(reference) => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": [reference], "message": "Referenced record not found" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
