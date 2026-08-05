//! Company portability routes: export/import JSON manifests.

use serde::Deserialize;
use serde_json::json;
use staple_data::{CompanyManifest, ImportStrategy};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, path_param, route},
};

use crate::{error::ApiError, routes::CompanyId, state::AppState};

/// Body for `POST /api/companies/{companyId}/import`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    /// Manifest to import.
    pub manifest: CompanyManifest,
    /// Import strategy (`skip` | `overwrite`).
    #[serde(default)]
    pub strategy: ImportStrategy,
}

/// `GET /api/companies/{companyId}/export` — exports the company's core
/// tables as a JSON manifest.
#[route(GET "/api/companies/{company_id}/export")]
pub async fn export_company(cx: &Cx) -> Result<Json<CompanyManifest>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let manifest = state
        .portability
        .export_company(&company_id)
        .await
        .map_err(portability_error_to_api)?;
    Ok(Json(manifest))
}

/// `POST /api/companies/{companyId}/import` — imports a manifest into the
/// company, minting fresh ids and rewriting references.
#[route(POST "/api/companies/{company_id}/import")]
pub async fn import_company(
    cx: &Cx,
    Json(body): Json<ImportRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let summary = state
        .portability
        .import_company(&company_id, body.manifest, body.strategy)
        .await
        .map_err(portability_error_to_api)?;
    Ok(Json(serde_json::to_value(&summary).unwrap_or_default()))
}

fn portability_error_to_api(error: staple_data::PortabilityError) -> ApiError {
    use staple_data::PortabilityError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::InvalidManifest(message) => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["manifest"], "message": message }]),
        ),
        E::CompanyNotEmpty => {
            ApiError::conflict("Target company already has data; use the overwrite strategy")
        }
        other => ApiError::internal(other.to_string()),
    }
}
