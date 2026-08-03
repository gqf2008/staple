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

/// Contract test: Postgres export produces the same row-level snapshot as the
/// Turso source for the same rows. Requires a running Postgres via
/// `STAPLE_TEST_POSTGRES_URL`; skipped otherwise (CI/local without Postgres).
#[tokio::test]
async fn postgres_export_matches_turso_snapshot() {
    let Ok(url) = std::env::var("STAPLE_TEST_POSTGRES_URL") else {
        eprintln!(
            "skipping postgres_export_matches_turso_snapshot: STAPLE_TEST_POSTGRES_URL not set"
        );
        return;
    };

    // Create a minimal schema covering every exported table (id-only for the
    // tables we do not seed) plus real columns for the seeded tables.
    let setup_url = url.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let url = setup_url;
        let mut client = postgres::Client::connect(&url, postgres::NoTls).unwrap();
            for table in staple_migrate::TABLE_ORDER {
                client
                    .batch_execute(&format!(
                        "DROP TABLE IF EXISTS {table} CASCADE; CREATE TABLE {table} (id TEXT PRIMARY KEY)"
                    ))
                    .unwrap();
            }
            client
                .batch_execute(
                    "ALTER TABLE companies ADD COLUMN name TEXT;
                     ALTER TABLE companies ADD COLUMN issue_prefix TEXT;
                     ALTER TABLE companies ADD COLUMN attachment_max_bytes BIGINT;
                     ALTER TABLE companies ADD COLUMN budget_monthly_cents BIGINT;
                     ALTER TABLE agents ADD COLUMN company_id TEXT;
                     ALTER TABLE agents ADD COLUMN name TEXT;
                     ALTER TABLE agents ADD COLUMN role TEXT;
                     ALTER TABLE agents ADD COLUMN adapter_type TEXT;
                     ALTER TABLE agents ADD COLUMN status TEXT;
                     ALTER TABLE issues ADD COLUMN company_id TEXT;
                     ALTER TABLE issues ADD COLUMN project_id TEXT;
                     ALTER TABLE issues ADD COLUMN title TEXT;
                     ALTER TABLE issues ADD COLUMN issue_number BIGINT;
                     ALTER TABLE issues ADD COLUMN identifier TEXT;
                     ALTER TABLE issues ADD COLUMN status TEXT;
                     ALTER TABLE issues ADD COLUMN description TEXT;
                     ALTER TABLE issues ADD COLUMN priority TEXT;
                     ALTER TABLE issues ADD COLUMN parent_id TEXT;
                     ALTER TABLE issues ADD COLUMN assignee_agent_id TEXT;
                     ALTER TABLE issues ADD COLUMN work_mode TEXT;
                     ALTER TABLE issues ADD COLUMN billing_code TEXT;
                     ALTER TABLE issues ADD COLUMN request_depth BIGINT;
                     ALTER TABLE issues ADD COLUMN hidden_at TEXT;
                     ALTER TABLE issues ADD COLUMN started_at TEXT;
                     ALTER TABLE issues ADD COLUMN completed_at TEXT;
                     ALTER TABLE issues ADD COLUMN cancelled_at TEXT;
                     ALTER TABLE issues ADD COLUMN created_by_user_id TEXT;
                     ALTER TABLE issues ADD COLUMN created_at TEXT;
                     ALTER TABLE issues ADD COLUMN updated_at TEXT;
                     ALTER TABLE issues ADD COLUMN execution_policy TEXT;
                     ALTER TABLE issues ADD COLUMN checkout_run_id TEXT;
                     ALTER TABLE issues ADD COLUMN execution_run_id TEXT;
                     ALTER TABLE issues ADD COLUMN execution_locked_at TEXT;
                     ALTER TABLE issues ADD COLUMN origin_kind TEXT;
                     ALTER TABLE issues ADD COLUMN origin_id TEXT;
                     ALTER TABLE issues ADD COLUMN sort_order BIGINT;
                     ALTER TABLE issues ADD COLUMN due_at TEXT;
                     ALTER TABLE issues ADD COLUMN estimate_points BIGINT;
                     ALTER TABLE issues ADD COLUMN confidence BIGINT;
                     ALTER TABLE issues ADD COLUMN complexity TEXT;
                     ALTER TABLE issues ADD COLUMN priority_score REAL;
                     ALTER TABLE issues ADD COLUMN derived_state TEXT;
                     ALTER TABLE issues ADD COLUMN plan TEXT;
                     ALTER TABLE issues ADD COLUMN plan_updated_at TEXT;
                     ALTER TABLE issues ADD COLUMN plan_locked_at TEXT;
                     ALTER TABLE issues ADD COLUMN plan_lock_holder_agent_id TEXT;
                     ALTER TABLE issues ADD COLUMN plan_lock_holder_user_id TEXT;
                     ALTER TABLE issues ADD COLUMN plan_lock_reason TEXT;
                     ALTER TABLE issues ADD COLUMN subtask TEXT;
                     ALTER TABLE issues ADD COLUMN spec TEXT;
                     ALTER TABLE issues ADD COLUMN execution_policy_source TEXT;
                     ALTER TABLE issues ADD COLUMN approve_plan_locked_at TEXT;
                     ALTER TABLE issues ADD COLUMN approve_plan_locked_by TEXT;
                     ALTER TABLE issues ADD COLUMN approve_plan_locked_reason TEXT;
                     ALTER TABLE issues ADD COLUMN approval_status TEXT;
                     ALTER TABLE issues ADD COLUMN approved_at TEXT;
                     ALTER TABLE issues ADD COLUMN rejected_at TEXT;
                     ALTER TABLE issues ADD COLUMN rejection_reason TEXT;
                     ALTER TABLE issues ADD COLUMN environment_id TEXT;
                     ALTER TABLE issues ADD COLUMN workspace_id TEXT;
                     ALTER TABLE issues ADD COLUMN execution_workspace_id TEXT;
                     ALTER TABLE issues ADD COLUMN last_synced_at TEXT;
                     ALTER TABLE issues ADD COLUMN sync_error TEXT;
                     ALTER TABLE issues ADD COLUMN searchable_text TEXT;
                     ALTER TABLE issues ADD COLUMN deleted_at TEXT;
                     ALTER TABLE issues ADD COLUMN duplicate_of_id TEXT;",
                )
                .unwrap();

            // Seed the same rows into Postgres.
            client
                .batch_execute(
                    "ALTER TABLE companies ADD COLUMN created_at TEXT;
                     ALTER TABLE companies ADD COLUMN updated_at TEXT;
                     ALTER TABLE agents ADD COLUMN created_at TEXT;
                     ALTER TABLE agents ADD COLUMN updated_at TEXT;
                     INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes, budget_monthly_cents, created_at, updated_at)
                     VALUES ('c1', 'Alpha', 'ALPHA', 1024, 500, '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z'),
                            ('c2', 'Beta', 'BETA', 2048, 0, '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z');
                     INSERT INTO agents (id, company_id, name, role, adapter_type, status, created_at, updated_at)
                     VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local', 'active', '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z'),
                            ('a2', 'c2', 'two', 'ceo', 'http', 'paused', '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z');
                     INSERT INTO issues (id, company_id, project_id, title, issue_number, identifier, status, created_at, updated_at)
                     VALUES ('i1', 'c1', NULL, 'Task one', 1, 'ALPHA-1', 'in_progress', '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z'),
                            ('i2', 'c2', NULL, 'Beta task', 2, 'BETA-1', 'done', '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z');",
                )
                .unwrap();

        Ok(())
    })
    .await
    .unwrap()
    .unwrap();
    let pg_snapshot = staple_migrate::export_postgres(&url).await.unwrap();
    assert_eq!(pg_snapshot["companies"].len(), 2);
    assert_eq!(pg_snapshot["agents"].len(), 2);
    assert_eq!(pg_snapshot["issues"].len(), 2);

    // Seed the same rows into a Turso database (full schema) and export it.
    let dir = TempDir::new().unwrap();
    let turso_path = dir.path().join("source.db");
    let turso_path_str = turso_path.display().to_string();
    let db = staple_migrate::open_local(&turso_path_str).await.unwrap();
    let conn = db.connect().unwrap();
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
        "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes, budget_monthly_cents,
                                created_at, updated_at)
         VALUES ('c1', 'Alpha', 'ALPHA', 1024, 500, '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z'),
                ('c2', 'Beta', 'BETA', 2048, 0, '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO agents (id, company_id, name, role, adapter_type, status,
                             created_at, updated_at)
         VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local', 'active',
                 '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z'),
                ('a2', 'c2', 'two', 'ceo', 'http', 'paused',
                 '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z')",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, company_id, project_id, title, issue_number, identifier, status,
                             created_at, updated_at)
         VALUES ('i1', 'c1', NULL, 'Task one', 1, 'ALPHA-1', 'in_progress',
                 '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z'),
                ('i2', 'c2', NULL, 'Beta task', 2, 'BETA-1', 'done',
                 '2026-08-03T00:00:00.000Z', '2026-08-03T00:00:00.000Z')",
        (),
    )
    .await
    .unwrap();
    let turso_snapshot = export(&turso_path_str).await.unwrap();

    // Row-level consistency: every explicitly seeded column matches between
    // the Postgres snapshot and the Turso snapshot for the same row id.
    let seeded_columns: &[(&str, &[&str])] = &[
        (
            "companies",
            &[
                "id",
                "name",
                "issue_prefix",
                "attachment_max_bytes",
                "budget_monthly_cents",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "agents",
            &[
                "id",
                "company_id",
                "name",
                "role",
                "adapter_type",
                "status",
                "created_at",
                "updated_at",
            ],
        ),
        (
            "issues",
            &[
                "id",
                "company_id",
                "project_id",
                "title",
                "issue_number",
                "identifier",
                "status",
                "created_at",
                "updated_at",
            ],
        ),
    ];
    for (table, columns) in seeded_columns {
        for pg_row in &pg_snapshot[*table] {
            let id = pg_row["id"].as_str().unwrap();
            let turso_row = turso_snapshot[*table]
                .iter()
                .find(|row| row["id"].as_str() == Some(id))
                .unwrap_or_else(|| panic!("{table} row {id} missing from Turso snapshot"));
            for key in *columns {
                assert_eq!(
                    pg_row.get(*key),
                    turso_row.get(*key),
                    "{table} {id} column {key} differs between Postgres and Turso"
                );
            }
        }
    }
}
