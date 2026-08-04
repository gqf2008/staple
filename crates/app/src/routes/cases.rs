//! Cases routes (upstream cases.ts / Cases pages).

use serde::Deserialize;
use serde_json::json;
use staple_data::{CasePatch, CaseRecord, NewCase};
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

/// Body for `POST /api/companies/{companyId}/cases`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCaseRequest {
    /// Project id.
    #[serde(default)]
    pub project_id: Option<String>,
    /// Case type.
    pub case_type: String,
    /// Type-scoped key.
    #[serde(default)]
    pub key: Option<String>,
    /// Title.
    pub title: String,
    /// Summary.
    #[serde(default)]
    pub summary: Option<String>,
    /// Fields JSON.
    #[serde(default)]
    pub fields: Option<serde_json::Value>,
    /// Parent case id.
    #[serde(default)]
    pub parent_case_id: Option<String>,
}

/// Body for `PATCH /api/cases/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCaseRequest {
    /// New title.
    #[serde(default)]
    pub title: Option<String>,
    /// New summary (`null` clears).
    #[serde(default)]
    pub summary: Option<Option<String>>,
    /// New fields.
    #[serde(default)]
    pub fields: Option<serde_json::Value>,
    /// New parent case (`null` clears).
    #[serde(default)]
    pub parent_case_id: Option<Option<String>>,
}

/// Body for `POST /api/cases/{id}/status`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetCaseStatusRequest {
    /// Target status.
    pub status: String,
}

fn validate_create(body: &CreateCaseRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.title.trim().is_empty() {
        issues.push(
            json!({ "path": ["title"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.case_type.trim().is_empty() {
        issues.push(json!({ "path": ["caseType"], "message": "String must contain at least 1 character(s)" }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/cases?projectId=...` — lists cases.
#[route(GET "/api/companies/{company_id}/cases")]
pub async fn list_cases(cx: &Cx) -> Result<Json<Vec<CaseRecord>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let project_id = topcoat::router::query_params::<CasesQuery>(cx)
        .ok()
        .and_then(|query| query.project_id.clone());
    let state = app_context::<AppState>(cx);
    let cases = state
        .cases
        .list(&company_id, project_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(cases))
}

/// `POST /api/companies/{companyId}/cases` — creates a case.
#[route(POST "/api/companies/{company_id}/cases")]
pub async fn create_case(
    cx: &Cx,
    Json(body): Json<CreateCaseRequest>,
) -> Result<(StatusCode, Json<CaseRecord>), ApiError> {
    validate_create(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let case = state
        .cases
        .create(NewCase {
            company_id: company_id.clone(),
            project_id: body.project_id,
            case_type: body.case_type,
            key: body.key,
            title: body.title,
            summary: body.summary,
            fields: body.fields,
            parent_case_id: body.parent_case_id,
            created_by_agent_id: None,
            created_by_user_id: Some("board".to_owned()),
        })
        .await
        .map_err(case_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "case.created",
        "case",
        &case.id,
        Some(json!({ "identifier": case.identifier, "title": case.title })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(case)))
}

/// `GET /api/cases/{id}` — fetches one case.
#[route(GET "/api/cases/{id}")]
pub async fn get_case(cx: &Cx) -> Result<Json<CaseRecord>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .cases
        .company_of(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    state
        .cases
        .get(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Case not found"))
}

/// `PATCH /api/cases/{id}` — partially updates a case.
#[route(PATCH "/api/cases/{id}")]
pub async fn update_case(
    cx: &Cx,
    Json(body): Json<UpdateCaseRequest>,
) -> Result<Json<CaseRecord>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .cases
        .company_of(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let case = state
        .cases
        .update(
            &company_id,
            &id,
            CasePatch {
                title: body.title,
                summary: body.summary,
                fields: body.fields,
                parent_case_id: body.parent_case_id,
            },
        )
        .await
        .map_err(case_error_to_api)?
        .ok_or_else(|| ApiError::not_found("Case not found"))?;
    log_activity(
        &state.activity,
        &company_id,
        "case.updated",
        "case",
        &case.id,
        None,
    )
    .await?;
    Ok(Json(case))
}

/// `POST /api/cases/{id}/status` — moves a case through the state machine.
#[route(POST "/api/cases/{id}/status")]
pub async fn set_case_status(
    cx: &Cx,
    Json(body): Json<SetCaseStatusRequest>,
) -> Result<Json<CaseRecord>, ApiError> {
    if !matches!(
        body.status.as_str(),
        "draft" | "in_progress" | "in_review" | "approved" | "done" | "cancelled"
    ) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{
                "path": ["status"],
                "message": "Invalid enum value. Expected 'draft' | 'in_progress' | 'in_review' | 'approved' | 'done' | 'cancelled'",
            }]),
        ));
    }
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .cases
        .company_of(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let case = state
        .cases
        .set_status(&company_id, &id, &body.status)
        .await
        .map_err(case_error_to_api)?
        .ok_or_else(|| ApiError::not_found("Case not found"))?;
    log_activity(
        &state.activity,
        &company_id,
        "case.status_updated",
        "case",
        &case.id,
        Some(json!({ "status": case.status })),
    )
    .await?;
    Ok(Json(case))
}

/// `DELETE /api/cases/{id}` — deletes a case.
#[route(DELETE "/api/cases/{id}")]
pub async fn delete_case(cx: &Cx) -> Result<StatusCode, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .cases
        .company_of(&id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    match state
        .cases
        .delete(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(case) => {
            log_activity(
                &state.activity,
                &company_id,
                "case.deleted",
                "case",
                &case.id,
                None,
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Case not found")),
    }
}

/// Query for listing cases.
#[topcoat::router::query_params]
struct CasesQuery {
    /// Optional project filter.
    #[serde(rename = "projectId")]
    project_id: Option<String>,
}

fn case_error_to_api(error: staple_data::CaseError) -> ApiError {
    use staple_data::CaseError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ReferenceNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "Referenced record not found" }]),
        ),
        E::CaseNotFound => ApiError::not_found("Case not found"),
        E::InvalidStatusTransition(from, to) => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["status"], "message": format!("Invalid status transition: {from} -> {to}") }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}

/// Silence unused import warnings when no uuid validation is used yet.
#[allow(dead_code)]
fn _is_uuid(value: &str) -> bool {
    is_uuid(value)
}
