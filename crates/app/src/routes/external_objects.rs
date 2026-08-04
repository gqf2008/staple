//! External object routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewExternalObject, NewExternalObjectCatalog, NewExternalObjectMention};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, query_params, route},
};

use crate::{
    error::ApiError,
    routes::{CompanyId, Id},
    state::AppState,
};

/// Body for `POST /api/issues/{issueId}/external-objects`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExternalObjectRequest {
    /// Kind.
    pub kind: String,
    /// External id.
    pub external_id: String,
    /// URL.
    #[serde(default)]
    pub url: Option<String>,
    /// Metadata.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// `GET /api/issues/{issueId}/external-objects` — lists links.
#[route(GET "/api/issues/{id}/external-objects")]
pub async fn list_external_objects(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let objects = state
        .external_objects
        .list_for_issue(&issue_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&objects).unwrap_or_default()))
}

/// `POST /api/issues/{issueId}/external-objects` — links an external object.
#[route(POST "/api/issues/{id}/external-objects")]
pub async fn create_external_object(
    cx: &Cx,
    Json(body): Json<CreateExternalObjectRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.kind.trim().is_empty() {
        issues.push(
            json!({ "path": ["kind"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.external_id.trim().is_empty() {
        issues.push(json!({ "path": ["externalId"], "message": "String must contain at least 1 character(s)" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let object = state
        .external_objects
        .create(NewExternalObject {
            issue_id,
            kind: body.kind,
            external_id: body.external_id,
            url: body.url,
            metadata: body.metadata.map(|value| value.to_string()),
        })
        .await
        .map_err(external_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&object).unwrap_or_default()),
    ))
}

/// `POST /api/external-objects/{id}/refresh` — refreshes status.
#[route(POST "/api/external-objects/{id}/refresh")]
pub async fn refresh_external_object(
    cx: &Cx,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .external_objects
        .refresh(&id, &body.status)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(object) => Ok(Json(serde_json::to_value(&object).unwrap_or_default())),
        None => Err(ApiError::not_found("External object not found")),
    }
}

/// Body for refresh.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    /// New status.
    pub status: String,
}

fn external_error_to_api(error: staple_data::ExternalObjectError) -> ApiError {
    use staple_data::ExternalObjectError as E;
    match error {
        E::IssueNotFound => ApiError::not_found("Issue not found"),
        E::AlreadyExists => ApiError::conflict("External object link already exists"),
        other => ApiError::internal(other.to_string()),
    }
}

// --- External-object catalog + mentions (upstream alignment) --------------

/// Body for `PUT /api/companies/{companyId}/external-objects/catalog`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCatalogRequest {
    /// Provider key (e.g. `github`).
    pub provider_key: String,
    /// Owning plugin id.
    #[serde(default)]
    pub plugin_id: Option<String>,
    /// Object type (e.g. `pull_request`).
    pub object_type: String,
    /// External id.
    pub external_id: String,
    /// Sanitized canonical URL.
    #[serde(default)]
    pub sanitized_canonical_url: Option<String>,
    /// Canonical identity hash.
    #[serde(default)]
    pub canonical_identity_hash: Option<String>,
    /// Display key.
    #[serde(default)]
    pub display_key: Option<String>,
    /// Icon key.
    #[serde(default)]
    pub icon_key: Option<String>,
    /// Display title.
    #[serde(default)]
    pub display_title: Option<String>,
    /// Status key.
    #[serde(default)]
    pub status_key: Option<String>,
    /// Status label.
    #[serde(default)]
    pub status_label: Option<String>,
    /// Status icon key.
    #[serde(default)]
    pub status_icon_key: Option<String>,
    /// Status category (default `unknown`).
    #[serde(default = "default_status_category")]
    pub status_category: String,
    /// Status tone (default `neutral`).
    #[serde(default = "default_status_tone")]
    pub status_tone: String,
    /// Liveness (default `unknown`).
    #[serde(default = "default_liveness")]
    pub liveness: String,
    /// Terminal flag.
    #[serde(default)]
    pub is_terminal: bool,
    /// Payload JSON.
    #[serde(default = "default_object")]
    pub data: serde_json::Value,
    /// Remote version.
    #[serde(default)]
    pub remote_version: Option<String>,
    /// ETag.
    #[serde(default)]
    pub etag: Option<String>,
    /// Refresh token.
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// Last error code.
    #[serde(default)]
    pub last_error_code: Option<String>,
    /// Last error message.
    #[serde(default)]
    pub last_error_message: Option<String>,
}

fn default_status_category() -> String {
    "unknown".to_owned()
}
fn default_status_tone() -> String {
    "neutral".to_owned()
}
fn default_liveness() -> String {
    "unknown".to_owned()
}
fn default_object() -> serde_json::Value {
    serde_json::json!({})
}

/// `PUT /api/companies/{companyId}/external-objects/catalog` — upserts a
/// catalog entry.
#[route(PUT "/api/companies/{company_id}/external-objects/catalog")]
pub async fn upsert_catalog(
    cx: &Cx,
    Json(body): Json<UpsertCatalogRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.provider_key.trim().is_empty() {
        issues.push(json!({ "path": ["providerKey"], "message": "String must contain at least 1 character(s)" }));
    }
    if body.object_type.trim().is_empty() {
        issues.push(json!({ "path": ["objectType"], "message": "String must contain at least 1 character(s)" }));
    }
    if body.external_id.trim().is_empty() {
        issues.push(json!({ "path": ["externalId"], "message": "String must contain at least 1 character(s)" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .external_object_catalog
        .upsert_catalog(NewExternalObjectCatalog {
            company_id,
            provider_key: body.provider_key,
            plugin_id: body.plugin_id,
            object_type: body.object_type,
            external_id: body.external_id,
            sanitized_canonical_url: body.sanitized_canonical_url,
            canonical_identity_hash: body.canonical_identity_hash,
            display_key: body.display_key,
            icon_key: body.icon_key,
            display_title: body.display_title,
            status_key: body.status_key,
            status_label: body.status_label,
            status_icon_key: body.status_icon_key,
            status_category: body.status_category,
            status_tone: body.status_tone,
            liveness: body.liveness,
            is_terminal: body.is_terminal,
            data: body.data,
            remote_version: body.remote_version,
            etag: body.etag,
            last_resolved_at: None,
            last_changed_at: None,
            last_error_at: None,
            next_refresh_at: None,
            refresh_started_at: None,
            refresh_token: body.refresh_token,
            last_error_code: body.last_error_code,
            last_error_message: body.last_error_message,
        })
        .await
        .map_err(external_error_to_api)?;
    Ok((
        StatusCode::OK,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/external-objects/catalog` — lists catalog
/// entries (`?providerKey=&objectType=`).
#[route(GET "/api/companies/{company_id}/external-objects/catalog")]
pub async fn list_catalog(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let query = query_params::<CatalogQuery>(cx).ok();
    let provider_key = query
        .as_ref()
        .and_then(|q| q.provider_key.clone())
        .unwrap_or_default();
    let object_type = query
        .as_ref()
        .and_then(|q| q.object_type.clone())
        .unwrap_or_default();
    let state = app_context::<AppState>(cx);
    let records = state
        .external_object_catalog
        .list_catalog(&company_id, &provider_key, &object_type)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `GET /api/companies/{companyId}/external-objects/catalog/{id}` — fetches
/// one catalog entry.
#[route(GET "/api/companies/{company_id}/external-objects/catalog/{id}")]
pub async fn get_catalog(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .external_object_catalog
        .get_catalog(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("External object not found")),
    }
}

/// Query for listing catalog entries.
#[query_params]
struct CatalogQuery {
    /// Provider key filter.
    #[serde(rename = "providerKey")]
    provider_key: Option<String>,
    /// Object type filter.
    #[serde(rename = "objectType")]
    object_type: Option<String>,
}

/// Body for `POST /api/companies/{companyId}/external-objects/mentions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMentionRequest {
    /// Source issue id.
    pub source_issue_id: String,
    /// Source kind (e.g. `issue_body`, `comment`).
    pub source_kind: String,
    /// Source record id.
    #[serde(default)]
    pub source_record_id: Option<String>,
    /// Document key.
    #[serde(default)]
    pub document_key: Option<String>,
    /// Property key.
    #[serde(default)]
    pub property_key: Option<String>,
    /// Matched text (redacted).
    #[serde(default)]
    pub matched_text_redacted: Option<String>,
    /// Sanitized display URL.
    #[serde(default)]
    pub sanitized_display_url: Option<String>,
    /// Canonical identity hash.
    #[serde(default)]
    pub canonical_identity_hash: Option<String>,
    /// Canonical identity JSON.
    #[serde(default)]
    pub canonical_identity: Option<serde_json::Value>,
    /// Catalog object id.
    #[serde(default)]
    pub object_id: Option<String>,
    /// Provider key.
    #[serde(default)]
    pub provider_key: Option<String>,
    /// Detector key.
    #[serde(default)]
    pub detector_key: Option<String>,
    /// Object type.
    #[serde(default)]
    pub object_type: Option<String>,
    /// Confidence (default `exact`).
    #[serde(default = "default_confidence")]
    pub confidence: String,
    /// Creating plugin id.
    #[serde(default)]
    pub created_by_plugin_id: Option<String>,
}

fn default_confidence() -> String {
    "exact".to_owned()
}

/// `POST /api/companies/{companyId}/external-objects/mentions` — records an
/// external-object mention.
#[route(POST "/api/companies/{company_id}/external-objects/mentions")]
pub async fn create_mention(
    cx: &Cx,
    Json(body): Json<CreateMentionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut issues = Vec::new();
    if body.source_issue_id.trim().is_empty() {
        issues.push(json!({ "path": ["sourceIssueId"], "message": "String must contain at least 1 character(s)" }));
    }
    if body.source_kind.trim().is_empty() {
        issues.push(json!({ "path": ["sourceKind"], "message": "String must contain at least 1 character(s)" }));
    }
    if !issues.is_empty() {
        return Err(ApiError::unprocessable("Validation error", json!(issues)));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .external_object_catalog
        .create_mention(NewExternalObjectMention {
            company_id,
            source_issue_id: body.source_issue_id,
            source_kind: body.source_kind,
            source_record_id: body.source_record_id,
            document_key: body.document_key,
            property_key: body.property_key,
            matched_text_redacted: body.matched_text_redacted,
            sanitized_display_url: body.sanitized_display_url,
            canonical_identity_hash: body.canonical_identity_hash,
            canonical_identity: body.canonical_identity,
            object_id: body.object_id,
            provider_key: body.provider_key,
            detector_key: body.detector_key,
            object_type: body.object_type,
            confidence: body.confidence,
            created_by_plugin_id: body.created_by_plugin_id,
        })
        .await
        .map_err(external_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `GET /api/companies/{companyId}/external-objects/mentions` — lists
/// mentions (`?sourceIssueId=` or `?objectId=`).
#[route(GET "/api/companies/{company_id}/external-objects/mentions")]
pub async fn list_mentions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let query = query_params::<MentionQuery>(cx).ok();
    let source_issue_id = query.as_ref().and_then(|q| q.source_issue_id.clone());
    let object_id = query.as_ref().and_then(|q| q.object_id.clone());
    let state = app_context::<AppState>(cx);
    let records = if let Some(source_issue_id) = source_issue_id {
        state
            .external_object_catalog
            .list_mentions_for_issue(&company_id, &source_issue_id)
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
    } else if let Some(object_id) = object_id {
        state
            .external_object_catalog
            .list_mentions_for_object(&company_id, &object_id)
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
    } else {
        Vec::new()
    };
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// Query for listing mentions.
#[query_params]
struct MentionQuery {
    /// Source issue id filter.
    #[serde(rename = "sourceIssueId")]
    source_issue_id: Option<String>,
    /// Catalog object id filter.
    #[serde(rename = "objectId")]
    object_id: Option<String>,
}
