//! Instruction document and agent instruction file routes.
//!
//! Company-scoped, board-only management surface for the instruction system:
//! a document library plus per-agent mounted bundle files (managed
//! instructions semantics, upstream `agent-instructions.ts` parity).

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    InstructionError, NewAgentInstructionFile, NewInstructionDocument, UpdateInstructionDocument,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::{enforce_company_scope, require_board},
    error::ApiError,
    instructions::validate_instruction_path,
    routes::{AgentId, CompanyId, Id},
    state::AppState,
};

/// `{path}` path parameter (instruction file path inside the bundle).
#[path_param(error = bad_request("Invalid instruction file path"))]
pub(crate) struct Path(String);

/// Body for `POST /api/companies/{companyId}/instruction-documents`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstructionDocumentRequest {
    /// Document name.
    pub name: String,
    /// Document content.
    pub content: String,
}

/// Body for `PATCH /api/instruction-documents/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstructionDocumentRequest {
    /// New document name.
    #[serde(default)]
    pub name: Option<String>,
    /// New document content.
    #[serde(default)]
    pub content: Option<String>,
}

/// Body for `PUT .../agents/{agentId}/instructions/{path}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAgentInstructionFileRequest {
    /// File content.
    pub content: String,
    /// Whether this file is the bundle entry file.
    #[serde(default)]
    pub is_entry: bool,
}

fn instruction_error_to_api(error: InstructionError) -> ApiError {
    use InstructionError as E;
    match error {
        E::ReferenceNotFound(reference) => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": [reference], "message": "Referenced record not found" }]),
        ),
        E::NotFound => ApiError::not_found("Instruction not found"),
        other => ApiError::internal(other.to_string()),
    }
}

/// `GET /api/companies/{companyId}/instruction-documents` — lists documents.
#[route(GET "/api/companies/{company_id}/instruction-documents")]
pub async fn list_instruction_documents(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::InstructionDocumentRecord>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let documents = state
        .instructions
        .list_documents(&company_id)
        .await
        .map_err(instruction_error_to_api)?;
    Ok(Json(documents))
}

/// `POST /api/companies/{companyId}/instruction-documents` — creates a
/// document.
#[route(POST "/api/companies/{company_id}/instruction-documents")]
pub async fn create_instruction_document(
    cx: &Cx,
    Json(body): Json<CreateInstructionDocumentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.name.trim().is_empty() {
        issues.push(json!({
            "path": ["name"],
            "message": "String must contain at least 1 character(s)"
        }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let document = state
        .instructions
        .create_document(NewInstructionDocument {
            company_id: company_id.clone(),
            name: body.name.trim().to_owned(),
            content: body.content,
        })
        .await
        .map_err(instruction_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "instruction_document.created",
        "instruction_document",
        &document.id,
        Some(json!({ "name": document.name })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&document).unwrap_or_default()),
    ))
}

/// `GET /api/instruction-documents/{id}` — fetches one document.
#[route(GET "/api/instruction-documents/{id}")]
pub async fn get_instruction_document(
    cx: &Cx,
) -> Result<Json<staple_data::InstructionDocumentRecord>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(document) = state
        .instructions
        .get_document(&id)
        .await
        .map_err(instruction_error_to_api)?
    else {
        return Err(ApiError::not_found("Instruction document not found"));
    };
    enforce_company_scope(cx, &document.company_id)?;
    require_board(cx)?;
    Ok(Json(document))
}

/// `PATCH /api/instruction-documents/{id}` — updates a document.
#[route(PATCH "/api/instruction-documents/{id}")]
pub async fn update_instruction_document(
    cx: &Cx,
    Json(body): Json<UpdateInstructionDocumentRequest>,
) -> Result<Json<staple_data::InstructionDocumentRecord>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(existing) = state
        .instructions
        .get_document(&id)
        .await
        .map_err(instruction_error_to_api)?
    else {
        return Err(ApiError::not_found("Instruction document not found"));
    };
    enforce_company_scope(cx, &existing.company_id)?;
    require_board(cx)?;
    let name = match body.name {
        Some(name) if name.trim().is_empty() => {
            return Err(ApiError::unprocessable(
                "Validation error",
                json!([{
                    "path": ["name"],
                    "message": "String must contain at least 1 character(s)"
                }]),
            ));
        }
        Some(name) => name.trim().to_owned(),
        None => existing.name,
    };
    let document = state
        .instructions
        .update_document(UpdateInstructionDocument {
            id,
            company_id: existing.company_id.clone(),
            name,
            content: body.content.unwrap_or(existing.content),
        })
        .await
        .map_err(instruction_error_to_api)?;
    log_activity(
        &state.activity,
        &existing.company_id,
        "instruction_document.updated",
        "instruction_document",
        &document.id,
        Some(json!({ "name": document.name })),
    )
    .await?;
    Ok(Json(document))
}

/// `DELETE /api/instruction-documents/{id}` — deletes a document.
#[route(DELETE "/api/instruction-documents/{id}")]
pub async fn delete_instruction_document(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(existing) = state
        .instructions
        .get_document(&id)
        .await
        .map_err(instruction_error_to_api)?
    else {
        return Err(ApiError::not_found("Instruction document not found"));
    };
    enforce_company_scope(cx, &existing.company_id)?;
    require_board(cx)?;
    let deleted = state
        .instructions
        .delete_document(&id, &existing.company_id)
        .await
        .map_err(instruction_error_to_api)?;
    log_activity(
        &state.activity,
        &existing.company_id,
        "instruction_document.deleted",
        "instruction_document",
        &id,
        Some(json!({ "name": existing.name, "deleted": deleted })),
    )
    .await?;
    Ok(Json(json!({ "deleted": deleted })))
}

/// `GET /api/companies/{companyId}/agents/{agentId}/instructions` — lists the
/// mounted instruction files for an agent.
#[route(GET "/api/companies/{company_id}/agents/{agent_id}/instructions")]
pub async fn list_agent_instruction_files(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::AgentInstructionFileRecord>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let files = state
        .instructions
        .list_agent_files(&company_id, &agent_id)
        .await
        .map_err(instruction_error_to_api)?;
    Ok(Json(files))
}

/// `PUT /api/companies/{companyId}/agents/{agentId}/instructions/{path}` —
/// creates or replaces an instruction file on an agent.
#[route(PUT "/api/companies/{company_id}/agents/{agent_id}/instructions/{path}")]
pub async fn upsert_agent_instruction_file(
    cx: &Cx,
    Json(body): Json<UpsertAgentInstructionFileRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let path = validate_instruction_path(&path_param::<Path>(cx)?.to_string())?;
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let file = state
        .instructions
        .upsert_agent_file(NewAgentInstructionFile {
            company_id: company_id.clone(),
            agent_id: agent_id.clone(),
            path: path.clone(),
            content: body.content,
            is_entry: body.is_entry,
        })
        .await
        .map_err(instruction_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "instruction_file.upserted",
        "instruction_file",
        &file.id,
        Some(json!({ "agentId": agent_id, "path": path, "isEntry": file.is_entry })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&file).unwrap_or_default()),
    ))
}

/// `DELETE /api/companies/{companyId}/agents/{agentId}/instructions/{path}` —
/// deletes an instruction file from an agent.
#[route(DELETE "/api/companies/{company_id}/agents/{agent_id}/instructions/{path}")]
pub async fn delete_agent_instruction_file(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let path = validate_instruction_path(&path_param::<Path>(cx)?.to_string())?;
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let deleted = state
        .instructions
        .delete_agent_file(&company_id, &agent_id, &path)
        .await
        .map_err(instruction_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "instruction_file.deleted",
        "instruction_file",
        &path,
        Some(json!({ "agentId": agent_id, "path": path, "deleted": deleted })),
    )
    .await?;
    Ok(Json(json!({ "deleted": deleted })))
}
