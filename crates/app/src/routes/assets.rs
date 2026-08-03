//! Asset upload and issue attachment routes (local-disk provider).

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use staple_data::{NewAsset, NewIssueAttachment};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        Body, Response, StatusCode, content::Json, content::multipart::Multipart, path_param, route,
    },
};

use crate::{
    audit::log_activity,
    dto::{AssetDto, IssueAttachmentDto},
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// `POST /api/companies/{companyId}/assets` — uploads one file (multipart
/// field `file`), stores it on local disk, and registers the asset.
#[route(POST "/api/companies/{company_id}/assets")]
pub async fn upload_asset(
    cx: &Cx,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<AssetDto>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);

    let mut uploaded: Option<(String, String, Vec<u8>)> = None; // (name, content_type, bytes)
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        if field.name() == Some("file") {
            let name = field
                .file_name()
                .map(str::to_owned)
                .unwrap_or_else(|| "file".to_owned());
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_owned();
            let bytes = field
                .bytes()
                .await
                .map_err(|error| ApiError::internal(error.to_string()))?;
            uploaded = Some((name, content_type, bytes.to_vec()));
            break;
        }
    }
    let Some((filename, content_type, bytes)) = uploaded else {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["file"], "message": "A file field is required" }]),
        ));
    };
    if bytes.is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["file"], "message": "File must not be empty" }]),
        ));
    }

    let digest = hex(Sha256::digest(&bytes));
    let object_key = format!("{company_id}/{}-{}", digest, sanitize_filename(&filename));
    state
        .storage
        .save(&object_key, &bytes)
        .map_err(|error| ApiError::internal(error.to_string()))?;

    let asset = state
        .assets
        .create_asset(NewAsset {
            company_id,
            provider: "local_disk".to_owned(),
            object_key,
            content_type,
            byte_size: bytes.len() as i64,
            sha256: digest,
            original_filename: Some(filename),
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    log_activity(
        &state.activity,
        &asset.company_id,
        "asset.uploaded",
        "asset",
        &asset.id,
        Some(json!({ "objectKey": asset.object_key, "byteSize": asset.byte_size })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(asset.into())))
}

/// `GET /api/assets/{assetId}/content` — streams the stored bytes.
#[route(GET "/api/assets/{asset_id}/content")]
pub async fn get_asset_content(cx: &Cx) -> Result<Response, ApiError> {
    let asset_id = path_param::<AssetId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(asset) = state
        .assets
        .get_asset(&asset_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    else {
        return Err(ApiError::not_found("Asset not found"));
    };
    let bytes = state
        .storage
        .read(&asset.object_key)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", asset.content_type)
        .header("Content-Length", bytes.len().to_string())
        .body(Body::from(bytes))
        .expect("valid response"))
}

/// `POST /api/issues/{issueId}/attachments` — links an asset to an issue.
#[route(POST "/api/issues/{id}/attachments")]
pub async fn attach_asset(
    cx: &Cx,
    Json(body): Json<AttachAssetRequest>,
) -> Result<(StatusCode, Json<IssueAttachmentDto>), ApiError> {
    if !is_uuid(&body.asset_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["assetId"], "message": "Invalid uuid" }]),
        ));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let attachment = state
        .assets
        .create_issue_attachment(NewIssueAttachment {
            issue_id,
            asset_id: body.asset_id,
        })
        .await
        .map_err(asset_error_to_api)?;
    log_activity(
        &state.activity,
        &attachment.company_id,
        "attachment.created",
        "issue_attachment",
        &attachment.id,
        Some(json!({ "issueId": attachment.issue_id, "assetId": attachment.asset_id })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(attachment.into())))
}

/// `GET /api/issues/{issueId}/attachments` — lists an issue's attachments.
#[route(GET "/api/issues/{id}/attachments")]
pub async fn list_attachments(cx: &Cx) -> Result<Json<Vec<IssueAttachmentDto>>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let attachments = state
        .assets
        .list_issue_attachments(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        attachments
            .into_iter()
            .map(IssueAttachmentDto::from)
            .collect(),
    ))
}

/// Body for `POST /api/issues/{issueId}/attachments`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachAssetRequest {
    /// Asset id to link.
    pub asset_id: String,
}

/// Shared `{asset_id}` path parameter.
#[path_param(error = bad_request("Invalid asset id"))]
pub(crate) struct AssetId(String);

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "file".to_owned()
    } else {
        sanitized
    }
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn asset_error_to_api(error: staple_data::AssetError) -> ApiError {
    use staple_data::AssetError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::AssetNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["assetId"], "message": "Asset not found" }]),
        ),
        E::AttachmentExists => ApiError::conflict("Asset already attached to this issue"),
        other => ApiError::internal(other.to_string()),
    }
}
