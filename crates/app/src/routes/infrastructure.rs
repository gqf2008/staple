//! Infrastructure routes: instance settings, users, folders, watchdogs,
//! plan decompositions, reference mentions, heartbeat events, approval
//! comments, inbox dismissals, built-in resources, agent config revisions,
//! and user preferences.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    NewAgentConfigRevision, NewApprovalComment, NewBuiltInResource, NewFolder,
    NewHeartbeatRunEvent, NewInboxAgentPolicy, NewInboxDismissal, NewIssuePlanDecomposition,
    NewIssueReferenceMention, NewIssueWatchdog, NewUser,
};
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

fn infrastructure_error_to_api(error: staple_data::InfrastructureError) -> ApiError {
    use staple_data::InfrastructureError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::ReferenceNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "Referenced record not found" }]),
        ),
        E::AlreadyExists => ApiError::conflict("Record already exists"),
        E::NotFound => ApiError::not_found("Record not found"),
        other => ApiError::internal(other.to_string()),
    }
}

// --- Instance settings ----------------------------------------------------

/// `GET /api/instance/settings`.
#[route(GET "/api/instance/settings")]
pub async fn get_instance_settings(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let state = app_context::<AppState>(cx);
    let settings = state
        .infrastructure
        .get_instance_settings()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&settings).unwrap_or_default()))
}

/// Body for `PUT /api/instance/settings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceSettingsRequest {
    #[serde(default)]
    pub default_environment_id: Option<Option<String>>,
    #[serde(default)]
    pub general: Option<serde_json::Value>,
    #[serde(default)]
    pub experimental: Option<serde_json::Value>,
}

/// `PUT /api/instance/settings`.
#[route(PUT "/api/instance/settings")]
pub async fn update_instance_settings(
    cx: &Cx,
    Json(body): Json<UpdateInstanceSettingsRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let state = app_context::<AppState>(cx);
    let settings = state
        .infrastructure
        .update_instance_settings(body.default_environment_id, body.general, body.experimental)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&settings).unwrap_or_default()))
}

// --- Users & sessions -----------------------------------------------------

/// Body for `POST /api/users`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub image: Option<String>,
}

/// `GET /api/users`.
#[route(GET "/api/users")]
pub async fn list_users(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let state = app_context::<AppState>(cx);
    let users = state
        .infrastructure
        .list_users()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&users).unwrap_or_default()))
}

/// `POST /api/users`.
#[route(POST "/api/users")]
pub async fn create_user(
    cx: &Cx,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if body.id.trim().is_empty() || body.email.trim().is_empty() {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["id"], "message": "id and email are required" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let user = state
        .infrastructure
        .create_user(NewUser {
            id: body.id,
            name: body.name,
            email: body.email,
            email_verified: body.email_verified,
            image: body.image,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&user).unwrap_or_default()),
    ))
}

/// `GET /api/users/{id}/sessions`.
#[route(GET "/api/users/{id}/sessions")]
pub async fn list_user_sessions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let sessions = state
        .infrastructure
        .list_sessions(&user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&sessions).unwrap_or_default()))
}

// --- Folders --------------------------------------------------------------

/// Body for `POST /api/companies/{companyId}/folders`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    pub kind: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub system_key: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub position: i64,
}

/// `GET /api/companies/{companyId}/folders?kind=`.
#[route(GET "/api/companies/{company_id}/folders")]
pub async fn list_folders(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let kind = query_params::<FoldersQuery>(cx)
        .ok()
        .and_then(|q| q.kind.clone());
    let state = app_context::<AppState>(cx);
    let folders = state
        .infrastructure
        .list_folders(&company_id, kind.as_deref())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&folders).unwrap_or_default()))
}

#[query_params]
struct FoldersQuery {
    #[serde(rename = "kind")]
    kind: Option<String>,
}

/// `POST /api/companies/{companyId}/folders`.
#[route(POST "/api/companies/{company_id}/folders")]
pub async fn create_folder(
    cx: &Cx,
    Json(body): Json<CreateFolderRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let folder = state
        .infrastructure
        .create_folder(NewFolder {
            company_id,
            kind: body.kind,
            parent_id: body.parent_id,
            name: body.name,
            slug: body.slug,
            system_key: body.system_key,
            color: body.color,
            position: body.position,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&folder).unwrap_or_default()),
    ))
}

/// `DELETE /api/companies/{companyId}/folders/{id}`.
#[route(DELETE "/api/companies/{company_id}/folders/{id}")]
pub async fn delete_folder(cx: &Cx) -> Result<StatusCode, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let removed = state
        .infrastructure
        .delete_folder(&company_id, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !removed {
        return Err(ApiError::not_found("Folder not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Issue watchdogs ------------------------------------------------------

/// Body for `POST /api/issues/{id}/watchdogs`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWatchdogRequest {
    pub watchdog_agent_id: String,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default = "default_active")]
    pub status: String,
    #[serde(default)]
    pub watchdog_issue_id: Option<String>,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
    #[serde(default)]
    pub created_by_run_id: Option<String>,
}

fn default_active() -> String {
    "active".to_owned()
}

/// `GET /api/issues/{id}/watchdogs`.
#[route(GET "/api/issues/{id}/watchdogs")]
pub async fn list_watchdogs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state
        .issues
        .get(&issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Issue not found"));
    };
    crate::auth::enforce_company_scope(cx, &issue.company_id)?;
    let company_id = issue.company_id;
    let watchdogs = state
        .infrastructure
        .list_watchdogs(&company_id, &issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&watchdogs).unwrap_or_default()))
}

/// `POST /api/issues/{id}/watchdogs`.
#[route(POST "/api/issues/{id}/watchdogs")]
pub async fn create_watchdog(
    cx: &Cx,
    Json(body): Json<CreateWatchdogRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state
        .issues
        .get(&issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Issue not found"));
    };
    crate::auth::enforce_company_scope(cx, &issue.company_id)?;
    let company_id = issue.company_id;
    let watchdog = state
        .infrastructure
        .create_watchdog(NewIssueWatchdog {
            company_id,
            issue_id,
            watchdog_agent_id: body.watchdog_agent_id,
            instructions: body.instructions,
            status: body.status,
            watchdog_issue_id: body.watchdog_issue_id,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
            created_by_run_id: body.created_by_run_id,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&watchdog).unwrap_or_default()),
    ))
}

// --- Issue plan decompositions --------------------------------------------

/// Body for `POST /api/issues/{id}/plan-decompositions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanDecompositionRequest {
    pub accepted_plan_revision_id: String,
    #[serde(default)]
    pub accepted_interaction_id: Option<String>,
    #[serde(default = "default_in_flight")]
    pub status: String,
    pub request_fingerprint: String,
    #[serde(default)]
    pub requested_child_count: i64,
    #[serde(default = "default_array")]
    pub requested_children: serde_json::Value,
    #[serde(default = "default_array")]
    pub child_issue_ids: serde_json::Value,
    #[serde(default)]
    pub owner_agent_id: Option<String>,
    #[serde(default)]
    pub owner_user_id: Option<String>,
    #[serde(default)]
    pub owner_run_id: Option<String>,
}

fn default_in_flight() -> String {
    "in_flight".to_owned()
}
fn default_array() -> serde_json::Value {
    serde_json::json!([])
}

/// `GET /api/issues/{id}/plan-decompositions`.
#[route(GET "/api/issues/{id}/plan-decompositions")]
pub async fn list_plan_decompositions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state
        .issues
        .get(&issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Issue not found"));
    };
    crate::auth::enforce_company_scope(cx, &issue.company_id)?;
    let company_id = issue.company_id;
    let records = state
        .infrastructure
        .list_plan_decompositions(&company_id, &issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/issues/{id}/plan-decompositions`.
#[route(POST "/api/issues/{id}/plan-decompositions")]
pub async fn create_plan_decomposition(
    cx: &Cx,
    Json(body): Json<CreatePlanDecompositionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state
        .issues
        .get(&issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Issue not found"));
    };
    crate::auth::enforce_company_scope(cx, &issue.company_id)?;
    let company_id = issue.company_id;
    let record = state
        .infrastructure
        .create_plan_decomposition(NewIssuePlanDecomposition {
            company_id,
            source_issue_id: issue_id,
            accepted_plan_revision_id: body.accepted_plan_revision_id,
            accepted_interaction_id: body.accepted_interaction_id,
            status: body.status,
            request_fingerprint: body.request_fingerprint,
            requested_child_count: body.requested_child_count,
            requested_children: body.requested_children,
            child_issue_ids: body.child_issue_ids,
            owner_agent_id: body.owner_agent_id,
            owner_user_id: body.owner_user_id,
            owner_run_id: body.owner_run_id,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Issue reference mentions ---------------------------------------------

/// Body for `POST /api/issues/{id}/reference-mentions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReferenceMentionRequest {
    pub target_issue_id: String,
    pub source_kind: String,
    #[serde(default)]
    pub source_record_id: Option<String>,
    #[serde(default)]
    pub document_key: Option<String>,
    #[serde(default)]
    pub matched_text: Option<String>,
}

/// `GET /api/issues/{id}/reference-mentions`.
#[route(GET "/api/issues/{id}/reference-mentions")]
pub async fn list_reference_mentions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state
        .issues
        .get(&issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Issue not found"));
    };
    crate::auth::enforce_company_scope(cx, &issue.company_id)?;
    let company_id = issue.company_id;
    let records = state
        .infrastructure
        .list_reference_mentions(&company_id, &issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/issues/{id}/reference-mentions`.
#[route(POST "/api/issues/{id}/reference-mentions")]
pub async fn create_reference_mention(
    cx: &Cx,
    Json(body): Json<CreateReferenceMentionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let issue_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(issue) = state
        .issues
        .get(&issue_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Issue not found"));
    };
    crate::auth::enforce_company_scope(cx, &issue.company_id)?;
    let company_id = issue.company_id;
    let record = state
        .infrastructure
        .create_reference_mention(NewIssueReferenceMention {
            company_id,
            source_issue_id: issue_id,
            target_issue_id: body.target_issue_id,
            source_kind: body.source_kind,
            source_record_id: body.source_record_id,
            document_key: body.document_key,
            matched_text: body.matched_text,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Heartbeat run events -------------------------------------------------

/// Body for `POST /api/heartbeat-runs/{id}/events`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendHeartbeatEventRequest {
    pub agent_id: String,
    pub seq: i64,
    pub event_type: String,
    #[serde(default)]
    pub stream: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// `GET /api/heartbeat-runs/{id}/events`.
#[route(GET "/api/heartbeat-runs/{id}/events")]
pub async fn list_heartbeat_events(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .infrastructure
        .heartbeat_run_company(&run_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Heartbeat run not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let records = state
        .infrastructure
        .list_heartbeat_events(&company_id, &run_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/heartbeat-runs/{id}/events`.
#[route(POST "/api/heartbeat-runs/{id}/events")]
pub async fn append_heartbeat_event(
    cx: &Cx,
    Json(body): Json<AppendHeartbeatEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let run_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let Some(company_id) = state
        .infrastructure
        .heartbeat_run_company(&run_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    else {
        return Err(ApiError::not_found("Heartbeat run not found"));
    };
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let record = state
        .infrastructure
        .append_heartbeat_event(NewHeartbeatRunEvent {
            company_id,
            run_id,
            agent_id: body.agent_id,
            seq: body.seq,
            event_type: body.event_type,
            stream: body.stream,
            level: body.level,
            color: body.color,
            message: body.message,
            payload: body.payload,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Approval comments ----------------------------------------------------

/// Body for `POST /api/companies/{companyId}/approvals/{id}/comments`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalCommentRequest {
    #[serde(default)]
    pub author_agent_id: Option<String>,
    #[serde(default)]
    pub author_user_id: Option<String>,
    pub body: String,
}

/// `GET /api/companies/{companyId}/approvals/{id}/comments`.
#[route(GET "/api/companies/{company_id}/approvals/{id}/comments")]
pub async fn list_approval_comments(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let approval_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let records = state
        .infrastructure
        .list_approval_comments(&company_id, &approval_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `POST /api/companies/{companyId}/approvals/{id}/comments`.
#[route(POST "/api/companies/{company_id}/approvals/{id}/comments")]
pub async fn create_approval_comment(
    cx: &Cx,
    Json(body): Json<CreateApprovalCommentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let approval_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .infrastructure
        .create_approval_comment(NewApprovalComment {
            company_id,
            approval_id,
            author_agent_id: body.author_agent_id,
            author_user_id: body.author_user_id,
            body: body.body,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Inbox dismissals -----------------------------------------------------

/// Body for `POST /api/companies/{companyId}/inbox/dismissals`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetInboxDismissalRequest {
    pub user_id: String,
    pub item_key: String,
    #[serde(default = "default_dismiss")]
    pub kind: String,
    #[serde(default)]
    pub snoozed_until: Option<String>,
}

fn default_dismiss() -> String {
    "dismiss".to_owned()
}

/// `GET /api/companies/{companyId}/inbox/dismissals?userId=`.
#[route(GET "/api/companies/{company_id}/inbox/dismissals")]
pub async fn list_inbox_dismissals(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let user_id = query_params::<DismissalsQuery>(cx)
        .ok()
        .and_then(|q| q.user_id.clone())
        .unwrap_or_default();
    let state = app_context::<AppState>(cx);
    let records = state
        .infrastructure
        .list_inbox_dismissals(&company_id, &user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

#[query_params]
struct DismissalsQuery {
    #[serde(rename = "userId")]
    user_id: Option<String>,
}

/// `POST /api/companies/{companyId}/inbox/dismissals`.
#[route(POST "/api/companies/{company_id}/inbox/dismissals")]
pub async fn set_inbox_dismissal(
    cx: &Cx,
    Json(body): Json<SetInboxDismissalRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .infrastructure
        .set_inbox_dismissal(NewInboxDismissal {
            company_id,
            user_id: body.user_id,
            item_key: body.item_key,
            kind: body.kind,
            snoozed_until: body.snoozed_until,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `DELETE /api/companies/{companyId}/inbox/dismissals/{item_key}`.
#[route(DELETE "/api/companies/{company_id}/inbox/dismissals/{item_key}")]
pub async fn remove_inbox_dismissal(cx: &Cx) -> Result<StatusCode, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let item_key = path_param::<Id>(cx)?.to_string();
    let user_id = query_params::<DismissalsQuery>(cx)
        .ok()
        .and_then(|q| q.user_id.clone())
        .unwrap_or_default();
    let state = app_context::<AppState>(cx);
    let removed = state
        .infrastructure
        .remove_inbox_dismissal(&company_id, &user_id, &item_key)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !removed {
        return Err(ApiError::not_found("Dismissal not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Built-in resources ---------------------------------------------------

/// Body for `PUT /api/companies/{companyId}/built-in-resources`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertBuiltInResourceRequest {
    pub bundle_key: String,
    pub resource_kind: String,
    pub resource_key: String,
    pub resource_id: String,
    pub stock_version: String,
    pub stock_hash: String,
    #[serde(default = "default_object")]
    pub defaults_json: serde_json::Value,
}

fn default_object() -> serde_json::Value {
    serde_json::json!({})
}

/// `GET /api/companies/{companyId}/built-in-resources`.
#[route(GET "/api/companies/{company_id}/built-in-resources")]
pub async fn list_built_in_resources(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let records = state
        .infrastructure
        .list_built_in_resources(&company_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

/// `PUT /api/companies/{companyId}/built-in-resources`.
#[route(PUT "/api/companies/{company_id}/built-in-resources")]
pub async fn upsert_built_in_resource(
    cx: &Cx,
    Json(body): Json<UpsertBuiltInResourceRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .infrastructure
        .upsert_built_in_resource(NewBuiltInResource {
            company_id,
            bundle_key: body.bundle_key,
            resource_kind: body.resource_kind,
            resource_key: body.resource_key,
            resource_id: body.resource_id,
            stock_version: body.stock_version,
            stock_hash: body.stock_hash,
            defaults_json: body.defaults_json,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok(Json(serde_json::to_value(&record).unwrap_or_default()))
}

// --- Agent config revisions -----------------------------------------------

/// Body for `POST /api/companies/{companyId}/agent-config-revisions`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentConfigRevisionRequest {
    pub agent_id: String,
    #[serde(default)]
    pub created_by_agent_id: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
    #[serde(default = "default_patch")]
    pub source: String,
    #[serde(default)]
    pub rolled_back_from_revision_id: Option<String>,
    #[serde(default = "default_array")]
    pub changed_keys: serde_json::Value,
    pub before_config: serde_json::Value,
    pub after_config: serde_json::Value,
}

fn default_patch() -> String {
    "patch".to_owned()
}

/// `GET /api/companies/{companyId}/agent-config-revisions?agentId=`.
#[route(GET "/api/companies/{company_id}/agent-config-revisions")]
pub async fn list_agent_config_revisions(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let agent_id = query_params::<AgentRevisionsQuery>(cx)
        .ok()
        .and_then(|q| q.agent_id.clone())
        .unwrap_or_default();
    let state = app_context::<AppState>(cx);
    let records = state
        .infrastructure
        .list_agent_config_revisions(&company_id, &agent_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(serde_json::to_value(&records).unwrap_or_default()))
}

#[query_params]
struct AgentRevisionsQuery {
    #[serde(rename = "agentId")]
    agent_id: Option<String>,
}

/// `POST /api/companies/{companyId}/agent-config-revisions`.
#[route(POST "/api/companies/{company_id}/agent-config-revisions")]
pub async fn create_agent_config_revision(
    cx: &Cx,
    Json(body): Json<CreateAgentConfigRevisionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .infrastructure
        .create_agent_config_revision(NewAgentConfigRevision {
            company_id,
            agent_id: body.agent_id,
            created_by_agent_id: body.created_by_agent_id,
            created_by_user_id: body.created_by_user_id,
            source: body.source,
            rolled_back_from_revision_id: body.rolled_back_from_revision_id,
            changed_keys: body.changed_keys,
            before_config: body.before_config,
            after_config: body.after_config,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

// --- Inbox agent policies -------------------------------------------------

/// Body for `PUT /api/companies/{companyId}/inbox-agent-policies`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetInboxAgentPolicyRequest {
    pub user_id: String,
    #[serde(default = "default_open")]
    pub mode: String,
    #[serde(default = "default_array")]
    pub allowed_agent_ids: serde_json::Value,
}

fn default_open() -> String {
    "open".to_owned()
}

/// `GET /api/companies/{companyId}/inbox-agent-policies/{user_id}`.
#[route(GET "/api/companies/{company_id}/inbox-agent-policies/{user_id}")]
pub async fn get_inbox_agent_policy(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let user_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .infrastructure
        .get_inbox_agent_policy(&company_id, &user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Policy not found")),
    }
}

/// `PUT /api/companies/{companyId}/inbox-agent-policies`.
#[route(PUT "/api/companies/{company_id}/inbox-agent-policies")]
pub async fn set_inbox_agent_policy(
    cx: &Cx,
    Json(body): Json<SetInboxAgentPolicyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let record = state
        .infrastructure
        .set_inbox_agent_policy(NewInboxAgentPolicy {
            company_id,
            user_id: body.user_id,
            mode: body.mode,
            allowed_agent_ids: body.allowed_agent_ids,
        })
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok(Json(serde_json::to_value(&record).unwrap_or_default()))
}

// --- User sidebar preferences ---------------------------------------------

/// Body for `PUT /api/users/{id}/sidebar-preferences`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSidebarPreferenceRequest {
    #[serde(default = "default_array")]
    pub company_order: serde_json::Value,
}

/// `GET /api/users/{id}/sidebar-preferences`.
#[route(GET "/api/users/{id}/sidebar-preferences")]
pub async fn get_user_sidebar_preference(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .infrastructure
        .get_user_sidebar_preference(&user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    {
        Some(record) => Ok(Json(serde_json::to_value(&record).unwrap_or_default())),
        None => Err(ApiError::not_found("Preference not found")),
    }
}

/// `PUT /api/users/{id}/sidebar-preferences`.
#[route(PUT "/api/users/{id}/sidebar-preferences")]
pub async fn set_user_sidebar_preference(
    cx: &Cx,
    Json(body): Json<SetSidebarPreferenceRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .infrastructure
        .set_user_sidebar_preference(&user_id, body.company_order)
        .await
        .map_err(infrastructure_error_to_api)?;
    Ok(Json(serde_json::to_value(&record).unwrap_or_default()))
}
