//! Activity log repository: the audit trail for mutating actions.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `activity_log` table.
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    /// Entry id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// `agent | user | system`.
    pub actor_type: String,
    /// Actor id.
    pub actor_id: String,
    /// Action name (e.g. `company.created`).
    pub action: String,
    /// Entity type.
    pub entity_type: String,
    /// Entity id.
    pub entity_id: String,
    /// Agent id (agent actors).
    pub agent_id: Option<String>,
    /// Heartbeat run id.
    pub run_id: Option<String>,
    /// Responsible user id.
    pub responsible_user_id: Option<String>,
    /// Details JSON.
    pub details: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for writing an audit entry.
#[derive(Debug, Clone)]
pub struct NewActivity {
    /// Owning company id.
    pub company_id: String,
    /// `agent | user | system`.
    pub actor_type: String,
    /// Actor id.
    pub actor_id: String,
    /// Action name.
    pub action: String,
    /// Entity type.
    pub entity_type: String,
    /// Entity id.
    pub entity_id: String,
    /// Details JSON.
    pub details: Option<String>,
}

/// Activity repository errors.
#[derive(Debug, Error)]
pub enum ActivityError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
}

/// Audit trail persistence contract.
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    /// Appends an audit entry.
    ///
    /// # Errors
    ///
    /// Returns [`ActivityError`] on database failure.
    async fn log(&self, input: NewActivity) -> Result<ActivityEntry, ActivityError>;

    /// Lists audit entries for a company, newest first.
    ///
    /// # Errors
    ///
    /// Returns [`ActivityError`] on database failure.
    async fn list(&self, company_id: &str, limit: i64)
    -> Result<Vec<ActivityEntry>, ActivityError>;
}

/// Turso/libSQL implementation of [`ActivityRepository`].
#[derive(Debug)]
pub struct TursoActivityRepository {
    db: Database,
}

impl TursoActivityRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const COLUMNS: &str = "id, company_id, actor_type, actor_id, action, entity_type, entity_id,
    agent_id, run_id, responsible_user_id, details, created_at";

fn row_to_entry(row: &libsql::Row) -> Result<ActivityEntry, libsql::Error> {
    Ok(ActivityEntry {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        actor_type: helpers::row_text(row, 2)?.expect("actor_type is NOT NULL"),
        actor_id: helpers::row_text(row, 3)?.expect("actor_id is NOT NULL"),
        action: helpers::row_text(row, 4)?.expect("action is NOT NULL"),
        entity_type: helpers::row_text(row, 5)?.expect("entity_type is NOT NULL"),
        entity_id: helpers::row_text(row, 6)?.expect("entity_id is NOT NULL"),
        agent_id: helpers::row_text(row, 7)?,
        run_id: helpers::row_text(row, 8)?,
        responsible_user_id: helpers::row_text(row, 9)?,
        details: helpers::row_text(row, 10)?,
        created_at: helpers::row_text(row, 11)?.expect("created_at is NOT NULL"),
    })
}

#[async_trait]
impl ActivityRepository for TursoActivityRepository {
    async fn log(&self, input: NewActivity) -> Result<ActivityEntry, ActivityError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO activity_log (id, company_id, actor_type, actor_id, action,
                                       entity_type, entity_id, details, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.actor_type,
                input.actor_id,
                input.action,
                input.entity_type,
                input.entity_id,
                input.details
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {COLUMNS} FROM activity_log WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("entry was just inserted");
        Ok(row_to_entry(&row)?)
    }

    async fn list(
        &self,
        company_id: &str,
        limit: i64,
    ) -> Result<Vec<ActivityEntry>, ActivityError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {COLUMNS} FROM activity_log WHERE company_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        );
        let mut rows = conn.query(&sql, libsql::params![company_id, limit]).await?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next().await? {
            entries.push(row_to_entry(&row)?);
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoActivityRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoActivityRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn log_and_list_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        let entry = repo
            .log(NewActivity {
                company_id: "c1".to_owned(),
                actor_type: "user".to_owned(),
                actor_id: "board".to_owned(),
                action: "company.created".to_owned(),
                entity_type: "company".to_owned(),
                entity_id: "c1".to_owned(),
                details: Some(r#"{"name":"Alpha"}"#.to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(entry.action, "company.created");

        let entries = repo.list("c1", 10).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].details.as_deref(), Some(r#"{"name":"Alpha"}"#));

        let empty = repo.list("other", 10).await.unwrap();
        assert!(empty.is_empty());
    }
}
