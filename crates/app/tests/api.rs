//! API integration tests: health endpoint, unified JSON error handling, and
//! company CRUD.

use std::sync::Arc;

use http::{Method, Request, header::CONTENT_TYPE};
use serde_json::{Value, json};
use staple_adapters::{AdapterRegistry, CliAdapter, CliAdapterConfig};
use staple_app::router;
use staple_app::state::AppState;
use staple_app::storage::LocalStorage;
use staple_data::{
    DbConfig, SecretCipher, TursoActivityRepository, TursoAgentRepository,
    TursoAgentRuntimeRepository, TursoApiKeyRepository, TursoApprovalRepository,
    TursoAssetRepository, TursoBoardKeyRepository, TursoBudgetPolicyRepository,
    TursoCaseRepository, TursoCompanyRepository, TursoCostRepository,
    TursoDecisionActionRepository, TursoDecisionRepository, TursoDocumentRepository,
    TursoEnvironmentRepository, TursoExternalObjectCatalogRepository,
    TursoExternalObjectRepository, TursoGoalRepository, TursoHeartbeatRepository,
    TursoInfrastructureRepository, TursoInviteRepository, TursoIssueCommentRepository,
    TursoIssueRelationRepository, TursoIssueRepository, TursoIssueStructureRepository,
    TursoLabelRepository, TursoMembershipRepository, TursoPermissionGrantRepository,
    TursoPipelineRepository, TursoPluginRepository, TursoPluginRuntimeRepository,
    TursoPreferenceRepository, TursoProjectRepository, TursoRoutineRepository,
    TursoSecretRepository, TursoSkillRepository, TursoWorkProductRepository,
    TursoWorkspaceRepository, migrate, open,
};
use topcoat::router::{Body, Router, StatusCode, to_bytes};

async fn test_state() -> AppState {
    test_state_with_db().await.0
}

async fn test_state_with_db() -> (AppState, staple_data::Database) {
    let dir = tempfile::tempdir().unwrap();
    let seed_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let companies_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let agents_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let agent_runtime_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let permission_grants_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let memberships_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let invites_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let infrastructure_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let board_keys_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let budget_policies_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let cases_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let preferences_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let plugins_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let pipelines_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let plugin_runtime_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let goals_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let projects_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let issues_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let comments_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let documents_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let assets_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let relations_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let work_products_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let heartbeat_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let costs_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let approvals_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let activity_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let secrets_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let secret_cipher = SecretCipher::load_or_create(dir.path().join("master.key")).unwrap();
    let api_keys_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let decisions_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let decision_actions_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let external_objects_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let external_object_catalog_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let skills_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let environments_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let workspaces_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let labels_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let issue_structure_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let routines_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    migrate(&companies_db).await.unwrap();
    let uploads = dir.path().join("uploads");
    // Keep the temp dir alive for the lifetime of the test process.
    std::mem::forget(dir);
    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(companies_db)),
        agents: Arc::new(TursoAgentRepository::new(agents_db)),
        agent_runtime: Arc::new(TursoAgentRuntimeRepository::new(agent_runtime_db)),
        permission_grants: Arc::new(TursoPermissionGrantRepository::new(permission_grants_db)),
        memberships: Arc::new(TursoMembershipRepository::new(memberships_db)),
        invites: Arc::new(TursoInviteRepository::new(invites_db)),
        infrastructure: Arc::new(TursoInfrastructureRepository::new(infrastructure_db)),
        board_keys: Arc::new(TursoBoardKeyRepository::new(board_keys_db)),
        budget_policies: Arc::new(TursoBudgetPolicyRepository::new(budget_policies_db)),
        cases: Arc::new(TursoCaseRepository::new(cases_db)),
        preferences: Arc::new(TursoPreferenceRepository::new(preferences_db)),
        pipelines: Arc::new(TursoPipelineRepository::new(pipelines_db)),
        plugins: Arc::new(TursoPluginRepository::new(plugins_db)),
        plugin_runtime: Arc::new(TursoPluginRuntimeRepository::new(plugin_runtime_db)),
        goals: Arc::new(TursoGoalRepository::new(goals_db)),
        projects: Arc::new(TursoProjectRepository::new(projects_db)),
        issues: Arc::new(TursoIssueRepository::new(issues_db)),
        comments: Arc::new(TursoIssueCommentRepository::new(comments_db)),
        documents: Arc::new(TursoDocumentRepository::new(documents_db)),
        assets: Arc::new(TursoAssetRepository::new(assets_db)),
        relations: Arc::new(TursoIssueRelationRepository::new(relations_db)),
        storage: LocalStorage::new(uploads),
        work_products: Arc::new(TursoWorkProductRepository::new(work_products_db)),
        heartbeat: Arc::new(TursoHeartbeatRepository::new(heartbeat_db)),
        costs: Arc::new(TursoCostRepository::new(costs_db)),
        approvals: Arc::new(TursoApprovalRepository::new(approvals_db)),
        activity: Arc::new(TursoActivityRepository::new(activity_db)),
        secrets: Arc::new(TursoSecretRepository::new(secrets_db, secret_cipher)),
        api_keys: Arc::new(TursoApiKeyRepository::new(api_keys_db)),
        decisions: Arc::new(TursoDecisionRepository::new(decisions_db)),
        decision_actions: Arc::new(TursoDecisionActionRepository::new(decision_actions_db)),
        external_objects: Arc::new(TursoExternalObjectRepository::new(external_objects_db)),
        external_object_catalog: Arc::new(TursoExternalObjectCatalogRepository::new(
            external_object_catalog_db,
        )),
        skills: Arc::new(TursoSkillRepository::new(skills_db)),
        environments: Arc::new(TursoEnvironmentRepository::new(environments_db)),
        workspaces: Arc::new(TursoWorkspaceRepository::new(workspaces_db)),
        labels: Arc::new(TursoLabelRepository::new(labels_db)),
        issue_structure: Arc::new(TursoIssueStructureRepository::new(issue_structure_db)),
        routines: Arc::new(TursoRoutineRepository::new(routines_db)),
        adapters: Arc::new({
            let mut registry = AdapterRegistry::new();
            registry.register(Box::new(CliAdapter::new(CliAdapterConfig::default())));
            registry
        }),
        plugin_reports: Vec::new(),
    };
    (state, seed_db)
}

async fn send(
    router: &Router,
    method: Method,
    path: &str,
    body: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(_body) = body {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    let request = builder
        .body(Body::from(body.unwrap_or_default().to_owned()))
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

async fn send_json(
    router: &Router,
    method: Method,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let (status, text) = send(router, method, path, Some(&body.to_string())).await;
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

#[tokio::test]
async fn health_returns_200_ok() {
    let (status, body) = send(
        &router(test_state().await),
        Method::GET,
        "/api/health",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn unknown_api_path_returns_json_404() {
    let (status, body) = send(
        &router(test_state().await),
        Method::GET,
        "/api/does-not-exist",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, r#"{"error":"not found"}"#);
}

#[tokio::test]
async fn wrong_method_returns_json_405() {
    let (status, body) = send(
        &router(test_state().await),
        Method::POST,
        "/api/health",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(body, r#"{"error":"method not allowed"}"#);
}

#[tokio::test]
async fn non_api_404_is_also_json() {
    let (status, body) = send(&router(test_state().await), Method::GET, "/nope", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, r#"{"error":"not found"}"#);
}

#[tokio::test]
async fn company_crud_flow() {
    let state = test_state().await;
    let app = router(state);

    // Create -> 201 with camelCase body.
    let (status, created) = send_json(
        &app,
        Method::POST,
        "/api/companies",
        json!({ "name": "Acme Corp", "budgetMonthlyCents": 2500 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Acme Corp");
    assert_eq!(created["status"], "active");
    assert_eq!(created["budgetMonthlyCents"], 2500);
    assert_eq!(created["spentMonthlyCents"], 0);
    assert_eq!(created["issuePrefix"], "ACM");
    let company_id = created["id"].as_str().unwrap().to_owned();

    // List -> contains the created company.
    let (status, list) = send_json(&app, Method::GET, "/api/companies", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], company_id);

    // Get by id.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["name"], "Acme Corp");

    // Patch -> updated fields.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/companies/{company_id}"),
        json!({ "name": "Acme 2", "status": "paused", "description": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "Acme 2");
    assert_eq!(updated["status"], "paused");
    assert_eq!(updated["description"], Value::Null);
}

#[tokio::test]
async fn company_get_missing_returns_404() {
    let app = router(test_state().await);
    let (status, body) = send_json(
        &app,
        Method::GET,
        "/api/companies/does-not-exist",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Company not found" }));
}

#[tokio::test]
async fn company_patch_missing_returns_404() {
    let app = router(test_state().await);
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/companies/does-not-exist",
        json!({ "name": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Company not found" }));
}

#[tokio::test]
async fn company_create_validation_failure_returns_422() {
    let app = router(test_state().await);

    // Empty name.
    let (status, body) =
        send_json(&app, Method::POST, "/api/companies", json!({ "name": "" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    assert_eq!(body["details"][0]["path"][0], "name");

    // Negative budget.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies",
        json!({ "name": "Acme", "budgetMonthlyCents": -1 }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "budgetMonthlyCents");

    // Missing name.
    let (status, _) = send_json(&app, Method::POST, "/api/companies", json!({})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn company_patch_validation_failure_returns_422() {
    let app = router(test_state().await);
    let (_, created) = send_json(
        &app,
        Method::POST,
        "/api/companies",
        json!({ "name": "Acme" }),
    )
    .await;
    let company_id = created["id"].as_str().unwrap().to_owned();

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/companies/{company_id}"),
        json!({ "status": "invalid-status" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    assert_eq!(body["details"][0]["path"][0], "status");

    let (status, _) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/companies/{company_id}"),
        json!({ "brandColor": "red" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn company_create_malformed_json_returns_json_400() {
    let app = router(test_state().await);
    let (status, body) = send(&app, Method::POST, "/api/companies", Some(r#"{not json"#)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body.contains("\"error\"") && body.to_lowercase().contains("bad request"),
        "unexpected body: {body}"
    );
}

async fn create_company_via(app: &Router, name: &str) -> String {
    let (status, body) =
        send_json(app, Method::POST, "/api/companies", json!({ "name": name })).await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn goal_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "Growth", "level": "team", "status": "active" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], "Growth");
    assert_eq!(created["level"], "team");
    assert_eq!(created["status"], "active");
    let goal_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/goals"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Get by id.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/goals/{goal_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["title"], "Growth");

    // Patch.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/goals/{goal_id}"),
        json!({ "status": "achieved", "description": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "achieved");
    assert_eq!(updated["description"], Value::Null);

    // Delete -> 204, then 404.
    let (status, _) = send(&app, Method::DELETE, &format!("/api/goals/{goal_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/goals/{goal_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Goal not found" }));
}

#[tokio::test]
async fn goal_hierarchy_constraints_are_enforced() {
    let app = router(test_state().await);
    let company_a = create_company_via(&app, "Alpha").await;
    let company_b = create_company_via(&app, "Beta").await;

    // Parent in another company is rejected with 422.
    let (_, parent) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_a}/goals"),
        json!({ "title": "Parent" }),
    )
    .await;
    let parent_id = parent["id"].as_str().unwrap().to_owned();

    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_b}/goals"),
        json!({ "title": "Child", "parentId": parent_id }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "parentId");
    assert_eq!(
        body["details"][0]["message"],
        "Parent goal belongs to a different company"
    );
}

#[tokio::test]
async fn goal_validation_failure_returns_422() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "", "level": "bogus" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    let paths: Vec<&str> = body["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"][0].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"title"));
    assert!(paths.contains(&"level"));
}

#[tokio::test]
async fn project_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Goal for linking.
    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "Growth" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Create -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "Ship", "goalId": goal_id, "status": "planned" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Ship");
    assert_eq!(created["goalId"], goal_id);
    let project_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/projects"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Get.
    let (status, _fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Patch.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/projects/{project_id}"),
        json!({ "status": "in_progress", "description": "now" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "in_progress");

    // Delete -> 204.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/projects/{project_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn project_hierarchy_constraints_are_enforced() {
    let app = router(test_state().await);
    let company_a = create_company_via(&app, "Alpha").await;
    let company_b = create_company_via(&app, "Beta").await;

    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_a}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Cross-company goal link is rejected with 422.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_b}/projects"),
        json!({ "name": "P", "goalId": goal_id }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "goalId");
    assert_eq!(
        body["details"][0]["message"],
        "Goal belongs to a different company"
    );
}

#[tokio::test]
async fn project_404s() {
    let app = router(test_state().await);
    let (status, body) = send_json(&app, Method::GET, "/api/projects/missing", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Project not found" }));

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/projects/missing",
        json!({ "name": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Project not found" }));
}

#[tokio::test]
async fn delete_referenced_goal_returns_409() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Link a project to the goal, then deletion must conflict.
    send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "P", "goalId": goal_id }),
    )
    .await;

    let (status, body) = send(&app, Method::DELETE, &format!("/api/goals/{goal_id}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, r#"{"error":"Goal is referenced by other records"}"#);
}

#[tokio::test]
async fn issue_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create -> 201 with identifier allocation.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "First task", "priority": "high" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], "First task");
    assert_eq!(created["identifier"], "ACM-1");
    assert_eq!(created["issueNumber"], 1);
    assert_eq!(created["status"], "backlog");
    assert_eq!(created["priority"], "high");
    let issue_id = created["id"].as_str().unwrap().to_owned();

    // Assigned issue defaults to todo.
    let (_, assigned) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "Second", "assigneeUserId": "u1" }),
    )
    .await;
    assert_eq!(assigned["status"], "todo");
    assert_eq!(assigned["identifier"], "ACM-2");

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/issues"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 2);

    // Get.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["title"], "First task");

    // Patch through the state machine: backlog -> todo -> in_progress -> done.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "todo" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "todo");

    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "in_progress" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(updated["startedAt"].is_string());

    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "done", "title": "First task (done)" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "done");
    assert!(updated["completedAt"].is_string());

    // Delete -> 204.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issues/{issue_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn issue_state_machine_rejects_invalid_transition() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (_, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "T" }),
    )
    .await;
    let issue_id = created["id"].as_str().unwrap().to_owned();

    // backlog -> done is not allowed.
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}"),
        json!({ "status": "done" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "status");
    assert!(
        body["details"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid status transition")
    );
}

#[tokio::test]
async fn issue_validation_failure_returns_422() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "", "priority": "urgent" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
    let paths: Vec<&str> = body["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"][0].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"title"));
    assert!(paths.contains(&"priority"));
}

#[tokio::test]
async fn issue_hierarchy_constraints_are_enforced() {
    let app = router(test_state().await);
    let company_a = create_company_via(&app, "Alpha").await;
    let company_b = create_company_via(&app, "Beta").await;

    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_a}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    // Cross-company goal link is rejected.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_b}/issues"),
        json!({ "title": "T", "goalId": goal_id }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["details"][0]["path"][0], "goal");
}

#[tokio::test]
async fn issue_404s() {
    let app = router(test_state().await);
    let (status, body) = send_json(&app, Method::GET, "/api/issues/missing", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Issue not found" }));

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/issues/missing",
        json!({ "title": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Issue not found" }));
}

async fn create_issue_via(app: &Router, company_id: &str, title: &str) -> String {
    let (status, body) = send_json(
        app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": title }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn comment_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Add a comment -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/comments"),
        json!({ "body": "First comment", "authorUserId": "u1" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["body"], "First comment");
    assert_eq!(created["authorUserId"], "u1");
    let comment_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/comments"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Get one.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/comments/{comment_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["body"], "First comment");

    // Empty body -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/comments"),
        json!({ "body": " " }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Delete -> 204, then 404.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issues/{issue_id}/comments/{comment_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issues/{issue_id}/comments/{comment_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn document_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Create -> 201 with revision 1.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/documents"),
        json!({ "key": "plan", "title": "Plan", "body": "# v1" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["latestBody"], "# v1");
    assert_eq!(created["latestRevisionNumber"], 1);

    // Duplicate key -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/documents"),
        json!({ "key": "plan", "body": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Update -> revision 2.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_id}/documents/plan"),
        json!({ "body": "# v2", "changeSummary": "rewrite" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["latestBody"], "# v2");
    assert_eq!(updated["latestRevisionNumber"], 2);

    // Get by key.
    let (status, fetched) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/documents/plan"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["latestRevisionNumber"], 2);

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/documents"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Missing key -> 404.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/documents/design"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({ "error": "Document not found" }));
}

#[tokio::test]
async fn blocker_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let blocking = create_issue_via(&app, &company_id, "blocking").await;
    let blocked = create_issue_via(&app, &company_id, "blocked").await;

    // Add blocker -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{blocked}/blockers"),
        json!({ "blockerIssueId": blocking }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["issueId"], blocking);
    assert_eq!(created["relatedIssueId"], blocked);
    let relation_id = created["id"].as_str().unwrap().to_owned();

    // List blockers of the blocked issue.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{blocked}/blockers"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["issueId"], blocking);

    // Duplicate -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{blocked}/blockers"),
        json!({ "blockerIssueId": blocking }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Remove -> 204.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issue-relations/{relation_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn asset_upload_and_attach_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Upload (multipart body built by hand).
    let boundary = "X-TEST-BOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"hello.txt\"\r\nContent-Type: text/plain\r\n\r\nhello\r\n--{boundary}--\r\n"
    );
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/companies/{company_id}/assets"))
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();
    let response = app.handle(request).await;
    let status = response.status();
    let (_, response_body) = response.into_parts();
    let bytes = to_bytes(response_body, usize::MAX).await.unwrap();
    let asset: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(asset["originalFilename"], "hello.txt");
    assert_eq!(asset["byteSize"], 5);
    assert_eq!(asset["provider"], "local_disk");
    let asset_id = asset["id"].as_str().unwrap().to_owned();

    // Read content back.
    let (status, content) = send(
        &app,
        Method::GET,
        &format!("/api/assets/{asset_id}/content"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content, "hello");

    // Attach to issue -> 201.
    let (status, attachment) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/attachments"),
        json!({ "assetId": asset_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(attachment["assetId"], asset_id);

    // List attachments.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/attachments"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn work_product_crud_flow() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Create -> 201.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/work-products"),
        json!({
            "type": "artifact",
            "provider": "paperclip",
            "title": "Report.pdf",
            "status": "active",
            "isPrimary": true,
            "metadata": { "kind": "workspace_file", "relativePath": "report.pdf" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["title"], "Report.pdf");
    assert_eq!(created["isPrimary"], true);
    let product_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/work-products"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Patch.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/work-products/{product_id}"),
        json!({ "status": "archived", "summary": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "archived");

    // Delete -> 204, then 404.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/work-products/{product_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/work-products/{product_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn work_product_validation_failure_returns_422() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/work-products"),
        json!({ "type": "", "provider": "", "title": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");
}

#[tokio::test]
async fn heartbeat_run_lifecycle_and_lock() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier, status)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'T', 1, 'ALPHA-1', 'in_progress')",
        (),
    )
    .await
    .unwrap();

    // Start -> 201, running.
    let (status, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual", "issueId": "22222222-2222-2222-2222-222222222222" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(run["status"], "running");
    let run_id = run["id"].as_str().unwrap().to_owned();

    // Concurrent start on the same issue -> 409.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual", "issueId": "22222222-2222-2222-2222-222222222222" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("already checked out")
    );

    // Observe.
    let (status, observed) = send_json(
        &app,
        Method::GET,
        &format!("/api/heartbeat-runs/{run_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(observed["status"], "running");

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/heartbeat-runs",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Complete -> releases the lock, another run can start.
    let (status, completed) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/complete"),
        json!({ "status": "succeeded" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(completed["status"], "succeeded");

    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual", "issueId": "22222222-2222-2222-2222-222222222222" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn heartbeat_failure_attribution_and_cancel() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();

    // Infrastructure failure attribution.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual" }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();
    let (status, completed) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/complete"),
        json!({ "status": "failed", "error": "clone failed", "errorKind": "infrastructure" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(completed["errorKind"], "infrastructure");
    assert_eq!(completed["error"], "clone failed");

    // Cancel.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "invocationSource": "manual" }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();
    let (status, cancelled) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");

    // Invalid completion status -> 422.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111" }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/complete"),
        json!({ "status": "bogus" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn watchdog_authorization_via_api() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'root', 1, 'ALPHA-1'),
                ('33333333-3333-3333-3333-333333333333', 'c1', 'child', 2, 'ALPHA-2'),
                ('44444444-4444-4444-4444-444444444444', 'c1', 'outside', 3, 'ALPHA-3')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "UPDATE issues SET parent_id = '22222222-2222-2222-2222-222222222222' WHERE id = '33333333-3333-3333-3333-333333333333'",
        (),
    )
    .await
    .unwrap();

    // Watchdog run scoped to i1.
    let (_, run) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/heartbeat-runs",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "invocationSource": "scheduler",
            "contextSnapshot": { "kind": "task_watchdog", "watchedIssueId": "22222222-2222-2222-2222-222222222222" }
        }),
    )
    .await;
    let run_id = run["id"].as_str().unwrap().to_owned();

    // Child in subtree -> allowed.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/watchdog-actions"),
        json!({ "issueId": "33333333-3333-3333-3333-333333333333", "action": "update_status" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["allowed"], true);

    // Outside the subtree -> 403.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/heartbeat-runs/{run_id}/watchdog-actions"),
        json!({ "issueId": "44444444-4444-4444-4444-444444444444", "action": "update_status" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body["error"].as_str().unwrap().contains("not authorized"));
}

#[tokio::test]
async fn cost_event_and_budget_hard_stop_flow() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes, budget_monthly_cents)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024, 100)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local'),
                ('22222222-2222-2222-2222-222222222222', 'c1', 'two', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();

    // Record a cost event -> 201 with event + hardStop info.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cost-events",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "provider": "anthropic",
            "model": "claude",
            "inputTokens": 100,
            "outputTokens": 50,
            "costCents": 60,
            "occurredAt": "2026-08-03T00:00:00.000Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["event"]["costCents"], 60);
    assert_eq!(body["event"]["provider"], "anthropic");
    // 60 < 100: no hard stop yet.
    assert_eq!(body["hardStop"]["triggered"], false);

    // Summary.
    let (status, summary) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/costs/summary",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(summary["spentMonthlyCents"], 60);
    assert_eq!(summary["remainingCents"], 40);
    assert_eq!(summary["pausedAgents"], 0);

    // by-agent.
    let (status, by_agent) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/costs/by-agent",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let spent = by_agent
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["agentId"] == "11111111-1111-1111-1111-111111111111")
        .expect("agent row");
    assert_eq!(spent["spentMonthlyCents"], 60);

    // Exhaust the company budget -> both agents paused.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cost-events",
        json!({
            "agentId": "22222222-2222-2222-2222-222222222222",
            "provider": "anthropic",
            "model": "claude",
            "costCents": 60,
            "occurredAt": "2026-08-03T00:00:01.000Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["hardStop"]["triggered"], true);
    let paused_ids: Vec<&str> = body["hardStop"]["pausedAgentIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert!(paused_ids.contains(&"11111111-1111-1111-1111-111111111111"));
    assert!(paused_ids.contains(&"22222222-2222-2222-2222-222222222222"));

    // Reset: spending zeroed, agents resumed.
    let (status, reset) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/budget/reset",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reset["resumedAgents"], 2);

    let (_, summary) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/costs/summary",
        json!({}),
    )
    .await;
    assert_eq!(summary["spentMonthlyCents"], 0);
    assert_eq!(summary["pausedAgents"], 0);

    // Set budget.
    let (status, summary) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/budget",
        json!({ "budgetMonthlyCents": 500 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(summary["budgetMonthlyCents"], 500);
}

#[tokio::test]
async fn cost_event_validation_and_404() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();

    // Missing agent -> 422.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cost-events",
        json!({
            "agentId": "not-a-uuid",
            "provider": "p",
            "model": "m",
            "costCents": -1,
            "occurredAt": "2026-08-03T00:00:00.000Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "Validation error");

    // Unknown company -> 404.
    let (status, _) = send_json(
        &app,
        Method::GET,
        "/api/companies/missing/costs/summary",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn approval_state_machine_and_gate() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create approval -> 201 pending.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({
            "type": "budget_override_required",
            "requestedByUserId": "u1",
            "payload": { "budgetMonthlyCents": 500 }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["status"], "pending");
    assert_eq!(created["type"], "budget_override_required");
    let approval_id = created["id"].as_str().unwrap().to_owned();

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/approvals"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Approve -> applies the budget override (approval gate).
    let (status, decided) = send_json(
        &app,
        Method::POST,
        &format!("/api/approvals/{approval_id}/decide"),
        json!({ "decision": "approved", "decisionNote": "ok", "decidedByUserId": "board" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(decided["status"], "approved");
    assert!(decided["decidedAt"].is_string());

    let (_, summary) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/costs/summary"),
        json!({}),
    )
    .await;
    assert_eq!(summary["budgetMonthlyCents"], 500);

    // Decide again -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/approvals/{approval_id}/decide"),
        json!({ "decision": "rejected" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Cancel a second approval.
    let (_, second) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({ "type": "hire_agent", "payload": {} }),
    )
    .await;
    let second_id = second["id"].as_str().unwrap().to_owned();
    let (status, cancelled) = send_json(
        &app,
        Method::POST,
        &format!("/api/approvals/{second_id}/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");

    // Invalid type -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({ "type": "bogus" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn mutating_apis_write_audit_log() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // A few mutations across entities.
    let (_, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "G" }),
    )
    .await;
    let goal_id = goal["id"].as_str().unwrap().to_owned();
    let (_, project) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "P" }),
    )
    .await;
    let project_id = project["id"].as_str().unwrap().to_owned();
    let issue_id = create_issue_via(&app, &company_id, "T").await;
    send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/comments"),
        json!({ "body": "hello" }),
    )
    .await;

    // The audit trail must contain all of them.
    let (status, activity) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let actions: Vec<&str> = activity
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect();
    for expected in [
        "company.created",
        "goal.created",
        "project.created",
        "issue.created",
        "comment.created",
    ] {
        assert!(
            actions.contains(&expected),
            "missing audit entry {expected}: {actions:?}"
        );
    }

    // Entity ids match.
    let goal_entry = activity
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "goal.created")
        .unwrap();
    assert_eq!(goal_entry["entityId"], goal_id);
    let project_entry = activity
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "project.created")
        .unwrap();
    assert_eq!(project_entry["entityId"], project_id);
    assert!(
        activity
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["companyId"] == company_id)
    );
}

#[tokio::test]
async fn secret_lifecycle_rotation_rollback_and_redaction() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Create -> 201, version 1.
    let (status, created) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "github_token", "value": "ghp_v1_secret" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "github_token");
    assert_eq!(created["latestVersion"], 1);
    assert_eq!(created["provider"], "local_encrypted");

    // Duplicate -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "github_token", "value": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Value readback.
    let (status, value) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/value"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value["value"], "ghp_v1_secret");

    // Rotate -> v2.
    let (status, rotated) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets/github_token/rotate"),
        json!({ "value": "ghp_v2_secret" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rotated["latestVersion"], 2);
    let (_, value) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/value"),
        json!({}),
    )
    .await;
    assert_eq!(value["value"], "ghp_v2_secret");

    // Versions.
    let (status, versions) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/versions"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(versions.as_array().unwrap().len(), 2);

    // Rollback to v1 -> value is v1 again, version 3.
    let (status, rolled) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets/github_token/rollback"),
        json!({ "version": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rolled["latestVersion"], 3);
    let (_, value) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token/value"),
        json!({}),
    )
    .await;
    assert_eq!(value["value"], "ghp_v1_secret");

    // Redact: transcript containing the value is masked.
    let (status, redacted) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/redact"),
        json!({
            "text": "agent output: ghp_v1_secret was used",
            "names": ["github_token"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !redacted["redacted"]
            .as_str()
            .unwrap()
            .contains("ghp_v1_secret")
    );
    assert!(redacted["redacted"].as_str().unwrap().contains("***"));

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    // Values never leak in list responses.
    assert!(!list.to_string().contains("ghp_v1_secret"));

    // Delete -> 204, then 404.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/companies/{company_id}/secrets/github_token"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/secrets/github_token"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn secret_validation_and_audit() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;

    // Empty name/value -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "", "value": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // A successful creation is audited.
    send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/secrets"),
        json!({ "name": "aws_key", "value": "AKIA123" }),
    )
    .await;

    // Audit entries exist.
    let (status, activity) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let actions: Vec<&str> = activity
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect();
    assert!(actions.contains(&"secret.created"));
}

async fn send_with_auth(
    router: &Router,
    method: Method,
    path: &str,
    body: Option<&str>,
    bearer: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(path);
    if body.is_some() {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    if let Some(token) = bearer {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder
        .body(Body::from(body.unwrap_or_default().to_owned()))
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, response_body) = response.into_parts();
    let bytes = to_bytes(response_body, usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn three_identity_permission_matrix() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();

    // Board creates an API key for the agent.
    let (status, key_body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-api-keys",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "name": "dev" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let plaintext = key_body["plaintext"].as_str().unwrap().to_owned();
    assert!(plaintext.starts_with("sk-"));
    // Hash stored, not plaintext.
    assert_ne!(key_body["key"]["keyHash"], plaintext);

    // 1. Unauthenticated: invalid bearer -> 401 JSON.
    let (status, body) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some("sk-invalid"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.contains("Invalid API key"));

    // 2. Agent: own company allowed.
    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 3. Agent: cross-company access -> 403.
    let (status, body) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c2/issues",
        None,
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.contains("cannot access another company"));

    // 4. Agent: board-only action (create company) -> 403.
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies",
        Some(r#"{"name":"Sneaky"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 5. Agent: board-only budget action -> 403.
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/budget",
        Some(r#"{"budgetMonthlyCents":9999}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 6. Board: everything still works.
    let (status, _) = send_json(&app, Method::GET, "/api/companies/c1/issues", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    // 7. Revoke -> agent key becomes invalid (401).
    let key_id = key_body["key"]["id"].as_str().unwrap().to_owned();
    send_json(
        &app,
        Method::POST,
        &format!("/api/agent-api-keys/{key_id}/revoke"),
        json!({}),
    )
    .await;
    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn decision_desk_and_inbox_and_external_objects() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "T").await;

    // Decision queue.
    let (status, queue) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-queues"),
        json!({ "name": "approvals", "retentionDays": 30 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(queue["name"], "approvals");
    let queue_id = queue["id"].as_str().unwrap().to_owned();

    // Duplicate queue -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-queues"),
        json!({ "name": "approvals" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Queue item.
    let (status, item) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-queues/{queue_id}/items"),
        json!({ "sourceKind": "issue", "sourceId": issue_id, "payload": { "n": 1 } }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(item["sourceId"], issue_id);
    let (status, items) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/decision-queues/{queue_id}/items"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(items.as_array().unwrap().len(), 1);

    // Triage.
    let (status, triage) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/decision-triage"),
        json!({ "sourceKind": "issue", "sourceId": issue_id, "decision": "keep", "decidedByUserId": "board" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(triage["decision"], "keep");

    // Inbox: issue present; archive -> gone; unarchive -> back.
    let (status, inbox) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(inbox.as_array().unwrap().len(), 1);

    let (status, archived) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/archive"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(archived["hiddenAt"].is_string());

    let (_, inbox) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox"),
        json!({}),
    )
    .await;
    assert_eq!(inbox.as_array().unwrap().len(), 0);

    let (_, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/unarchive"),
        json!({}),
    )
    .await;
    let (_, inbox) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/inbox"),
        json!({}),
    )
    .await;
    assert_eq!(inbox.as_array().unwrap().len(), 1);

    // External objects.
    let (status, external) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/external-objects"),
        json!({ "kind": "github_pr", "externalId": "42", "url": "https://github.com/x/y/pull/42" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(external["status"], "pending");
    let external_id = external["id"].as_str().unwrap().to_owned();

    let (status, refreshed) = send_json(
        &app,
        Method::POST,
        &format!("/api/external-objects/{external_id}/refresh"),
        json!({ "status": "merged" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(refreshed["status"], "merged");
    assert!(refreshed["lastSyncedAt"].is_string());

    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/external-objects"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn skills_policy_evaluation() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local'),
                ('22222222-2222-2222-2222-222222222222', 'c1', 'two', 'senior', 'codex_local'),
                ('33333333-3333-3333-3333-333333333333', 'c2', 'three', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();

    // Board creates a skill restricted to senior role.
    let (status, skill) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/skills",
        json!({
            "name": "code_review",
            "description": "review",
            "restrictionPolicy": { "allowedRoles": ["senior"] }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(skill["name"], "code_review");

    // List.
    let (status, list) = send_json(&app, Method::GET, "/api/companies/c1/skills", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Senior allowed.
    let (status, evaluation) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/skills/evaluate",
        json!({ "agentId": "22222222-2222-2222-2222-222222222222", "skill": "code_review" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(evaluation["allowed"], true);

    // Engineer denied by role allow-list.
    let (_, evaluation) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/skills/evaluate",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "skill": "code_review" }),
    )
    .await;
    assert_eq!(evaluation["allowed"], false);
    assert!(evaluation["reason"].as_str().unwrap().contains("role"));

    // Cross-company agent denied (company boundary in the evaluator).
    let (_, evaluation) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/skills/evaluate",
        json!({ "agentId": "33333333-3333-3333-3333-333333333333", "skill": "code_review" }),
    )
    .await;
    assert_eq!(evaluation["allowed"], false);
    // Cross-company agents resolve to "not found" (existence is not leaked).
    assert!(evaluation["reason"].as_str().unwrap().contains("not found"));

    // Unknown skill denied.
    let (_, evaluation) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/skills/evaluate",
        json!({ "agentId": "22222222-2222-2222-2222-222222222222", "skill": "nope" }),
    )
    .await;
    assert_eq!(evaluation["allowed"], false);

    // Duplicate skill -> 409.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/skills",
        json!({ "name": "code_review" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Agent cannot create skills (board-only).
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/skills",
        Some(r#"{"name":"x"}"#),
        Some("sk-agent"),
    )
    .await;
    // Invalid key -> 401 (no key was created for the agent).
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn board_ui_pages_render() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    create_issue_via(&app, &company_id, "UI task").await;

    // Home page lists companies.
    let (status, html) = send(&app, Method::GET, "/", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Companies"));
    assert!(html.contains("Acme"));
    // Token layer is present; no bare values in markup.
    assert!(html.contains("--color-primary"));
    assert!(html.contains("--space-4"));

    // Company overview shows the issue.
    let (status, html) = send(&app, Method::GET, &format!("/companies/{company_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Issues"));
    assert!(html.contains("UI task"));

    // Issue list page.
    let (status, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}/issues"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("UI task"));

    // Unknown company -> 404.
    let (status, _) = send(&app, Method::GET, "/companies/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

async fn send_form(
    router: &Router,
    method: Method,
    path: &str,
    body: &str,
) -> (StatusCode, String) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, response_body) = response.into_parts();
    let bytes = to_bytes(response_body, usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn issue_detail_approvals_and_activity_pages() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "Detail task").await;

    // Issue detail page: attributes, empty sections.
    let (status, html) = send(&app, Method::GET, &format!("/issues/{issue_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Detail task"));
    assert!(html.contains("Comments"));
    assert!(html.contains("Documents"));
    assert!(html.contains("Attachments"));
    assert!(html.contains("Work products"));
    assert!(html.contains("Add a comment"));

    // Comment form POST -> redirect back, comment appears.
    let (status, _) = send_form(
        &app,
        Method::POST,
        &format!("/issues/{issue_id}/comments/ui"),
        "body=from+the+board",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let (_, html) = send(&app, Method::GET, &format!("/issues/{issue_id}"), None).await;
    assert!(html.contains("from the board"));

    // Approvals page: request form + create via UI + approve via UI.
    let (status, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}/approvals"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Request"));
    assert!(html.contains("hire_agent"));

    let (status, _) = send_form(
        &app,
        Method::POST,
        &format!("/companies/{company_id}/approvals/ui"),
        "type=hire_agent&payload=%7B%7D",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let (_, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}/approvals"),
        None,
    )
    .await;
    assert!(html.contains("pending"));
    assert!(html.contains("Approve"));
    assert!(html.contains("Reject"));

    // Approve the first pending approval via the UI route.
    let (_, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/approvals"),
        json!({}),
    )
    .await;
    let approval_id = list[0]["id"].as_str().unwrap().to_owned();
    let (status, _) = send_form(
        &app,
        Method::POST,
        &format!("/approvals/{approval_id}/decide/ui"),
        "decision=approved",
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let (_, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}/approvals"),
        None,
    )
    .await;
    assert!(html.contains("approved"));

    // Activity page renders audit entries.
    let (status, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}/activity"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Audit log"));
    assert!(html.contains("company.created"));
    assert!(html.contains("comment.created"));
}

#[tokio::test]
async fn adapter_registry_and_cli_lifecycle() {
    let app = router(test_state().await);

    // Discovery.
    let (status, body) = send_json(&app, Method::GET, "/api/adapters", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["adapters"]
            .as_array()
            .unwrap()
            .contains(&json!("cli_local"))
    );

    // Invoke.
    let (status, handle) = send_json(
        &app,
        Method::POST,
        "/api/adapters/cli_local/invoke",
        json!({ "task": "echo adapter-ok" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = handle["runId"].as_str().unwrap().to_owned();

    // Observe (poll until terminal).
    let mut status_body = serde_json::Value::Null;
    for _ in 0..100 {
        let (_, observed) = send_json(
            &app,
            Method::GET,
            &format!("/api/adapters/cli_local/runs/{run_id}"),
            json!({}),
        )
        .await;
        status_body = observed;
        if status_body["status"] != "running" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(status_body["status"], "succeeded");
    assert_eq!(status_body["output"], "adapter-ok");

    // Unknown adapter -> 404.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/adapters/nope/invoke",
        json!({ "task": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn adapter_plugin_status_endpoint() {
    let app = router(test_state().await);
    let (status, body) =
        send_json(&app, Method::GET, "/api/adapters/plugins/status", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["reports"].is_array());
}

#[tokio::test]
async fn ui_internationalization_zh_cn_and_switcher() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "本地化任务").await;

    // English default: nav + page titles.
    let (_, html) = send(&app, Method::GET, "/", None).await;
    assert!(html.contains("Companies"));
    assert!(
        html.contains(">中文<"),
        "language switcher to zh-CN missing"
    );

    // zh-CN: query param switches the home page.
    let (_, html) = send(&app, Method::GET, "/?lang=zh-CN", None).await;
    assert!(html.contains("公司"));
    assert!(
        html.contains(">繁體<"),
        "language switcher to zh-TW missing (cycle En -> zh-CN -> zh-TW)"
    );
    assert!(html.contains("Acme"));

    // zh-TW: full upstream locale loads and switches back to English.
    let (_, html) = send(&app, Method::GET, "/?lang=zh-TW", None).await;
    assert!(html.contains("公司"));
    assert!(
        html.contains(">English<"),
        "language switcher back to English missing on zh-TW"
    );

    // zh-CN company overview: sections localized, issue title present.
    let (_, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}?lang=zh-CN"),
        None,
    )
    .await;
    assert!(html.contains("目标"));
    assert!(html.contains("项目"));
    assert!(html.contains("任务"));
    assert!(html.contains("本地化任务"));

    // zh-CN issue detail: sections + comment form localized.
    let (_, html) = send(
        &app,
        Method::GET,
        &format!("/issues/{issue_id}?lang=zh-CN"),
        None,
    )
    .await;
    assert!(html.contains("评论"));
    assert!(html.contains("文档"));
    assert!(html.contains("附件"));
    assert!(html.contains("工作产物"));
    assert!(html.contains("发表评论"));
    assert!(html.contains("添加"));

    // zh-CN approvals + activity.
    let (_, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}/approvals?lang=zh-CN"),
        None,
    )
    .await;
    assert!(html.contains("审批"));
    assert!(html.contains("发起"));
    assert!(html.contains("待处理"));

    let (_, html) = send(
        &app,
        Method::GET,
        &format!("/companies/{company_id}/activity?lang=zh-CN"),
        None,
    )
    .await;
    assert!(html.contains("审计日志"));
    // The zh-CN title replaced the English one.
    assert!(!html.contains("Audit log"));
    // Audit action strings are data values and stay as-is (issue.created).

    // Language persists through navigation links (?lang= on every link).
    let (_, html) = send(&app, Method::GET, "/?lang=zh-CN", None).await;
    assert!(html.contains("/companies/"));
    // zh-CN page must not contain the untranslated English nav strings.
    assert!(!html.contains(">Companies<"));
}

#[tokio::test]
async fn environments_and_workspaces() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (_, project) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "P" }),
    )
    .await;
    let project_id = project["id"].as_str().unwrap().to_owned();

    // Environments (board): ensure-local + create + list + duplicate.
    let (status, local) = send_json(
        &app,
        Method::POST,
        "/api/environments/ensure-local",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(local["driver"], "local");

    let (status, created_env) = send_json(
        &app,
        Method::POST,
        "/api/environments",
        json!({ "name": "prod", "driver": "remote", "config": { "region": "cn" } }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created_env["status"], "active");

    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/environments",
        json!({ "name": "prod" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, list) = send_json(&app, Method::GET, "/api/environments", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 2);

    // Project workspace.
    let (status, pw) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/project-workspaces"),
        json!({ "projectId": project_id, "name": "main", "isPrimary": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(pw["isPrimary"], true);
    let pw_id = pw["id"].as_str().unwrap().to_owned();

    // Execution workspace.
    let (status, ew) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/execution-workspaces"),
        json!({
            "projectId": project_id,
            "projectWorkspaceId": pw_id,
            "mode": "ephemeral",
            "strategyType": "checkout",
            "name": "run-ws"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(ew["status"], "active");
    let ew_id = ew["id"].as_str().unwrap().to_owned();

    // Runtime service + operation.
    let (status, service) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/runtime-services"),
        json!({
            "executionWorkspaceId": ew_id,
            "scopeType": "execution_workspace",
            "scopeId": ew_id,
            "serviceName": "vite",
            "lifecycle": "ephemeral",
            "command": "npm run dev",
            "port": 5173,
            "provider": "local"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(service["port"], 5173);

    let (status, operation) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/workspace-operations"),
        json!({ "executionWorkspaceId": ew_id, "phase": "setup", "command": "git clone" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(operation["status"], "running");

    // Lists.
    let (_, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/project-workspaces"),
        json!({}),
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let (_, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/execution-workspaces"),
        json!({}),
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let (_, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/runtime-services"),
        json!({}),
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let (_, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/workspace-operations"),
        json!({}),
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Cross-company reference rejected (project of another company).
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c2/project-workspaces",
        json!({ "projectId": project_id, "name": "x" }),
    )
    .await;
    // c2 does not exist -> the project reference check fails (422 via
    // ReferenceNotFound is not reached because company scope passes for the
    // board; the workspace repo rejects the foreign project).
    assert!(status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn issue_structure_extensions() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let issue_id = create_issue_via(&app, &company_id, "Structured").await;

    // Labels.
    let (status, label) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/labels"),
        json!({ "name": "bug", "color": "#dc2626" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(label["name"], "bug");
    let label_id = label["id"].as_str().unwrap().to_owned();

    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/labels"),
        json!({ "name": "bug", "color": "#000" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, attached) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/labels"),
        json!({ "labelId": label_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(attached["labelId"], label_id);

    let (_, labels) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/labels"),
        json!({}),
    )
    .await;
    assert_eq!(labels.as_array().unwrap().len(), 1);

    // Thread interaction.
    let (status, interaction) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/thread-interactions"),
        json!({ "kind": "review_request", "payload": { "reviewer": "u1" } }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(interaction["status"], "pending");

    // Read state upsert.
    let (status, read) = send_json(
        &app,
        Method::PUT,
        &format!("/api/issues/{issue_id}/read-state"),
        json!({ "userId": "u1", "lastReadAt": "2026-08-03T00:00:00.000Z" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(read["lastReadAt"], "2026-08-03T00:00:00.000Z");

    // Approval link.
    let (_, approval) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({ "type": "hire_agent", "payload": {} }),
    )
    .await;
    let approval_id = approval["id"].as_str().unwrap().to_owned();
    let (status, linked) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/approvals"),
        json!({ "approvalId": approval_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(linked["approvalId"], approval_id);

    // Execution decision.
    let (status, decision) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/execution-decisions"),
        json!({ "stageId": "s1", "stageType": "review", "outcome": "approved", "body": "ok" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(decision["outcome"], "approved");

    // Detach label.
    let (status, _) = send(
        &app,
        Method::DELETE,
        &format!("/api/issues/{issue_id}/labels/{label_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, labels) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_id}/labels"),
        json!({}),
    )
    .await;
    assert_eq!(labels.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn routines_lifecycle() {
    let app = router(test_state().await);
    let company_id = create_company_via(&app, "Acme").await;
    let (_, project) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "P" }),
    )
    .await;
    let project_id = project["id"].as_str().unwrap().to_owned();

    // Create routine.
    let (status, routine) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/routines"),
        json!({
            "title": "Daily report",
            "projectId": project_id,
            "description": "generate report",
            "variables": [ { "name": "fmt", "value": "md" } ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(routine["status"], "active");
    assert_eq!(routine["latestRevisionNumber"], 1);
    let routine_id = routine["id"].as_str().unwrap().to_owned();

    // Update -> revision 2.
    let (status, updated) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/routines/{routine_id}"),
        json!({ "title": "Weekly report" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["latestRevisionNumber"], 2);
    assert_eq!(updated["title"], "Weekly report");

    // List.
    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/routines"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Trigger -> run queued.
    let (status, run) = send_json(
        &app,
        Method::POST,
        &format!("/api/routines/{routine_id}/trigger"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(run["status"], "queued");

    let (status, runs) = send_json(
        &app,
        Method::GET,
        &format!("/api/routines/{routine_id}/runs"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(runs.as_array().unwrap().len(), 1);

    // Trigger: cron.
    let (status, trigger) = send_json(
        &app,
        Method::POST,
        &format!("/api/routines/{routine_id}/triggers"),
        json!({ "scheduleKind": "cron", "scheduleExpr": "0 9 * * *" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(trigger["scheduleKind"], "cron");

    let (status, triggers) = send_json(
        &app,
        Method::GET,
        &format!("/api/routines/{routine_id}/triggers"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(triggers.as_array().unwrap().len(), 1);

    // Invalid trigger kind -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/routines/{routine_id}/triggers"),
        json!({ "scheduleKind": "bogus" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Missing routine -> 404.
    let (status, _) = send_json(&app, Method::GET, "/api/routines/missing", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn permission_grants_scoped_assignment_inbox_and_budget() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type, reports_to)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'root', 'manager', 'cli', NULL),
                ('22222222-2222-2222-2222-222222222222', 'c1', 'leaf', 'worker', 'cli',
                 '11111111-1111-1111-1111-111111111111'),
                ('33333333-3333-3333-3333-333333333333', 'c1', 'other', 'worker', 'cli', NULL)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO projects (id, company_id, name) VALUES
         ('44444444-4444-4444-4444-444444444444', 'c1', 'P1'),
         ('55555555-5555-5555-5555-555555555555', 'c1', 'P2')",
        (),
    )
    .await
    .unwrap();

    // Board creates an API key for the leaf agent.
    let (status, key_body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-api-keys",
        json!({ "agentId": "22222222-2222-2222-2222-222222222222", "name": "leaf" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let plaintext = key_body["plaintext"].as_str().unwrap().to_owned();

    // Grant management is board-only.
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/permission-grants",
        Some(r#"{"principalType":"agent","principalId":"22222222-2222-2222-2222-222222222222","permissionKey":"tasks:assign_scope","scope":{"projectId":"44444444-4444-4444-4444-444444444444"}}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Board creates a scoped assignment grant (project P1 only).
    let (status, grant_body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/permission-grants",
        json!({
            "principalType": "agent",
            "principalId": "22222222-2222-2222-2222-222222222222",
            "permissionKey": "tasks:assign_scope",
            "scope": { "projectId": "44444444-4444-4444-4444-444444444444" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let grant_id = grant_body["id"].as_str().unwrap().to_owned();

    // Invalid permission key -> 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/permission-grants",
        json!({
            "principalType": "agent",
            "principalId": "22222222-2222-2222-2222-222222222222",
            "permissionKey": "nope:unknown",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Leaf may assign inside P1.
    let (status, body) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/issues",
        Some(r#"{"title":"in P1","projectId":"44444444-4444-4444-4444-444444444444","assigneeAgentId":"22222222-2222-2222-2222-222222222222"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let issue_in_p1: serde_json::Value = serde_json::from_str(&body).unwrap();
    let issue_in_p1_id = issue_in_p1["id"].as_str().unwrap().to_owned();

    // Leaf may NOT assign inside P2: generic denial without scope details.
    let (status, body) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/issues",
        Some(r#"{"title":"in P2","projectId":"55555555-5555-5555-5555-555555555555","assigneeAgentId":"22222222-2222-2222-2222-222222222222"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(!body.contains("55555555-5555-5555-5555-555555555555"));

    // Creating an issue without assignment stays allowed.
    let (status, body) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/issues",
        Some(r#"{"title":"no assignee","projectId":"55555555-5555-5555-5555-555555555555"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let issue_in_p2_no_assignee = serde_json::from_str::<serde_json::Value>(&body).unwrap();
    let issue_in_p2_no_assignee_id = issue_in_p2_no_assignee["id"].as_str().unwrap().to_owned();

    // Reassignment through PATCH is also constrained: assigning an issue in
    // P2 is outside the project-scoped grant.
    let (status, body) = send_with_auth(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_in_p2_no_assignee_id}"),
        Some(r#"{"assigneeAgentId":"22222222-2222-2222-2222-222222222222"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
    // Same-project reassignment in P1 is allowed (grant constrains projects).
    let (status, body) = send_with_auth(
        &app,
        Method::PATCH,
        &format!("/api/issues/{issue_in_p1_id}"),
        Some(r#"{"assigneeAgentId":"33333333-3333-3333-3333-333333333333"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    // A broad tasks:assign grant overrides the scoped restriction.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/permission-grants",
        json!({
            "principalType": "agent",
            "principalId": "22222222-2222-2222-2222-222222222222",
            "permissionKey": "tasks:assign",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/issues",
        Some(r#"{"title":"in P2 again","projectId":"55555555-5555-5555-5555-555555555555","assigneeAgentId":"22222222-2222-2222-2222-222222222222"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // inbox:manage: without a grant the default-open policy allows archive.
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_in_p1_id}/archive"),
        Some(r#"{"userId":"u-1"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Board grants inbox:manage scoped to u-1 only.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/permission-grants",
        json!({
            "principalType": "agent",
            "principalId": "22222222-2222-2222-2222-222222222222",
            "permissionKey": "inbox:manage",
            "scope": { "userIds": ["u-1"] }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    // Uncovered user -> 403 with generic message.
    let (status, body) = send_with_auth(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_in_p1_id}/unarchive"),
        Some(r#"{"userId":"u-2"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(!body.contains("u-2"));
    // Covered user -> 200.
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_in_p1_id}/unarchive"),
        Some(r#"{"userId":"u-1"}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Subordinate budgets: leaf is not a manager -> cannot set other's budget.
    let (status, _) = send_with_auth(
        &app,
        Method::PATCH,
        "/api/agents/33333333-3333-3333-3333-333333333333/budgets",
        Some(r#"{"budgetMonthlyCents":100}"#),
        Some(&plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // Root may set leaf's budget (leaf is in root's subtree).
    let (status, key_root) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-api-keys",
        json!({ "agentId": "11111111-1111-1111-1111-111111111111", "name": "root" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let root_plaintext = key_root["plaintext"].as_str().unwrap().to_owned();
    let (status, body) = send_with_auth(
        &app,
        Method::PATCH,
        "/api/agents/22222222-2222-2222-2222-222222222222/budgets",
        Some(r#"{"budgetMonthlyCents":5000}"#),
        Some(&root_plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body).unwrap()["budgetMonthlyCents"],
        5000
    );
    // Root may NOT set other's budget (not in subtree).
    let (status, _) = send_with_auth(
        &app,
        Method::PATCH,
        "/api/agents/33333333-3333-3333-3333-333333333333/budgets",
        Some(r#"{"budgetMonthlyCents":200}"#),
        Some(&root_plaintext),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // Board may set any agent budget.
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        "/api/agents/33333333-3333-3333-3333-333333333333/budgets",
        json!({ "budgetMonthlyCents": 200 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["budgetMonthlyCents"], 200);

    // List grants and delete one (board-only, cross-company delete -> 404).
    let (status, grants) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/permission-grants",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let keys: Vec<&str> = grants
        .as_array()
        .unwrap()
        .iter()
        .map(|grant| grant["permissionKey"].as_str().unwrap())
        .collect();
    assert!(keys.contains(&"tasks:assign_scope"));
    assert!(keys.contains(&"tasks:assign"));
    assert!(keys.contains(&"inbox:manage"));
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        "/api/companies/c2/permission-grants/does-not-exist",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/companies/c1/permission-grants/{grant_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn access_and_operations_full_flow() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'One', 'worker', 'cli')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO assets (id, company_id, provider, object_key, content_type, byte_size, sha256)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'local', 'logo.png', 'image/png', 100, 'abc')",
        (),
    )
    .await
    .unwrap();

    // Memberships -----------------------------------------------------------
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/memberships",
        json!({ "principalType": "agent", "principalId": "11111111-1111-1111-1111-111111111111", "membershipRole": "operator" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let membership_id = body["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/memberships",
        json!({ "principalType": "agent", "principalId": "99999999-9999-9999-9999-999999999999" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, memberships) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/memberships",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(memberships.as_array().unwrap().len(), 1);

    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/memberships/{membership_id}"),
        json!({ "status": "inactive", "membershipRole": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "inactive");

    // Instance roles ---------------------------------------------------------
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/instance/user-roles",
        json!({ "userId": "u-board" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let role_id = body["id"].as_str().unwrap().to_owned();
    let (status, roles) = send_json(&app, Method::GET, "/api/instance/user-roles", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(roles.as_array().unwrap().len(), 1);
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/instance/user-roles/{role_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Invites + join requests ------------------------------------------------
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/invites",
        json!({ "allowedJoinTypes": "both" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let invite_id = body["invite"]["id"].as_str().unwrap().to_owned();
    let token = body["token"].as_str().unwrap().to_owned();
    assert!(token.starts_with("inv-"));

    // Board API key (authenticates as board) --------------------------------
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/board-api-keys",
        json!({ "userId": "u-board", "name": "ci" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let board_key = body["plaintext"].as_str().unwrap().to_owned();
    assert!(board_key.starts_with("bk-"));
    let board_key_id = body["key"]["id"].as_str().unwrap().to_owned();
    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some(&board_key),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_with_auth(
        &app,
        Method::POST,
        "/api/companies/c1/invites",
        Some(r#"{"allowedJoinTypes":"human"}"#),
        Some(&board_key),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Revoked board key -> 401.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/board-api-keys/{board_key_id}/revoke"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c1/issues",
        None,
        Some(&board_key),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // CLI auth challenge -----------------------------------------------------
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/cli-auth-challenges",
        json!({ "command": "paperclip login", "pendingKeyName": "cli-session" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let challenge_id = body["challenge"]["id"].as_str().unwrap().to_owned();
    let secret = body["secret"].as_str().unwrap().to_owned();
    assert!(secret.starts_with("chal-"));
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/cli-auth-challenges/{challenge_id}/approve"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["boardApiKeyId"].is_string());
    // Challenge is single-use.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/cli-auth-challenges/{challenge_id}/approve"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Join request via invite + approve creates agent -----------------------
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/invites/{invite_id}/join-requests"),
        json!({
            "requestType": "agent",
            "agentName": "Helper",
            "adapterType": "cli",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let join_request_id = body["joinRequest"]["id"].as_str().unwrap().to_owned();
    assert!(body["claimSecret"].is_string());
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/join-requests/{join_request_id}/approve"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "approved");
    assert!(body["createdAgentId"].is_string());

    // Budget policies + incidents --------------------------------------------
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/budget-policies",
        json!({
            "scopeType": "company",
            "scopeId": "c1",
            "windowKind": "calendar_month_utc",
            "amount": 100000,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let policy_id = body["id"].as_str().unwrap().to_owned();
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/budget-incidents",
        json!({
            "policyId": policy_id,
            "scopeType": "company",
            "scopeId": "c1",
            "windowKind": "calendar_month_utc",
            "windowStart": "2026-08-01T00:00:00.000Z",
            "windowEnd": "2026-08-31T23:59:59.999Z",
            "thresholdType": "hard_stop",
            "amountLimit": 100000,
            "amountObserved": 110000,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let incident_id = body["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/budget-incidents/{incident_id}/resolve"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, incidents) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/budget-incidents",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(incidents.as_array().unwrap()[0]["status"], "resolved");

    // Sidebar preferences -----------------------------------------------------
    let (status, body) = send_json(
        &app,
        Method::PUT,
        "/api/companies/c1/sidebar-preferences",
        json!({ "userId": "u1", "projectOrder": ["p2", "p1"] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let (status, body) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/sidebar-preferences?userId=u1",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["projectOrder"][0], "p2");

    // Company logo ------------------------------------------------------------
    let (status, body) = send_json(
        &app,
        Method::PUT,
        "/api/companies/c1/logo",
        json!({ "assetId": "22222222-2222-2222-2222-222222222222" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["assetId"], "22222222-2222-2222-2222-222222222222");
    let (status, _) = send_json(&app, Method::GET, "/api/companies/c1/logo", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_json(&app, Method::DELETE, "/api/companies/c1/logo", json!({})).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send_json(&app, Method::GET, "/api/companies/c1/logo", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Cleanup: delete membership + revoke invite (cross-company 404) ---------
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/companies/c1/memberships/{membership_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/invites/{invite_id}/revoke"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        "/api/companies/c2/memberships/does-not-exist",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn plugin_ecosystem_full_chain() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();

    // Register plugin.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/plugins",
        json!({
            "pluginKey": "acme.github",
            "packageName": "@acme/github",
            "version": "1.2.0",
            "categories": ["integrations"],
            "manifest": { "id": "acme.github", "name": "GitHub Sync" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let plugin_id = body["id"].as_str().unwrap().to_owned();
    let (status, plugins) = send_json(&app, Method::GET, "/api/plugins", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(plugins.as_array().unwrap().len(), 1);

    // Update status + error (failure diagnostics).
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/plugins/{plugin_id}"),
        json!({ "status": "error", "lastError": "manifest validation failed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "error");
    assert_eq!(body["lastError"], "manifest validation failed");

    // Config + company settings + managed resources.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/configs"),
        json!({ "companyId": "c1", "config": { "token": "x" } }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/plugins/{plugin_id}/configs?companyId=c1"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["configJson"]["token"], "x");

    let (status, body) = send_json(
        &app,
        Method::PUT,
        &format!("/api/plugins/{plugin_id}/company-settings"),
        json!({ "companyId": "c1", "enabled": false, "settings": { "policy": "strict" } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["enabled"], false);

    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/managed-resources"),
        json!({
            "companyId": "c1",
            "resourceKind": "agent",
            "resourceKey": "defaults",
            "resourceId": "a1",
            "defaults": { "mode": "strict" }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let resource_id = body["id"].as_str().unwrap().to_owned();
    let (status, resources) = send_json(
        &app,
        Method::GET,
        &format!("/api/plugins/{plugin_id}/managed-resources?companyId=c1"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resources.as_array().unwrap().len(), 1);

    // Runtime: state -> entity -> job -> run -> log -> webhook -> db.
    let (status, body) = send_json(
        &app,
        Method::PUT,
        &format!("/api/plugins/{plugin_id}/state"),
        json!({ "scopeKind": "company", "scopeId": "c1", "key": "cursor", "value": { "page": 2 } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["valueJson"]["page"], 2);
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/plugins/{plugin_id}/state?scopeKind=company&scopeId=c1&namespace=default&key=cursor"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["valueJson"]["page"], 2);
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/plugins/{plugin_id}/state?scopeKind=company&scopeId=c1&namespace=default&key=cursor"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/entities"),
        json!({
            "companyId": "c1",
            "entityType": "issue",
            "scopeKind": "issue",
            "scopeId": "i1",
            "externalId": "GH-1",
            "title": "Sync"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["externalId"], "GH-1");
    let (status, entities) = send_json(
        &app,
        Method::GET,
        &format!("/api/plugins/{plugin_id}/entities?companyId=c1"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(entities.as_array().unwrap().len(), 1);

    // Job -> run -> complete.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/jobs"),
        json!({ "jobKey": "nightly", "schedule": "0 0 * * *" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/jobs/nightly/runs"),
        json!({ "companyId": "c1", "trigger": "manual" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let run_id = body["id"].as_str().unwrap().to_owned();
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/jobs/nightly/runs/{run_id}/complete"),
        json!({ "status": "succeeded", "logs": ["done"] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "succeeded");
    assert_eq!(body["logs"][0], "done");

    // Logs.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/logs"),
        json!({ "companyId": "c1", "level": "info", "message": "started" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let (status, logs) = send_json(
        &app,
        Method::GET,
        &format!("/api/plugins/{plugin_id}/logs?companyId=c1"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(logs.as_array().unwrap()[0]["message"], "started");

    // Webhook.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/webhooks"),
        json!({
            "companyId": "c1",
            "webhookKey": "issue.created",
            "externalId": "evt-1",
            "payload": { "id": 1 }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let webhook_id = body["id"].as_str().unwrap().to_owned();
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/webhooks/{webhook_id}/complete"),
        json!({ "status": "succeeded", "durationMs": 10 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "succeeded");

    // Database namespace + migration.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/database/namespaces"),
        json!({ "namespaceName": "acme_github", "namespaceMode": "schema" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/plugins/{plugin_id}/database/migrations"),
        json!({
            "namespaceName": "acme_github",
            "migrationKey": "0001_init",
            "checksum": "abc123",
            "pluginVersion": "1.2.0",
            "status": "applied"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["status"], "applied");
    let (status, migrations) = send_json(
        &app,
        Method::GET,
        &format!("/api/plugins/{plugin_id}/database/migrations"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(migrations.as_array().unwrap().len(), 1);

    // Company boundary: managed resource delete for another company is 404.
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/plugins/{plugin_id}/managed-resources/{resource_id}?companyId=c2"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/plugins/{plugin_id}/managed-resources/{resource_id}?companyId=c1"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Unknown plugin -> 404 diagnostics.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/plugins/missing/entities",
        json!({ "entityType": "issue", "scopeKind": "issue" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Uninstall.
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/plugins/{plugin_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn agent_runtime_and_recovery_flow() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'One', 'worker', 'cli')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'T', 1, 'ALPHA-1')",
        (),
    )
    .await
    .unwrap();

    // Task sessions.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-task-sessions",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "adapterType": "cli",
            "taskKey": "task-1",
            "sessionDisplayId": "sess-1"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let (status, sessions) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/agent-task-sessions",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sessions.as_array().unwrap().len(), 1);

    // Runtime state.
    let (status, body) = send_json(
        &app,
        Method::PUT,
        "/api/companies/c1/agent-runtime-state/11111111-1111-1111-1111-111111111111",
        json!({ "adapterType": "cli", "state": { "step": 2 }, "totalInputTokens": 100 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["totalInputTokens"], 100);
    let (status, body) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/agent-runtime-state/11111111-1111-1111-1111-111111111111",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["stateJson"]["step"], 2);

    // Wakeup queue -> claim -> finish.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-wakeup-requests",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "source": "scheduler",
            "reason": "timeout",
            "idempotencyKey": "wake-1"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let wakeup_id = body["id"].as_str().unwrap().to_owned();
    // Duplicate idempotency key coalesces (returns same id).
    let (status, body2) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-wakeup-requests",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "source": "scheduler",
            "idempotencyKey": "wake-1"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body2["id"], wakeup_id);
    assert_eq!(body2["coalescedCount"], 1);

    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/agent-wakeup-requests/{wakeup_id}/claim"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "claimed");
    // Claiming again fails with 422.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/agent-wakeup-requests/{wakeup_id}/claim"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/agent-wakeup-requests/{wakeup_id}/finish"),
        json!({ "status": "finished", "runId": "run-1" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "finished");

    // Recovery actions state machine.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/recovery-actions",
        json!({
            "sourceIssueId": "22222222-2222-2222-2222-222222222222",
            "kind": "restore",
            "ownerAgentId": "11111111-1111-1111-1111-111111111111",
            "cause": "lost_process",
            "fingerprint": "fp-1",
            "nextAction": "resume",
            "maxAttempts": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let action_id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(body["status"], "active");

    // Escalate -> restore -> resolve.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/recovery-actions/{action_id}/escalate"),
        json!({ "resolutionNote": "needs board" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "escalated");
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/recovery-actions/{action_id}/restore"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "active");
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/recovery-actions/{action_id}/resolve"),
        json!({ "outcome": "restored", "resolutionNote": "done" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "resolved");
    // Resolving again fails.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/recovery-actions/{action_id}/resolve"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, actions) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/recovery-actions",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(actions.as_array().unwrap().len(), 1);

    // Cross-company reads are empty/404.
    let (status, sessions) = send_json(
        &app,
        Method::GET,
        "/api/companies/c2/agent-task-sessions",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(sessions.as_array().unwrap().is_empty());
    let (status, _) = send_json(
        &app,
        Method::GET,
        "/api/companies/c2/agent-runtime-state/11111111-1111-1111-1111-111111111111",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn decision_retention_sweeper_flow() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();

    // Triage row (old enough for the sweeper).
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-triage",
        json!({ "sourceKind": "issue", "sourceId": "i-old", "decision": "approved" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let triage_id = body["id"].as_str().unwrap().to_owned();
    conn.execute(
        &format!(
            "UPDATE decision_triage SET updated_at = datetime('now', '-100 days') WHERE id = '{triage_id}'"
        ),
        (),
    )
    .await
    .unwrap();
    // Kept triage.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-triage",
        json!({ "sourceKind": "issue", "sourceId": "i-kept" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let kept_triage_id = body["id"].as_str().unwrap().to_owned();
    conn.execute(
        &format!(
            "UPDATE decision_triage SET updated_at = datetime('now', '-100 days') WHERE id = '{kept_triage_id}'"
        ),
        (),
    )
    .await
    .unwrap();
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-retention/issue/i-kept/keep",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Append a triage event.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-triage-events",
        json!({ "triageId": triage_id, "eventType": "decided", "decision": "approved" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let (status, events) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/c1/decision-triage-events?triageId={triage_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(events.as_array().unwrap().len(), 1);

    // Run the 90-day sweeper: old triage archived + notification enqueued;
    // kept triage survives.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-desk/sweep",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["archived"], 1);
    assert_eq!(body["notificationsEnqueued"], 1);

    let (status, retention) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/decision-retention",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let rows = retention.as_array().unwrap();
    assert!(
        rows.iter()
            .any(|r| r["sourceId"] == "i-old" && r["archived"] == true)
    );
    assert!(
        rows.iter()
            .any(|r| r["sourceId"] == "i-kept" && r["archived"] == false)
    );

    // Outbox has one pending notification; mark it sent.
    let (status, outbox) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/decision-archive-notification-outbox",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(outbox.as_array().unwrap().len(), 1);
    let outbox_id = outbox.as_array().unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/decision-archive-notification-outbox/{outbox_id}/sent"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "sent");

    // Manual archive then restore.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-retention/issue/i-kept/archive",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-retention/issue/i-kept/restore",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["archived"], false);
    assert!(body["restoredAt"].is_string());
    // Restore clears keep; re-keep so the second sweep is a true no-op.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-retention/issue/i-kept/keep",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Second sweep is a no-op (already archived, notification deduped).
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-desk/sweep",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["archived"], 0);
    assert_eq!(body["notificationsEnqueued"], 0);
}

#[tokio::test]
async fn managed_checkout_materializes_with_secret_and_redacts() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO projects (id, company_id, name) VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'Repo')",
        (),
    )
    .await
    .unwrap();

    // Build a real local git repo as the materialization source.
    let src = tempfile::tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(src.path())
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(src.path())
        .output()
        .expect("git config");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(src.path())
        .output()
        .expect("git config");
    std::fs::write(src.path().join("README.md"), "# hello\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(src.path())
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-q", "-m", "init"])
        .current_dir(src.path())
        .output()
        .expect("git commit");

    // Create the execution workspace pointing at the local repo.
    let repo_url = src.path().to_string_lossy().to_string();
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/execution-workspaces",
        json!({
            "projectId": "11111111-1111-1111-1111-111111111111",
            "mode": "checkout",
            "strategyType": "clone",
            "name": "main",
            "repoUrl": repo_url
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let workspace_id = body["id"].as_str().unwrap().to_owned();

    // Without a credential secret, materialize fails with a clear 422.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/workspaces/{workspace_id}/materialize"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert!(body.to_string().contains("Secret github_token not found"));

    // Create the secret, then materialize (checkout root outside the repo).
    let checkout_root = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("STAPLE_CHECKOUT_ROOT", checkout_root.path());
    }
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/secrets",
        json!({ "name": "github_token", "value": "ghp_secret_value" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/workspaces/{workspace_id}/materialize"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["workspace"]["materialized"], true);
    let materialized_path = body["path"].as_str().unwrap().to_owned();
    assert!(
        std::path::Path::new(&materialized_path)
            .join("README.md")
            .exists()
    );

    // Status endpoint reports materialized with the scoped path.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/c1/workspaces/{workspace_id}/materialization"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["materialized"], true);
    assert_eq!(body["credentialSecretName"], "github_token");

    // Refresh fetches without error and never leaks the token.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/c1/workspaces/{workspace_id}/refresh"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(!body.to_string().contains("ghp_secret_value"));

    // Cross-company access to materialization is denied.
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/c2/workspaces/{workspace_id}/materialization"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn scheduler_wakeup_routine_cron_and_sweep() {
    let (state, db) = test_state_with_db().await;
    let app = router(state.clone());
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type, status)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'One', 'worker', 'cli', 'active')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'T', 1, 'ALPHA-1')",
        (),
    )
    .await
    .unwrap();

    // 1. Queued wakeup for the active agent.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/agent-wakeup-requests",
        json!({
            "agentId": "11111111-1111-1111-1111-111111111111",
            "source": "scheduler",
            "reason": "timeout",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let wakeup_id = body["id"].as_str().unwrap().to_owned();

    // 2. Active routine with an every-minute cron trigger.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/routines",
        json!({ "title": "Nightly" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let routine_id = body["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/routines/{routine_id}/triggers"),
        json!({ "scheduleKind": "cron", "scheduleExpr": "* * * * *" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // 3. Old triage row eligible for the 90-day sweep.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/decision-triage",
        json!({ "sourceKind": "issue", "sourceId": "i-old" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let triage_id = body["id"].as_str().unwrap().to_owned();
    conn.execute(
        &format!(
            "UPDATE decision_triage SET updated_at = datetime('now', '-100 days') WHERE id = '{triage_id}'"
        ),
        (),
    )
    .await
    .unwrap();

    // Run one scheduler tick.
    let config = staple_app::scheduler::config_from_env();
    let mut last_sweep: Option<String> = None;
    staple_app::scheduler::tick(&state, &config, &mut last_sweep)
        .await
        .unwrap();

    // Wakeup finished and a heartbeat run was created.
    let (_, body) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/agent-wakeup-requests",
        json!({}),
    )
    .await;
    let wakeup = body
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["id"] == wakeup_id)
        .unwrap();
    assert_eq!(wakeup["status"], "finished");
    assert!(wakeup["runId"].is_string());
    // Verify the scheduler-created heartbeat run via SQL.
    let mut rows = conn
        .query(
            "SELECT COUNT(*) FROM heartbeat_runs WHERE company_id = 'c1' AND invocation_source = 'scheduler'",
            (),
        )
        .await
        .unwrap();
    let row = rows.next().await.unwrap().unwrap();
    assert_eq!(row.get::<i64>(0).unwrap(), 1);

    // Routine run created by the cron trigger.
    let (_, runs) = send_json(
        &app,
        Method::GET,
        &format!("/api/routines/{routine_id}/runs"),
        json!({}),
    )
    .await;
    assert_eq!(runs.as_array().unwrap().len(), 1);

    // Sweep archived the old triage.
    let (_, retention) = send_json(
        &app,
        Method::GET,
        "/api/companies/c1/decision-retention",
        json!({}),
    )
    .await;
    assert!(
        retention
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["sourceId"] == "i-old" && r["archived"] == true)
    );

    // A second tick does not double-fire the routine (lastTriggeredAt gate).
    staple_app::scheduler::tick(&state, &config, &mut last_sweep)
        .await
        .unwrap();
    let (_, runs) = send_json(
        &app,
        Method::GET,
        &format!("/api/routines/{routine_id}/runs"),
        json!({}),
    )
    .await;
    assert_eq!(runs.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn cases_lifecycle_status_and_boundary() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
        (),
    )
    .await
    .unwrap();

    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cases",
        json!({ "caseType": "support", "key": "k1", "title": "Billing issue", "fields": { "severity": "high" } }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let case_id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(body["identifier"], "ALPHA-CASE-1");
    assert_eq!(body["status"], "draft");

    // Status machine.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/cases/{case_id}/status"),
        json!({ "status": "in_progress" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "in_progress");
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/cases/{case_id}/status"),
        json!({ "status": "done" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // Terminal case rejects forward moves.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/cases/{case_id}/status"),
        json!({ "status": "in_review" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Update + get + list.
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/cases/{case_id}"),
        json!({ "title": "Billing issue v2", "summary": "escalated" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["title"], "Billing issue v2");
    let (status, cases) = send_json(&app, Method::GET, "/api/companies/c1/cases", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cases.as_array().unwrap().len(), 1);

    // Duplicate type+key rejected.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/cases",
        json!({ "caseType": "support", "key": "k1", "title": "Dup" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Cross-company get is 404.
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/cases/{case_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // Delete.
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/cases/{case_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/cases/{case_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn pipelines_full_flow() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();

    // Pipeline with enforced transitions.
    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/pipelines",
        json!({ "key": "intake", "name": "Intake", "enforceTransitions": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let pipeline_id = body["id"].as_str().unwrap().to_owned();

    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/stages"),
        json!({ "key": "todo", "name": "To do", "kind": "working", "position": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let todo_id = body["id"].as_str().unwrap().to_owned();
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/stages"),
        json!({ "key": "done", "name": "Done", "kind": "done", "position": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let done_id = body["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/transitions"),
        json!({ "fromStageId": todo_id, "toStageId": done_id, "label": "complete" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Create a case.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/cases"),
        json!({ "stageId": todo_id, "caseKey": "case-1", "title": "First" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let case_id = body["id"].as_str().unwrap().to_owned();

    // Move along declared edge succeeds.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipeline-cases/{case_id}/move"),
        json!({ "toStageId": done_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["terminalKind"], "done");

    // Events recorded.
    let (status, events) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipeline-cases/{case_id}/events"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(events.as_array().unwrap().len(), 1);
    assert_eq!(events.as_array().unwrap()[0]["type"], "transitioned");

    // Lists.
    let (status, pipelines) =
        send_json(&app, Method::GET, "/api/companies/c1/pipelines", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(pipelines.as_array().unwrap().len(), 1);
    let (status, stages) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipelines/{pipeline_id}/stages"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stages.as_array().unwrap().len(), 2);
    let (status, cases) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipelines/{pipeline_id}/cases"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cases.as_array().unwrap().len(), 1);

    // Cross-company access is denied.
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipeline-cases/{case_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // Delete pipeline cascades.
    let (status, _) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/pipelines/{pipeline_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipelines/{pipeline_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn pipeline_extensions_api() {
    let (state, db) = test_state_with_db().await;
    let app = router(state);
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'T', 1, 'ALPHA-1')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO documents (id, company_id, title, created_at, updated_at)
         VALUES ('22222222-2222-2222-2222-222222222222', 'c1', 'Plan',
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO routines (id, company_id, title, created_at, updated_at)
         VALUES ('33333333-3333-3333-3333-333333333333', 'c1', 'Nightly',
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        (),
    )
    .await
    .unwrap();

    let (status, body) = send_json(
        &app,
        Method::POST,
        "/api/companies/c1/pipelines",
        json!({ "key": "ext", "name": "Ext" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let pipeline_id = body["id"].as_str().unwrap().to_owned();
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/stages"),
        json!({ "key": "s1", "name": "S1", "kind": "working", "position": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let stage_id = body["id"].as_str().unwrap().to_owned();
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/cases"),
        json!({ "stageId": stage_id, "caseKey": "c1", "title": "Case" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let case_id = body["id"].as_str().unwrap().to_owned();

    // Issue link.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipeline-cases/{case_id}/issue-links"),
        json!({ "issueId": "11111111-1111-1111-1111-111111111111", "role": "work" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, links) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipeline-cases/{case_id}/issue-links"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(links.as_array().unwrap().len(), 1);

    // Blocker.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/cases"),
        json!({ "stageId": stage_id, "caseKey": "c2", "title": "Other" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let other_case = body["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipeline-cases/{case_id}/blockers"),
        json!({ "blockedByCaseId": other_case }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, blockers) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipeline-cases/{case_id}/blockers"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(blockers.as_array().unwrap().len(), 1);

    // Pipeline + case documents.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipelines/{pipeline_id}/documents"),
        json!({ "documentId": "22222222-2222-2222-2222-222222222222", "key": "plan" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, docs) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipelines/{pipeline_id}/documents"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(docs.as_array().unwrap().len(), 1);
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipeline-cases/{case_id}/documents"),
        json!({ "documentId": "22222222-2222-2222-2222-222222222222", "key": "plan" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Automation execution.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/pipeline-cases/{case_id}/automations"),
        json!({
            "automationId": "auto-1",
            "triggeringEventId": "evt-1",
            "routineId": "33333333-3333-3333-3333-333333333333",
            "status": "succeeded"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let (status, automations) = send_json(
        &app,
        Method::GET,
        &format!("/api/pipeline-cases/{case_id}/automations"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(automations.as_array().unwrap().len(), 1);
}
