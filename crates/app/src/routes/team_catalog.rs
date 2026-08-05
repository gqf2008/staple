//! Teams catalog routes (read-only browse surface).

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{content::Json, path_param, route},
};

use crate::{
    auth::{enforce_company_scope, require_board},
    error::ApiError,
    routes::CompanyId,
    state::AppState,
    team_catalog,
};

/// `{catalog_id}` path parameter for team catalog routes.
#[path_param(error = bad_request("Invalid catalog id"))]
pub(crate) struct CatalogId(String);

/// `GET /api/teams/catalog` — lists catalog teams.
#[route(GET "/api/teams/catalog")]
pub async fn list_catalog(cx: &Cx) -> Result<Json<Vec<team_catalog::CatalogTeam>>, ApiError> {
    require_board(cx)?;
    Ok(Json(team_catalog::list()))
}

/// `GET /api/teams/catalog/{catalog_id}/files?path=` — reads a team file.
#[route(GET "/api/teams/catalog/{catalog_id}/files")]
pub async fn catalog_file(cx: &Cx) -> Result<Json<team_catalog::CatalogTeamFileDetail>, ApiError> {
    require_board(cx)?;
    let catalog_id = path_param::<CatalogId>(cx)?.to_string();
    let relative_path = topcoat::context::try_request_context::<http::request::Parts>(cx)
        .and_then(|parts| {
            parts.uri.query().and_then(|query| {
                query.split('&').find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    (key == "path").then(|| value.to_owned())
                })
            })
        })
        .unwrap_or_else(|| "TEAM.md".to_owned());
    let file = team_catalog::files(&catalog_id, &relative_path)
        .ok_or_else(|| ApiError::not_found("Catalog team file not found"))?;
    Ok(Json(file))
}

/// `GET /api/companies/{companyId}/teams/catalog/installed` — installed teams.
#[route(GET "/api/companies/{company_id}/teams/catalog/installed")]
pub async fn installed_catalog(
    cx: &Cx,
) -> Result<Json<Vec<team_catalog::InstalledCatalogTeam>>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let state = app_context::<AppState>(cx);
    let agents = state
        .agents
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(team_catalog::installed(&agents)))
}
