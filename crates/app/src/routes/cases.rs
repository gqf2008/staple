//! Cases routes (upstream cases.ts / Cases pages).

use serde::Deserialize;
use serde_json::json;
use staple_data::{CasePatch, CaseRecord, NewCase, NewCaseEvent};
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

/// Body for `POST /api/cases/{id}/issue-links`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkIssueRequest {
    /// Issue id.
    pub issue_id: String,
    /// Link role (`origin` | `work` | `reference`).
    #[serde(default = "default_link_role")]
    pub role: String,
}

fn default_link_role() -> String {
    "work".to_owned()
}

/// Body for `POST /api/cases/{id}/events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCaseEventRequest {
    /// Event kind (upstream `case_events.kind` CHECK).
    pub kind: String,
    /// Actor type (`user` | `agent` | `system`).
    pub actor_type: String,
    /// Actor user id.
    #[serde(default)]
    pub actor_user_id: Option<String>,
    /// Actor agent id.
    #[serde(default)]
    pub actor_agent_id: Option<String>,
    /// Originating run id.
    #[serde(default)]
    pub run_id: Option<String>,
    /// Event payload.
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Body for `POST /api/cases/{id}/documents`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkDocumentRequest {
    /// Document id.
    pub document_id: String,
    /// Document key within the case.
    pub key: String,
}

/// Body for `POST /api/cases/{id}/labels`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCaseLabelRequest {
    /// Label id.
    pub label_id: String,
}

/// Body for `POST /api/cases/{id}/attachments`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCaseAttachmentRequest {
    /// Asset id.
    pub asset_id: String,
}

const CASE_EVENT_KINDS: [&str; 11] = [
    "created",
    "updated",
    "fields_changed",
    "status_changed",
    "issue_linked",
    "issue_unlinked",
    "document_revised",
    "child_linked",
    "attachment_added",
    "label_added",
    "label_removed",
];

/// Resolves the owning company of `case_id` and enforces board scope.
async fn case_company(cx: &Cx, state: &AppState, case_id: &str) -> Result<String, ApiError> {
    let Some(company_id) = state
        .cases
        .company_of(case_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Case not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    Ok(company_id)
}

/// `POST /api/cases/{id}/issue-links` — links an issue to a case.
#[route(POST "/api/cases/{id}/issue-links")]
pub async fn link_case_issue(
    cx: &Cx,
    Json(body): Json<LinkIssueRequest>,
) -> Result<(StatusCode, Json<staple_data::CaseIssueLinkRecord>), ApiError> {
    if body.issue_id.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["issueId"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    if !matches!(body.role.as_str(), "origin" | "work" | "reference") {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["role"], "message": "Invalid enum value. Expected 'origin' | 'work' | 'reference'" }]),
        ));
    }
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let record = state
        .cases
        .link_issue(&company_id, &case_id, &body.issue_id, &body.role)
        .await
        .map_err(case_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "case.issue_linked",
        "case",
        &case_id,
        Some(json!({ "issueId": body.issue_id, "role": body.role })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/cases/{id}/issue-links` — lists linked issues.
#[route(GET "/api/cases/{id}/issue-links")]
pub async fn list_case_issue_links(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::CaseIssueLinkRecord>>, ApiError> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let records = state
        .cases
        .list_issue_links(&company_id, &case_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `DELETE /api/cases/{id}/issue-links/{issue_id}` — unlinks an issue.
#[route(DELETE "/api/cases/{id}/issue-links/{issue_id}")]
pub async fn unlink_case_issue(cx: &Cx) -> Result<StatusCode, ApiError> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let removed = state
        .cases
        .unlink_issue(&company_id, &case_id, &issue_id)
        .await
        .map_err(case_error_to_api)?;
    if !removed {
        return Err(ApiError::not_found("Issue link not found"));
    }
    log_activity(
        &state.activity,
        &company_id,
        "case.issue_unlinked",
        "case",
        &case_id,
        Some(json!({ "issueId": issue_id })),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/cases/{id}/events` — records a case event.
#[route(POST "/api/cases/{id}/events")]
pub async fn add_case_event(
    cx: &Cx,
    Json(body): Json<AddCaseEventRequest>,
) -> Result<(StatusCode, Json<staple_data::CaseEventRecord>), ApiError> {
    if !CASE_EVENT_KINDS.contains(&body.kind.as_str()) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["kind"], "message": "Invalid case event kind" }]),
        ));
    }
    if !matches!(body.actor_type.as_str(), "user" | "agent" | "system") {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["actorType"], "message": "Invalid enum value. Expected 'user' | 'agent' | 'system'" }]),
        ));
    }
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let record = state
        .cases
        .add_event(NewCaseEvent {
            company_id,
            case_id,
            kind: body.kind,
            actor_type: body.actor_type,
            actor_user_id: body.actor_user_id,
            actor_agent_id: body.actor_agent_id,
            run_id: body.run_id,
            payload: body.payload,
        })
        .await
        .map_err(case_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/cases/{id}/events` — lists case events.
#[route(GET "/api/cases/{id}/events")]
pub async fn list_case_events(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::CaseEventRecord>>, ApiError> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let records = state
        .cases
        .list_events(&company_id, &case_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `POST /api/cases/{id}/documents` — links a document to a case.
#[route(POST "/api/cases/{id}/documents")]
pub async fn link_case_document(
    cx: &Cx,
    Json(body): Json<LinkDocumentRequest>,
) -> Result<(StatusCode, Json<staple_data::CaseDocumentRecord>), ApiError> {
    if body.document_id.trim().is_empty() || body.key.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["documentId"], "message": "documentId and key are required" }]),
        ));
    }
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let record = state
        .cases
        .link_document(&company_id, &case_id, &body.document_id, &body.key)
        .await
        .map_err(case_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/cases/{id}/documents` — lists case documents.
#[route(GET "/api/cases/{id}/documents")]
pub async fn list_case_documents(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::CaseDocumentRecord>>, ApiError> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let records = state
        .cases
        .list_documents(&company_id, &case_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `POST /api/cases/{id}/labels` — adds a label to a case.
#[route(POST "/api/cases/{id}/labels")]
pub async fn add_case_label(
    cx: &Cx,
    Json(body): Json<AddCaseLabelRequest>,
) -> Result<(StatusCode, Json<staple_data::CaseLabelRecord>), ApiError> {
    if body.label_id.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["labelId"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let record = state
        .cases
        .add_label(&company_id, &case_id, &body.label_id)
        .await
        .map_err(case_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/cases/{id}/labels` — lists case labels.
#[route(GET "/api/cases/{id}/labels")]
pub async fn list_case_labels(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::CaseLabelRecord>>, ApiError> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let records = state
        .cases
        .list_labels(&company_id, &case_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `DELETE /api/cases/{id}/labels/{label_id}` — removes a case label.
#[route(DELETE "/api/cases/{id}/labels/{label_id}")]
pub async fn remove_case_label(cx: &Cx) -> Result<StatusCode, ApiError> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let label_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let removed = state
        .cases
        .remove_label(&company_id, &case_id, &label_id)
        .await
        .map_err(case_error_to_api)?;
    if !removed {
        return Err(ApiError::not_found("Case label not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/cases/{id}/attachments` — attaches an asset to a case.
#[route(POST "/api/cases/{id}/attachments")]
pub async fn add_case_attachment(
    cx: &Cx,
    Json(body): Json<AddCaseAttachmentRequest>,
) -> Result<(StatusCode, Json<staple_data::CaseAttachmentRecord>), ApiError> {
    if body.asset_id.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["assetId"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let record = state
        .cases
        .add_attachment(&company_id, &case_id, &body.asset_id)
        .await
        .map_err(case_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/cases/{id}/attachments` — lists case attachments.
#[route(GET "/api/cases/{id}/attachments")]
pub async fn list_case_attachments(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::CaseAttachmentRecord>>, ApiError> {
    let case_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = case_company(cx, state, &case_id).await?;
    let records = state
        .cases
        .list_attachments(&company_id, &case_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// Silence unused import warnings when no uuid validation is used yet.
#[allow(dead_code)]
fn _is_uuid(value: &str) -> bool {
    is_uuid(value)
}
