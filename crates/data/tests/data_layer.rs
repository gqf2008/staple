//! Data-layer integration tests: migrations, schema coverage, and
//! SQL-enforced company isolation.

use libsql::{Connection, Database};
use staple_data::{DbConfig, load_migrations, migrate, migrate_down, open};
use tempfile::TempDir;

const CORE_TABLES: &[&str] = &[
    "companies",
    "agents",
    "agent_api_keys",
    "goals",
    "projects",
    "issues",
    "issue_comments",
    "heartbeat_runs",
    "cost_events",
    "approvals",
    "activity_log",
    "project_memberships",
    "agent_memberships",
    "company_secrets",
    "company_secret_versions",
    "assets",
    "issue_attachments",
    "documents",
    "document_revisions",
    "issue_documents",
];

const REQUIRED_INDEXES: &[&str] = &[
    "idx_agents_company_status",
    "idx_agents_company_reports_to",
    "idx_issues_company_status",
    "idx_issues_company_assignee_status",
    "idx_issues_company_parent",
    "idx_issues_company_project",
    "idx_cost_events_company_occurred",
    "idx_cost_events_company_agent_occurred",
    "idx_heartbeat_runs_company_agent_started",
    "idx_approvals_company_status_type",
    "idx_activity_log_company_created",
    "idx_assets_company_created",
    "idx_issue_attachments_company_issue",
    "idx_project_memberships_company_user",
    "idx_agent_memberships_company_user",
];

async fn migrated_db() -> (TempDir, Database) {
    let dir = TempDir::new().unwrap();
    let db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    let applied = migrate(&db).await.unwrap();
    assert_eq!(applied.len(), load_migrations("migrations").unwrap().len());
    (dir, db)
}

async fn connect_conn(db: &Database) -> Connection {
    staple_data::connect(db).await.unwrap()
}

async fn table_names(conn: &Connection) -> Vec<String> {
    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            (),
        )
        .await
        .unwrap();
    let mut names = Vec::new();
    while let Some(row) = rows.next().await.unwrap() {
        names.push(row.get::<String>(0).unwrap());
    }
    names
}

async fn index_names(conn: &Connection) -> Vec<String> {
    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name IS NOT NULL ORDER BY name",
            (),
        )
        .await
        .unwrap();
    let mut names = Vec::new();
    while let Some(row) = rows.next().await.unwrap() {
        names.push(row.get::<String>(0).unwrap());
    }
    names
}

async fn insert_company(conn: &Connection, id: &str, name: &str, prefix: &str) {
    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
         VALUES (?1, ?2, ?3, 10485760)",
        (id, name, prefix),
    )
    .await
    .unwrap();
}

async fn insert_agent(conn: &Connection, id: &str, company_id: &str, name: &str) {
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type)
         VALUES (?1, ?2, ?3, 'engineer', 'codex_local')",
        (id, company_id, name),
    )
    .await
    .unwrap();
}

async fn insert_project(conn: &Connection, id: &str, company_id: &str, name: &str) {
    conn.execute(
        "INSERT INTO projects (id, company_id, name) VALUES (?1, ?2, ?3)",
        (id, company_id, name),
    )
    .await
    .unwrap();
}

async fn insert_asset(conn: &Connection, id: &str, company_id: &str, key: &str) {
    conn.execute(
        "INSERT INTO assets (id, company_id, provider, object_key, content_type, byte_size, sha256)
         VALUES (?1, ?2, 'local_disk', ?3, 'text/plain', 4, 'abc123')",
        (id, company_id, key),
    )
    .await
    .unwrap();
}

async fn insert_issue(
    conn: &Connection,
    id: &str,
    company_id: &str,
    project_id: Option<&str>,
    number: i64,
    identifier: &str,
) -> Result<u64, libsql::Error> {
    conn.execute(
        "INSERT INTO issues (id, company_id, project_id, title, issue_number, identifier)
         VALUES (?1, ?2, ?3, 'task', ?4, ?5)",
        (id, company_id, project_id, number, identifier),
    )
    .await
}

#[tokio::test]
async fn migrate_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();

    let first = migrate(&db).await.unwrap();
    assert!(!first.is_empty());

    let second = migrate(&db).await.unwrap();
    assert!(second.is_empty(), "second migrate must be a no-op");

    let conn = connect_conn(&db).await;
    let mut rows = conn
        .query("SELECT COUNT(*) FROM schema_migrations", ())
        .await
        .unwrap();
    let row = rows.next().await.unwrap().unwrap();
    let count: i64 = row.get(0).unwrap();
    assert_eq!(count, load_migrations("migrations").unwrap().len() as i64);
}

#[tokio::test]
async fn schema_covers_all_core_tables() {
    let (_dir, db) = migrated_db().await;
    let conn = connect_conn(&db).await;
    let names = table_names(&conn).await;
    for table in CORE_TABLES {
        assert!(
            names.iter().any(|name| name == table),
            "missing table {table}"
        );
    }
    assert!(names.iter().any(|name| name == "schema_migrations"));
}

#[tokio::test]
async fn schema_creates_required_indexes() {
    let (_dir, db) = migrated_db().await;
    let conn = connect_conn(&db).await;
    let names = index_names(&conn).await;
    for index in REQUIRED_INDEXES {
        assert!(
            names.iter().any(|name| name == index),
            "missing index {index}"
        );
    }
}

#[tokio::test]
async fn migrate_down_rolls_back_and_can_reapply() {
    let dir = TempDir::new().unwrap();
    let db = open(&DbConfig::local(dir.path().join("test.db")))
        .await
        .unwrap();
    migrate(&db).await.unwrap();

    let migration_count = load_migrations("migrations").unwrap().len();
    let rolled_back = migrate_down(&db, 0).await.unwrap();
    assert_eq!(rolled_back.len(), migration_count);

    let conn = connect_conn(&db).await;
    let names = table_names(&conn).await;
    assert!(!names.iter().any(|name| name == "companies"));

    // Reapply: up must work again from a clean slate.
    let applied = migrate(&db).await.unwrap();
    assert_eq!(applied.len(), migration_count);
}

#[tokio::test]
async fn company_isolation_rejects_unknown_company() {
    let (_dir, db) = migrated_db().await;
    let conn = connect_conn(&db).await;

    let error = conn
        .execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'missing-company', 'x', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("FOREIGN KEY constraint failed"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn company_isolation_rejects_cross_company_project_reference() {
    let (_dir, db) = migrated_db().await;
    let conn = connect_conn(&db).await;
    insert_company(&conn, "c1", "Alpha", "ALPHA").await;
    insert_company(&conn, "c2", "Beta", "BETA").await;
    insert_project(&conn, "p1", "c1", "Project One").await;
    insert_agent(&conn, "a1", "c1", "Agent One").await;

    // c2 issue referencing c1's project must be rejected by the composite FK.
    let error = insert_issue(&conn, "i1", "c2", Some("p1"), 1, "BETA-1")
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("FOREIGN KEY constraint failed"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn company_isolation_rejects_cross_company_asset_reference() {
    let (_dir, db) = migrated_db().await;
    let conn = connect_conn(&db).await;
    insert_company(&conn, "c1", "Alpha", "ALPHA").await;
    insert_company(&conn, "c2", "Beta", "BETA").await;
    insert_agent(&conn, "a1", "c1", "Agent One").await;
    insert_project(&conn, "p1", "c1", "Project One").await;
    insert_asset(&conn, "asset1", "c1", "k1").await;
    insert_issue(&conn, "i1", "c1", Some("p1"), 1, "ALPHA-1")
        .await
        .unwrap();

    // c2 attachment referencing c1's asset must be rejected.
    let error = conn
        .execute(
            "INSERT INTO issue_attachments (id, company_id, issue_id, asset_id)
             VALUES ('att1', 'c2', 'i1', 'asset1')",
            (),
        )
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("FOREIGN KEY constraint failed"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn valid_same_company_inserts_succeed() {
    let (_dir, db) = migrated_db().await;
    let conn = connect_conn(&db).await;
    insert_company(&conn, "c1", "Alpha", "ALPHA").await;
    insert_agent(&conn, "a1", "c1", "Agent One").await;
    insert_project(&conn, "p1", "c1", "Project One").await;
    insert_issue(&conn, "i1", "c1", Some("p1"), 1, "ALPHA-1")
        .await
        .unwrap();
    insert_asset(&conn, "asset1", "c1", "k1").await;
    conn.execute(
        "INSERT INTO issue_attachments (id, company_id, issue_id, asset_id)
         VALUES ('att1', 'c1', 'i1', 'asset1')",
        (),
    )
    .await
    .unwrap();
}
