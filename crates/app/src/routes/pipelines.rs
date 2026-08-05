//! Pipelines routes (upstream Pipelines pages).

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewPipeline, NewPipelineCase, NewPipelineCaseEvent, NewStage, NewTransition,
    PipelineCaseEventRecord, PipelineCaseRecord, PipelineRecord, PipelineStageRecord,
    PipelineTransitionRecord,
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
        .add_event(NewPipelineCaseEvent {
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

/// Reads a query parameter value from the request.
fn query_param(cx: &Cx, key: &str) -> Option<String> {
    topcoat::context::try_request_context::<http::request::Parts>(cx).and_then(|parts| {
        parts.uri.query().and_then(|query| {
            query.split('&').find_map(|pair| {
                let (k, value) = pair.split_once('=')?;
                (k == key).then(|| value.to_owned())
            })
        })
    })
}

/// Body for `POST /api/companies/{companyId}/review-cases/bulk`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkReviewItem {
    /// Case id.
    pub case_id: String,
    /// Decision (`approve` | `request_changes` | `reject`).
    pub decision: String,
    /// Reason.
    #[serde(default)]
    pub reason: Option<String>,
    /// Expected case version (optimistic lock).
    #[serde(default)]
    pub expected_version: Option<i64>,
}

/// Body for `POST /api/companies/{companyId}/review-cases/bulk`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkReviewRequest {
    /// Review decisions.
    pub items: Vec<BulkReviewItem>,
}

/// Body for `POST /api/cases/{caseId}/resolve-suggestion`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveSuggestionRequest {
    /// Suggestion id.
    pub suggestion_id: String,
    /// Decision (`accept` | `dismiss`).
    pub decision: String,
    /// Reason.
    #[serde(default)]
    pub reason: Option<String>,
    /// Expected case version (optimistic lock).
    #[serde(default)]
    pub expected_version: Option<i64>,
}

/// `GET /api/companies/{companyId}/pipelines-attention` — attention feed.
#[route(GET "/api/companies/{company_id}/pipelines-attention")]
pub async fn pipelines_attention(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let limit = query_param(cx, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(50)
        .min(100);
    let state = app_context::<AppState>(cx);
    let feed = state
        .pipelines
        .list_attention(&company_id, limit)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(feed))
}

/// `GET /api/companies/{companyId}/case-events` — company-wide case events.
#[route(GET "/api/companies/{company_id}/case-events")]
pub async fn company_case_events(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let types: Vec<String> = query_param(cx, "types")
        .map(|value| value.split(',').map(str::to_owned).collect())
        .unwrap_or_default();
    let limit = query_param(cx, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(50)
        .min(100);
    let offset = query_param(cx, "offset")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(0);
    let state = app_context::<AppState>(cx);
    let events = state
        .pipelines
        .list_company_case_events(&company_id, &types, limit, offset)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(events))
}

/// `GET /api/companies/{companyId}/review-cases` — review-stage cases.
#[route(GET "/api/companies/{company_id}/review-cases")]
pub async fn review_cases(cx: &Cx) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let pipeline_id = query_param(cx, "pipelineId");
    let parent_case_id = query_param(cx, "parentCaseId");
    let state = app_context::<AppState>(cx);
    let cases = state
        .pipelines
        .list_review_cases(
            &company_id,
            pipeline_id.as_deref(),
            parent_case_id.as_deref(),
        )
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(cases))
}

/// `POST /api/companies/{companyId}/review-cases/bulk` — bulk review decisions.
#[route(POST "/api/companies/{company_id}/review-cases/bulk")]
pub async fn bulk_review_cases(
    cx: &Cx,
    Json(body): Json<BulkReviewRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let mut results = Vec::new();
    for item in body.items {
        let expected = item.expected_version.unwrap_or(0);
        let result = state
            .pipelines
            .review_case(
                &company_id,
                &item.case_id,
                &item.decision,
                expected,
                item.reason.as_deref(),
                "user",
                Some("board"),
                None,
            )
            .await;
        match result {
            Ok(case) => results.push(serde_json::json!({
                "caseId": item.case_id,
                "ok": true,
                "result": case,
            })),
            Err(error) => results.push(serde_json::json!({
                "caseId": item.case_id,
                "ok": false,
                "error": {
                    "status": pipeline_error_status(&error),
                    "message": error.to_string(),
                },
            })),
        }
    }
    Ok(Json(json!({ "results": results })))
}

/// `POST /api/cases/{case_id}/resolve-suggestion` — resolve a pending suggestion.
#[route(POST "/api/cases/{case_id}/resolve-suggestion")]
pub async fn resolve_suggestion(
    cx: &Cx,
    Json(body): Json<ResolveSuggestionRequest>,
) -> Result<Json<PipelineCaseRecord>, ApiError> {
    let case_id = path_param::<CaseId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = state
        .pipelines
        .company_of_case(&case_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Case not found"))?;
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let case = state
        .pipelines
        .resolve_suggestion(
            &company_id,
            &case_id,
            &body.suggestion_id,
            &body.decision,
            body.expected_version,
            body.reason.as_deref(),
            "user",
            Some("board"),
            None,
        )
        .await
        .map_err(pipeline_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "pipeline.suggestion_resolved",
        "pipeline_case",
        &case_id,
        Some(json!({ "decision": body.decision, "suggestionId": body.suggestion_id })),
    )
    .await?;
    Ok(Json(case))
}

fn pipeline_error_status(error: &staple_data::PipelineError) -> u16 {
    use staple_data::PipelineError as E;
    match error {
        E::SuggestionNotPending | E::VersionConflict => 409,
        E::NotFound => 404,
        E::ReferenceNotFound => 422,
        _ => 500,
    }
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
        E::SuggestionNotPending => ApiError::conflict("Pipeline suggestion is not pending"),
        E::VersionConflict => ApiError::conflict("Case version conflict"),
        other => ApiError::internal(other.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Extension surfaces: issue links, blockers, documents, automation executions
// ---------------------------------------------------------------------------

/// `POST /api/pipeline-cases/{id}/issue-links` — links an issue to a case.
#[route(POST "/api/pipeline-cases/{id}/issue-links")]
pub async fn link_issue(
    cx: &Cx,
    Json(body): Json<LinkIssueRequest>,
) -> Result<(StatusCode, Json<staple_data::PipelineCaseIssueLinkRecord>), ApiError> {
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
    let link = state
        .pipelines
        .link_issue(
            &company_id,
            &id,
            &body.issue_id,
            body.role.as_deref().unwrap_or("work"),
        )
        .await
        .map_err(pipeline_error_to_api)?;
    Ok((StatusCode::CREATED, Json(link)))
}

/// `GET /api/pipeline-cases/{id}/issue-links` — lists issue links.
#[route(GET "/api/pipeline-cases/{id}/issue-links")]
pub async fn list_issue_links(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PipelineCaseIssueLinkRecord>>, ApiError> {
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
        .list_issue_links(&company_id, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(rows))
}

/// `DELETE /api/pipeline-cases/{id}/issue-links/{issue_id}` — unlinks.
#[route(DELETE "/api/pipeline-cases/{id}/issue-links/{issue_id}")]
pub async fn unlink_issue(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let issue_id = path_param::<IssueId>(cx)?.to_string();
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
    if state
        .pipelines
        .unlink_issue(&company_id, &id, &issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found("Issue link not found"))
    }
}

/// `POST /api/pipeline-cases/{id}/blockers` — adds a blocker edge.
#[route(POST "/api/pipeline-cases/{id}/blockers")]
pub async fn add_blocker(
    cx: &Cx,
    Json(body): Json<BlockerRequest>,
) -> Result<(StatusCode, Json<staple_data::PipelineCaseBlockerRecord>), ApiError> {
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
    let blocker = state
        .pipelines
        .add_blocker(&company_id, &id, &body.blocked_by_case_id)
        .await
        .map_err(pipeline_error_to_api)?;
    Ok((StatusCode::CREATED, Json(blocker)))
}

/// `GET /api/pipeline-cases/{id}/blockers` — lists blockers.
#[route(GET "/api/pipeline-cases/{id}/blockers")]
pub async fn list_blockers(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PipelineCaseBlockerRecord>>, ApiError> {
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
        .list_blockers(&company_id, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(rows))
}

/// `DELETE /api/pipeline-cases/{id}/blockers/{blocked_by_case_id}` — removes.
#[route(DELETE "/api/pipeline-cases/{id}/blockers/{blocked_by_case_id}")]
pub async fn remove_blocker(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let blocked_by = path_param::<BlockerId>(cx)?.to_string();
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
    if state
        .pipelines
        .remove_blocker(&company_id, &id, &blocked_by)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found("Blocker not found"))
    }
}

/// `POST /api/pipelines/{id}/documents` — links a document to a pipeline.
#[route(POST "/api/pipelines/{id}/documents")]
pub async fn link_pipeline_document(
    cx: &Cx,
    Json(body): Json<DocumentLinkRequest>,
) -> Result<(StatusCode, Json<staple_data::PipelineDocumentRecord>), ApiError> {
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
    let link = state
        .pipelines
        .link_pipeline_document(&company_id, &id, &body.document_id, &body.key)
        .await
        .map_err(pipeline_error_to_api)?;
    Ok((StatusCode::CREATED, Json(link)))
}

/// `GET /api/pipelines/{id}/documents` — lists pipeline documents.
#[route(GET "/api/pipelines/{id}/documents")]
pub async fn list_pipeline_documents(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PipelineDocumentRecord>>, ApiError> {
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
        .list_pipeline_documents(&company_id, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/pipeline-cases/{id}/documents` — links a document to a case.
#[route(POST "/api/pipeline-cases/{id}/documents")]
pub async fn link_case_document(
    cx: &Cx,
    Json(body): Json<DocumentLinkRequest>,
) -> Result<(StatusCode, Json<staple_data::PipelineCaseDocumentRecord>), ApiError> {
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
    let link = state
        .pipelines
        .link_case_document(&company_id, &id, &body.document_id, &body.key)
        .await
        .map_err(pipeline_error_to_api)?;
    Ok((StatusCode::CREATED, Json(link)))
}

/// `GET /api/pipeline-cases/{id}/documents` — lists case documents.
#[route(GET "/api/pipeline-cases/{id}/documents")]
pub async fn list_case_documents(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PipelineCaseDocumentRecord>>, ApiError> {
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
        .list_case_documents(&company_id, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/pipeline-cases/{id}/automations` — records an automation execution.
#[route(POST "/api/pipeline-cases/{id}/automations")]
pub async fn record_automation(
    cx: &Cx,
    Json(body): Json<AutomationRequest>,
) -> Result<
    (
        StatusCode,
        Json<staple_data::PipelineAutomationExecutionRecord>,
    ),
    ApiError,
> {
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
    let record = state
        .pipelines
        .record_automation(
            &company_id,
            &id,
            &body.automation_id,
            &body.triggering_event_id,
            &body.routine_id,
            &body.status,
            body.execution_issue_id,
            body.error,
        )
        .await
        .map_err(pipeline_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/pipeline-cases/{id}/automations` — lists automation executions.
#[route(GET "/api/pipeline-cases/{id}/automations")]
pub async fn list_automations(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PipelineAutomationExecutionRecord>>, ApiError> {
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
        .list_automations(&company_id, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(rows))
}

/// Body for linking an issue.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkIssueRequest {
    /// Issue id.
    pub issue_id: String,
    /// Role (`origin` | `conversation` | `work` | `automation`).
    #[serde(default)]
    pub role: Option<String>,
}

/// Body for adding a blocker.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockerRequest {
    /// Blocking case id.
    pub blocked_by_case_id: String,
}

/// Body for linking a document.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkRequest {
    /// Document id.
    pub document_id: String,
    /// Key.
    pub key: String,
}

/// Body for recording an automation execution.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRequest {
    /// Automation id.
    pub automation_id: String,
    /// Triggering event id.
    pub triggering_event_id: String,
    /// Routine id.
    pub routine_id: String,
    /// Status (`succeeded` | `failed`).
    pub status: String,
    /// Execution issue id.
    #[serde(default)]
    pub execution_issue_id: Option<String>,
    /// Error.
    #[serde(default)]
    pub error: Option<String>,
}

/// `{issue_id}` path parameter.
#[path_param(error = bad_request("Invalid issue id"))]
pub(crate) struct IssueId(String);

/// `{case_id}` path parameter.
#[path_param(error = bad_request("Invalid case id"))]
pub(crate) struct CaseId(String);

/// `{blocked_by_case_id}` path parameter.
#[path_param(error = bad_request("Invalid case id"))]
pub(crate) struct BlockerId(String);
