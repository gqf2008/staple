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

/// `POST /api/companies/{companyId}/teams/catalog/{catalogId}/preview` —
/// previews an install plan.
#[route(POST "/api/companies/{company_id}/teams/catalog/{catalog_id}/preview")]
pub async fn preview_install(cx: &Cx) -> Result<Json<team_catalog::PreviewResult>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let catalog_id = path_param::<CatalogId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let team = team_catalog::detail(&catalog_id)
        .ok_or_else(|| ApiError::not_found("Catalog team not found"))?;
    let agents = state
        .agents
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let projects = state
        .projects
        .list(&company_id)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(team_catalog::preview(&team, &agents, &projects)))
}

/// `POST /api/companies/{companyId}/teams/catalog/{catalogId}/install` —
/// installs a team (creates agents + projects with provenance).
#[route(POST "/api/companies/{company_id}/teams/catalog/{catalog_id}/install")]
pub async fn install_team(cx: &Cx) -> Result<Json<serde_json::Value>, ApiError> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let catalog_id = path_param::<CatalogId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    let team = team_catalog::detail(&catalog_id)
        .ok_or_else(|| ApiError::not_found("Catalog team not found"))?;
    let mut created_agents = 0i64;
    let mut created_projects = 0i64;
    for slug in &team.agent_slugs {
        let agent = state
            .agents
            .create(staple_data::NewAgent {
                company_id: company_id.clone(),
                name: slug.clone(),
                role: "worker".to_owned(),
                title: None,
                icon: None,
                reports_to: None,
                adapter_type: "cli".to_owned(),
                budget_monthly_cents: 0,
            })
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;
        let metadata = team_catalog::provenance_metadata(&team.id, &team.key, &team.content_hash);
        let _ = state
            .agents
            .set_agent_metadata(&company_id, &agent.id, metadata)
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;
        created_agents += 1;
    }
    for slug in &team.project_slugs {
        let _ = state
            .projects
            .create(staple_data::NewProject {
                company_id: company_id.clone(),
                goal_id: None,
                name: slug.clone(),
                description: None,
                status: "backlog".to_owned(),
                lead_agent_id: None,
                target_date: None,
                env: None,
            })
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?;
        created_projects += 1;
    }
    crate::audit::log_activity(
        &state.activity,
        &company_id,
        "teams_catalog.installed",
        "team_catalog",
        &team.id,
        Some(serde_json::json!({
            "catalogId": team.id,
            "createdAgents": created_agents,
            "createdProjects": created_projects,
        })),
    )
    .await?;
    Ok(Json(serde_json::json!({
        "catalogId": team.id,
        "createdAgents": created_agents,
        "createdProjects": created_projects,
        "skillsDeferred": true,
    })))
}

/// `POST /companies/{company_id}/teams/catalog/{catalog_id}/install/ui` —
/// installs a team from the detail page and redirects back.
#[route(POST "/companies/{company_id}/teams/catalog/{catalog_id}/install/ui")]
pub async fn install_team_ui(cx: &Cx) -> Result<topcoat::router::error::SeeOther> {
    let company_id = path_param::<CompanyId>(cx)?.to_string();
    enforce_company_scope(cx, &company_id)?;
    let catalog_id = path_param::<CatalogId>(cx)?.to_string();
    let state = app_context::<AppState>(cx);
    if let Some(team) = team_catalog::detail(&catalog_id) {
        for slug in &team.agent_slugs {
            if let Ok(agent) = state
                .agents
                .create(staple_data::NewAgent {
                    company_id: company_id.clone(),
                    name: slug.clone(),
                    role: "worker".to_owned(),
                    title: None,
                    icon: None,
                    reports_to: None,
                    adapter_type: "cli".to_owned(),
                    budget_monthly_cents: 0,
                })
                .await
            {
                let metadata =
                    team_catalog::provenance_metadata(&team.id, &team.key, &team.content_hash);
                let _ = state
                    .agents
                    .set_agent_metadata(&company_id, &agent.id, metadata)
                    .await;
            }
        }
        for slug in &team.project_slugs {
            let _ = state
                .projects
                .create(staple_data::NewProject {
                    company_id: company_id.clone(),
                    goal_id: None,
                    name: slug.clone(),
                    description: None,
                    status: "backlog".to_owned(),
                    lead_agent_id: None,
                    target_date: None,
                    env: None,
                })
                .await;
        }
    }
    Ok(topcoat::router::error::see_other(&format!(
        "/companies/{company_id}/teams/catalog/{catalog_id}?result=installed"
    )))
}
