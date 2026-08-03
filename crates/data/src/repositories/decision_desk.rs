//! Decision desk: queues, queue items, and triage.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A decision queue.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionQueueRecord {
    /// Queue id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Queue name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Retention override in days.
    pub retention_days: Option<i64>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A queue item.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionQueueItemRecord {
    /// Item id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Queue id.
    pub queue_id: String,
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Payload JSON.
    pub payload: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for upserting triage state.
#[derive(Debug, Clone)]
pub struct TriageInput {
    /// Decide-by time.
    pub decide_by: Option<String>,
    /// Snoozed until time.
    pub snoozed_until: Option<String>,
    /// Decision.
    pub decision: Option<String>,
    /// Deciding user.
    pub decided_by_user_id: Option<String>,
}

/// Triage state for an attention source.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTriageRecord {
    /// Triage id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Source kind.
    pub source_kind: String,
    /// Source id.
    pub source_id: String,
    /// Decide-by time.
    pub decide_by: Option<String>,
    /// Snoozed until time.
    pub snoozed_until: Option<String>,
    /// Decision.
    pub decision: Option<String>,
    /// Deciding user.
    pub decided_by_user_id: Option<String>,
}

/// Decision desk errors.
#[derive(Debug, Error)]
pub enum DecisionError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The queue does not exist in this company.
    #[error("queue not found")]
    QueueNotFound,
    /// The queue name is already taken.
    #[error("queue already exists")]
    QueueExists,
    /// The item is already in the queue.
    #[error("queue item already exists")]
    ItemExists,
}

/// Decision desk persistence contract.
#[async_trait]
pub trait DecisionRepository: Send + Sync {
    /// Creates a queue.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on invalid references or duplicate names.
    async fn create_queue(
        &self,
        company_id: &str,
        name: &str,
        description: Option<String>,
        retention_days: Option<i64>,
    ) -> Result<DecisionQueueRecord, DecisionError>;

    /// Lists queues.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_queues(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionQueueRecord>, DecisionError>;

    /// Adds an item to a queue.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on invalid references or duplicates.
    async fn add_item(
        &self,
        company_id: &str,
        queue_id: &str,
        source_kind: &str,
        source_id: &str,
        payload: Option<String>,
    ) -> Result<DecisionQueueItemRecord, DecisionError>;

    /// Lists queue items.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_items(
        &self,
        company_id: &str,
        queue_id: &str,
    ) -> Result<Vec<DecisionQueueItemRecord>, DecisionError>;

    /// Upserts triage state for an attention source.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn set_triage(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        input: TriageInput,
    ) -> Result<DecisionTriageRecord, DecisionError>;

    /// Lists triage state for a company.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] on database failure.
    async fn list_triage(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionTriageRecord>, DecisionError>;
}

/// Turso/libSQL implementation of [`DecisionRepository`].
#[derive(Debug)]
pub struct TursoDecisionRepository {
    db: Database,
}

impl TursoDecisionRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_opt_i64(row: &libsql::Row, idx: i32) -> Result<Option<i64>, libsql::Error> {
    let value = row.get_value(idx)?;
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(*value.as_integer().expect("INTEGER column")))
    }
}

#[async_trait]
impl DecisionRepository for TursoDecisionRepository {
    async fn create_queue(
        &self,
        company_id: &str,
        name: &str,
        description: Option<String>,
        retention_days: Option<i64>,
    ) -> Result<DecisionQueueRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(DecisionError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO decision_queues (id, company_id, name, description, retention_days,
                                              created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, name, description, retention_days],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, name, description, retention_days, created_at
                         FROM decision_queues WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("queue was just inserted");
                Ok(DecisionQueueRecord {
                    id: helpers::row_text(&row, 0)?.expect("id"),
                    company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                    name: helpers::row_text(&row, 2)?.expect("name"),
                    description: helpers::row_text(&row, 3)?,
                    retention_days: row_opt_i64(&row, 4)?,
                    created_at: helpers::row_text(&row, 5)?.expect("created_at"),
                })
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionError::QueueExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_queues(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionQueueRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, description, retention_days, created_at
                 FROM decision_queues WHERE company_id = ?1 ORDER BY name",
                libsql::params![company_id],
            )
            .await?;
        let mut queues = Vec::new();
        while let Some(row) = rows.next().await? {
            queues.push(DecisionQueueRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                name: helpers::row_text(&row, 2)?.expect("name"),
                description: helpers::row_text(&row, 3)?,
                retention_days: row_opt_i64(&row, 4)?,
                created_at: helpers::row_text(&row, 5)?.expect("created_at"),
            });
        }
        Ok(queues)
    }

    async fn add_item(
        &self,
        company_id: &str,
        queue_id: &str,
        source_kind: &str,
        source_id: &str,
        payload: Option<String>,
    ) -> Result<DecisionQueueItemRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "decision_queues", queue_id, company_id).await? {
            return Err(DecisionError::QueueNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO decision_queue_items (id, company_id, queue_id, source_kind,
                                                   source_id, payload, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    company_id,
                    queue_id,
                    source_kind,
                    source_id,
                    payload
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, queue_id, source_kind, source_id, payload, created_at
                         FROM decision_queue_items WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("item was just inserted");
                Ok(DecisionQueueItemRecord {
                    id: helpers::row_text(&row, 0)?.expect("id"),
                    company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                    queue_id: helpers::row_text(&row, 2)?.expect("queue_id"),
                    source_kind: helpers::row_text(&row, 3)?.expect("source_kind"),
                    source_id: helpers::row_text(&row, 4)?.expect("source_id"),
                    payload: helpers::row_text(&row, 5)?,
                    created_at: helpers::row_text(&row, 6)?.expect("created_at"),
                })
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionError::ItemExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_items(
        &self,
        company_id: &str,
        queue_id: &str,
    ) -> Result<Vec<DecisionQueueItemRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, queue_id, source_kind, source_id, payload, created_at
                 FROM decision_queue_items WHERE company_id = ?1 AND queue_id = ?2 ORDER BY created_at",
                libsql::params![company_id, queue_id],
            )
            .await?;
        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push(DecisionQueueItemRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                queue_id: helpers::row_text(&row, 2)?.expect("queue_id"),
                source_kind: helpers::row_text(&row, 3)?.expect("source_kind"),
                source_id: helpers::row_text(&row, 4)?.expect("source_id"),
                payload: helpers::row_text(&row, 5)?,
                created_at: helpers::row_text(&row, 6)?.expect("created_at"),
            });
        }
        Ok(items)
    }

    async fn set_triage(
        &self,
        company_id: &str,
        source_kind: &str,
        source_id: &str,
        input: TriageInput,
    ) -> Result<DecisionTriageRecord, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        conn.execute(
            "INSERT INTO decision_triage (id, company_id, source_kind, source_id, decide_by,
                                          snoozed_until, decision, decided_by_user_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, source_kind, source_id) DO UPDATE SET
                 decide_by = excluded.decide_by,
                 snoozed_until = excluded.snoozed_until,
                 decision = excluded.decision,
                 decided_by_user_id = excluded.decided_by_user_id,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                Uuid::new_v4().to_string(),
                company_id,
                source_kind,
                source_id,
                input.decide_by,
                input.snoozed_until,
                input.decision,
                input.decided_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, source_kind, source_id, decide_by, snoozed_until,
                        decision, decided_by_user_id
                 FROM decision_triage WHERE company_id = ?1 AND source_kind = ?2 AND source_id = ?3",
                libsql::params![company_id, source_kind, source_id],
            )
            .await?;
        let row = rows.next().await?.expect("triage exists");
        Ok(DecisionTriageRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            source_kind: helpers::row_text(&row, 2)?.expect("source_kind"),
            source_id: helpers::row_text(&row, 3)?.expect("source_id"),
            decide_by: helpers::row_text(&row, 4)?,
            snoozed_until: helpers::row_text(&row, 5)?,
            decision: helpers::row_text(&row, 6)?,
            decided_by_user_id: helpers::row_text(&row, 7)?,
        })
    }

    async fn list_triage(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionTriageRecord>, DecisionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, source_kind, source_id, decide_by, snoozed_until,
                        decision, decided_by_user_id
                 FROM decision_triage WHERE company_id = ?1 ORDER BY updated_at DESC",
                libsql::params![company_id],
            )
            .await?;
        let mut triage = Vec::new();
        while let Some(row) = rows.next().await? {
            triage.push(DecisionTriageRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                source_kind: helpers::row_text(&row, 2)?.expect("source_kind"),
                source_id: helpers::row_text(&row, 3)?.expect("source_id"),
                decide_by: helpers::row_text(&row, 4)?,
                snoozed_until: helpers::row_text(&row, 5)?,
                decision: helpers::row_text(&row, 6)?,
                decided_by_user_id: helpers::row_text(&row, 7)?,
            });
        }
        Ok(triage)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoDecisionRepository) {
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
        let repo = TursoDecisionRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn queue_item_triage_roundtrip() {
        let (_dir, repo) = repo().await;
        let queue = repo
            .create_queue(
                "c1",
                "approvals",
                Some("pending approvals".to_owned()),
                Some(30),
            )
            .await
            .unwrap();
        assert_eq!(queue.name, "approvals");

        // Duplicate queue name rejected.
        let error = repo
            .create_queue("c1", "approvals", None, None)
            .await
            .unwrap_err();
        assert!(matches!(error, DecisionError::QueueExists));

        // Items.
        let item = repo
            .add_item("c1", &queue.id, "approval", "a1", Some("{}".to_owned()))
            .await
            .unwrap();
        assert_eq!(item.source_id, "a1");
        let error = repo
            .add_item("c1", &queue.id, "approval", "a1", None)
            .await
            .unwrap_err();
        assert!(matches!(error, DecisionError::ItemExists));
        let items = repo.list_items("c1", &queue.id).await.unwrap();
        assert_eq!(items.len(), 1);

        // Triage upsert.
        let triage = repo
            .set_triage(
                "c1",
                "approval",
                "a1",
                TriageInput {
                    decide_by: Some("2026-08-10T00:00:00Z".to_owned()),
                    snoozed_until: None,
                    decision: Some("approved".to_owned()),
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap();
        assert_eq!(triage.decision.as_deref(), Some("approved"));
        let triage2 = repo
            .set_triage(
                "c1",
                "approval",
                "a1",
                TriageInput {
                    decide_by: None,
                    snoozed_until: None,
                    decision: Some("rejected".to_owned()),
                    decided_by_user_id: Some("board".to_owned()),
                },
            )
            .await
            .unwrap();
        assert_eq!(triage2.decision.as_deref(), Some("rejected"));
        let all = repo.list_triage("c1").await.unwrap();
        assert_eq!(all.len(), 1);
    }
}
