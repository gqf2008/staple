//! Company secret routes: versioned, encrypted, redacted.

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewSecret, redact};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    dto::{CompanySecretDto, SecretVersionDto},
    error::ApiError,
    routes::CompanyId,
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/secrets`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    /// Secret name (unique per company).
    pub name: String,
    /// Plaintext value (encrypted before storage).
    pub value: String,
}

/// Body for `POST /api/companies/{companyId}/secrets/{name}/rotate`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RotateSecretRequest {
    /// New plaintext value.
    pub value: String,
}

/// Body for `POST /api/companies/{companyId}/secrets/{name}/rollback`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackSecretRequest {
    /// Version to restore.
    pub version: i64,
}

/// Body for `POST /api/companies/{companyId}/redact`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactRequest {
    /// Text to redact.
    pub text: String,
    /// Secret names whose values should be hidden.
    pub names: Vec<String>,
}

/// Typed `{name}` path parameter for secrets.
#[path_param(error = bad_request("Invalid secret name"))]
pub(crate) struct Name(String);

fn validate_create(body: &CreateSecretRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.name.trim().is_empty() {
        issues.push(
            json!({ "path": ["name"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if body.value.is_empty() {
        issues.push(
            json!({ "path": ["value"], "message": "String must contain at least 1 character(s)" }),
        );
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `POST /api/companies/{companyId}/secrets` — creates a secret (version 1).
#[route(POST "/api/companies/{company_id}/secrets")]
pub async fn create_secret(
    cx: &Cx,
    Json(body): Json<CreateSecretRequest>,
) -> Result<(StatusCode, Json<CompanySecretDto>), ApiError> {
    validate_create(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let secret = state
        .secrets
        .create_secret(NewSecret {
            company_id: company_id.clone(),
            name: body.name.trim().to_owned(),
            value: body.value,
        })
        .await
        .map_err(secret_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "secret.created",
        "company_secret",
        &secret.id,
        Some(json!({ "name": secret.name })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(secret.into())))
}

/// `GET /api/companies/{companyId}/secrets` — lists secrets (no values).
#[route(GET "/api/companies/{company_id}/secrets")]
pub async fn list_secrets(cx: &Cx) -> Result<Json<Vec<CompanySecretDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let secrets = state
        .secrets
        .list_secrets(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        secrets.into_iter().map(CompanySecretDto::from).collect(),
    ))
}

/// `GET /api/companies/{companyId}/secrets/{name}` — secret metadata.
#[route(GET "/api/companies/{company_id}/secrets/{name}")]
pub async fn get_secret(cx: &Cx) -> Result<Json<CompanySecretDto>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let name = path_param::<Name>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .secrets
        .get_secret(&company_id, &name)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(secret) => Ok(Json(secret.into())),
        None => Err(ApiError::not_found("Secret not found")),
    }
}

/// `GET /api/companies/{companyId}/secrets/{name}/value` — reads the current
/// plaintext value (board-only surface until #28).
#[route(GET "/api/companies/{company_id}/secrets/{name}/value")]
pub async fn get_secret_value(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let name = path_param::<Name>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .secrets
        .get_secret_value(&company_id, &name)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(value) => Ok(Json(json!({ "name": name, "value": value }))),
        None => Err(ApiError::not_found("Secret not found")),
    }
}

/// `POST /api/companies/{companyId}/secrets/{name}/rotate` — new version.
#[route(POST "/api/companies/{company_id}/secrets/{name}/rotate")]
pub async fn rotate_secret(
    cx: &Cx,
    Json(body): Json<RotateSecretRequest>,
) -> Result<Json<CompanySecretDto>, ApiError> {
    if body.value.is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["value"], "message": "String must contain at least 1 character(s)" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let name = path_param::<Name>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let secret = state
        .secrets
        .rotate_secret(&company_id, &name, body.value)
        .await
        .map_err(secret_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "secret.rotated",
        "company_secret",
        &secret.id,
        Some(json!({ "name": secret.name, "version": secret.latest_version })),
    )
    .await?;
    Ok(Json(secret.into()))
}

/// `POST /api/companies/{companyId}/secrets/{name}/rollback` — restores a
/// previous version as the newest version.
#[route(POST "/api/companies/{company_id}/secrets/{name}/rollback")]
pub async fn rollback_secret(
    cx: &Cx,
    Json(body): Json<RollbackSecretRequest>,
) -> Result<Json<CompanySecretDto>, ApiError> {
    if body.version < 1 {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["version"], "message": "Version must be a positive integer" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let name = path_param::<Name>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let secret = state
        .secrets
        .rollback_secret(&company_id, &name, body.version)
        .await
        .map_err(secret_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "secret.rolled_back",
        "company_secret",
        &secret.id,
        Some(json!({ "name": secret.name, "version": secret.latest_version })),
    )
    .await?;
    Ok(Json(secret.into()))
}

/// `GET /api/companies/{companyId}/secrets/{name}/versions` — version list.
#[route(GET "/api/companies/{company_id}/secrets/{name}/versions")]
pub async fn list_secret_versions(cx: &Cx) -> Result<Json<Vec<SecretVersionDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let name = path_param::<Name>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let versions = state
        .secrets
        .list_versions(&company_id, &name)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        versions.into_iter().map(SecretVersionDto::from).collect(),
    ))
}

/// `DELETE /api/companies/{companyId}/secrets/{name}` — deletes a secret.
#[route(DELETE "/api/companies/{company_id}/secrets/{name}")]
pub async fn delete_secret(cx: &Cx) -> Result<StatusCode, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let name = path_param::<Name>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .secrets
        .delete_secret(&company_id, &name)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(secret) => {
            log_activity(
                &state.activity,
                &company_id,
                "secret.deleted",
                "company_secret",
                &secret.id,
                Some(json!({ "name": secret.name })),
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Secret not found")),
    }
}

/// `POST /api/companies/{companyId}/redact` — redacts secret values from
/// arbitrary text (logs, transcripts, outputs).
#[route(POST "/api/companies/{company_id}/redact")]
pub async fn redact_text(
    cx: &Cx,
    Json(body): Json<RedactRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let mut values = Vec::new();
    for name in &body.names {
        if let Some(value) = state
            .secrets
            .get_secret_value(&company_id, name)
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
        {
            values.push(value);
        }
    }
    Ok(Json(json!({ "redacted": redact(&body.text, &values) })))
}

fn secret_error_to_api(error: staple_data::SecretError) -> ApiError {
    use staple_data::SecretError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::AlreadyExists => ApiError::conflict("Secret already exists"),
        E::SecretNotFound => ApiError::not_found("Secret not found"),
        E::VersionNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["version"], "message": "Secret version not found" }]),
        ),
        E::Cipher(_) => ApiError::internal("secret cipher error"),
        other => ApiError::internal(other.to_string()),
    }
}
