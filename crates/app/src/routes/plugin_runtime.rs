//! Plugin runtime routes: scoped state, entities, jobs/runs, logs, webhook
//! deliveries, database namespaces, and migration ledger.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewPluginEntity, NewPluginJob, NewPluginJobRun, NewPluginLog, NewPluginMigration,
    NewPluginNamespace, NewPluginWebhook,
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{auth::require_board, error::ApiError, routes::Id, state::AppState};

/// Query for runtime lists (company filter).
#[topcoat::router::query_params]
struct CompanyQuery {
    /// Optional company filter.
    #[serde(rename = "companyId")]
    company_id: Option<String>,
}

/// Body for `PUT /api/plugins/{id}/state`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetStateRequest {
    /// Scope kind.
    pub scope_kind: String,
    /// Scope id.
    #[serde(default)]
    pub scope_id: Option<String>,
    /// Namespace.
    #[serde(default)]
    pub namespace: Option<String>,
    /// State key.
    pub key: String,
    /// JSON value.
    pub value: serde_json::Value,
}

/// Body for `POST /api/plugins/{id}/entities`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertEntityRequest {
    /// Company id.
    #[serde(default)]
    pub company_id: Option<String>,
    /// Entity type.
    pub entity_type: String,
    /// Scope kind.
    pub scope_kind: String,
    /// Scope id.
    #[serde(default)]
    pub scope_id: Option<String>,
    /// External id.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Title.
    #[serde(default)]
    pub title: Option<String>,
    /// Status.
    #[serde(default)]
    pub status: Option<String>,
    /// Data JSON.
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Body for `POST /api/plugins/{id}/jobs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertJobRequest {
    /// Job key.
    pub job_key: String,
    /// Schedule.
    pub schedule: String,
}

/// Body for `PATCH /api/plugins/{id}/jobs/{job_key}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJobRequest {
    /// New schedule.
    #[serde(default)]
    pub schedule: Option<String>,
    /// New status.
    #[serde(default)]
    pub status: Option<String>,
}

/// Body for `POST /api/plugins/{id}/jobs/{job_key}/runs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobRunRequest {
    /// Company id.
    #[serde(default)]
    pub company_id: Option<String>,
    /// Trigger.
    #[serde(default)]
    pub trigger: Option<String>,
}

/// Body for `POST /api/plugins/{id}/jobs/{job_key}/runs/{run_id}/complete`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteJobRunRequest {
    /// Status (`succeeded` | `failed` | `cancelled`).
    pub status: String,
    /// Error.
    #[serde(default)]
    pub error: Option<String>,
    /// Log lines.
    #[serde(default)]
    pub logs: Option<Vec<String>>,
}

/// Body for `POST /api/plugins/{id}/logs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendLogRequest {
    /// Company id.
    #[serde(default)]
    pub company_id: Option<String>,
    /// Level.
    #[serde(default)]
    pub level: Option<String>,
    /// Message.
    pub message: String,
    /// Meta JSON.
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
}

/// Body for `POST /api/plugins/{id}/webhooks`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    /// Company id.
    #[serde(default)]
    pub company_id: Option<String>,
    /// Webhook key.
    pub webhook_key: String,
    /// External dedup id.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Payload JSON.
    pub payload: serde_json::Value,
    /// Headers JSON.
    #[serde(default)]
    pub headers: Option<serde_json::Value>,
}

/// Body for `POST /api/plugins/{id}/webhooks/{webhook_id}/complete`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteWebhookRequest {
    /// Status (`succeeded` | `failed`).
    pub status: String,
    /// Error.
    #[serde(default)]
    pub error: Option<String>,
    /// Duration ms.
    #[serde(default)]
    pub duration_ms: Option<i64>,
}

/// Body for `POST /api/plugins/{id}/database/namespaces`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertNamespaceRequest {
    /// Namespace name.
    pub namespace_name: String,
    /// Namespace mode.
    #[serde(default)]
    pub namespace_mode: Option<String>,
}

/// Body for `POST /api/plugins/{id}/database/migrations`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordMigrationRequest {
    /// Namespace name.
    pub namespace_name: String,
    /// Migration key.
    pub migration_key: String,
    /// Checksum.
    pub checksum: String,
    /// Plugin version.
    pub plugin_version: String,
    /// Status (`applied` | `failed` | `pending`).
    pub status: String,
    /// Error message.
    #[serde(default)]
    pub error_message: Option<String>,
}

/// `PUT /api/plugins/{id}/state` — sets scoped plugin state.
#[route(PUT "/api/plugins/{id}/state")]
pub async fn set_state(
    cx: &Cx,
    Json(body): Json<SetStateRequest>,
) -> Result<Json<staple_data::PluginStateRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .state_set(
            &id,
            &body.scope_kind,
            body.scope_id.as_deref(),
            body.namespace.as_deref().unwrap_or("default"),
            &body.key,
            body.value,
        )
        .await
        .map_err(runtime_error_to_api)?;
    Ok(Json(record))
}

/// `GET /api/plugins/{id}/state?scopeKind=...&scopeId=...&namespace=...&key=...`
/// — reads plugin state.
#[route(GET "/api/plugins/{id}/state")]
pub async fn get_state(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let query = topcoat::router::query_params::<StateQuery>(cx).map_err(|_| {
        ApiError::bad_request("scopeKind, namespace and key query parameters are required")
    })?;
    let state = app_context::<AppState>(cx);
    match state
        .plugin_runtime
        .state_get(
            &id,
            &query.scope_kind,
            query.scope_id.as_deref(),
            &query.namespace,
            &query.key,
        )
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Ok(Json(json!({ "value": null }))),
    }
}

/// `DELETE /api/plugins/{id}/state?scopeKind=...&scopeId=...&namespace=...&key=...`.
#[route(DELETE "/api/plugins/{id}/state")]
pub async fn delete_state(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let query = topcoat::router::query_params::<StateQuery>(cx).map_err(|_| {
        ApiError::bad_request("scopeKind, namespace and key query parameters are required")
    })?;
    let state = app_context::<AppState>(cx);
    match state
        .plugin_runtime
        .state_delete(
            &id,
            &query.scope_kind,
            query.scope_id.as_deref(),
            &query.namespace,
            &query.key,
        )
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("State entry not found")),
    }
}

/// Query for state operations.
#[topcoat::router::query_params]
struct StateQuery {
    /// Scope kind.
    #[serde(rename = "scopeKind")]
    scope_kind: String,
    /// Scope id.
    #[serde(rename = "scopeId")]
    scope_id: Option<String>,
    /// Namespace.
    namespace: String,
    /// State key.
    key: String,
}

/// `GET /api/plugins/{id}/state/list?scopeKind=...&scopeId=...&namespace=...`.
#[route(GET "/api/plugins/{id}/state/list")]
pub async fn list_state(cx: &Cx) -> Result<Json<Vec<staple_data::PluginStateRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let query = topcoat::router::query_params::<StateListQuery>(cx).map_err(|_| {
        ApiError::bad_request("scopeKind and namespace query parameters are required")
    })?;
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .state_list(
            &id,
            &query.scope_kind,
            query.scope_id.as_deref(),
            &query.namespace,
        )
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// Query for state listing.
#[topcoat::router::query_params]
struct StateListQuery {
    /// Scope kind.
    #[serde(rename = "scopeKind")]
    scope_kind: String,
    /// Scope id.
    #[serde(rename = "scopeId")]
    scope_id: Option<String>,
    /// Namespace.
    namespace: String,
}

/// `GET /api/plugins/{id}/entities` — lists entities.
#[route(GET "/api/plugins/{id}/entities")]
pub async fn list_entities(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PluginEntityRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let company_id = topcoat::router::query_params::<CompanyQuery>(cx)
        .ok()
        .and_then(|query| query.company_id.clone());
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .entity_list(&id, company_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `POST /api/plugins/{id}/entities` — upserts an entity.
#[route(POST "/api/plugins/{id}/entities")]
pub async fn upsert_entity(
    cx: &Cx,
    Json(body): Json<UpsertEntityRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginEntityRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .entity_upsert(NewPluginEntity {
            plugin_id: id,
            company_id: body.company_id,
            entity_type: body.entity_type,
            scope_kind: body.scope_kind,
            scope_id: body.scope_id,
            external_id: body.external_id,
            title: body.title,
            status: body.status,
            data: body.data.unwrap_or_else(|| json!({})),
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `DELETE /api/plugins/{id}/entities/{entity_id}` — deletes an entity.
#[route(DELETE "/api/plugins/{id}/entities/{entity_id}")]
pub async fn delete_entity(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let entity_id = path_param::<EntityId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .plugin_runtime
        .entity_delete(&id, &entity_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Entity not found")),
    }
}

/// `{entity_id}` path parameter.
#[path_param(error = bad_request("Invalid entity id"))]
pub(crate) struct EntityId(String);

/// `GET /api/plugins/{id}/jobs` — lists jobs.
#[route(GET "/api/plugins/{id}/jobs")]
pub async fn list_jobs(cx: &Cx) -> Result<Json<Vec<staple_data::PluginJobRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .job_list(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `POST /api/plugins/{id}/jobs` — upserts a job.
#[route(POST "/api/plugins/{id}/jobs")]
pub async fn upsert_job(
    cx: &Cx,
    Json(body): Json<UpsertJobRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginJobRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .job_upsert(NewPluginJob {
            plugin_id: id,
            job_key: body.job_key,
            schedule: body.schedule,
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `PATCH /api/plugins/{id}/jobs/{job_key}` — updates a job.
#[route(PATCH "/api/plugins/{id}/jobs/{job_key}")]
pub async fn update_job(
    cx: &Cx,
    Json(body): Json<UpdateJobRequest>,
) -> Result<Json<staple_data::PluginJobRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let job_key = path_param::<JobKey>(cx)?.to_string();
    if let Some(status) = &body.status
        && !matches!(status.as_str(), "active" | "paused" | "error")
    {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["status"], "message": "Invalid enum value. Expected 'active' | 'paused' | 'error'" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .job_update(&id, &job_key, body.schedule, body.status)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Job not found"))?;
    Ok(Json(record))
}

/// `{job_key}` path parameter.
#[path_param(error = bad_request("Invalid job key"))]
pub(crate) struct JobKey(String);

/// `POST /api/plugins/{id}/jobs/{job_key}/runs` — creates a job run.
#[route(POST "/api/plugins/{id}/jobs/{job_key}/runs")]
pub async fn create_job_run(
    cx: &Cx,
    Json(body): Json<CreateJobRunRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginJobRunRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let job_key = path_param::<JobKey>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let job = state
        .plugin_runtime
        .job_list(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .into_iter()
        .find(|job| job.job_key == job_key)
        .ok_or_else(|| ApiError::not_found("Job not found"))?;
    let record = state
        .plugin_runtime
        .job_run_create(NewPluginJobRun {
            job_id: job.id,
            plugin_id: id,
            company_id: body.company_id,
            trigger: body.trigger.unwrap_or_else(|| "manual".to_owned()),
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `POST /api/plugins/{id}/jobs/{job_key}/runs/{run_id}/complete`.
#[route(POST "/api/plugins/{id}/jobs/{job_key}/runs/{run_id}/complete")]
pub async fn complete_job_run(
    cx: &Cx,
    Json(body): Json<CompleteJobRunRequest>,
) -> Result<Json<staple_data::PluginJobRunRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let run_id = path_param::<RunId>(cx)?.to_string();
    if !matches!(body.status.as_str(), "succeeded" | "failed" | "cancelled") {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["status"], "message": "Invalid enum value. Expected 'succeeded' | 'failed' | 'cancelled'" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .job_run_complete(
            &id,
            &run_id,
            &body.status,
            body.error,
            body.logs.unwrap_or_default(),
        )
        .await
        .map_err(runtime_error_to_api)?
        .ok_or_else(|| ApiError::not_found("Job run not found"))?;
    Ok(Json(record))
}

/// `{run_id}` path parameter.
#[path_param(error = bad_request("Invalid run id"))]
pub(crate) struct RunId(String);

/// `GET /api/plugins/{id}/job-runs` — lists job runs.
#[route(GET "/api/plugins/{id}/job-runs")]
pub async fn list_job_runs(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PluginJobRunRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let company_id = topcoat::router::query_params::<CompanyQuery>(cx)
        .ok()
        .and_then(|query| query.company_id.clone());
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .job_run_list(&id, company_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `GET /api/plugins/{id}/logs` — lists logs.
#[route(GET "/api/plugins/{id}/logs")]
pub async fn list_logs(cx: &Cx) -> Result<Json<Vec<staple_data::PluginLogRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let company_id = topcoat::router::query_params::<CompanyQuery>(cx)
        .ok()
        .and_then(|query| query.company_id.clone());
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .log_list(&id, company_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `POST /api/plugins/{id}/logs` — appends a log.
#[route(POST "/api/plugins/{id}/logs")]
pub async fn append_log(
    cx: &Cx,
    Json(body): Json<AppendLogRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginLogRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .log_append(NewPluginLog {
            plugin_id: id,
            company_id: body.company_id,
            level: body.level.unwrap_or_else(|| "info".to_owned()),
            message: body.message,
            meta: body.meta,
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/plugins/{id}/webhooks` — lists webhook deliveries.
#[route(GET "/api/plugins/{id}/webhooks")]
pub async fn list_webhooks(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PluginWebhookDeliveryRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let company_id = topcoat::router::query_params::<CompanyQuery>(cx)
        .ok()
        .and_then(|query| query.company_id.clone());
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .webhook_list(&id, company_id.as_deref())
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `POST /api/plugins/{id}/webhooks` — records an inbound delivery.
#[route(POST "/api/plugins/{id}/webhooks")]
pub async fn create_webhook(
    cx: &Cx,
    Json(body): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginWebhookDeliveryRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .webhook_create(NewPluginWebhook {
            plugin_id: id,
            company_id: body.company_id,
            webhook_key: body.webhook_key,
            external_id: body.external_id,
            payload: body.payload,
            headers: body.headers.unwrap_or_else(|| json!({})),
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `POST /api/plugins/{id}/webhooks/{webhook_id}/complete`.
#[route(POST "/api/plugins/{id}/webhooks/{webhook_id}/complete")]
pub async fn complete_webhook(
    cx: &Cx,
    Json(body): Json<CompleteWebhookRequest>,
) -> Result<Json<staple_data::PluginWebhookDeliveryRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let webhook_id = path_param::<WebhookId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .plugin_runtime
        .webhook_complete(&id, &webhook_id, &body.status, body.error, body.duration_ms)
        .await
        .map_err(runtime_error_to_api)?
        .ok_or_else(|| ApiError::not_found("Webhook delivery not found"))?;
    Ok(Json(record))
}

/// `{webhook_id}` path parameter.
#[path_param(error = bad_request("Invalid webhook id"))]
pub(crate) struct WebhookId(String);

/// `POST /api/plugins/{id}/database/namespaces` — upserts a namespace.
#[route(POST "/api/plugins/{id}/database/namespaces")]
pub async fn upsert_namespace(
    cx: &Cx,
    Json(body): Json<UpsertNamespaceRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginDatabaseNamespaceRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let plugin = state
        .plugins
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Plugin not found"))?;
    let record = state
        .plugin_runtime
        .namespace_upsert(NewPluginNamespace {
            plugin_id: id,
            plugin_key: plugin.plugin_key,
            namespace_name: body.namespace_name,
            namespace_mode: body.namespace_mode.unwrap_or_else(|| "schema".to_owned()),
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/plugins/{id}/database/namespaces` — lists namespaces.
#[route(GET "/api/plugins/{id}/database/namespaces")]
pub async fn list_namespaces(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PluginDatabaseNamespaceRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .namespace_list(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

/// `POST /api/plugins/{id}/database/migrations` — records a migration.
#[route(POST "/api/plugins/{id}/database/migrations")]
pub async fn record_migration(
    cx: &Cx,
    Json(body): Json<RecordMigrationRequest>,
) -> Result<(StatusCode, Json<staple_data::PluginMigrationRecord>), ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let plugin = state
        .plugins
        .get(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Plugin not found"))?;
    let record = state
        .plugin_runtime
        .migration_record(NewPluginMigration {
            plugin_id: id,
            plugin_key: plugin.plugin_key,
            namespace_name: body.namespace_name,
            migration_key: body.migration_key,
            checksum: body.checksum,
            plugin_version: body.plugin_version,
            status: body.status,
            error_message: body.error_message,
        })
        .await
        .map_err(runtime_error_to_api)?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `GET /api/plugins/{id}/database/migrations` — lists migrations.
#[route(GET "/api/plugins/{id}/database/migrations")]
pub async fn list_migrations(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::PluginMigrationRecord>>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .plugin_runtime
        .migration_list(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(records))
}

fn runtime_error_to_api(error: staple_data::PluginRuntimeError) -> ApiError {
    use staple_data::PluginRuntimeError as E;
    match error {
        E::PluginNotFound => ApiError::not_found("Plugin not found"),
        E::JobNotFound => ApiError::not_found("Job not found"),
        E::RunTerminal => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["status"], "message": "Job run is already terminal" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
