//! Company portability: export/import a company's core business tables as a
//! JSON manifest. Ids are re-minted and references rewritten on import so a
//! manifest can be restored into any company (upstream CompanyExport/
//! CompanyImport — JSON-manifest baseline; zip packaging is a later batch).

use async_trait::async_trait;
use libsql::Database;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::helpers;

/// Core tables included in a company manifest (parents before children).
const CORE_TABLES: [&str; 7] = [
    "agents",
    "goals",
    "projects",
    "labels",
    "issues",
    "issue_comments",
    "issue_labels",
];

/// Per-table import specification: portable columns and columns whose values
/// reference other rows in the manifest (rewritten via the id map). Columns
/// referencing tables outside the manifest (runs, workspaces, documents,
/// etc.) are intentionally excluded so imports stay valid.
struct TableSpec {
    name: &'static str,
    columns: &'static [&'static str],
    mapped: &'static [&'static str],
}

const TABLE_SPECS: [TableSpec; 7] = [
    TableSpec {
        name: "agents",
        columns: &[
            "id",
            "company_id",
            "name",
            "role",
            "title",
            "icon",
            "status",
            "reports_to",
            "capabilities",
            "adapter_type",
            "adapter_config",
            "runtime_config",
            "context_mode",
            "budget_monthly_cents",
            "spent_monthly_cents",
            "pause_reason",
            "paused_at",
            "permissions",
            "last_heartbeat_at",
            "metadata",
        ],
        mapped: &["reports_to"],
    },
    TableSpec {
        name: "goals",
        columns: &[
            "id",
            "company_id",
            "title",
            "description",
            "level",
            "parent_id",
            "owner_agent_id",
            "status",
        ],
        mapped: &["parent_id", "owner_agent_id"],
    },
    TableSpec {
        name: "projects",
        columns: &[
            "id",
            "company_id",
            "goal_id",
            "name",
            "description",
            "status",
            "lead_agent_id",
            "target_date",
            "env",
        ],
        mapped: &["goal_id", "lead_agent_id"],
    },
    TableSpec {
        name: "labels",
        columns: &["id", "company_id", "name", "color"],
        mapped: &[],
    },
    TableSpec {
        name: "issues",
        columns: &[
            "id",
            "company_id",
            "project_id",
            "goal_id",
            "parent_id",
            "title",
            "description",
            "status",
            "priority",
            "assignee_agent_id",
            "billing_code",
            "created_by_agent_id",
            "created_by_user_id",
            "hidden_at",
            "issue_number",
            "identifier",
            "work_mode",
            "execution_policy",
            "execution_state",
            "started_at",
            "completed_at",
            "cancelled_at",
            "source_trust",
            "unblock_descriptor",
            "responsible_user_id",
            "request_depth",
            "origin_kind",
            "origin_id",
            "origin_fingerprint",
        ],
        mapped: &[
            "project_id",
            "goal_id",
            "parent_id",
            "assignee_agent_id",
            "created_by_agent_id",
        ],
    },
    TableSpec {
        name: "issue_comments",
        columns: &[
            "id",
            "company_id",
            "issue_id",
            "author_agent_id",
            "author_user_id",
            "body",
        ],
        mapped: &["issue_id", "author_agent_id"],
    },
    TableSpec {
        name: "issue_labels",
        columns: &["company_id", "issue_id", "label_id"],
        mapped: &["issue_id", "label_id"],
    },
];

/// One table's rows in a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestTable {
    /// Table name (whitelisted core table).
    pub name: String,
    /// Rows as column -> value objects.
    pub rows: Vec<serde_json::Value>,
}

/// Company portability manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyManifest {
    /// Manifest version (currently 1).
    pub version: u32,
    /// ISO 8601 export time.
    pub exported_at: String,
    /// Source company id (informational).
    pub company_id: String,
    /// Core tables in dependency order.
    pub tables: Vec<ManifestTable>,
}

/// Import strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportStrategy {
    /// Skip: require an empty target company (idempotent restore).
    #[default]
    Skip,
    /// Delete the company's core rows first, then insert.
    Overwrite,
}

/// Import result summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    /// Rows inserted.
    pub imported: u64,
    /// Rows skipped (existing ids or conflicts).
    pub skipped: u64,
    /// Rows failed to insert.
    pub failed: u64,
}

/// Portability repository errors.
#[derive(Debug, Error)]
pub enum PortabilityError {
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    #[error("company not found")]
    CompanyNotFound,
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("target company already has data (use overwrite strategy)")]
    CompanyNotEmpty,
}

/// Portability persistence contract.
#[async_trait]
pub trait PortabilityRepository: Send + Sync {
    /// Exports a company's core tables as a manifest.
    ///
    /// # Errors
    ///
    /// Returns [`PortabilityError`] when the company is missing or the
    /// database fails.
    async fn export_company(&self, company_id: &str) -> Result<CompanyManifest, PortabilityError>;

    /// Counts existing core rows per table for a company (conflict preview).
    ///
    /// # Errors
    ///
    /// Returns [`PortabilityError`] when the company is missing or the
    /// database fails.
    async fn company_row_counts(
        &self,
        company_id: &str,
    ) -> Result<std::collections::HashMap<String, u64>, PortabilityError>;

    /// Imports a manifest into a company, minting fresh ids and rewriting
    /// references so the rows belong to the target company.
    ///
    /// # Errors
    ///
    /// Returns [`PortabilityError`] on invalid manifests or database
    /// failures.
    async fn import_company(
        &self,
        company_id: &str,
        manifest: CompanyManifest,
        strategy: ImportStrategy,
    ) -> Result<ImportSummary, PortabilityError>;
}

/// Turso/libSQL implementation of [`PortabilityRepository`].
#[derive(Debug)]
pub struct TursoPortabilityRepository {
    db: Database,
}

impl TursoPortabilityRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

/// Column names for a table via `PRAGMA table_info`.
async fn table_columns(
    conn: &libsql::Connection,
    table: &str,
) -> Result<Vec<String>, PortabilityError> {
    let mut rows = conn
        .query(&format!("PRAGMA table_info({table})"), ())
        .await?;
    let mut columns = Vec::new();
    while let Some(row) = rows.next().await? {
        if let Some(name) = helpers::row_text(&row, 1)? {
            columns.push(name);
        }
    }
    Ok(columns)
}

/// Converts a libsql row into a JSON object keyed by column name.
fn row_to_json(
    row: &libsql::Row,
    columns: &[String],
) -> Result<serde_json::Value, PortabilityError> {
    let mut object = serde_json::Map::new();
    for (index, column) in columns.iter().enumerate() {
        let value = row.get_value(index as i32)?;
        let json = if value.is_null() {
            serde_json::Value::Null
        } else if value.is_integer() {
            serde_json::json!(value.as_integer().expect("integer"))
        } else if value.is_real() {
            serde_json::json!(value.as_real().expect("real"))
        } else if value.is_text() {
            serde_json::json!(value.as_text().expect("text").clone())
        } else {
            // Blob columns are not part of the core tables.
            serde_json::Value::Null
        };
        object.insert(column.clone(), json);
    }
    Ok(serde_json::Value::Object(object))
}

/// Converts a JSON value back into a libsql value (objects/arrays/bools are
/// stored as their JSON text, matching the TEXT JSON columns).
fn json_to_value(value: &serde_json::Value) -> libsql::Value {
    match value {
        serde_json::Value::Null => libsql::Value::Null,
        serde_json::Value::Bool(flag) => libsql::Value::from(i64::from(*flag)),
        serde_json::Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                libsql::Value::from(integer)
            } else {
                libsql::Value::from(number.as_f64().unwrap_or_default())
            }
        }
        serde_json::Value::String(text) => libsql::Value::from(text.clone()),
        other => libsql::Value::from(other.to_string()),
    }
}

#[async_trait]
impl PortabilityRepository for TursoPortabilityRepository {
    async fn export_company(&self, company_id: &str) -> Result<CompanyManifest, PortabilityError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(PortabilityError::CompanyNotFound);
        }
        let mut tables = Vec::new();
        for table in CORE_TABLES {
            let columns = table_columns(&conn, table).await?;
            let mut rows = conn
                .query(
                    &format!(
                        "SELECT {columns} FROM {table} WHERE company_id = ?1",
                        columns = columns.join(", ")
                    ),
                    libsql::params![company_id],
                )
                .await?;
            let mut row_values = Vec::new();
            while let Some(row) = rows.next().await? {
                row_values.push(row_to_json(&row, &columns)?);
            }
            tables.push(ManifestTable {
                name: table.to_owned(),
                rows: row_values,
            });
        }
        Ok(CompanyManifest {
            version: 1,
            exported_at: "2026-08-05T00:00:00.000Z".to_owned(),
            company_id: company_id.to_owned(),
            tables,
        })
    }

    async fn company_row_counts(
        &self,
        company_id: &str,
    ) -> Result<std::collections::HashMap<String, u64>, PortabilityError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(PortabilityError::CompanyNotFound);
        }
        let mut counts = std::collections::HashMap::new();
        for table in CORE_TABLES {
            let mut rows = conn
                .query(
                    &format!("SELECT COUNT(*) FROM {table} WHERE company_id = ?1"),
                    libsql::params![company_id],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                counts.insert(table.to_owned(), helpers::row_i64(&row, 0)? as u64);
            }
        }
        // Package-level tables (docs/skills) used by the conflict preview.
        for table in ["documents", "company_skills"] {
            let mut rows = conn
                .query(
                    &format!("SELECT COUNT(*) FROM {table} WHERE company_id = ?1"),
                    libsql::params![company_id],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                counts.insert(table.to_owned(), helpers::row_i64(&row, 0)? as u64);
            }
        }
        Ok(counts)
    }

    async fn import_company(
        &self,
        company_id: &str,
        manifest: CompanyManifest,
        strategy: ImportStrategy,
    ) -> Result<ImportSummary, PortabilityError> {
        if manifest.version != 1 {
            return Err(PortabilityError::InvalidManifest(
                "unsupported manifest version".to_owned(),
            ));
        }
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(PortabilityError::CompanyNotFound);
        }
        let manifest_tables: std::collections::HashMap<String, Vec<serde_json::Value>> = manifest
            .tables
            .into_iter()
            .map(|table| (table.name, table.rows))
            .collect();
        for table in &manifest_tables {
            if !CORE_TABLES.contains(&table.0.as_str()) {
                return Err(PortabilityError::InvalidManifest(format!(
                    "unknown table {}",
                    table.0
                )));
            }
        }

        // Overwrite: delete the target company's core rows first (children
        // before parents).
        if strategy == ImportStrategy::Overwrite {
            for table in CORE_TABLES.iter().rev() {
                conn.execute(
                    &format!("DELETE FROM {table} WHERE company_id = ?1"),
                    libsql::params![company_id],
                )
                .await?;
            }
        }

        // Id remapping across tables: id columns are globally unique, so a
        // cross-company import must mint fresh ids and rewrite references.
        // Skip requires an empty target company (idempotent restore).
        if strategy == ImportStrategy::Skip {
            for table in CORE_TABLES {
                let mut rows = conn
                    .query(
                        &format!("SELECT 1 FROM {table} WHERE company_id = ?1 LIMIT 1"),
                        libsql::params![company_id],
                    )
                    .await?;
                if rows.next().await?.is_some() {
                    return Err(PortabilityError::CompanyNotEmpty);
                }
            }
        }

        // Id remapping across tables: id columns are globally unique, so a
        // cross-company import must mint fresh ids and rewrite references.
        let mut id_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut imported = 0u64;
        let mut skipped = 0u64;
        let mut failed = 0u64;

        for spec in TABLE_SPECS {
            let Some(rows) = manifest_tables.get(spec.name) else {
                continue;
            };
            let valid_columns = table_columns(&conn, spec.name).await?;
            for row in rows {
                let Some(object) = row.as_object() else {
                    return Err(PortabilityError::InvalidManifest(format!(
                        "row in {} is not an object",
                        spec.name
                    )));
                };
                let mut values: Vec<libsql::Value> = Vec::new();
                let mut insert_columns: Vec<&str> = Vec::new();
                for column in spec.columns {
                    if !valid_columns.contains(&column.to_string()) {
                        continue;
                    }
                    let value = object.get(*column);
                    if spec.mapped.contains(column) {
                        // Rewrite internal references to the fresh ids.
                        let rewritten = value
                            .and_then(serde_json::Value::as_str)
                            .and_then(|old| id_map.get(old).cloned());
                        insert_columns.push(column);
                        values.push(match rewritten {
                            Some(new_id) => libsql::Value::from(new_id),
                            None => libsql::Value::Null,
                        });
                    } else if *column == "id" && spec.name != "issue_labels" {
                        // Mint a fresh id and remember the mapping.
                        let new_id = uuid::Uuid::new_v4().to_string();
                        if let Some(old) = value.and_then(serde_json::Value::as_str) {
                            id_map.insert(old.to_owned(), new_id.clone());
                        }
                        insert_columns.push(column);
                        values.push(libsql::Value::from(new_id));
                    } else if *column == "company_id" {
                        insert_columns.push(column);
                        values.push(libsql::Value::from(company_id.to_owned()));
                    } else {
                        insert_columns.push(column);
                        values.push(json_to_value(value.unwrap_or(&serde_json::Value::Null)));
                    }
                }

                if insert_columns.is_empty() {
                    skipped += 1;
                    continue;
                }
                let placeholders: Vec<String> = (1..=insert_columns.len())
                    .map(|index| format!("?{index}"))
                    .collect();
                let sql = format!(
                    "INSERT INTO {table} ({columns}) VALUES ({placeholders})",
                    table = spec.name,
                    columns = insert_columns.join(", "),
                    placeholders = placeholders.join(", ")
                );
                match conn.execute(&sql, values).await {
                    Ok(_) => imported += 1,
                    Err(error) => {
                        tracing::warn!(table = spec.name, error = %error, "row import failed");
                        failed += 1;
                    }
                }
            }
        }
        Ok(ImportSummary {
            imported,
            skipped,
            failed,
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoPortabilityRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'Agent One', 'engineer', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO goals (id, company_id, title, level)
             VALUES ('g1', 'c1', 'Goal One', 'company')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO projects (id, company_id, goal_id, name)
             VALUES ('p1', 'c1', 'g1', 'Project One')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, project_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'p1', 'Issue One', 1, 'ALPHA-1')",
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
            "INSERT INTO labels (id, company_id, name, color) VALUES ('l1', 'c1', 'bug', '#f00')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issue_labels (issue_id, label_id, company_id)
             VALUES ('i1', 'l1', 'c1')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoPortabilityRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn export_import_roundtrip_preserves_rows() {
        let (_dir, repo) = repo().await;
        let manifest = repo.export_company("c1").await.unwrap();
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.company_id, "c1");

        let summary = repo
            .import_company("c2", manifest.clone(), ImportStrategy::Skip)
            .await
            .unwrap();
        assert!(summary.imported >= 6, "{summary:?}");
        assert_eq!(summary.failed, 0);

        // Row counts match, and ids were re-minted so both companies coexist.
        let exported_again = repo.export_company("c2").await.unwrap();
        let count = |tables: &[ManifestTable], name: &str| {
            tables
                .iter()
                .find(|table| table.name == name)
                .map(|table| table.rows.len())
                .unwrap_or(0)
        };
        for table in CORE_TABLES {
            assert_eq!(
                count(&exported_again.tables, table),
                count(&manifest.tables, table),
                "table {table} row count mismatch"
            );
        }
        let c1_agents = repo.export_company("c1").await.unwrap();
        let c2_agents = repo.export_company("c2").await.unwrap();
        let agent_ids = |tables: &[ManifestTable]| -> Vec<String> {
            tables
                .iter()
                .find(|table| table.name == "agents")
                .map(|table| {
                    table
                        .rows
                        .iter()
                        .filter_map(|row| row.get("id").and_then(serde_json::Value::as_str))
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        };
        let ids1 = agent_ids(&c1_agents.tables);
        let ids2 = agent_ids(&c2_agents.tables);
        assert!(!ids1.is_empty());
        for id in &ids2 {
            assert!(!ids1.contains(id), "id {id} must be re-minted");
        }

        // Skip: importing again is rejected because the target is not empty.
        assert!(matches!(
            repo.import_company("c2", manifest, ImportStrategy::Skip)
                .await
                .unwrap_err(),
            PortabilityError::CompanyNotEmpty
        ));
    }

    #[tokio::test]
    async fn overwrite_replaces_existing_rows() {
        let (_dir, repo) = repo().await;
        let manifest = repo.export_company("c1").await.unwrap();
        repo.import_company("c2", manifest.clone(), ImportStrategy::Skip)
            .await
            .unwrap();
        let before = repo.export_company("c2").await.unwrap();
        let count = |tables: &[ManifestTable], name: &str| {
            tables
                .iter()
                .find(|table| table.name == name)
                .map(|table| table.rows.len())
                .unwrap_or(0)
        };
        let before_issues = count(&before.tables, "issues");

        // Overwrite clears and re-imports (fresh ids again, same counts).
        let summary = repo
            .import_company("c2", manifest, ImportStrategy::Overwrite)
            .await
            .unwrap();
        assert!(summary.imported >= 6, "{summary:?}");
        assert_eq!(summary.failed, 0);
        let after = repo.export_company("c2").await.unwrap();
        assert_eq!(count(&after.tables, "issues"), before_issues);
    }

    #[tokio::test]
    async fn company_row_counts_reflect_existing_rows() {
        let (_dir, repo) = repo().await;
        let counts = repo.company_row_counts("c1").await.unwrap();
        assert_eq!(counts.get("agents"), Some(&1));
        assert_eq!(counts.get("issues"), Some(&1));
        assert_eq!(counts.get("issue_comments"), Some(&1));
        assert!(counts.get("projects").copied().unwrap_or(0) >= 1);
        let c2 = repo.company_row_counts("c2").await.unwrap();
        assert_eq!(c2.get("agents"), Some(&0));
        assert!(matches!(
            repo.company_row_counts("nope").await.unwrap_err(),
            PortabilityError::CompanyNotFound
        ));
    }

    #[tokio::test]
    async fn rejects_unknown_company_and_bad_manifest() {
        let (_dir, repo) = repo().await;
        let manifest = repo.export_company("c1").await.unwrap();
        assert!(matches!(
            repo.import_company("nope", manifest.clone(), ImportStrategy::Skip)
                .await
                .unwrap_err(),
            PortabilityError::CompanyNotFound
        ));
        let mut unknown = manifest;
        unknown.version = 99;
        assert!(matches!(
            repo.import_company("c2", unknown, ImportStrategy::Skip)
                .await
                .unwrap_err(),
            PortabilityError::InvalidManifest(_)
        ));
    }
}
