//! Issue thread interaction supersede integration tests: creating a new
//! `request_confirmation` from an agent expires older pending requests from
//! the same agent on the same issue (`superseded_by_newer_request`), the
//! scheduler sweep cleans up stragglers, and board actors are recorded.

use std::sync::Arc;
use std::time::Duration;

use http::{Method, Request, header::CONTENT_TYPE};
use serde_json::{Value, json};
use staple_adapters::{AdapterRegistry, CliAdapter, CliAdapterConfig};
use staple_app::router;
use staple_app::scheduler::{SchedulerConfig, tick};
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
    TursoInfrastructureRepository, TursoInstructionRepository, TursoInviteRepository,
    TursoIssueCommentRepository, TursoIssueRelationRepository, TursoIssueRepository,
    TursoIssueStructureRepository, TursoLabelRepository, TursoMembershipRepository,
    TursoPermissionGrantRepository, TursoPipelineRepository, TursoPluginRepository,
    TursoPluginRuntimeRepository, TursoPortabilityRepository, TursoPreferenceRepository,
    TursoProjectRepository, TursoRoutineRepository, TursoScatteredRepository,
    TursoSecretBindingRepository, TursoSecretRepository, TursoSkillCatalogRepository,
    TursoSkillRepository, TursoToolCatalogRepository, TursoToolConnectionRepository,
    TursoToolGatewayRepository, TursoWorkProductRepository, TursoWorkspaceRepository, migrate,
    open,
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

async fn send_with_auth(
    router: &Router,
    method: Method,
    path: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if !body.is_null() {
        builder = builder.header(CONTENT_TYPE, "application/json");
    }
    if let Some(token) = bearer {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request = builder.body(Body::from(body.to_string())).unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

const AGENT_ONE: &str = "11111111-1111-1111-1111-111111111111";
const AGENT_TWO: &str = "22222222-2222-2222-2222-222222222222";

/// Seeds two companies, two agents in `c1`, and two issues in `c1`.
async fn seed_company_agents_issues(db: &staple_data::Database) -> String {
    let conn = staple_data::connect(db).await.unwrap();
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
                ('22222222-2222-2222-2222-222222222222', 'c1', 'two', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, title, issue_number, identifier)
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'T1', 1, 'ALPHA-1'),
                ('22222222-2222-2222-2222-222222222222', 'c1', 'T2', 2, 'ALPHA-2')",
        (),
    )
    .await
    .unwrap();
    "c1".to_owned()
}

/// Creates an agent API key through the API and returns the plaintext.
async fn create_agent_key(app: &Router, company_id: &str, agent_id: &str) -> String {
    let (status, body) = send_json(
        app,
        Method::POST,
        &format!("/api/companies/{company_id}/agent-api-keys"),
        json!({ "agentId": agent_id, "name": "dev" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    body["plaintext"].as_str().unwrap().to_owned()
}

/// Creates a `request_confirmation` thread interaction as the given bearer.
async fn create_request_confirmation(
    app: &Router,
    path: &str,
    bearer: &str,
) -> (StatusCode, Value) {
    send_with_auth(
        app,
        Method::POST,
        path,
        json!({ "kind": "request_confirmation", "payload": { "target": { "type": "none" } } }),
        Some(bearer),
    )
    .await
}

async fn test_state() -> (AppState, staple_data::Database) {
    let dir = tempfile::tempdir().unwrap();
    let seed_db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let companies_db = open(&DbConfig::local(dir.path().join("test.db")))
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
        companies: Arc::new(TursoCompanyRepository::new(companies_db)),
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
        board_claim: Arc::new(staple_app::board_claim::BoardClaimManager::new()),
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
        instructions: Arc::new(TursoInstructionRepository::new(
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
        portability: Arc::new(TursoPortabilityRepository::new(
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
        storage: LocalStorage::new(dir.path().join("uploads")),
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
    };
    // Keep the temp dir alive for the lifetime of the test process.
    std::mem::forget(dir);
    (state, seed_db)
}

#[tokio::test]
async fn agent_request_confirmation_supersedes_older_pending() {
    let (state, db) = test_state().await;
    let company_id = seed_company_agents_issues(&db).await;
    let app = router(state);

    let key_one = create_agent_key(&app, &company_id, AGENT_ONE).await;
    let key_two = create_agent_key(&app, &company_id, AGENT_TWO).await;

    let issue_one = "11111111-1111-1111-1111-111111111111";
    let issue_two = "22222222-2222-2222-2222-222222222222";

    // First request from agent one: pending, no supersedes.
    let (status, first) = create_request_confirmation(
        &app,
        &format!("/api/issues/{issue_one}/thread-interactions"),
        &key_one,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {first}");
    assert_eq!(first["status"], "pending");
    assert_eq!(first["createdByAgentId"], AGENT_ONE);
    assert_eq!(first["superseded"], json!([]));

    // A request from a different agent must not supersede it.
    let (status, other_agent) = create_request_confirmation(
        &app,
        &format!("/api/issues/{issue_one}/thread-interactions"),
        &key_two,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {other_agent}");
    assert_eq!(other_agent["createdByAgentId"], AGENT_TWO);
    assert_eq!(other_agent["superseded"], json!([]));

    // The same agent on a different issue must not supersede it either.
    let (status, other_issue) = create_request_confirmation(
        &app,
        &format!("/api/issues/{issue_two}/thread-interactions"),
        &key_one,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {other_issue}");
    assert_eq!(other_issue["superseded"], json!([]));

    // A newer request from the same agent + issue supersedes the first one.
    let (status, replacement) = create_request_confirmation(
        &app,
        &format!("/api/issues/{issue_one}/thread-interactions"),
        &key_one,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {replacement}");
    assert_eq!(replacement["status"], "pending");
    assert_eq!(replacement["createdByAgentId"], AGENT_ONE);
    let superseded = replacement["superseded"].as_array().unwrap();
    assert_eq!(superseded.len(), 1);
    assert_eq!(superseded[0]["id"], first["id"]);
    assert_eq!(superseded[0]["status"], "expired");
    assert_eq!(superseded[0]["result"]["version"], 1);
    assert_eq!(
        superseded[0]["result"]["outcome"],
        "superseded_by_newer_request"
    );
    assert_eq!(
        superseded[0]["result"]["supersededByInteractionId"],
        replacement["id"]
    );
    assert!(superseded[0]["resolvedByAgentId"].as_str().is_some());
    assert!(superseded[0]["resolvedAt"].as_str().is_some());

    // Read-back: only the first request expired; the others stay pending.
    let (status, list) = send_with_auth(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_one}/thread-interactions"),
        json!({}),
        Some(&key_one),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {list}");
    let by_id = |id: &str| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == id)
            .unwrap()
    };
    assert_eq!(by_id(first["id"].as_str().unwrap())["status"], "expired");
    assert_eq!(
        by_id(first["id"].as_str().unwrap())["result"]["outcome"],
        "superseded_by_newer_request"
    );
    assert_eq!(
        by_id(first["id"].as_str().unwrap())["result"]["supersededByInteractionId"],
        replacement["id"]
    );
    assert_eq!(
        by_id(other_agent["id"].as_str().unwrap())["status"],
        "pending"
    );
    assert_eq!(
        by_id(replacement["id"].as_str().unwrap())["status"],
        "pending"
    );
}

#[tokio::test]
async fn board_actor_is_recorded_on_thread_interaction() {
    let (state, db) = test_state().await;
    let _company_id = seed_company_agents_issues(&db).await;
    let app = router(state);
    let issue_one = "11111111-1111-1111-1111-111111111111";

    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/issues/{issue_one}/thread-interactions"))
        .header(CONTENT_TYPE, "application/json")
        .header("X-Board-User", "u-board-1")
        .body(Body::from(
            json!({ "kind": "request_confirmation", "payload": {} }).to_string(),
        ))
        .unwrap();
    let response = app.handle(request).await;
    let status = response.status();
    let (_, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["createdByUserId"], "u-board-1");
    assert!(body["createdByAgentId"].is_null());
    assert_eq!(body["superseded"], json!([]));
}

#[tokio::test]
async fn scheduler_sweep_expires_stale_pending_confirmations() {
    let (state, db) = test_state().await;
    let _company_id = seed_company_agents_issues(&db).await;
    let app = router(state.clone());
    let issue_one = "11111111-1111-1111-1111-111111111111";
    let issue_two = "22222222-2222-2222-2222-222222222222";

    // Seed duplicates directly (bypassing the create-time supersede) to
    // simulate legacy/racing stragglers the sweep must clean up.
    let conn = staple_data::connect(&db).await.unwrap();
    conn.execute(
        "INSERT INTO issue_thread_interactions
            (id, company_id, issue_id, kind, status, payload, created_by_agent_id,
             created_at, updated_at)
         VALUES ('old-1', 'c1', '11111111-1111-1111-1111-111111111111',
                 'request_confirmation', 'pending', '{}', '11111111-1111-1111-1111-111111111111',
                 '2026-08-01T00:00:00.000Z', '2026-08-01T00:00:00.000Z')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issue_thread_interactions
            (id, company_id, issue_id, kind, status, payload, created_by_agent_id,
             created_at, updated_at)
         VALUES ('new-1', 'c1', '11111111-1111-1111-1111-111111111111',
                 'request_confirmation', 'pending', '{}', '11111111-1111-1111-1111-111111111111',
                 '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issue_thread_interactions
            (id, company_id, issue_id, kind, status, payload, created_by_agent_id,
             created_at, updated_at)
         VALUES ('other-issue', 'c1', '22222222-2222-2222-2222-222222222222',
                 'request_confirmation', 'pending', '{}', '11111111-1111-1111-1111-111111111111',
                 '2026-08-01T00:00:00.000Z', '2026-08-01T00:00:00.000Z')",
        (),
    )
    .await
    .unwrap();

    // One scheduler tick runs the daily sweep (decision retention +
    // confirmation supersede).
    let config = SchedulerConfig {
        tick: Duration::from_secs(60),
        wakeup_batch: 10,
        sweep_interval_days: 1,
    };
    tick(&state, &config, &mut None).await.unwrap();

    let (status, list) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_one}/thread-interactions"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {list}");
    let by_id = |id: &str| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == id)
            .unwrap()
    };
    assert_eq!(by_id("old-1")["status"], "expired");
    assert_eq!(
        by_id("old-1")["result"]["supersededByInteractionId"],
        "new-1"
    );
    assert_eq!(by_id("new-1")["status"], "pending");
    let (_, other) = send_json(
        &app,
        Method::GET,
        &format!("/api/issues/{issue_two}/thread-interactions"),
        json!({}),
    )
    .await;
    assert_eq!(other.as_array().unwrap()[0]["status"], "pending");
}
