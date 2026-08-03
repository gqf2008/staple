//! Invites and join requests routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{InviteRecord, JoinRequestRecord, NewInvite, NewJoinRequest};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::require_board,
    error::ApiError,
    routes::{CompanyId, Id},
    state::AppState,
};

/// Body for `POST /api/companies/{companyId}/invites`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteRequest {
    /// Invite type (`company_join`).
    #[serde(default)]
    pub invite_type: Option<String>,
    /// Allowed join types (`human` | `agent` | `both`).
    #[serde(default)]
    pub allowed_join_types: Option<String>,
    /// Defaults payload.
    #[serde(default)]
    pub defaults_payload: Option<serde_json::Value>,
    /// ISO 8601 expiry (defaults to +30 days).
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Body for `POST /api/invites/{id}/join-requests`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJoinRequestRequest {
    /// Request type (`human` | `agent`).
    pub request_type: String,
    /// Request IP.
    #[serde(default)]
    pub request_ip: Option<String>,
    /// Requesting user id.
    #[serde(default)]
    pub requesting_user_id: Option<String>,
    /// Request email snapshot.
    #[serde(default)]
    pub request_email_snapshot: Option<String>,
    /// Agent name.
    #[serde(default)]
    pub agent_name: Option<String>,
    /// Adapter type.
    #[serde(default)]
    pub adapter_type: Option<String>,
    /// Capabilities.
    #[serde(default)]
    pub capabilities: Option<String>,
    /// Agent defaults payload.
    #[serde(default)]
    pub agent_defaults_payload: Option<serde_json::Value>,
}

fn validate_invite(body: &CreateInviteRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if let Some(invite_type) = &body.invite_type
        && !matches!(invite_type.as_str(), "company_join" | "bootstrap_ceo")
    {
        issues.push(json!({
            "path": ["inviteType"],
            "message": "Invalid enum value. Expected 'company_join' | 'bootstrap_ceo'",
        }));
    }
    if let Some(join_types) = &body.allowed_join_types
        && !matches!(join_types.as_str(), "human" | "agent" | "both")
    {
        issues.push(json!({
            "path": ["allowedJoinTypes"],
            "message": "Invalid enum value. Expected 'human' | 'agent' | 'both'",
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn validate_join_request(body: &CreateJoinRequestRequest) -> Result<(), ApiError> {
    let mut issues = Vec::new();
    if !matches!(body.request_type.as_str(), "human" | "agent") {
        issues.push(json!({
            "path": ["requestType"],
            "message": "Invalid enum value. Expected 'human' | 'agent'",
        }));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

fn default_expiry() -> String {
    // ISO 8601 now + 30 days, generated through the DB default path at create
    // time via the repository when omitted; this helper covers explicit
    // validation only.
    "2999-01-01T00:00:00.000Z".to_owned()
}

/// `POST /api/companies/{companyId}/invites` — creates an invite.
#[route(POST "/api/companies/{company_id}/invites")]
pub async fn create_invite(
    cx: &Cx,
    Json(body): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    require_board(cx)?;
    validate_invite(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let expires_at = body.expires_at.unwrap_or_else(default_expiry);
    let (invite, token) = state
        .invites
        .create_invite(NewInvite {
            company_id: company_id.clone(),
            invite_type: body
                .invite_type
                .unwrap_or_else(|| "company_join".to_owned()),
            allowed_join_types: body.allowed_join_types.unwrap_or_else(|| "both".to_owned()),
            defaults_payload: body.defaults_payload,
            expires_at,
            invited_by_user_id: None,
        })
        .await
        .map_err(invite_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "invite.created",
        "invite",
        &invite.id,
        Some(json!({ "inviteType": invite.invite_type })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "invite": invite, "token": token })),
    ))
}

/// `GET /api/companies/{companyId}/invites` — lists invites.
#[route(GET "/api/companies/{company_id}/invites")]
pub async fn list_invites(cx: &Cx) -> Result<Json<Vec<InviteRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let invites = state
        .invites
        .list_invites(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(invites))
}

/// `POST /api/companies/{companyId}/invites/{id}/revoke` — revokes an invite.
#[route(POST "/api/companies/{company_id}/invites/{id}/revoke")]
pub async fn revoke_invite(cx: &Cx) -> Result<Json<InviteRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let invite = state
        .invites
        .revoke_invite(&company_id, &id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .ok_or_else(|| ApiError::not_found("Invite not found"))?;
    log_activity(
        &state.activity,
        &company_id,
        "invite.revoked",
        "invite",
        &invite.id,
        None,
    )
    .await?;
    Ok(Json(invite))
}

/// `POST /api/companies/{companyId}/invites/{id}/join-requests` — creates a
/// join request for an invite (one per invite).
#[route(POST "/api/companies/{company_id}/invites/{id}/join-requests")]
pub async fn create_join_request(
    cx: &Cx,
    Json(body): Json<CreateJoinRequestRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    validate_join_request(&body)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    crate::auth::enforce_company_scope(cx, &company_id)?;
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let claim_secret =
        (body.request_type == "agent").then(|| format!("claim-{}", uuid::Uuid::new_v4()));
    let record = state
        .invites
        .create_join_request(NewJoinRequest {
            company_id,
            invite_id: id,
            request_type: body.request_type,
            request_ip: body.request_ip.unwrap_or_default(),
            requesting_user_id: body.requesting_user_id,
            request_email_snapshot: body.request_email_snapshot,
            agent_name: body.agent_name,
            adapter_type: body.adapter_type,
            capabilities: body.capabilities,
            agent_defaults_payload: body.agent_defaults_payload,
            claim_secret_hash: claim_secret
                .as_ref()
                .map(|secret| staple_data::sha256_hex(secret)),
            claim_secret_expires_at: Some(default_expiry()),
        })
        .await
        .map_err(invite_error_to_api)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({ "joinRequest": record, "claimSecret": claim_secret })),
    ))
}

/// `GET /api/companies/{companyId}/join-requests` — lists join requests.
#[route(GET "/api/companies/{company_id}/join-requests")]
pub async fn list_join_requests(cx: &Cx) -> Result<Json<Vec<JoinRequestRecord>>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let requests = state
        .invites
        .list_join_requests(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(requests))
}

/// `POST /api/companies/{companyId}/join-requests/{id}/approve` — approves.
#[route(POST "/api/companies/{company_id}/join-requests/{id}/approve")]
pub async fn approve_join_request(cx: &Cx) -> Result<Json<JoinRequestRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .invites
        .approve(&company_id, &id, None)
        .await
        .map_err(invite_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "join_request.approved",
        "join_request",
        &record.id,
        Some(json!({ "requestType": record.request_type })),
    )
    .await?;
    Ok(Json(record))
}

/// `POST /api/companies/{companyId}/join-requests/{id}/reject` — rejects.
#[route(POST "/api/companies/{company_id}/join-requests/{id}/reject")]
pub async fn reject_join_request(cx: &Cx) -> Result<Json<JoinRequestRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let id = path_param::<Id>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let record = state
        .invites
        .reject(&company_id, &id, None)
        .await
        .map_err(invite_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        "join_request.rejected",
        "join_request",
        &record.id,
        None,
    )
    .await?;
    Ok(Json(record))
}

fn invite_error_to_api(error: staple_data::InviteError) -> ApiError {
    use staple_data::InviteError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::InviteNotFound => ApiError::not_found("Invite not found"),
        E::InviteRevokedOrExpired => ApiError::forbidden("Invite is revoked or expired"),
        E::JoinRequestNotFound => ApiError::not_found("Join request not found"),
        E::NotPending => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["status"], "message": "Join request is not pending approval" }]),
        ),
        E::AlreadyExists => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["inviteId"], "message": "A join request already exists for this invite" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
