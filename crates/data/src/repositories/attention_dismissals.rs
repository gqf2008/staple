//! Attention inbox dismissals/snoozes.
//!
//! Mirrors the upstream `inbox_dismissals` table surface used by the
//! issue-based attention feed: one row per `(company_id, user_id, item_key)`
//! holding either a permanent `dismiss` or a `snooze` with a future
//! `snoozed_until`. Rows are company- and user-scoped at the SQL level.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// One inbox dismissal/snooze row.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DismissalRecord {
    /// Row id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Board user the dismissal belongs to.
    pub user_id: String,
    /// Attention item key (`{sourceKind}:{dedupKey}`).
    pub item_key: String,
    /// `dismiss` or `snooze`.
    pub kind: String,
    /// ISO 8601 time the item was dismissed/snoozed.
    pub dismissed_at: String,
    /// ISO 8601 time a snooze becomes visible again (`None` for dismiss).
    pub snoozed_until: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for upserting an inbox dismissal/snooze.
#[derive(Debug, Clone)]
pub struct NewDismissal {
    /// Owning company id.
    pub company_id: String,
    /// Board user the dismissal belongs to.
    pub user_id: String,
    /// Attention item key (`{sourceKind}:{dedupKey}`).
    pub item_key: String,
    /// `dismiss` or `snooze`.
    pub kind: String,
    /// ISO 8601 time a snooze becomes visible again (`None` for dismiss).
    pub snoozed_until: Option<String>,
}

/// Dismissal repository errors.
#[derive(Debug, Error)]
pub enum DismissalError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
}

/// Attention dismissal persistence contract.
#[async_trait]
pub trait AttentionDismissalRepository: Send + Sync {
    /// Lists dismissals/snoozes for one company + user, newest first.
    ///
    /// # Errors
    ///
    /// Returns [`DismissalError`] on database failure.
    async fn list(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Vec<DismissalRecord>, DismissalError>;

    /// Creates or replaces the dismissal for `(company_id, user_id,
    /// item_key)`. On a unique-key conflict the row's `kind`,
    /// `snoozed_until`, `dismissed_at`, and `updated_at` are replaced.
    ///
    /// # Errors
    ///
    /// Returns [`DismissalError`] on database failure (including a missing
    /// company, which violates the foreign key).
    async fn upsert(&self, input: NewDismissal) -> Result<DismissalRecord, DismissalError>;

    /// Deletes the dismissal for `(company_id, user_id, item_key)`, returning
    /// whether a row was removed.
    ///
    /// # Errors
    ///
    /// Returns [`DismissalError`] on database failure.
    async fn clear(
        &self,
        company_id: &str,
        user_id: &str,
        item_key: &str,
    ) -> Result<bool, DismissalError>;
}

/// Turso/libSQL implementation of [`AttentionDismissalRepository`].
#[derive(Debug)]
pub struct TursoAttentionDismissalRepository {
    db: Database,
}

impl TursoAttentionDismissalRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const COLUMNS: &str = "id, company_id, user_id, item_key, kind, dismissed_at, snoozed_until,
                       created_at, updated_at";

#[async_trait]
impl AttentionDismissalRepository for TursoAttentionDismissalRepository {
    async fn list(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Vec<DismissalRecord>, DismissalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {COLUMNS} FROM inbox_dismissals
                     WHERE company_id = ?1 AND user_id = ?2 ORDER BY updated_at DESC"
                ),
                libsql::params![company_id, user_id],
            )
            .await?;
        let mut dismissals = Vec::new();
        while let Some(row) = rows.next().await? {
            dismissals.push(row_to_dismissal(&row)?);
        }
        Ok(dismissals)
    }

    async fn upsert(&self, input: NewDismissal) -> Result<DismissalRecord, DismissalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_user_id = input.user_id.clone();
        let key_item_key = input.item_key.clone();
        conn.execute(
            &format!(
                "INSERT INTO inbox_dismissals
                   (id, company_id, user_id, item_key, kind, dismissed_at, snoozed_until,
                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, {now}, ?6, {now}, {now})
                 ON CONFLICT (company_id, user_id, item_key) DO UPDATE SET
                   kind = excluded.kind,
                   snoozed_until = excluded.snoozed_until,
                   dismissed_at = excluded.dismissed_at,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id,
                input.company_id,
                input.user_id,
                input.item_key,
                input.kind,
                input.snoozed_until
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {COLUMNS} FROM inbox_dismissals
                     WHERE company_id = ?1 AND user_id = ?2 AND item_key = ?3"
                ),
                libsql::params![key_company_id, key_user_id, key_item_key],
            )
            .await?;
        let row = rows.next().await?.expect("dismissal was just upserted");
        Ok(row_to_dismissal(&row)?)
    }

    async fn clear(
        &self,
        company_id: &str,
        user_id: &str,
        item_key: &str,
    ) -> Result<bool, DismissalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let deleted = conn
            .execute(
                "DELETE FROM inbox_dismissals
                 WHERE company_id = ?1 AND user_id = ?2 AND item_key = ?3",
                libsql::params![company_id, user_id, item_key],
            )
            .await?;
        Ok(deleted > 0)
    }
}

fn row_to_dismissal(row: &libsql::Row) -> Result<DismissalRecord, libsql::Error> {
    Ok(DismissalRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        user_id: helpers::row_text(row, 2)?.expect("user_id"),
        item_key: helpers::row_text(row, 3)?.expect("item_key"),
        kind: helpers::row_text(row, 4)?.expect("kind"),
        dismissed_at: helpers::row_text(row, 5)?.expect("dismissed_at"),
        snoozed_until: helpers::row_text(row, 6)?,
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
        updated_at: helpers::row_text(row, 8)?.expect("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoAttentionDismissalRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        let repo = TursoAttentionDismissalRepository::new(db);
        (dir, repo)
    }

    fn new_dismissal() -> NewDismissal {
        NewDismissal {
            company_id: "c1".to_owned(),
            user_id: "u1".to_owned(),
            item_key: "attention:issue-1".to_owned(),
            kind: "dismiss".to_owned(),
            snoozed_until: None,
        }
    }

    #[tokio::test]
    async fn upsert_overwrites_same_key_and_list_scopes_by_user() {
        let (_dir, repo) = repo().await;
        let dismissed = repo.upsert(new_dismissal()).await.unwrap();
        assert_eq!(dismissed.kind, "dismiss");
        assert!(dismissed.snoozed_until.is_none());
        assert_eq!(dismissed.company_id, "c1");
        assert_eq!(dismissed.user_id, "u1");

        // Upserting the same key as a snooze replaces the row (same id).
        let snoozed = repo
            .upsert(NewDismissal {
                kind: "snooze".to_owned(),
                snoozed_until: Some("2026-09-01T00:00:00.000Z".to_owned()),
                ..new_dismissal()
            })
            .await
            .unwrap();
        assert_eq!(snoozed.id, dismissed.id);
        assert_eq!(snoozed.kind, "snooze");
        assert_eq!(
            snoozed.snoozed_until.as_deref(),
            Some("2026-09-01T00:00:00.000Z")
        );

        let rows = repo.list("c1", "u1").await.unwrap();
        assert_eq!(rows.len(), 1, "upsert must not create duplicates");
        assert_eq!(rows[0].kind, "snooze");

        // Another user's list is empty.
        assert!(repo.list("c1", "u2").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_orders_by_updated_at_and_separates_users() {
        let (_dir, repo) = repo().await;
        repo.upsert(NewDismissal {
            item_key: "attention:first".to_owned(),
            ..new_dismissal()
        })
        .await
        .unwrap();
        repo.upsert(NewDismissal {
            item_key: "attention:second".to_owned(),
            ..new_dismissal()
        })
        .await
        .unwrap();
        repo.upsert(NewDismissal {
            user_id: "u2".to_owned(),
            ..new_dismissal()
        })
        .await
        .unwrap();

        let rows = repo.list("c1", "u1").await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].item_key, "attention:second");

        let other = repo.list("c1", "u2").await.unwrap();
        assert_eq!(other.len(), 1);
        assert_eq!(other[0].item_key, "attention:issue-1");
    }

    #[tokio::test]
    async fn clear_removes_only_the_scoped_row() {
        let (_dir, repo) = repo().await;
        repo.upsert(new_dismissal()).await.unwrap();
        repo.upsert(NewDismissal {
            company_id: "c2".to_owned(),
            ..new_dismissal()
        })
        .await
        .unwrap();

        assert!(repo.clear("c1", "u1", "attention:issue-1").await.unwrap());
        assert!(!repo.clear("c1", "u1", "attention:issue-1").await.unwrap());
        assert!(repo.list("c1", "u1").await.unwrap().is_empty());
        // The c2 row is untouched.
        assert_eq!(repo.list("c2", "u1").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn companies_are_isolated() {
        let (_dir, repo) = repo().await;
        repo.upsert(new_dismissal()).await.unwrap();
        repo.upsert(NewDismissal {
            company_id: "c2".to_owned(),
            item_key: "attention:other".to_owned(),
            ..new_dismissal()
        })
        .await
        .unwrap();

        let c1 = repo.list("c1", "u1").await.unwrap();
        assert_eq!(c1.len(), 1);
        assert_eq!(c1[0].item_key, "attention:issue-1");

        let c2 = repo.list("c2", "u1").await.unwrap();
        assert_eq!(c2.len(), 1);
        assert_eq!(c2[0].item_key, "attention:other");
    }
}
