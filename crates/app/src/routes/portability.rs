//! Company portability routes: export/import JSON manifests.

use std::io::Write;

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

// --- Zip archive routes ---------------------------------------------------

/// A node in the archive file tree (directories nested, files leaf).
#[derive(Default)]
struct FileNode {
    dirs: std::collections::BTreeMap<String, FileNode>,
    files: Vec<(String, u64)>,
}

impl FileNode {
    fn insert(&mut self, parts: &[&str], size: u64) {
        if parts.is_empty() {
            return;
        }
        if parts.len() == 1 {
            self.files.push((parts[0].to_owned(), size));
        } else {
            self.dirs
                .entry(parts[0].to_owned())
                .or_default()
                .insert(&parts[1..], size);
        }
    }

    fn to_json(&self) -> Vec<serde_json::Value> {
        let mut nodes = Vec::new();
        for (name, child) in &self.dirs {
            nodes.push(serde_json::json!({
                "name": name,
                "type": "dir",
                "children": child.to_json(),
            }));
        }
        for (name, size) in &self.files {
            nodes.push(serde_json::json!({ "name": name, "type": "file", "size": size }));
        }
        nodes
    }
}

fn build_file_tree(entries: &[(String, u64)]) -> Vec<serde_json::Value> {
    let mut root = FileNode::default();
    for (name, size) in entries {
        let parts: Vec<&str> = name.split('/').collect();
        root.insert(&parts, *size);
    }
    root.to_json()
}

const MANIFEST_ENTRY: &str = "manifest.json";
const ATTACHMENTS_PREFIX: &str = "attachments/";

/// `GET /api/companies/{companyId}/export/archive` — downloads a zip archive
/// containing `manifest.json` plus the company's attachments.
#[route(GET "/api/companies/{company_id}/export/archive")]
pub async fn export_company_archive(cx: &Cx) -> Result<topcoat::router::Response, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let manifest = state
        .portability
        .export_company(&company_id)
        .await
        .map_err(portability_error_to_api)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| ApiError::internal(error.to_string()))?;

    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    writer
        .start_file(MANIFEST_ENTRY, options)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    writer
        .write_all(&manifest_bytes)
        .map_err(|error| ApiError::internal(error.to_string()))?;

    let assets = state
        .assets
        .list_for_company(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    for asset in assets {
        let bytes = match state.storage.read(&asset.object_key) {
            Ok(bytes) => bytes,
            Err(_) => continue, // missing attachment files are skipped
        };
        let name = format!("{ATTACHMENTS_PREFIX}{}", asset.object_key);
        writer
            .start_file(name, options)
            .map_err(|error| ApiError::internal(error.to_string()))?;
        writer
            .write_all(&bytes)
            .map_err(|error| ApiError::internal(error.to_string()))?;
    }
    let cursor = writer
        .finish()
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let bytes = cursor.into_inner();
    topcoat::router::Response::builder()
        .header("Content-Type", "application/zip")
        .header(
            "Content-Disposition",
            format!("attachment; filename=company-{company_id}.zip"),
        )
        .body(topcoat::router::Body::from(bytes))
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// `POST /api/companies/{companyId}/import/archive/preview` — parses a zip
/// archive (octet-stream body) and returns the file list + manifest summary.
#[route(POST "/api/companies/{company_id}/import/archive/preview")]
pub async fn preview_company_archive(
    cx: &Cx,
    bytes: bytes::Bytes,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).map_err(|error| {
            ApiError::unprocessable(
                "Validation error",
                json!([{ "path": ["archive"], "message": format!("invalid zip: {error}") }]),
            )
        })?;
    let mut files = Vec::new();
    let mut attachment_entries: Vec<(String, u64)> = Vec::new();
    let mut manifest_value = None;
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|error| ApiError::internal(error.to_string()))?;
        let name = file.name().to_owned();
        if name == MANIFEST_ENTRY {
            let mut reader = file;
            let mut content = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut content)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            manifest_value = serde_json::from_slice::<serde_json::Value>(&content)
                .ok()
                .map(|value| {
                    let tables = value
                        .get("tables")
                        .and_then(serde_json::Value::as_array)
                        .map(|tables| {
                            tables
                                .iter()
                                .map(|table| {
                                    serde_json::json!({
                                        "name": table.get("name").cloned().unwrap_or_default(),
                                        "rows": table
                                            .get("rows")
                                            .and_then(serde_json::Value::as_array)
                                            .map(Vec::len)
                                            .unwrap_or(0),
                                    })
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    serde_json::json!({
                        "version": value.get("version").cloned().unwrap_or_default(),
                        "companyId": value.get("companyId").cloned().unwrap_or_default(),
                        "tables": tables,
                    })
                });
        } else if name.starts_with(ATTACHMENTS_PREFIX) {
            let size = file.size();
            files.push(serde_json::json!({ "name": name, "size": size }));
            attachment_entries.push((name.clone(), size));
        }
    }
    let Some(manifest) = manifest_value else {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["archive"], "message": "missing manifest.json" }]),
        ));
    };
    let existing = state
        .portability
        .company_row_counts(&company_id)
        .await
        .map_err(portability_error_to_api)?;
    Ok(Json(json!({
        "files": files,
        "filesTree": build_file_tree(&attachment_entries),
        "manifest": manifest,
        "existing": existing,
    })))
}

/// `POST /api/companies/{companyId}/import/archive?strategy=` — applies a zip
/// archive import (manifest + attachments).
#[route(POST "/api/companies/{company_id}/import/archive")]
pub async fn import_company_archive(
    cx: &Cx,
    bytes: bytes::Bytes,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let strategy = match topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| parts.uri.query())
        .and_then(|query| {
            query.split('&').find_map(|pair| {
                pair.split_once('=')
                    .filter(|(k, _)| *k == "strategy")
                    .map(|(_, v)| v)
            })
        }) {
        Some("overwrite") => ImportStrategy::Overwrite,
        _ => ImportStrategy::Skip,
    };
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).map_err(|error| {
            ApiError::unprocessable(
                "Validation error",
                json!([{ "path": ["archive"], "message": format!("invalid zip: {error}") }]),
            )
        })?;
    let content = {
        let mut manifest = archive
            .by_name(MANIFEST_ENTRY)
            .map_err(|error| ApiError::unprocessable("Validation error", json!([{ "path": ["archive"], "message": format!("missing manifest.json: {error}") }])))?;
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut manifest, &mut content)
            .map_err(|error| ApiError::internal(error.to_string()))?;
        content
    };
    let manifest: staple_data::CompanyManifest =
        serde_json::from_slice(&content).map_err(|error| {
            ApiError::unprocessable(
                "Validation error",
                json!([{ "path": ["manifest"], "message": error.to_string() }]),
            )
        })?;

    let state = app_context::<AppState>(cx);
    let summary = state
        .portability
        .import_company(&company_id, manifest, strategy)
        .await
        .map_err(portability_error_to_api)?;

    // Restore attachments from the archive.
    let mut restored = 0u64;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| ApiError::internal(error.to_string()))?;
        let name = file.name().to_owned();
        if let Some(key) = name.strip_prefix(ATTACHMENTS_PREFIX) {
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut bytes)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            if state.storage.save(key, &bytes).is_ok() {
                restored += 1;
            }
        }
    }
    Ok(Json(
        json!({ "summary": summary, "attachmentsRestored": restored }),
    ))
}
