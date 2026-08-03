//! Activity log routes (audit trail).

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, path_param, route},
};

use crate::{dto::ActivityEntryDto, error::ApiError, routes::CompanyId, state::AppState};

/// `GET /api/companies/{companyId}/activity` — lists audit entries, newest
/// first.
#[route(GET "/api/companies/{company_id}/activity")]
pub async fn list_activity(cx: &Cx) -> Result<Json<Vec<ActivityEntryDto>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let entries = state
        .activity
        .list(&company_id, 200)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(
        entries.into_iter().map(ActivityEntryDto::from).collect(),
    ))
}
