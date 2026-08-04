//! Pipelines routes (upstream Pipelines pages).

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewCaseEvent, NewPipeline, NewPipelineCase, NewStage, NewTransition, PipelineCaseEventRecord,
    PipelineCaseRecord, PipelineRecord, PipelineStageRecord, PipelineTransitionRecord,
};
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

/// Body for `POST /api/companies/{companyId}/pipelines`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePipelineRequest {
    /// Project id.
    #[serde(default)]
    pub project_id: Option<String>,
    /// Key (unique per company).
    pub key: String,
    /// Name.
    pub name: String,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Enforce transitions.
    #[serde(default)]
    pub enforce_transitions: Option<bool>,
}

/// Body for `POST /api/pipelines/{id}/stages`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStageRequest {
    /// Stage key.
    pub key: String,
    /// Stage name.
    pub name: String,
    /// Kind (`working` | `review` | `done` | `cancelled`).
    pub kind: String,
    /// Position.
    pub position: i64,
    /// Config JSON.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// Body for `POST /api/pipelines/{id}/transitions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransitionRequest {
    /// From stage id.
    pub from_stage_id: String,
    /// To stage id.
    pub to_stage_id: String,
    /// Label.
    #[serde(default)]
    pub label: Option<String>,
}

/// Body for `POST /api/pipelines/{id}/cases`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePipelineCaseRequest {
    /// Stage id.
    pub stage_id: String,
    /// Case key (unique per pipeline).
    pub case_key: String,
    /// Title.
    pub title: String,
    /// Summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// Fields JSON.
    #[serde(default)]
    pub fields: Option<serde_json::Value>,
    /// Workspace ref JSON.
    #[serde(default)]
    pub workspace_ref: Option<serde_json::Value>,
    /// Parent case id.
    #[serde(default)]
    pub parent_case_id: Option<String>,
}

/// Body for `POST /api/pipeline-cases/{id}/move`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveCaseRequest {
    /// Target stage id.
    pub to_stage_id: String,
    /// Force (bypass transition checks).
    #[serde(default)]
    pub force: Option<bool>,
}

/// Body for `POST /api/pipeline-cases/{id}/events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEventRequest {
    /// Event type.
    pub r#type: String,
    /// Actor type.
    pub actor_type: String,
    /// Actor user id.
    #[serde(default)]
    pub actor_user_id: Option<String>,
    /// Actor agent id.
    #[serde(default)]
    pub actor_agent_id: Option<String>,
    /// Run id.
    #[serde(default)]
    pub run_id: Option<String>,
    /// Payload JSON.
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

fn validate_pipeline(body: &CreatePipelineRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.key.trim().is_empty() {
        issues.push(
            json!({ "path": ["key"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.name.trim().is_empty() {
        issues.push(
            json!({ "path": ["name"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn validate_stage(body: &CreateStageRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.key.trim().is_empty() {
        issues.push(
            json!({ "path": ["key"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.name.trim().is_empty() {
        issues.push(
            json!({ "path": ["name"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if !matches!(
        body.kind.as_str(),
        "working" | "review" | "done" | "cancelled"
    ) {
        issues.push(json!({
            "path": ["kind"],
            "message": "Invalid enum value. Expected 'working' | 'review' | 'done' | 'cancelled'",
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn validate_case(body: &CreatePipelineCaseRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.case_key.trim().is_empty() {
        issues.push(json!({ "path": ["caseKey"], "message": "String must contain at least 1 character(s)" }));
    }
    if body.title.trim().is_empty() {
        issues.push(
            json!({ "path": ["title"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if !is_uuid(&body.stage_id) {
        issues.push(json!({ "path": ["stageId"], "message": "Invalid uuid" }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/pipelines` — lists pipelines.
#[route(GET "/api/companies/{company_id}/pipelines")]
pub async fn list_pipelines(cx: &Cx) -> Result<Json<Vec<PipelineRecord>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let rows = state
        .pipelines
        .list_pipelines(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/companies/{companyId}/pipelines` — creates a pipeline.
#[route(POST "/api/companies/{company_id}/pipelines")]
pub async fn create_pipeline(
    cx: &Cx,
    Json(body): Json<CreatePipelineRequest>,
) -> Result<(StatusCode, Json<PipelineRecord>), ApiError> {
    validate_pipeline(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let pipeline = state
        .pipelines
        .create_pipeline(NewPipeline {
            company_id: company_id.clone(),
            project_id: body.project_id,
            key: body.key,
            name: body.name,
            description: body.description,
            enforce_transitions: body.enforce_transitions.unwrap_or(false),
            created_by_user_id: Some("board".to_owned()),
        })
        .await
        .map_err(pipeline_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "pipeline.created",
        "pipeline",
        &pipeline.id,
        Some(json!({ "key": pipeline.key, "name": pipeline.name })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(pipeline)))
}

/// `GET /api/pipelines/{id}` — fetches a pipeline.
#[route(GET "/api/pipelines/{id}")]
pub async fn get_pipeline(cx: &Cx) -> Result<Json<PipelineRecord>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    state
        .pipelines
        .get_pipeline(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Pipeline not found"))
}

/// `POST /api/pipelines/{id}/archive` — archives/unarchives a pipeline.
#[route(POST "/api/pipelines/{id}/archive")]
pub async fn archive_pipeline(
    cx: &Cx,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<PipelineRecord>, ApiError> {
    let archived = body
        .get("archived")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    state
        .pipelines
        .set_pipeline_archived(&company_id, &id, archived)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Pipeline not found"))
}

/// `DELETE /api/pipelines/{id}` — deletes a pipeline.
#[route(DELETE "/api/pipelines/{id}")]
pub async fn delete_pipeline(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    match state
        .pipelines
        .delete_pipeline(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Pipeline not found")),
    }
}

/// `GET /api/pipelines/{id}/stages` — lists stages.
#[route(GET "/api/pipelines/{id}/stages")]
pub async fn list_stages(cx: &Cx) -> Result<Json<Vec<PipelineStageRecord>>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let rows = state
        .pipelines
        .list_stages(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/pipelines/{id}/stages` — creates a stage.
#[route(POST "/api/pipelines/{id}/stages")]
pub async fn create_stage(
    cx: &Cx,
    Json(body): Json<CreateStageRequest>,
) -> Result<(StatusCode, Json<PipelineStageRecord>), ApiError> {
    validate_stage(&body)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let stage = state
        .pipelines
        .create_stage(NewStage {
            company_id: company_id.clone(),
            pipeline_id: id,
            key: body.key,
            name: body.name,
            kind: body.kind,
            position: body.position,
            config: body.config,
        })
        .await
        .map_err(pipeline_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "pipeline_stage.created",
        "pipeline_stage",
        &stage.id,
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(stage)))
}

/// `DELETE /api/pipeline-stages/{id}` — deletes a stage.
#[route(DELETE "/api/pipeline-stages/{id}")]
pub async fn delete_stage(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_stage(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Stage not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    match state
        .pipelines
        .delete_stage(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Stage not found")),
    }
}

/// `GET /api/pipelines/{id}/transitions` — lists transitions.
#[route(GET "/api/pipelines/{id}/transitions")]
pub async fn list_transitions(cx: &Cx) -> Result<Json<Vec<PipelineTransitionRecord>>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let rows = state
        .pipelines
        .list_transitions(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/pipelines/{id}/transitions` — creates a transition edge.
#[route(POST "/api/pipelines/{id}/transitions")]
pub async fn create_transition(
    cx: &Cx,
    Json(body): Json<CreateTransitionRequest>,
) -> Result<(StatusCode, Json<PipelineTransitionRecord>), ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let transition = state
        .pipelines
        .create_transition(NewTransition {
            company_id: company_id.clone(),
            pipeline_id: id,
            from_stage_id: body.from_stage_id,
            to_stage_id: body.to_stage_id,
            label: body.label,
        })
        .await
        .map_err(pipeline_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "pipeline_transition.created",
        "pipeline_transition",
        &transition.id,
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(transition)))
}

/// `GET /api/pipelines/{id}/cases` — lists pipeline cases.
#[route(GET "/api/pipelines/{id}/cases")]
pub async fn list_cases(cx: &Cx) -> Result<Json<Vec<PipelineCaseRecord>>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let rows = state
        .pipelines
        .list_cases(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/pipelines/{id}/cases` — creates a pipeline case.
#[route(POST "/api/pipelines/{id}/cases")]
pub async fn create_case(
    cx: &Cx,
    Json(body): Json<CreatePipelineCaseRequest>,
) -> Result<(StatusCode, Json<PipelineCaseRecord>), ApiError> {
    validate_case(&body)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_pipeline(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Pipeline not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let case = state
        .pipelines
        .create_case(NewPipelineCase {
            company_id: company_id.clone(),
            pipeline_id: id,
            stage_id: body.stage_id,
            case_key: body.case_key,
            title: body.title,
            summary: body.summary,
            fields: body.fields,
            workspace_ref: body.workspace_ref,
            parent_case_id: body.parent_case_id,
            created_by_user_id: Some("board".to_owned()),
        })
        .await
        .map_err(pipeline_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "pipeline_case.created",
        "pipeline_case",
        &case.id,
        Some(json!({ "caseKey": case.case_key, "title": case.title })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(case)))
}

/// `GET /api/pipeline-cases/{id}` — fetches a pipeline case.
#[route(GET "/api/pipeline-cases/{id}")]
pub async fn get_case(cx: &Cx) -> Result<Json<PipelineCaseRecord>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_case(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    state
        .pipelines
        .get_case(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Case not found"))
}

/// `POST /api/pipeline-cases/{id}/move` — moves a case to another stage.
#[route(POST "/api/pipeline-cases/{id}/move")]
pub async fn move_case(
    cx: &Cx,
    Json(body): Json<MoveCaseRequest>,
) -> Result<Json<PipelineCaseRecord>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_case(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let case = state
        .pipelines
        .move_case(
            &company_id,
            &id,
            &body.to_stage_id,
            "user",
            Some("board".to_owned()),
            None,
            body.force.unwrap_or(false),
        )
        .await
        .map_err(pipeline_error_to_api)?
        .ok_or_else(|| ApiError::not_found("Case not found"))?;
    log_activity(
        &state.activity,
        &company_id,
        "pipeline_case.moved",
        "pipeline_case",
        &case.id,
        Some(json!({ "stageId": case.stage_id })),
    )
    .await?;
    Ok(Json(case))
}

/// `GET /api/pipeline-cases/{id}/events` — lists case events.
#[route(GET "/api/pipeline-cases/{id}/events")]
pub async fn list_events(cx: &Cx) -> Result<Json<Vec<PipelineCaseEventRecord>>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_case(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let rows = state
        .pipelines
        .list_events(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/pipeline-cases/{id}/events` — appends a case event.
#[route(POST "/api/pipeline-cases/{id}/events")]
pub async fn add_event(
    cx: &Cx,
    Json(body): Json<AddEventRequest>,
) -> Result<(StatusCode, Json<PipelineCaseEventRecord>), ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .pipelines
        .company_of_case(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let event = state
        .pipelines
        .add_event(NewCaseEvent {
            company_id: company_id.clone(),
            case_id: id,
            r#type: body.r#type,
            actor_type: body.actor_type,
            actor_user_id: body.actor_user_id,
            actor_agent_id: body.actor_agent_id,
            run_id: body.run_id,
            from_stage_id: None,
            to_stage_id: None,
            payload: body.payload,
        })
        .await
        .map_err(pipeline_error_to_api)?;
    Ok((StatusCode::CREATED, Json(event)))
}

fn pipeline_error_to_api(error: staple_data::PipelineError) -> ApiError {
    use staple_data::PipelineError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ReferenceNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "Referenced record not found" }]),
        ),
        E::NotFound => ApiError::not_found("Not found"),
        E::Duplicate => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["key"], "message": "Duplicate key" }]),
        ),
        E::TransitionNotAllowed => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["toStageId"], "message": "Transition not allowed by pipeline" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
