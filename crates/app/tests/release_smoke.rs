//! Release smoke test: the core business flow end-to-end through the single
//! Rust binary's HTTP surface (mirrors the upstream release-smoke idea).

use std::sync::Arc;

use http::{Method, Request, header::CONTENT_TYPE};
use serde_json::{Value, json};
use staple_adapters::{AdapterRegistry, CliAdapter, CliAdapterConfig};
use staple_app::storage::LocalStorage;
use staple_app::{router, state::AppState};
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
    TursoScatteredRepository, TursoSecretBindingRepository, TursoSecretRepository,
    TursoSkillCatalogRepository, TursoSkillRepository, TursoToolCatalogRepository,
    TursoToolConnectionRepository, TursoToolGatewayRepository, TursoWorkProductRepository,
    TursoWorkspaceRepository, migrate, open,
};
use topcoat::router::{Body, Router, StatusCode, to_bytes};

async fn send_json(
    router: &Router,
    method: Method,
    path: &str,
    body: Value,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Percent-encodes a form field value (spaces become `+`).
fn form_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(byte));
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// POSTs an `application/x-www-form-urlencoded` form and returns the response
/// status and body (UI form handlers redirect with `303 See Other`).
async fn send_form(router: &Router, path: &str, fields: &[(&str, &str)]) -> (StatusCode, String) {
    let body = fields
        .iter()
        .map(|(key, value)| format!("{}={}", form_encode(key), form_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let request = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

#[tokio::test]
async fn core_business_flow_smoke() {
    // Boot the full app state (single binary surface).
    let dir = tempfile::tempdir().unwrap();
    let seed_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let api_keys_db = open(&DbConfig::local(dir.path().join("test.db")))
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
    let cipher = SecretCipher::load_or_create(dir.path().join("key")).unwrap();
    migrate(&seed_db).await.unwrap();

    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(seed_db)),
        agents: Arc::new(TursoAgentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        agent_runtime: Arc::new(TursoAgentRuntimeRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        permission_grants: Arc::new(TursoPermissionGrantRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        memberships: Arc::new(TursoMembershipRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        invites: Arc::new(TursoInviteRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        infrastructure: Arc::new(TursoInfrastructureRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        board_keys: Arc::new(TursoBoardKeyRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        budget_policies: Arc::new(TursoBudgetPolicyRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        cases: Arc::new(TursoCaseRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        preferences: Arc::new(TursoPreferenceRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        pipelines: Arc::new(TursoPipelineRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        plugins: Arc::new(TursoPluginRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        plugin_runtime: Arc::new(TursoPluginRuntimeRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        goals: Arc::new(TursoGoalRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        projects: Arc::new(TursoProjectRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        issues: Arc::new(TursoIssueRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        comments: Arc::new(TursoIssueCommentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        documents: Arc::new(TursoDocumentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        assets: Arc::new(TursoAssetRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        relations: Arc::new(TursoIssueRelationRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        work_products: Arc::new(TursoWorkProductRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        heartbeat: Arc::new(TursoHeartbeatRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        costs: Arc::new(TursoCostRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        approvals: Arc::new(TursoApprovalRepository::new(approvals_db)),
        activity: Arc::new(TursoActivityRepository::new(activity_db)),
        secrets: Arc::new(TursoSecretRepository::new(secrets_db, cipher)),
        api_keys: Arc::new(TursoApiKeyRepository::new(api_keys_db)),
        decisions: Arc::new(TursoDecisionRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        decision_actions: Arc::new(TursoDecisionActionRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        external_objects: Arc::new(TursoExternalObjectRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        external_object_catalog: Arc::new(TursoExternalObjectCatalogRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        skill_catalog: Arc::new(TursoSkillCatalogRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        secret_bindings: Arc::new(TursoSecretBindingRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        skills: Arc::new(TursoSkillRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        environments: Arc::new(TursoEnvironmentRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        labels: Arc::new(TursoLabelRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        issue_structure: Arc::new(TursoIssueStructureRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        routines: Arc::new(TursoRoutineRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        tool_catalog: Arc::new(TursoToolCatalogRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        tool_connections: Arc::new(TursoToolConnectionRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        tool_gateway: Arc::new(TursoToolGatewayRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        scattered: Arc::new(TursoScatteredRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        workspaces: Arc::new(TursoWorkspaceRepository::new(
            open(&DbConfig::local(dir.path().join("test.db")))
                .await
                .unwrap(),
        )),
        adapters: Arc::new({
            let mut registry = AdapterRegistry::new();
            registry.register(Box::new(CliAdapter::new(CliAdapterConfig::default())));
            registry
        }),
        plugin_reports: Vec::new(),
        storage: LocalStorage::new(dir.path().join("uploads")),
    };
    std::mem::forget(dir);
    let app = router(state);

    // 1. Health.
    let (status, _) = send_json(&app, Method::GET, "/api/health", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    // 2. Company → goal → project → issue → comment.
    let (status, company) = send_json(
        &app,
        Method::POST,
        "/api/companies",
        json!({ "name": "Smoke Co", "budgetMonthlyCents": 1000 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let company_id = company["id"].as_str().unwrap().to_owned();

    let (status, goal) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/goals"),
        json!({ "title": "Ship" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let goal_id = goal["id"].as_str().unwrap().to_owned();

    let (status, project) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/projects"),
        json!({ "name": "Project X", "goalId": goal_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let project_id = project["id"].as_str().unwrap().to_owned();

    let (status, issue) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/issues"),
        json!({ "title": "Core task", "projectId": project_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let issue_id = issue["id"].as_str().unwrap().to_owned();
    assert_eq!(issue["identifier"], "SMO-1");

    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/comments"),
        json!({ "body": "smoke comment" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // 3. Governance: approval request + decision + audit.
    let (status, approval) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals"),
        json!({ "type": "hire_agent", "payload": {} }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let approval_id = approval["id"].as_str().unwrap().to_owned();
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/approvals/{approval_id}/decide"),
        json!({ "decision": "approved", "decidedByUserId": "board" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, activity) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(activity.as_array().unwrap().len() >= 6);

    // 3.5. Create a skill for the skill detail UI check.
    let (status, skill) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/skills"),
        json!({ "name": "Reviewer" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let skill_id = skill["id"].as_str().unwrap().to_owned();

    // 3.6. Routine + run for the routine detail UI check.
    let (status, routine) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/routines"),
        json!({ "title": "Smoke routine", "projectId": project_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let routine_id = routine["id"].as_str().unwrap().to_owned();
    let (status, run) = send_json(
        &app,
        Method::POST,
        &format!("/api/routines/{routine_id}/trigger"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(run["status"], "queued");

    // 3.7. Approval comment for the approval detail UI check.
    let (status, _) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/approvals/{approval_id}/comments"),
        json!({ "body": "smoke approval comment", "authorUserId": "board" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // 3.8. Workspaces for the workspace detail UI check.
    let (status, project_workspace) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/project-workspaces"),
        json!({ "projectId": project_id, "name": "Smoke project workspace" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {project_workspace}");
    let project_workspace_id = project_workspace["id"].as_str().unwrap().to_owned();
    let (status, execution_workspace) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/execution-workspaces"),
        json!({
            "projectId": project_id,
            "projectWorkspaceId": project_workspace_id,
            "mode": "checkout",
            "strategyType": "clone",
            "name": "Smoke execution workspace"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {execution_workspace}");
    let execution_workspace_id = execution_workspace["id"].as_str().unwrap().to_owned();

    // 3.9. User for the profile settings UI check.
    let (status, _) = send_json(
        &app,
        Method::POST,
        "/api/users",
        json!({ "id": "smoke-user", "name": "Smoke User", "email": "smoke@example.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // 3.10. Create a status card for the status-card updates UI check.
    let (status, card) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/status-cards"),
        json!({
            "interestPrompt": "what changed",
            "refreshPolicy": { "interval": "15m" },
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let card_id = card["id"].as_str().unwrap().to_owned();

    // 3.11. Pipeline + work product for the management page UI checks.
    let (status, pipeline) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/pipelines"),
        json!({ "key": "smoke", "name": "Smoke Pipeline" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "pipeline body: {pipeline}");
    let pipeline_id = pipeline["id"].as_str().unwrap().to_owned();

    let (status, work_product) = send_json(
        &app,
        Method::POST,
        &format!("/api/issues/{issue_id}/work-products"),
        json!({
            "type": "artifact",
            "provider": "paperclip",
            "title": "Smoke artifact"
        }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "work product body: {work_product}"
    );
    let work_product_id = work_product["id"].as_str().unwrap().to_owned();

    // 3.12. UI form handlers: create company, create agent, pipeline settings.
    let (status, _) = send_form(
        &app,
        "/companies/ui",
        &[
            ("name", "Smoke UI Co"),
            ("description", "created from the home page"),
            ("budgetMonthlyCents", "500"),
            ("attachmentMaxBytes", "1024"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);

    let (status, _) = send_form(
        &app,
        &format!("/companies/{company_id}/agents/ui"),
        &[
            ("name", "Smoke Agent"),
            ("role", "engineer"),
            ("title", "Smoke engineer"),
            ("adapter_type", "cli_local"),
            ("budgetMonthlyCents", "1000"),
            ("reports_to", ""),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);

    let (status, _) = send_form(
        &app,
        &format!("/pipelines/{pipeline_id}/settings/ui"),
        &[
            ("name", "Smoke Pipeline Renamed"),
            ("description", "updated via settings"),
            ("status", "archived"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);

    // 4. UI renders the company overview.
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/companies/{company_id}"))
        .body(Body::empty())
        .unwrap();
    let response = app.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("Core task"));

    // 4.1. Home page shows the company created through the UI form.
    let request = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let response = app.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(
        html.contains("Smoke UI Co"),
        "home page missing UI-created company"
    );

    // 5. New UI surfaces render: board, search, settings.
    for (path, needle) in [
        (format!("/companies/{company_id}/board"), "board-card"),
        (
            format!("/companies/{company_id}/search?q=Core"),
            "Core task",
        ),
        (format!("/companies/{company_id}/settings"), "settings"),
        (format!("/companies/{company_id}/agents"), "Agents"),
        (format!("/companies/{company_id}/agents"), "Smoke Agent"),
        (
            format!("/companies/{company_id}/artifacts"),
            "Smoke artifact",
        ),
        (
            format!("/companies/{company_id}/artifacts"),
            work_product_id.as_str(),
        ),
        (
            format!("/pipelines/{pipeline_id}"),
            "Smoke Pipeline Renamed",
        ),
        (format!("/pipelines/{pipeline_id}"), "updated via settings"),
        (format!("/companies/{company_id}/inbox"), "Inbox"),
        (format!("/companies/{company_id}/decision-desk"), "Decision"),
        (format!("/companies/{company_id}/access"), "Access"),
        (format!("/companies/{company_id}/costs"), "Costs"),
        (format!("/companies/{company_id}/routines"), "Routines"),
        (format!("/companies/{company_id}/goals"), "Ship"),
        (format!("/companies/{company_id}/projects"), "Project X"),
        (format!("/goals/{goal_id}"), "Ship"),
        (format!("/companies/{company_id}/decisions"), "Decisions"),
        (
            format!("/companies/{company_id}/decision-training-examples"),
            "Training examples",
        ),
        (
            format!("/companies/{company_id}/status-cards"),
            "Status cards",
        ),
        (
            format!("/companies/{company_id}/summary-slots"),
            "Summary slots",
        ),
        (
            format!("/companies/{company_id}/finance-events"),
            "Finance events",
        ),
        (
            format!("/companies/{company_id}/feedback-votes"),
            "Feedback",
        ),
        (format!("/companies/{company_id}/secrets"), "Secrets"),
        (format!("/companies/{company_id}/skills"), "Skills"),
        (
            format!("/companies/{company_id}/skills/{skill_id}"),
            "Reviewer",
        ),
        (
            format!("/companies/{company_id}/secret-bindings"),
            "Secret bindings",
        ),
        (
            format!("/companies/{company_id}/user-secrets"),
            "User secrets",
        ),
        (format!("/companies/{company_id}/folders"), "Folders"),
        (
            format!("/companies/{company_id}/tools/profiles"),
            "Tool profiles",
        ),
        (
            format!("/companies/{company_id}/tools/connections"),
            "Tool connections",
        ),
        (
            format!("/companies/{company_id}/tools/gateways"),
            "Tool gateways",
        ),
        (
            format!("/companies/{company_id}/tools/catalog"),
            "Tool catalog",
        ),
        (
            format!("/companies/{company_id}/tools/invocations"),
            "Tool invocations",
        ),
        (format!("/issues/{issue_id}/watchdogs"), "Issue watchdogs"),
        (format!("/companies/{company_id}/board/chat"), "Board chat"),
        (format!("/issues/{issue_id}"), "Claim"),
        ("/users".to_string(), "Users"),
        ("/environments".to_string(), "Environments"),
        ("/auth".to_string(), "Auth"),
        ("/cli-auth".to_string(), "CLI auth"),
        ("/invite/demo-token".to_string(), "Invitation"),
        (format!("/companies/{company_id}/my-issues"), "My issues"),
        (
            format!("/companies/{company_id}/what-needs-me"),
            "What needs me",
        ),
        (format!("/companies/{company_id}/timeline"), "Timeline"),
        (
            format!("/companies/{company_id}/status-cards/{card_id}/updates"),
            "Status card updates",
        ),
        (format!("/companies/{company_id}/smoke-runs"), "Smoke runs"),
        (
            format!("/companies/{company_id}/feedback-exports"),
            "Feedback exports",
        ),
        ("/instance/settings".to_string(), "Instance"),
        (format!("/companies/{company_id}/dashboard"), "Dashboard"),
        (format!("/projects/{project_id}"), project_id.as_str()),
        (format!("/companies/{company_id}/workspaces"), "Workspaces"),
        ("/adapters".to_string(), "Adapters"),
        (format!("/companies/{company_id}/org-chart"), "Org"),
        (format!("/companies/{company_id}/dashboard/live"), "Live"),
        ("/adapters/cli_local".to_string(), "Invoke"),
        (format!("/companies/{company_id}/cases"), "Cases"),
        (format!("/companies/{company_id}/pipelines"), "Pipelines"),
        (format!("/routines/{routine_id}"), "Run history"),
        (
            format!("/approvals/{approval_id}"),
            "smoke approval comment",
        ),
        (
            format!("/workspaces/{project_workspace_id}"),
            "Smoke project workspace",
        ),
        (
            format!("/workspaces/{execution_workspace_id}"),
            "Smoke execution workspace",
        ),
        ("/profile/settings".to_string(), "Smoke User"),
        ("/no-such-page".to_string(), "Page not found"),
    ] {
        let request = Request::builder()
            .method(Method::GET)
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let response = app.handle(request).await;
        assert_eq!(response.status(), StatusCode::OK);
        let (_, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains(needle), "page missing {needle}");
    }

    // Board concierge chat script is served.
    let request = Request::builder()
        .method(Method::GET)
        .uri("/static/board_chat.js")
        .body(Body::empty())
        .unwrap();
    let response = app.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let js = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(js.contains("chat-form"), "board_chat.js missing chat-form");

    // Board drag & drop script is served.
    let request = Request::builder()
        .method(Method::GET)
        .uri("/static/board.js")
        .body(Body::empty())
        .unwrap();
    let response = app.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let js = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(js.contains("draggable"));

    println!("smoke OK: single binary covered the core business flow");
}
