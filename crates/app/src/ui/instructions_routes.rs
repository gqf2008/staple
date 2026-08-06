//! Instruction UI form handlers: accept HTML form posts and redirect back to
//! the instruction pages (mirrors `ui/routes.rs` for the instruction domain).

use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Form, error::see_other, path_param, route},
};

use crate::{audit::log_activity, instructions::validate_instruction_path, state::AppState};

/// Shared `{company_id}` path parameter for instruction UI routes.
#[path_param(error = bad_request("Invalid company id"))]
pub(crate) struct CompanyId(String);

/// Shared `{agent_id}` path parameter for instruction UI routes.
#[path_param(error = bad_request("Invalid agent id"))]
pub(crate) struct AgentId(String);

/// Shared `{id}` path parameter for instruction UI routes.
#[path_param(error = bad_request("Invalid id"))]
pub(crate) struct Id(String);

/// `{path}` path parameter for instruction UI routes.
#[path_param(error = bad_request("Invalid instruction file path"))]
pub(crate) struct Path(String);

/// Instruction document create form.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionDocumentForm {
    /// Document name.
    pub name: String,
    /// Document content.
    #[serde(default)]
    pub content: String,
}

/// Agent instruction file upsert form.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstructionFileForm {
    /// Relative path inside the bundle.
    pub path: String,
    /// File content.
    #[serde(default)]
    pub content: String,
    /// Whether this file is the bundle entry file.
    #[serde(default)]
    pub is_entry: bool,
}

/// `POST /companies/{company_id}/instruction-documents/ui` — creates a
/// document, redirects to the company instruction page.
#[route(POST "/companies/{company_id}/instruction-documents/ui")]
pub async fn create_instruction_document_ui(
    cx: &Cx,
    Form(form): Form<InstructionDocumentForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let name = form.name.trim().to_owned();
    if !name.is_empty()
        && let Ok(document) = state
            .instructions
            .create_document(staple_data::NewInstructionDocument {
                company_id: company_id.clone(),
                name,
                content: form.content,
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &company_id,
            "instruction_document.created",
            "instruction_document",
            &document.id,
            Some(serde_json::json!({ "name": document.name })),
        )
        .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/instructions")))
}

/// `POST /instruction-documents/{id}/delete/ui` — deletes a document,
/// redirects to its company's instruction page.
#[route(POST "/instruction-documents/{id}/delete/ui")]
pub async fn delete_instruction_document_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .instructions
        .get_document(&id)
        .await
        .ok()
        .flatten()
        .map(|document| document.company_id)
    else {
        return Ok(see_other("/"));
    };
    if state
        .instructions
        .delete_document(&id, &company_id)
        .await
        .is_ok()
    {
        let _ = log_activity(
            &state.activity,
            &company_id,
            "instruction_document.deleted",
            "instruction_document",
            &id,
            None,
        )
        .await;
    }
    Ok(see_other(&format!("/companies/{company_id}/instructions")))
}

/// `POST /companies/{company_id}/agents/{agent_id}/instructions/ui` — creates
/// or replaces an agent instruction file, redirects back to the page.
#[route(POST "/companies/{company_id}/agents/{agent_id}/instructions/ui")]
pub async fn upsert_agent_instruction_file_ui(
    cx: &Cx,
    Form(form): Form<AgentInstructionFileForm>,
) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Ok(path) = validate_instruction_path(&form.path)
        && !path.is_empty()
        && let Ok(file) = state
            .instructions
            .upsert_agent_file(staple_data::NewAgentInstructionFile {
                company_id: company_id.clone(),
                agent_id: agent_id.clone(),
                path: path.clone(),
                content: form.content,
                is_entry: form.is_entry,
            })
            .await
    {
        let _ = log_activity(
            &state.activity,
            &company_id,
            "instruction_file.upserted",
            "instruction_file",
            &file.id,
            Some(serde_json::json!({ "agentId": agent_id, "path": path })),
        )
        .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/agents/{agent_id}/instructions"
    )))
}

/// `POST /companies/{company_id}/agents/{agent_id}/instructions/{path}/delete/ui`
/// — deletes an agent instruction file, redirects back to the page.
#[route(POST "/companies/{company_id}/agents/{agent_id}/instructions/{path}/delete/ui")]
pub async fn delete_agent_instruction_file_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let agent_id = path_param::<AgentId>(cx)?.to_string();
    let path = path_param::<Path>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Ok(path) = validate_instruction_path(&path)
        && !path.is_empty()
        && state
            .instructions
            .delete_agent_file(&company_id, &agent_id, &path)
            .await
            .is_ok()
    {
        let _ = log_activity(
            &state.activity,
            &company_id,
            "instruction_file.deleted",
            "instruction_file",
            &path,
            Some(serde_json::json!({ "agentId": agent_id, "path": path })),
        )
        .await;
    }
    Ok(see_other(&format!(
        "/companies/{company_id}/agents/{agent_id}/instructions"
    )))
}
