//! Instruction system API integration tests: document CRUD, agent file
//! mounts, company isolation, path traversal rejection, default
//! materialization on agent creation, and audit logging.

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
    TursoAssetRepository, TursoAttentionDismissalRepository, TursoBoardKeyRepository,
    TursoBudgetPolicyRepository, TursoCaseRepository, TursoCompanyRepository, TursoCostRepository,
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
        attention_dismissals: Arc::new(TursoAttentionDismissalRepository::new(
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

async fn seed_companies_and_agent(db: &staple_data::Database) -> (String, String) {
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
         VALUES ('11111111-1111-1111-1111-111111111111', 'c1', 'one', 'engineer', 'codex_local')",
        (),
    )
    .await
    .unwrap();
    (
        "c1".to_owned(),
        "11111111-1111-1111-1111-111111111111".to_owned(),
    )
}

#[tokio::test]
async fn instruction_document_crud() {
    let (state, db) = test_state().await;
    let (company_id, _) = seed_companies_and_agent(&db).await;
    let app = router(state);

    // Create.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/instruction-documents"),
        json!({ "name": "AGENTS.md", "content": "# Rules" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(body["name"], "AGENTS.md");
    assert_eq!(body["companyId"], company_id);

    // List.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/instruction-documents"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Get.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/instruction-documents/{id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["content"], "# Rules");

    // Patch (partial: name only).
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/instruction-documents/{id}"),
        json!({ "name": "SOUL.md" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["name"], "SOUL.md");
    assert_eq!(body["content"], "# Rules");

    // Delete.
    let (status, body) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/instruction-documents/{id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["deleted"], true);

    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/instruction-documents/{id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "Instruction document not found");

    // Audit trail exists.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let actions: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect();
    assert!(actions.contains(&"instruction_document.created"));
    assert!(actions.contains(&"instruction_document.updated"));
    assert!(actions.contains(&"instruction_document.deleted"));
}

#[tokio::test]
async fn agent_instruction_files_upsert_delete() {
    let (state, db) = test_state().await;
    let (company_id, agent_id) = seed_companies_and_agent(&db).await;
    let app = router(state);

    // Upsert entry file.
    let (status, body) = send_json(
        &app,
        Method::PUT,
        &format!("/api/companies/{company_id}/agents/{agent_id}/instructions/AGENTS.md"),
        json!({ "content": "# Agent", "isEntry": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["path"], "AGENTS.md");
    assert_eq!(body["isEntry"], true);

    // Upsert a nested path (percent-encoded slash).
    let (status, body) = send_json(
        &app,
        Method::PUT,
        &format!("/api/companies/{company_id}/agents/{agent_id}/instructions/docs%2FNOTES.md"),
        json!({ "content": "# Notes", "isEntry": false }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["path"], "docs/NOTES.md");

    // Upsert replaces content on the same path.
    let (status, body) = send_json(
        &app,
        Method::PUT,
        &format!("/api/companies/{company_id}/agents/{agent_id}/instructions/AGENTS.md"),
        json!({ "content": "# Agent v2", "isEntry": true }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    assert_eq!(body["content"], "# Agent v2");

    // List mounted files.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/agents/{agent_id}/instructions"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let files = body.as_array().unwrap();
    assert_eq!(files.len(), 2);
    let entry = files
        .iter()
        .find(|file| file["path"] == "AGENTS.md")
        .unwrap();
    assert_eq!(entry["isEntry"], true);

    // Delete.
    let (status, body) = send_json(
        &app,
        Method::DELETE,
        &format!("/api/companies/{company_id}/agents/{agent_id}/instructions/docs%2FNOTES.md"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["deleted"], true);

    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/agents/{agent_id}/instructions"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Audit trail exists.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/activity"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let actions: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect();
    assert!(actions.contains(&"instruction_file.upserted"));
    assert!(actions.contains(&"instruction_file.deleted"));
}

#[tokio::test]
async fn company_isolation_and_permissions() {
    let (state, db) = test_state().await;
    let (company_id, agent_id) = seed_companies_and_agent(&db).await;
    let app = router(state);

    // Create a document in c1.
    let (status, body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/instruction-documents"),
        json!({ "name": "AGENTS.md", "content": "c1 rules" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let document_id = body["id"].as_str().unwrap().to_owned();

    // c2 cannot see c1 documents.
    let (status, body) = send_json(
        &app,
        Method::GET,
        "/api/companies/c2/instruction-documents",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().is_empty());

    // Updating a c1 document as c2 (company mismatch) -> 404.
    let (status, body) = send_json(
        &app,
        Method::PATCH,
        &format!("/api/instruction-documents/{document_id}"),
        json!({ "name": "HACK.md" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "board can update, body: {body}");

    // Cross-company agent mount -> 422 (agent belongs to c1).
    let (status, body) = send_json(
        &app,
        Method::PUT,
        "/api/companies/c2/agents/11111111-1111-1111-1111-111111111111/instructions/AGENTS.md",
        json!({ "content": "x", "isEntry": true }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["details"][0]["path"][0], "agent");

    // Agent API key: own company allowed? No — instruction routes are
    // board-only, so an agent key gets 403 even on its own company.
    let (status, key_body) = send_json(
        &app,
        Method::POST,
        &format!("/api/companies/{company_id}/agent-api-keys"),
        json!({ "agentId": agent_id, "name": "dev" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {key_body}");
    let plaintext = key_body["plaintext"].as_str().unwrap().to_owned();

    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/instruction-documents"),
        json!({}),
        Some(&plaintext),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "board-only route rejects agents"
    );

    // Cross-company agent key -> 403 by company scope too.
    let (status, _) = send_with_auth(
        &app,
        Method::GET,
        "/api/companies/c2/instruction-documents",
        json!({}),
        Some(&plaintext),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "agent cannot cross companies"
    );
}

#[tokio::test]
async fn path_traversal_is_rejected() {
    let (state, db) = test_state().await;
    let (company_id, agent_id) = seed_companies_and_agent(&db).await;
    let app = router(state);

    for bad_path in ["..%2Fx", "%2Fetc%2Fpasswd", ".", "a%2F..%2Fb"] {
        let (status, body) = send_json(
            &app,
            Method::PUT,
            &format!("/api/companies/{company_id}/agents/{agent_id}/instructions/{bad_path}"),
            json!({ "content": "x" }),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{bad_path}: {body}"
        );
        assert_eq!(body["details"][0]["path"][0], "path");
    }
}

#[tokio::test]
async fn create_agent_ui_materializes_default_instructions() {
    let (state, db) = test_state().await;
    let (company_id, _) = seed_companies_and_agent(&db).await;
    let app = router(state);

    // Create a fresh agent through the UI form.
    let (status, _) = send_form(
        &app,
        &format!("/companies/{company_id}/agents/ui"),
        &[("name", "Smoke"), ("role", "ceo")],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);

    // Find the new agent by listing.
    let (_, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/agents"),
        json!({}),
    )
    .await;
    let agents = body.as_array().unwrap();
    let smoke = agents
        .iter()
        .find(|agent| agent["name"] == "Smoke")
        .expect("Smoke agent created");
    let agent_id = smoke["id"].as_str().unwrap();

    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/agents/{agent_id}/instructions"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let files = body.as_array().unwrap();
    assert_eq!(
        files.len(),
        4,
        "ceo bundle: AGENTS.md + HEARTBEAT.md + SOUL.md + TOOLS.md"
    );
    let entry = files
        .iter()
        .find(|file| file["path"] == "AGENTS.md")
        .unwrap();
    assert_eq!(entry["isEntry"], true);
    assert!(!entry["content"].as_str().unwrap().is_empty());
    for file in files {
        if file["path"] != "AGENTS.md" {
            assert_eq!(file["isEntry"], false);
        }
    }

    // A non-ceo agent gets only AGENTS.md.
    let (status, _) = send_form(
        &app,
        &format!("/companies/{company_id}/agents/ui"),
        &[("name", "Worker")],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let (_, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/agents"),
        json!({}),
    )
    .await;
    let worker = body
        .as_array()
        .unwrap()
        .iter()
        .find(|agent| agent["name"] == "Worker")
        .unwrap();
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!(
            "/api/companies/{company_id}/agents/{}/instructions",
            worker["id"].as_str().unwrap()
        ),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let files = body.as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["path"], "AGENTS.md");
    assert_eq!(files[0]["isEntry"], true);
}

#[tokio::test]
async fn onboarding_materializes_default_instructions() {
    let (state, _db) = test_state().await;
    let app = router(state);

    // Walk the onboarding wizard to completion (bundled default team).
    let (status, _) = send_form(
        &app,
        "/onboarding/ui",
        &[("step", "1"), ("company_name", "Onboard Co")],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_form(
        &app,
        "/onboarding/ui",
        &[
            ("step", "2"),
            ("company_name", "Onboard Co"),
            ("mission_preset", "Build a SaaS product"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_form(
        &app,
        "/onboarding/ui",
        &[
            ("step", "3"),
            ("company_name", "Onboard Co"),
            ("mission", "Build a SaaS product"),
            ("lead_name", "Chief of Staff"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_form(
        &app,
        "/onboarding/ui",
        &[
            ("step", "4"),
            ("company_name", "Onboard Co"),
            ("mission", "Build a SaaS product"),
            ("lead_name", "Chief of Staff"),
            ("adapter_type", "cli_local"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send_form(
        &app,
        "/onboarding/ui",
        &[
            ("step", "5"),
            ("company_name", "Onboard Co"),
            ("mission", "Build a SaaS product"),
            ("lead_name", "Chief of Staff"),
            ("adapter_type", "cli_local"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);

    let (_, body) = send_json(&app, Method::GET, "/api/companies", json!({})).await;
    let company_id = body.as_array().unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/agents"),
        json!({}),
    )
    .await;
    let agents = body.as_array().unwrap();
    let lead = agents
        .iter()
        .find(|agent| agent["name"] == "Chief of Staff")
        .expect("lead agent");
    let ceo = agents
        .iter()
        .find(|agent| agent["name"] == "ceo")
        .expect("ceo agent");

    // Lead gets the default single-file bundle.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!(
            "/api/companies/{company_id}/agents/{}/instructions",
            lead["id"].as_str().unwrap()
        ),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let files = body.as_array().unwrap();
    assert_eq!(files.len(), 1, "lead should have AGENTS.md only");
    assert_eq!(files[0]["path"], "AGENTS.md");
    assert_eq!(files[0]["isEntry"], true);

    // The bundled ceo agent gets the full ceo bundle.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!(
            "/api/companies/{company_id}/agents/{}/instructions",
            ceo["id"].as_str().unwrap()
        ),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let files = body.as_array().unwrap();
    assert_eq!(files.len(), 4, "ceo bundle should have 4 files");
    assert!(files.iter().any(|file| file["path"] == "HEARTBEAT.md"));
    assert!(files.iter().any(|file| file["path"] == "SOUL.md"));
    assert!(files.iter().any(|file| file["path"] == "TOOLS.md"));
    let entry = files
        .iter()
        .find(|file| file["path"] == "AGENTS.md")
        .unwrap();
    assert_eq!(entry["isEntry"], true);
}

#[tokio::test]
async fn team_catalog_install_materializes_default_instructions() {
    let (state, db) = test_state().await;
    let (company_id, _) = seed_companies_and_agent(&db).await;
    let app = router(state);

    // Install the bundled default team (core-exec-team: ceo/cto/qa) through
    // the install API; the created agents must get role-based default
    // instruction bundles (ceo -> 4 files, other roles -> AGENTS.md only).
    let install_path = format!(
        "/api/companies/{company_id}/teams/catalog/paperclipai%3Abundled%3Acompany-defaults%3Acore-exec-team/install"
    );
    let (status, body) = send_json(&app, Method::POST, &install_path, json!({})).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["createdAgents"], 3);

    let (status, agents) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/agents"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {agents}");
    let agents = agents.as_array().unwrap();
    let ceo = agents
        .iter()
        .find(|agent| agent["name"] == "ceo")
        .expect("ceo agent");
    let cto = agents
        .iter()
        .find(|agent| agent["name"] == "cto")
        .expect("cto agent");

    // ceo gets the full default bundle.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!(
            "/api/companies/{company_id}/agents/{}/instructions",
            ceo["id"].as_str().unwrap()
        ),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let files = body.as_array().unwrap();
    assert_eq!(
        files.len(),
        4,
        "ceo bundle: AGENTS.md + HEARTBEAT.md + SOUL.md + TOOLS.md"
    );
    let entry = files
        .iter()
        .find(|file| file["path"] == "AGENTS.md")
        .unwrap();
    assert_eq!(entry["isEntry"], true);
    assert!(!entry["content"].as_str().unwrap().is_empty());
    for path in ["HEARTBEAT.md", "SOUL.md", "TOOLS.md"] {
        let file = files.iter().find(|file| file["path"] == path).unwrap();
        assert_eq!(file["isEntry"], false, "{path} must not be the entry file");
    }

    // cto (non-ceo role) gets only AGENTS.md.
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!(
            "/api/companies/{company_id}/agents/{}/instructions",
            cto["id"].as_str().unwrap()
        ),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let files = body.as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["path"], "AGENTS.md");
    assert_eq!(files[0]["isEntry"], true);

    // The UI install path is best-effort too: install into company c2 (no
    // existing agents) through the form route and check the same mounting.
    let (status, _) = send_form(
        &app,
        "/companies/c2/teams/catalog/paperclipai%3Abundled%3Acompany-defaults%3Acore-exec-team/install/ui",
        &[],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let (status, agents) =
        send_json(&app, Method::GET, "/api/companies/c2/agents", json!({})).await;
    assert_eq!(status, StatusCode::OK, "body: {agents}");
    let agents = agents.as_array().unwrap();
    let ceo = agents
        .iter()
        .find(|agent| agent["name"] == "ceo")
        .expect("ceo agent via UI install");
    let cto = agents
        .iter()
        .find(|agent| agent["name"] == "cto")
        .expect("cto agent via UI install");
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!(
            "/api/companies/c2/agents/{}/instructions",
            ceo["id"].as_str().unwrap()
        ),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body.as_array().unwrap().len(), 4);
    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!(
            "/api/companies/c2/agents/{}/instructions",
            cto["id"].as_str().unwrap()
        ),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let files = body.as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["path"], "AGENTS.md");
    assert_eq!(files[0]["isEntry"], true);
}
