//! Company memberships and instance user roles routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{
    CompanyAccessRow, CompanyMembershipRecord, InstanceUserRoleRecord, NewCompanyMembership,
    NewInstanceUserRole, UserAccessSummary,
};
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

/// Body for `POST /api/companies/{companyId}/memberships`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMembershipRequest {
    /// Principal type (`agent` | `user`).
    pub principal_type: String,
    /// Principal id.
    pub principal_id: String,
    /// Membership role.
    #[serde(default)]
    pub membership_role: Option<String>,
}

/// Body for `PATCH /api/memberships/{id}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMembershipRequest {
    /// New status (`active` | `inactive` | `pending` | `removed`).
    #[serde(default)]
    pub status: Option<String>,
    /// New role (`null` clears).
    #[serde(default)]
    pub membership_role: Option<Option<String>>,
}

/// Body for `POST /api/instance/user-roles`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceRoleRequest {
    /// User id.
    pub user_id: String,
    /// Role (`instance_admin`).
    #[serde(default)]
    pub role: Option<String>,
}

fn validate_membership(body: &CreateMembershipRequest) -> Result<(), ApiError> {
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
    if let Some(role) = &body.membership_role
        && !matches!(role.as_str(), "owner" | "admin" | "operator" | "viewer")
    {
        issues.push(json!({
            "path": ["membershipRole"],
            "message": "Invalid enum value. Expected 'owner' | 'admin' | 'operator' | 'viewer'",
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/memberships` — lists memberships.
#[route(GET "/api/companies/{company_id}/memberships")]
pub async fn list_memberships(cx: &Cx) -> Result<Json<Vec<CompanyMembershipRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let rows = state
        .memberships
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

/// `POST /api/companies/{companyId}/memberships` — creates a membership.
#[route(POST "/api/companies/{company_id}/memberships")]
pub async fn create_membership(
    cx: &Cx,
    Json(body): Json<CreateMembershipRequest>,
) -> Result<(StatusCode, Json<CompanyMembershipRecord>), ApiError> {
    require_board(cx)?;
    validate_membership(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .memberships
        .upsert(NewCompanyMembership {
            company_id,
            principal_type: body.principal_type,
            principal_id: body.principal_id,
            membership_role: body.membership_role,
        })
        .await
        .map_err(membership_error_to_api)?;
    log_activity(
        &state.activity,
        &record.company_id,
        "membership.upserted",
        "company_membership",
        &record.id,
        Some(json!({
            "principalType": record.principal_type,
            "principalId": record.principal_id,
            "role": record.membership_role,
        })),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `PATCH /api/memberships/{id}` — updates status/role.
#[route(PATCH "/api/memberships/{id}")]
pub async fn update_membership(
    cx: &Cx,
    Json(body): Json<UpdateMembershipRequest>,
) -> Result<Json<CompanyMembershipRecord>, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let existing = state
        .memberships
        .company_of(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Membership not found"))?;
    let record = state
        .memberships
        .update(&existing, &id, body.status, body.membership_role)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Membership not found"))?;
    log_activity(
        &state.activity,
        &record.company_id,
        "membership.updated",
        "company_membership",
        &record.id,
        Some(json!({ "status": record.status, "role": record.membership_role })),
    )
    .await?;
    Ok(Json(record))
}

/// `DELETE /api/companies/{companyId}/memberships/{id}` — deletes a membership.
#[route(DELETE "/api/companies/{company_id}/memberships/{id}")]
pub async fn delete_membership(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .memberships
        .delete(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(record) => {
            log_activity(
                &state.activity,
                &company_id,
                "membership.deleted",
                "company_membership",
                &record.id,
                None,
            )
            .await?;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("Membership not found")),
    }
}

/// `GET /api/instance/user-roles` — lists instance roles.
#[route(GET "/api/instance/user-roles")]
pub async fn list_instance_roles(cx: &Cx) -> Result<Json<Vec<InstanceUserRoleRecord>>, ApiError> {
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let roles = state
        .memberships
        .list_roles()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(roles))
}

/// `POST /api/instance/user-roles` — grants an instance role.
#[route(POST "/api/instance/user-roles")]
pub async fn create_instance_role(
    cx: &Cx,
    Json(body): Json<CreateInstanceRoleRequest>,
) -> Result<(StatusCode, Json<InstanceUserRoleRecord>), ApiError> {
    require_board(cx)?;
    let role = body.role.unwrap_or_else(|| "instance_admin".to_owned());
    if role != "instance_admin" {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["role"], "message": "Invalid enum value. Expected 'instance_admin'" }]),
        ));
    }
    let state = app_context::<AppState>(cx);
    let record = state
        .memberships
        .upsert_role(NewInstanceUserRole {
            user_id: body.user_id,
            role,
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// `DELETE /api/instance/user-roles/{id}` — removes an instance role.
#[route(DELETE "/api/instance/user-roles/{id}")]
pub async fn delete_instance_role(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .memberships
        .delete_role(&id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Instance role not found")),
    }
}

fn membership_error_to_api(error: staple_data::MembershipError) -> ApiError {
    use staple_data::MembershipError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::PrincipalNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["principalId"], "message": "Principal not found" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}

/// `{user_id}` path parameter for instance user access routes.
#[path_param(error = bad_request("Invalid user id"))]
pub(crate) struct UserId(String);

/// Body for `PUT /api/instance/users/{userId}/company-access`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUserCompanyAccessRequest {
    /// Company ids the user may access.
    pub company_ids: Vec<String>,
}

/// `GET /api/instance/users` — lists instance users with access summaries.
#[route(GET "/api/instance/users")]
pub async fn list_instance_users(cx: &Cx) -> Result<Json<Vec<UserAccessSummary>>, ApiError> {
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let search = topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| {
            parts.uri.query().and_then(|query| {
                query.split('&').find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    (key == "search").then_some(value.to_owned())
                })
            })
        })
        .unwrap_or_default();
    let mut users = state
        .memberships
        .list_users()
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    if !search.is_empty() {
        users.retain(|user| user.user_id.starts_with(&search));
    }
    Ok(Json(users))
}

/// `GET /api/instance/users/{userId}/company-access` — one user's access.
#[route(GET "/api/instance/users/{user_id}/company-access")]
pub async fn user_company_access(cx: &Cx) -> Result<Json<Vec<CompanyAccessRow>>, ApiError> {
    require_board(cx)?;
    let user_id = path_param::<UserId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let access = state
        .memberships
        .user_company_access(&user_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(access))
}

/// `PUT /api/instance/users/{userId}/company-access` — sets a user's companies.
#[route(PUT "/api/instance/users/{user_id}/company-access")]
pub async fn set_user_company_access(
    cx: &Cx,
    Json(body): Json<SetUserCompanyAccessRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_board(cx)?;
    let user_id = path_param::<UserId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let active = state
        .memberships
        .set_user_company_access(&user_id, &body.company_ids)
        .await
        .map_err(|error| match error {
            staple_data::MembershipError::CompanyNotFound => {
                ApiError::not_found("Company not found")
            }
            other => ApiError::internal(other.to_string()),
        })?;
    // Instance-level user access management has no single owning company, so
    // it is intentionally not written to the company-scoped activity log.
    Ok(Json(
        json!({ "userId": user_id, "activeCompanyCount": active }),
    ))
}
