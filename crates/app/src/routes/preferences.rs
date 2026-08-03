//! Sidebar preferences and company logo routes.

use serde::Deserialize;
use serde_json::json;
use staple_data::{CompanyLogoRecord, SidebarPreferenceRecord};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{StatusCode, content::Json, path_param, route},
};

use crate::{
    auth::require_board,
    error::ApiError,
    routes::{CompanyId, is_uuid},
    state::AppState,
};

/// Body for `PUT /api/companies/{companyId}/sidebar-preferences`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSidebarPrefsRequest {
    /// User id.
    pub user_id: String,
    /// Project order (array of project ids).
    #[serde(default)]
    pub project_order: Vec<String>,
}

/// Body for `PUT /api/companies/{companyId}/logo`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLogoRequest {
    /// Asset id.
    pub asset_id: String,
}

/// Query for `GET /api/companies/{companyId}/sidebar-preferences`.
#[topcoat::router::query_params]
struct SidebarPrefsQuery {
    /// Target user id.
    #[serde(rename = "userId")]
    user_id: String,
}

/// `GET /api/companies/{companyId}/sidebar-preferences?userId=...` — reads
/// sidebar preferences for a user.
#[route(GET "/api/companies/{company_id}/sidebar-preferences")]
pub async fn get_sidebar_prefs(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let query = topcoat::router::query_params::<SidebarPrefsQuery>(cx)
        .map_err(|_| ApiError::bad_request("userId query parameter is required"))?;
    let user_id = query.user_id.clone();
    let state = app_context::<AppState>(cx);
    match state
        .preferences
        .sidebar_prefs(&company_id, &user_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(prefs) => Ok(Json(serde_json::to_value(&prefs).unwrap_or_default())),
        None => Ok(Json(json!({ "projectOrder": [] }))),
    }
}

/// `PUT /api/companies/{companyId}/sidebar-preferences` — upserts prefs.
#[route(PUT "/api/companies/{company_id}/sidebar-preferences")]
pub async fn upsert_sidebar_prefs(
    cx: &Cx,
    Json(body): Json<UpsertSidebarPrefsRequest>,
) -> Result<Json<SidebarPreferenceRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let prefs = state
        .preferences
        .upsert_sidebar_prefs(&company_id, &body.user_id, body.project_order)
        .await
        .map_err(preference_error_to_api)?;
    Ok(Json(prefs))
}

/// `GET /api/companies/{companyId}/logo` — reads the company logo.
#[route(GET "/api/companies/{company_id}/logo")]
pub async fn get_logo(cx: &Cx) -> Result<Json<CompanyLogoRecord>, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    state
        .preferences
        .logo(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Company logo not found"))
}

/// `PUT /api/companies/{companyId}/logo` — sets the company logo.
#[route(PUT "/api/companies/{company_id}/logo")]
pub async fn set_logo(
    cx: &Cx,
    Json(body): Json<SetLogoRequest>,
) -> Result<Json<CompanyLogoRecord>, ApiError> {
    require_board(cx)?;
    if !is_uuid(&body.asset_id) {
        return Err(ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["assetId"], "message": "Invalid uuid" }]),
        ));
    }
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let logo = state
        .preferences
        .set_logo(&company_id, &body.asset_id)
        .await
        .map_err(preference_error_to_api)?;
    Ok(Json(logo))
}

/// `DELETE /api/companies/{companyId}/logo` — removes the company logo.
#[route(DELETE "/api/companies/{company_id}/logo")]
pub async fn delete_logo(cx: &Cx) -> Result<StatusCode, ApiError> {
    require_board(cx)?;
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    match state
        .preferences
        .delete_logo(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
    {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(ApiError::not_found("Company logo not found")),
    }
}

fn preference_error_to_api(error: staple_data::PreferenceError) -> ApiError {
    use staple_data::PreferenceError as E;
    match error {
        E::CompanyNotFound => ApiError::not_found("Company not found"),
        E::AssetNotFound => ApiError::unprocessable(
            "Validation error",
            json!([{ "path": ["assetId"], "message": "Asset not found" }]),
        ),
        other => ApiError::internal(other.to_string()),
    }
}
