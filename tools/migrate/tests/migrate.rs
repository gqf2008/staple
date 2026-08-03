//! End-to-end snapshot migration test: seed a source database, export,
//! import into a clean database, verify counts and constraints.

use std::path::Path;

use staple_migrate::{export, import, load_snapshot, verify};
use tempfile::TempDir;

/// Seeds a source database with data covering parent/child FK relations.
async fn seed_source(path: &Path) {
    let db = staple_migrate::open_local(&path.display().to_string())
        .await
        .unwrap();
    let conn = db.connect().unwrap();
    conn.execute("PRAGMA foreign_keys = ON", ()).await.unwrap();
    // Use the same migrations as the data layer.
    let migrations = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("crates/data/migrations");
    staple_migrate::apply_migrations(&conn, &migrations)
        .await
        .unwrap();

    conn.execute(
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes, budget_monthly_cents)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024, 500), ('c2', 'Beta', 'BETA', 2048, 0)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type, status)
         VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local', 'active'),
                ('a2', 'c1', 'two', 'ceo', 'codex_local', 'paused'),
                ('a3', 'c2', 'three', 'engineer', 'http', 'active')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO goals (id, company_id, title, level)
         VALUES ('g1', 'c1', 'Growth', 'company'), ('g2', 'c2', 'Beta Goal', 'company')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO projects (id, company_id, goal_id, name)
         VALUES ('p1', 'c1', 'g1', 'Ship'), ('p2', 'c2', NULL, 'Ops')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, project_id, title, issue_number, identifier, status)
         VALUES ('i1', 'c1', 'p1', 'Task one', 1, 'ALPHA-1', 'in_progress'),
                ('i2', 'c1', 'p1', 'Task two', 2, 'ALPHA-2', 'backlog'),
                ('i3', 'c2', NULL, 'Beta task', 1, 'BETA-1', 'done')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issue_comments (id, company_id, issue_id, author_user_id, body)
         VALUES ('cm1', 'c1', 'i1', 'u1', 'hello')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO cost_events (id, company_id, agent_id, provider, model, cost_cents, occurred_at)
         VALUES ('ce1', 'c1', 'a1', 'anthropic', 'claude', 25, '2026-08-03T00:00:00.000Z')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO approvals (id, company_id, type, payload, status)
         VALUES ('ap1', 'c1', 'hire_agent', '{}', 'pending')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO activity_log (id, company_id, actor_type, actor_id, action, entity_type, entity_id)
         VALUES ('act1', 'c1', 'user', 'board', 'company.created', 'company', 'c1')",
        (),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn export_import_verify_roundtrip() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("source.db");
    let target = dir.path().join("target.db");
    let snapshot_path = dir.path().join("snapshot.json");

    seed_source(&source).await;

    // Export.
    let snapshot = export(&source.display().to_string()).await.unwrap();
    let bytes = serde_json::to_vec_pretty(&snapshot).unwrap();
    std::fs::write(&snapshot_path, bytes).unwrap();

    // Import into a clean database.
    let counts = import(&target.display().to_string(), &snapshot)
        .await
        .unwrap();
    assert_eq!(counts["companies"], 2);
    assert_eq!(counts["agents"], 3);
    assert_eq!(counts["goals"], 2);
    assert_eq!(counts["projects"], 2);
    assert_eq!(counts["issues"], 3);
    assert_eq!(counts["issue_comments"], 1);
    assert_eq!(counts["cost_events"], 1);
    assert_eq!(counts["approvals"], 1);
    assert_eq!(counts["activity_log"], 1);

    // Verify counts against the snapshot.
    verify(
        &target.display().to_string(),
        &load_snapshot(&snapshot_path).unwrap(),
    )
    .await
    .unwrap();

    // Constraints still hold after import: cross-company references are
    // rejected by the composite FKs.
    let db = staple_migrate::open_local(&target.display().to_string())
        .await
        .unwrap();
    let conn = db.connect().unwrap();
    conn.execute("PRAGMA foreign_keys = ON", ()).await.unwrap();
    let error = conn
        .execute(
            "INSERT INTO issues (id, company_id, project_id, title, issue_number, identifier)
             VALUES ('bad', 'c2', 'p1', 'cross', 99, 'BAD-99')",
            (),
        )
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("FOREIGN KEY constraint failed"),
        "unexpected: {error}"
    );

    // Boolean/status values survived as typed values.
    let mut rows = conn
        .query("SELECT status FROM agents WHERE id = 'a2'", ())
        .await
        .unwrap();
    let row = rows.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>(0).unwrap(), "paused");
}

#[tokio::test]
async fn verify_detects_missing_rows() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("source.db");
    let target = dir.path().join("target.db");

    seed_source(&source).await;
    let snapshot = export(&source.display().to_string()).await.unwrap();
    import(&target.display().to_string(), &snapshot)
        .await
        .unwrap();

    // Delete one row from the target, then verify must fail.
    let db = staple_migrate::open_local(&target.display().to_string())
        .await
        .unwrap();
    let conn = db.connect().unwrap();
    conn.execute("DELETE FROM issue_comments WHERE id = 'cm1'", ())
        .await
        .unwrap();

    let error = verify(&target.display().to_string(), &snapshot)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("issue_comments"));
}
