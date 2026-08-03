//! Plugin registry, per-company config, company settings, and managed
//! resources routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewManagedResource, NewPlugin, PluginCompanySettingRecord, PluginConfigRecord, PluginRecord,
    UpsertCompanySettings, UpsertPluginConfig,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity, auth::require_board, error::ApiError, routes::Id, state::AppState,
};

/// Body for `POST /api/plugins`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterPluginRequest {
    /// Unique plugin key.
    pub plugin_key: String,
    /// Package name.
    pub package_name: String,
    /// Version.
    pub version: String,
    /// Plugin API version.
    #[serde(default)]
    pub api_version: Option<i64>,
    /// Categories.
    #[serde(default)]
    pub categories: Option<Vec<String>>,
    /// Manifest JSON.
    pub manifest: serde_json::Value,
    /// Install order.
    #[serde(default)]
    pub install_order: Option<i64>,
    /// Package path.
    #[serde(default)]
    pub package_path: Option<String>,
}

/// Body for `PATCH /api/plugins/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePluginRequest {
    /// New status.
    #[serde(default)]
    pub status: Option<String>,
    /// Last error (`null` clears).
    #[serde(default)]
    pub last_error: Option<Option<String>>,
}

/// Body for `POST /api/plugins/{id}/configs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertConfigRequest {
    /// Company id.
    pub company_id: String,
    /// Config JSON.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// Body for `PUT /api/plugins/{id}/company-settings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCompanySettingsRequest {
    /// Company id.
    pub company_id: String,
    /// Enabled.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Settings JSON.
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
}

/// Body for `POST /api/plugins/{id}/managed-resources`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManagedResourceRequest {
    /// Company id.
    pub company_id: String,
    /// Resource kind.
    pub resource_kind: String,
    /// Resource key.
    pub resource_key: String,
    /// Resource id.
    pub resource_id: String,
    /// Defaults JSON.
    #[serde(default)]
    pub defaults: Option<serde_json::Value>,
}

fn validate_register(body: &RegisterPluginRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if body.plugin_key.trim().is_empty() {
        issues.push(json!({
            "path": ["pluginKey"],
            "message": "String must contain at least 1 character(s)",
        }));
    }
    if body.package_name.trim().is_empty() {
        issues.push(json!({
            "path": ["packageName"],
            "message": "String must contain at least 1 character(s)",
        }));
    }
    if body.version.trim().is_empty() {
        issues.push(json!({
            "path": ["version"],
            "message": "String must contain at least 1 character(s)",
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `POST /api/plugins` — registers (or re-registers) a plugin.
#[route(POST "/api/plugins")]
pub async fn register_plugin(
    cx: &Cx,
    Json(body): Json<RegisterPluginRequest>,
) -> Result<(StatusCode, Json<PluginRecord>), ApiError> {
    require_board(cx)?;
    validate_register(&body)?;
    let state = app_context::<AppState>(cx);
    let plugin = state
        .plugins
        .register(NewPlugin {
            plugin_key: body.plugin_key,
            package_name: body.package_name,
            version: body.version,
            api_version: body.api_version.unwrap_or(1),
            categories: body.categories.unwrap_or_default(),
            manifest_json: body.manifest,
            install_order: body.install_order,
            package_path: body.package_path,
        })
        .await
        .map_err(plugin_error_to_api)?;
    Ok((StatusCode::CREATED, Json(plugin)))
}

/// `GET /api/plugins` — lists plugins.
#[route(GET "/api/plugins")]
pub async fn list_plugins(cx: &Cx) -> Result<Json<Vec<PluginRecord>>, ApiError> {
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let plugins = state
        .plugins
        .list()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(plugins))
}

/// `GET /api/plugins/{id}` — fetches a plugin.
#[route(GET "/api/plugins/{id}")]
pub async fn get_plugin(cx: &Cx) -> Result<Json<PluginRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .plugins
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Plugin not found"))
}

/// `PATCH /api/plugins/{id}` — updates status/error.
#[route(PATCH "/api/plugins/{id}")]
pub async fn update_plugin(
    cx: &Cx,
    Json(body): Json<UpdatePluginRequest>,
) -> Result<Json<PluginRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    if let Some(status) = &body.status
        && !matches!(
            status.as_str(),
            "installed" | "enabled" | "disabled" | "error" | "uninstalled"
        )
    {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{
                "path": ["status"],
                "message": "Invalid enum value. Expected 'installed' | 'enabled' | 'disabled' | 'error' | 'uninstalled'",
            }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let plugin = state
        .plugins
        .update_status(
            &id,
            body.status.as_deref().unwrap_or("enabled"),
            body.last_error,
        )
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Plugin not found"))?;
    Ok(Json(plugin))
}

/// `DELETE /api/plugins/{id}` — uninstalls a plugin.
#[route(DELETE "/api/plugins/{id}")]
pub async fn delete_plugin(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .plugins
        .delete(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Plugin not found")),
    }
}

/// `POST /api/plugins/{id}/configs` — upserts per-company config.
#[route(POST "/api/plugins/{id}/configs")]
pub async fn upsert_config(
    cx: &Cx,
    Json(body): Json<UpsertConfigRequest>,
) -> Result<(StatusCode, Json<PluginConfigRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let config = state
        .plugins
        .upsert_config(UpsertPluginConfig {
            plugin_id: id,
            company_id: body.company_id,
            config_json: body.config.unwrap_or_else(|| json!({})),
        })
        .await
        .map_err(plugin_error_to_api)?;
    log_activity(
        &state.activity,
        &config.company_id,
        "plugin.config_upserted",
        "plugin_config",
        &config.id,
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(config)))
}

/// `GET /api/plugins/{id}/configs?companyId=...` — lists configs (or one).
#[route(GET "/api/plugins/{id}/configs")]
pub async fn list_configs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = topcoat::router::query_params::<ConfigQuery>(cx)
        .ok()
        .map(|query| query.company_id.clone());
    match company_id {
        Some(company_id) => {
            let config = state
                .plugins
                .get_config(&id, &company_id)
                .await
                .map_err(|error| ApiError::internal(error.to_string()))?;
            Ok(Json(serde_json::to_value(config).unwrap_or_default()))
        }
        None => {
            let configs = state
                .plugins
                .list_configs(&id)
                .await
                .map_err(|error| ApiError::internal(error.to_string()))?;
            Ok(Json(serde_json::to_value(configs).unwrap_or_default()))
        }
    }
}

/// Query for configs listing.
#[topcoat::router::query_params]
struct ConfigQuery {
    /// Optional company filter.
    #[serde(rename = "companyId")]
    company_id: String,
}

/// `PUT /api/plugins/{id}/company-settings` — upserts company settings.
#[route(PUT "/api/plugins/{id}/company-settings")]
pub async fn upsert_company_settings(
    cx: &Cx,
    Json(body): Json<UpsertCompanySettingsRequest>,
) -> Result<Json<PluginCompanySettingRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let setting = state
        .plugins
        .upsert_company_settings(UpsertCompanySettings {
            company_id: body.company_id,
            plugin_id: id,
            enabled: body.enabled.unwrap_or(true),
            settings_json: body.settings.unwrap_or_else(|| json!({})),
        })
        .await
        .map_err(plugin_error_to_api)?;
    Ok(Json(setting))
}

/// `GET /api/plugins/{id}/company-settings` — lists company settings.
#[route(GET "/api/plugins/{id}/company-settings")]
pub async fn list_company_settings(
    cx: &Cx,
) -> Result<Json<Vec<PluginCompanySettingRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let settings = state
        .plugins
        .list_company_settings(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(settings))
}

/// `POST /api/plugins/{id}/managed-resources` — upserts a managed resource.
#[route(POST "/api/plugins/{id}/managed-resources")]
pub async fn upsert_managed_resource(
    cx: &Cx,
    Json(body): Json<CreateManagedResourceRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginManagedResourceRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let plugin = state
        .plugins
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Plugin not found"))?;
    let resource = state
        .plugins
        .upsert_managed_resource(NewManagedResource {
            company_id: body.company_id,
            plugin_id: id,
            plugin_key: plugin.plugin_key,
            resource_kind: body.resource_kind,
            resource_key: body.resource_key,
            resource_id: body.resource_id,
            defaults_json: body.defaults.unwrap_or_else(|| json!({})),
        })
        .await
        .map_err(plugin_error_to_api)?;
    Ok((StatusCode::CREATED, Json(resource)))
}

/// `GET /api/plugins/{id}/managed-resources?companyId=...` — lists resources.
#[route(GET "/api/plugins/{id}/managed-resources")]
pub async fn list_managed_resources(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PluginManagedResourceRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let company_id = topcoat::router::query_params::<ConfigQuery>(cx)
        .ok()
        .map(|query| query.company_id.clone())
        .ok_or_else(|| ApiError::bad_request("companyId query parameter is required"))?;
    let resources = state
        .plugins
        .list_managed_resources(&id, &company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(resources))
}

/// `DELETE /api/plugins/{id}/managed-resources/{resource_id}?companyId=...`
/// — deletes a managed resource (company-scoped).
#[route(DELETE "/api/plugins/{id}/managed-resources/{resource_id}")]
pub async fn delete_managed_resource(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let resource_id = path_param::<ResourceId>(cx)?.to_string();
    let company_id = topcoat::router::query_params::<ConfigQuery>(cx)
        .ok()
        .map(|query| query.company_id.clone())
        .ok_or_else(|| ApiError::bad_request("companyId query parameter is required"))?;
    let state = app_context::<AppState>(cx);
    match state
        .plugins
        .delete_managed_resource(&company_id, &resource_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Managed resource not found")),
    }
}

/// `{resource_id}` path parameter.
#[path_param(error = bad_request("Invalid resource id"))]
pub(crate) struct ResourceId(String);

fn plugin_error_to_api(error: staple_data::PluginError) -> ApiError {
    use staple_data::PluginError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::PluginNotFound => ApiError::not_found("Plugin not found"),
        E::AlreadyExists => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["pluginKey"], "message": "Plugin already exists" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
