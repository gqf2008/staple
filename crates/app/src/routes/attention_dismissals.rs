//! Attention inbox dismissal routes (upstream `inbox-dismissals.ts` parity,
//! issue #204 A2).
//!
//! Board-only, company-scoped surface for the dismiss/snooze state consumed
//! by the issue-based attention feed. The acting user is resolved from the
//! `X-Board-User` header (falling back to `"board"`), matching the board
//! actor resolution used by `issue_structure.rs`.

use serde::Deserialize;
use serde_json::json;
use staple_data::{DismissalError, NewDismissal};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    audit::log_activity,
    auth::{current_actor, enforce_company_scope, require_board},
    error::ApiError,
    routes::CompanyId,
    state::AppState,
};

/// `{item_key}` path parameter (attention item key).
#[path_param(error = bad_request("Invalid item key"))]
pub(crate) struct ItemKey(String);

/// Body for `POST /api/companies/{companyId}/inbox-dismissals`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertInboxDismissalRequest {
    /// Attention item key (`{sourceKind}:{dedupKey}`).
    pub item_key: String,
    /// `dismiss` or `snooze`.
    pub kind: String,
    /// ISO 8601 time a snooze becomes visible again.
    #[serde(default)]
    pub snoozed_until: Option<String>,
}

fn dismissal_error_to_api(error: DismissalError) -> ApiError {
    ApiError::internal(error.to_string())
}

/// Validates an upsert body and returns the normalized `snoozedUntil` (UTC
/// RFC 3339) for snoozes.
fn validate_upsert(body: &UpsertInboxDismissalRequest) -> Result<Option<String>, ApiError> {
    let mut issues = Vec::new();
    if body.item_key.trim().is_empty() {
        issues.push(json!({
            "path": ["itemKey"],
            "message": "String must contain at least 1 character(s)"
        }));
    }
    let kind = body.kind.trim();
    if kind != "dismiss" && kind != "snooze" {
        issues.push(json!({
            "path": ["kind"],
            "message": "kind must be one of: dismiss, snooze"
        }));
    }
    let mut normalized_until = None;
    if kind == "snooze" {
        match body
            .snoozed_until
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(raw) => match chrono::DateTime::parse_from_rfc3339(raw) {
                Ok(parsed) => {
                    let parsed = parsed.with_timezone(&chrono::Utc);
                    if parsed <= chrono::Utc::now() {
                        issues.push(json!({
                            "path": ["snoozedUntil"],
                            "message": "snoozedUntil must be in the future"
                        }));
                    } else {
                        normalized_until =
                            Some(parsed.to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
                    }
                }
                Err(_) => issues.push(json!({
                    "path": ["snoozedUntil"],
                    "message": "snoozedUntil must be an ISO timestamp"
                })),
            },
            None => issues.push(json!({
                "path": ["snoozedUntil"],
                "message": "Snooze requires snoozedUntil"
            })),
        }
    } else if kind == "dismiss" && body.snoozed_until.is_some() {
        issues.push(json!({
            "path": ["snoozedUntil"],
            "message": "Dismissals must not include snoozedUntil"
        }));
    }
    if issues.is_empty() {
        Ok(normalized_until)
    } else {
        Err(ApiError::unprocessable("Validation error", json!(issues)))
    }
}

/// `GET /api/companies/{companyId}/inbox-dismissals` — lists the current
/// user's dismissals/snoozes for the company.
#[route(GET "/api/companies/{company_id}/inbox-dismissals")]
pub async fn list_attention_dismissals(
    cx: &Cx,
) -> Result<Json<Vec<staple_data::DismissalRecord>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let user_id = current_actor(cx);
    let records = state
        .attention_dismissals
        .list(&company_id, &user_id)
        .await
        .map_err(dismissal_error_to_api)?;
    Ok(Json(records))
}

/// `POST /api/companies/{companyId}/inbox-dismissals` — dismisses or snoozes
/// an attention item for the current user.
#[route(POST "/api/companies/{company_id}/inbox-dismissals")]
pub async fn upsert_attention_dismissal(
    cx: &Cx,
    Json(body): Json<UpsertInboxDismissalRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    if state
        .companies
        .get(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .is_none()
    {
        return Err(ApiError::not_found("Company not found"));
    }
    let snoozed_until = validate_upsert(&body)?;
    let user_id = current_actor(cx);
    let record = state
        .attention_dismissals
        .upsert(NewDismissal {
            company_id: company_id.clone(),
            user_id: user_id.clone(),
            item_key: body.item_key.trim().to_owned(),
            kind: body.kind.trim().to_owned(),
            snoozed_until,
        })
        .await
        .map_err(dismissal_error_to_api)?;
    log_activity(
        &state.activity,
        &company_id,
        if record.kind == "snooze" {
            "inbox.snoozed"
        } else {
            "inbox.dismissed"
        },
        "company",
        &company_id,
        Some(json!({
            "userId": user_id,
            "itemKey": record.item_key,
            "kind": record.kind,
            "dismissedAt": record.dismissed_at,
            "snoozedUntil": record.snoozed_until,
        })),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&record).unwrap_or_default()),
    ))
}

/// `DELETE /api/companies/{companyId}/inbox-dismissals/{item_key}` — restores
/// (clears) a dismissal/snooze for the current user.
#[route(DELETE "/api/companies/{company_id}/inbox-dismissals/{item_key}")]
pub async fn clear_attention_dismissal(cx: &Cx) -> Result<StatusCode, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let item_key = path_param::<ItemKey>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    require_board(cx)?;
    let state = app_context::<AppState>(cx);
    let user_id = current_actor(cx);
    let removed = state
        .attention_dismissals
        .clear(&company_id, &user_id, &item_key)
        .await
        .map_err(dismissal_error_to_api)?;
    if removed {
        log_activity(
            &state.activity,
            &company_id,
            "inbox.restored",
            "company",
            &company_id,
            Some(json!({ "userId": user_id, "itemKey": item_key })),
        )
        .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
