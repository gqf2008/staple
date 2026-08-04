//! Secret binding routes: provider configs, bindings, user secret
//! definitions/declarations, and access events.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewSecretAccessEvent, NewSecretBinding, NewSecretProviderConfig, NewUserSecretDeclaration,
    NewUserSecretDefinition,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{error::ApiError, routes::CompanyId, state::AppState};

/// Body for `POST .../secret-provider-configs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProviderConfigRequest {
    pub provider: String,
    pub display_name: String,
    #[serde(default = "default_ready")]
    pub status: String,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default = "default_object")]
    pub config: serde_json::Value,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `PUT .../secret-bindings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBindingRequest {
    pub secret_id: String,
    pub target_type: String,
    pub target_id: String,
    pub config_path: String,
    #[serde(default = "default_latest")]
    pub version_selector: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_unclassified")]
    pub projection_class: String,
    #[serde(default)]
    pub projection_allowlist_key: Option<String>,
}

/// Body for `POST .../user-secret-definitions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserSecretDefinitionRequest {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_active")]
    pub status: String,
    #[serde(default = "default_local_encrypted")]
    pub provider: String,
    #[serde(default = "default_paperclip_managed")]
    pub managed_mode: String,
    #[serde(default)]
    pub provider_config_id: Option<String>,
    #[serde(default)]
    pub provider_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub usage_guidance: Option<String>,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
}

/// Body for `POST .../user-secret-declarations`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserSecretDeclarationRequest {
    pub user_secret_definition_id: String,
    pub target_type: String,
    pub target_id: String,
    pub config_path: String,
    pub env_key: String,
    #[serde(default = "default_latest")]
    pub version_selector: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub allow_missing_override: bool,
    #[serde(default)]
    pub label: Option<String>,
}

/// Body for `POST .../secret-access-events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccessEventRequest {
    #[serde(default)]
    pub secret_id: Option<String>,
    #[serde(default)]
    pub user_secret_definition_id: Option<String>,
    #[serde(default = "default_company")]
    pub secret_scope: String,
    #[serde(default)]
    pub version: Option<i64>,
    pub provider: String,
    #[serde(default)]
    pub responsible_user_id: Option<String>,
    #[serde(default)]
    pub credential_owner_user_id: Option<String>,
    #[serde(default)]
    pub credential_subject_type: Option<String>,
    #[serde(default)]
    pub credential_subject_id: Option<String>,
    pub actor_type: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    pub consumer_type: String,
    pub consumer_id: String,
    #[serde(default)]
    pub config_path: Option<String>,
    #[serde(default)]
    pub issue_id: Option<String>,
    #[serde(default)]
    pub heartbeat_run_id: Option<String>,
    #[serde(default)]
    pub plugin_id: Option<String>,
    pub outcome: String,
    #[serde(default)]
    pub error_code: Option<String>,
}

fn default_object() -> serde_json::Value {
    serde_json::json!({})
}
fn default_ready() -> String {
    "ready".to_owned()
}
fn default_latest() -> String {
    "latest".to_owned()
}
fn default_true() -> bool {
    true
}
fn default_unclassified() -> String {
    "unclassified".to_owned()
}
fn default_active() -> String {
    "active".to_owned()
}
fn default_local_encrypted() -> String {
    "local_encrypted".to_owned()
}
fn default_paperclip_managed() -> String {
    "paperclip_managed".to_owned()
}
fn default_company() -> String {
    "company".to_owned()
}

/// `POST .../secret-provider-configs` — creates a provider config.
#[route(POST "/api/companies/{company_id}/secret-provider-configs")]
pub async fn create_provider_config(
    cx: &Cx,
    Json(body): Json<CreateProviderConfigRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let config = state
        .secret_bindings
        .create_provider_config(NewSecretProviderConfig {
            company_id,
            provider: body.provider,
            display_name: body.display_name,
            status: body.status,
            is_default: body.is_default,
            config: body.config,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(secret_binding_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&config).unwrap_or_default()),
    ))
}

/// `GET .../secret-provider-configs` — lists provider configs.
#[route(GET "/api/companies/{company_id}/secret-provider-configs")]
pub async fn list_provider_configs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let configs = state
        .secret_bindings
        .list_provider_configs(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&configs).unwrap_or_default()))
}

/// `PUT .../secret-bindings` — sets (upserts) a secret binding.
#[route(PUT "/api/companies/{company_id}/secret-bindings")]
pub async fn set_binding(
    cx: &Cx,
    Json(body): Json<SetBindingRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let binding = state
        .secret_bindings
        .set_binding(NewSecretBinding {
            company_id,
            secret_id: body.secret_id,
            target_type: body.target_type,
            target_id: body.target_id,
            config_path: body.config_path,
            version_selector: body.version_selector,
            required: body.required,
            label: body.label,
            projection_class: body.projection_class,
            projection_allowlist_key: body.projection_allowlist_key,
        })
        .await
        .map_err(secret_binding_error_to_api)?;
    Ok(Json(serde_json::to_value(&binding).unwrap_or_default()))
}

/// `GET .../secret-bindings` — lists secret bindings.
#[route(GET "/api/companies/{company_id}/secret-bindings")]
pub async fn list_bindings(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let bindings = state
        .secret_bindings
        .list_bindings(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&bindings).unwrap_or_default()))
}

/// `POST .../user-secret-definitions` — creates a user secret definition.
#[route(POST "/api/companies/{company_id}/user-secret-definitions")]
pub async fn create_user_secret_definition(
    cx: &Cx,
    Json(body): Json<CreateUserSecretDefinitionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let definition = state
        .secret_bindings
        .create_user_secret_definition(NewUserSecretDefinition {
            company_id,
            key: body.key,
            name: body.name,
            description: body.description,
            status: body.status,
            provider: body.provider,
            managed_mode: body.managed_mode,
            provider_config_id: body.provider_config_id,
            provider_metadata: body.provider_metadata,
            usage_guidance: body.usage_guidance,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
        })
        .await
        .map_err(secret_binding_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&definition).unwrap_or_default()),
    ))
}

/// `GET .../user-secret-definitions` — lists user secret definitions.
#[route(GET "/api/companies/{company_id}/user-secret-definitions")]
pub async fn list_user_secret_definitions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let definitions = state
        .secret_bindings
        .list_user_secret_definitions(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&definitions).unwrap_or_default()))
}

/// `POST .../user-secret-declarations` — creates a user secret declaration.
#[route(POST "/api/companies/{company_id}/user-secret-declarations")]
pub async fn create_user_secret_declaration(
    cx: &Cx,
    Json(body): Json<CreateUserSecretDeclarationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let declaration = state
        .secret_bindings
        .create_user_secret_declaration(NewUserSecretDeclaration {
            company_id,
            user_secret_definition_id: body.user_secret_definition_id,
            target_type: body.target_type,
            target_id: body.target_id,
            config_path: body.config_path,
            env_key: body.env_key,
            version_selector: body.version_selector,
            required: body.required,
            allow_missing_override: body.allow_missing_override,
            label: body.label,
        })
        .await
        .map_err(secret_binding_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&declaration).unwrap_or_default()),
    ))
}

/// `GET .../user-secret-declarations` — lists user secret declarations.
#[route(GET "/api/companies/{company_id}/user-secret-declarations")]
pub async fn list_user_secret_declarations(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let declarations = state
        .secret_bindings
        .list_user_secret_declarations(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        serde_json::to_value(&declarations).unwrap_or_default(),
    ))
}

/// `POST .../secret-access-events` — records a secret access event.
#[route(POST "/api/companies/{company_id}/secret-access-events")]
pub async fn create_access_event(
    cx: &Cx,
    Json(body): Json<CreateAccessEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    crate::auth::require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let event = state
        .secret_bindings
        .create_access_event(NewSecretAccessEvent {
            company_id,
            secret_id: body.secret_id,
            user_secret_definition_id: body.user_secret_definition_id,
            secret_scope: body.secret_scope,
            version: body.version,
            provider: body.provider,
            responsible_user_id: body.responsible_user_id,
            credential_owner_user_id: body.credential_owner_user_id,
            credential_subject_type: body.credential_subject_type,
            credential_subject_id: body.credential_subject_id,
            actor_type: body.actor_type,
            actor_id: body.actor_id,
            consumer_type: body.consumer_type,
            consumer_id: body.consumer_id,
            config_path: body.config_path,
            issue_id: body.issue_id,
            heartbeat_run_id: body.heartbeat_run_id,
            plugin_id: body.plugin_id,
            outcome: body.outcome,
            error_code: body.error_code,
        })
        .await
        .map_err(secret_binding_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&event).unwrap_or_default()),
    ))
}

/// `GET .../secret-access-events` — lists secret access events.
#[route(GET "/api/companies/{company_id}/secret-access-events")]
pub async fn list_access_events(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let events = state
        .secret_bindings
        .list_access_events(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(serde_json::to_value(&events).unwrap_or_default()))
}

fn secret_binding_error_to_api(error: staple_data::SecretBindingError) -> ApiError {
    use staple_data::SecretBindingError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::SecretNotFound => ApiError::not_found("Secret not found"),
        E::DefinitionNotFound => ApiError::not_found("User secret definition not found"),
        E::ProviderConfigNotFound => ApiError::not_found("Provider config not found"),
        E::ReferenceNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["references"], "message": "Referenced record not found or out of company" }]),
        ),
        E::AlreadyExists => ApiError::conflict("Record already exists"),
        other => ApiError::internal(other.to_string()),
    }
}
