//! Principal permission grant routes (upstream §9.8 / access.ts).

use serde::Deserialize;
use serde_json::json;
use staple_data::{NewPermissionGrant, PermissionGrantRecord};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::require_board,
    error::ApiError,
    routes::{CompanyId, Id, is_uuid},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/permission-grants`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePermissionGrantRequest {
    /// Principal type (`agent` or `user`).
    pub principal_type: String,
    /// Principal id.
    pub principal_id: String,
    /// Permission key (e.g. `tasks:assign_scope`, `inbox:manage`).
    pub permission_key: String,
    /// JSON scope object (project/agent/user/subtree constraints).
    #[serde(default)]
    pub scope: Option<serde_json::Value>,
}

fn validate(body: &CreatePermissionGrantRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if !matches!(body.principal_type.as_str(), "agent" | "user") {
        issues.push(json!({
            "path": ["principalType"],
            "message": "Invalid enum value. Expected 'agent' | 'user'",
        }));
    }
    if !is_uuid(&body.principal_id) {
        issues.push(json!({ "path": ["principalId"], "message": "Invalid uuid" }));
    }
    if !staple_domain::is_permission_key(&body.permission_key) {
        issues.push(json!({
            "path": ["permissionKey"],
            "message": "Unknown permission key",
        }));
    }
    if let Some(scope) = &body.scope
        && !scope.is_object()
        && !scope.is_null()
    {
        issues.push(json!({ "path": ["scope"], "message": "Scope must be an object" }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/permission-grants` — lists grants.
#[route(GET "/api/companies/{company_id}/permission-grants")]
pub async fn list_grants(cx: &Cx) -> Result<Json<Vec<PermissionGrantRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let grants = state
        .permission_grants
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(grants))
}

/// `POST /api/companies/{companyId}/permission-grants` — creates or replaces
/// a grant (upsert on company + principal + permission key).
#[route(POST "/api/companies/{company_id}/permission-grants")]
pub async fn create_grant(
    cx: &Cx,
    Json(body): Json<CreatePermissionGrantRequest>,
) -> Result<(StatusCode, Json<PermissionGrantRecord>), ApiError> {
    require_board(cx)?;
    validate(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let grant = state
        .permission_grants
        .upsert(NewPermissionGrant {
            company_id,
            principal_type: body.principal_type,
            principal_id: body.principal_id,
            permission_key: body.permission_key,
            scope: body.scope,
            granted_by_user_id: None,
        })
        .await
        .map_err(grant_error_to_api)?;
    log_activity(
        &state.activity,
        &grant.company_id,
        "permission_grant.upserted",
        "principal_permission_grant",
        &grant.id,
        Some(json!({
            "principalType": grant.principal_type,
            "principalId": grant.principal_id,
            "permissionKey": grant.permission_key,
        })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(grant)))
}

/// `DELETE /api/companies/{companyId}/permission-grants/{id}` — deletes a grant.
#[route(DELETE "/api/companies/{company_id}/permission-grants/{id}")]
pub async fn delete_grant(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .permission_grants
        .delete(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(grant) => {
            log_activity(
                &state.activity,
                &company_id,
                "permission_grant.deleted",
                "principal_permission_grant",
                &grant.id,
                Some(json!({
                    "principalType": grant.principal_type,
                    "principalId": grant.principal_id,
                    "permissionKey": grant.permission_key,
                })),
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Permission grant not found")),
    }
}

fn grant_error_to_api(error: staple_data::PermissionGrantError) -> ApiError {
    use staple_data::PermissionGrantError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::PrincipalNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["principalId"], "message": "Principal not found" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
