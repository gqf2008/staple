//! Issue-based attention feed API integration tests (A1 core).

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

async fn seed_attention_fixture(
    state: &AppState,
    db: &staple_data::Database,
) -> (String, String, String, String) {
    let company = state
        .companies
        .create(staple_data::NewCompany {
            name: "Attention Co".to_owned(),
            description: Some("attention".to_owned()),
            budget_monthly_cents: 100,
            attachment_max_bytes: 1024,
        })
        .await
        .unwrap();
    // Exhaust the monthly budget (100 cents -> 200 cents spent).
    let conn = staple_data::connect(db).await.unwrap();
    conn.execute(
        "UPDATE companies SET spent_monthly_cents = 200 WHERE id = ?1",
        [company.id.clone()],
    )
    .await
    .unwrap();

    let agent = state
        .agents
        .create(staple_data::NewAgent {
            company_id: company.id.clone(),
            name: "worker".to_owned(),
            role: "worker".to_owned(),
            title: None,
            icon: None,
            reports_to: None,
            adapter_type: "cli".to_owned(),
            budget_monthly_cents: 0,
        })
        .await
        .unwrap();

    // Issues: one blocker (in progress) and one blocked issue.
    let blocker = state
        .issues
        .create(staple_data::NewIssue {
            company_id: company.id.clone(),
            project_id: None,
            goal_id: None,
            parent_id: None,
            title: "Blocker task".to_owned(),
            description: None,
            status: Some("in_progress".to_owned()),
            priority: Some("high".to_owned()),
            assignee_agent_id: Some(agent.id.clone()),
            assignee_user_id: None,
            created_by_user_id: None,
            work_mode: None,
            billing_code: None,
            execution_workspace_settings: None,
        })
        .await
        .unwrap();
    let blocked = state
        .issues
        .create(staple_data::NewIssue {
            company_id: company.id.clone(),
            project_id: None,
            goal_id: None,
            parent_id: None,
            title: "Blocked task".to_owned(),
            description: None,
            status: Some("blocked".to_owned()),
            priority: Some("high".to_owned()),
            assignee_agent_id: Some(agent.id.clone()),
            assignee_user_id: None,
            created_by_user_id: None,
            work_mode: None,
            billing_code: None,
            execution_workspace_settings: None,
        })
        .await
        .unwrap();
    state
        .relations
        .add_blocker(staple_data::NewIssueRelation {
            issue_id: blocker.id.clone(),
            related_issue_id: blocked.id.clone(),
        })
        .await
        .unwrap();

    // Pending approval linked to the blocked issue.
    let approval = state
        .approvals
        .create(staple_data::NewApproval {
            company_id: company.id.clone(),
            r#type: "request_board_approval".to_owned(),
            requested_by_agent_id: Some(agent.id.clone()),
            requested_by_user_id: None,
            payload: "{\"kind\":\"test\"}".to_owned(),
        })
        .await
        .unwrap();
    state
        .issue_structure
        .link_approval(&company.id, &blocked.id, &approval.id)
        .await
        .unwrap();

    // Two pending request confirmations on the same issue (feed collapses to
    // the newest) plus one question interaction.
    let _ = state
        .issue_structure
        .create_thread_interaction(staple_data::NewThreadInteraction {
            company_id: company.id.clone(),
            issue_id: blocked.id.clone(),
            kind: "request_confirmation".to_owned(),
            payload: "{\"prompt\":\"first\"}".to_owned(),
            created_by_agent_id: Some(agent.id.clone()),
            created_by_user_id: None,
        })
        .await
        .unwrap();
    let newest = state
        .issue_structure
        .create_thread_interaction(staple_data::NewThreadInteraction {
            company_id: company.id.clone(),
            issue_id: blocked.id.clone(),
            kind: "request_confirmation".to_owned(),
            payload: "{\"prompt\":\"second\"}".to_owned(),
            created_by_agent_id: Some(agent.id.clone()),
            created_by_user_id: None,
        })
        .await
        .unwrap();
    let _ = state
        .issue_structure
        .create_thread_interaction(staple_data::NewThreadInteraction {
            company_id: company.id.clone(),
            issue_id: blocker.id.clone(),
            kind: "ask_user_questions".to_owned(),
            payload: "{\"questions\":[]}".to_owned(),
            created_by_agent_id: Some(agent.id.clone()),
            created_by_user_id: None,
        })
        .await
        .unwrap();

    (company.id, agent.id, blocked.id, newest.interaction.id)
}

#[tokio::test]
async fn attention_feed_returns_all_source_kinds() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let (company_id, _agent_id, blocked_id, newest_interaction_id) =
        seed_attention_fixture(&state, &db).await;

    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/attention"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["companyId"], company_id);
    assert!(body["generatedAt"].is_string());
    assert_eq!(
        body["totalCount"], 5,
        "approval + 2 interactions + blocker + budget"
    );
    assert!(body["deskBadgeCount"].as_u64().unwrap() >= 1);
    assert_eq!(body["countsBySourceKind"]["approval"], 1);
    assert_eq!(body["countsBySourceKind"]["issue_thread_interaction"], 2);
    assert_eq!(body["countsBySourceKind"]["blocker_attention"], 1);
    assert_eq!(body["countsBySourceKind"]["budget_alert"], 1);
    assert!(body["nextCursor"].is_null());

    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 5);
    // Budget alert present.
    assert!(
        items
            .iter()
            .any(|item| item["sourceKind"] == "budget_alert")
    );
    // Approval item.
    let approval = items
        .iter()
        .find(|item| item["sourceKind"] == "approval")
        .expect("approval item");
    assert_eq!(approval["subject"]["metadata"]["issueId"], blocked_id);
    assert_eq!(approval["decisionVerbs"].as_array().unwrap().len(), 3);
    // Interactions: request_confirmation collapsed to the newest.
    let interactions: Vec<&Value> = items
        .iter()
        .filter(|item| item["sourceKind"] == "issue_thread_interaction")
        .collect();
    assert_eq!(interactions.len(), 2);
    let confirmations: Vec<&Value> = interactions
        .iter()
        .copied()
        .filter(|item| item["subject"]["metadata"]["kind"] == "request_confirmation")
        .collect();
    assert_eq!(confirmations.len(), 1, "request confirmations collapsed");
    assert_eq!(confirmations[0]["subject"]["id"], newest_interaction_id);
    // Blocker item with #10785 triage fields.
    let blocker = items
        .iter()
        .find(|item| item["sourceKind"] == "blocker_attention")
        .expect("blocker item");
    assert_eq!(blocker["subject"]["id"], blocked_id);
    assert!(blocker["detail"]["blockingTreeLive"].as_bool().unwrap());
    assert!(blocker["detail"]["terminalBlockerIssueId"].is_string());
    assert_eq!(blocker["detail"]["blockedTaskCount"], 0);
    assert_eq!(blocker["severity"], "high");
}

#[tokio::test]
async fn attention_feed_cursor_pagination() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let (company_id, _agent_id, _blocked_id, _newest_id) =
        seed_attention_fixture(&state, &db).await;

    let (status, first) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/attention?limit=2"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {first}");
    assert_eq!(first["items"].as_array().unwrap().len(), 2);
    let cursor = first["nextCursor"].as_str().expect("next cursor");
    assert!(!cursor.is_empty());

    let (status, second) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/attention?limit=2&cursor={cursor}"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {second}");
    let second_items = second["items"].as_array().unwrap();
    assert_eq!(second_items.len(), 2);
    let first_ids: Vec<&str> = first["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    for item in second_items {
        assert!(!first_ids.contains(&item["id"].as_str().unwrap()));
    }

    // Invalid cursor -> 400.
    let (status, _) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/attention?cursor=not-a-cursor"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn attention_feed_decide_sort_is_oldest_first() {
    let (state, db) = test_state().await;
    let app = router(state.clone());
    let (company_id, _agent_id, _blocked_id, _newest_id) =
        seed_attention_fixture(&state, &db).await;

    let (status, body) = send_json(
        &app,
        Method::GET,
        &format!("/api/companies/{company_id}/attention?sort=decide"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let items = body["items"].as_array().unwrap();
    let created: Vec<&str> = items
        .iter()
        .map(|item| item["createdAt"].as_str().unwrap())
        .collect();
    let mut sorted = created.clone();
    sorted.sort();
    assert_eq!(created, sorted, "decide sort must be oldest-first");
}
