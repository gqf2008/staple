//! Issue work products repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `issue_work_products` table.
#[derive(Debug, Clone)]
pub struct WorkProductRecord {
    /// Work product id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Issue id.
    pub issue_id: String,
    /// Type.
    pub r#type: String,
    /// Provider.
    pub provider: String,
    /// External id.
    pub external_id: Option<String>,
    /// Title.
    pub title: String,
    /// URL.
    pub url: Option<String>,
    /// Status.
    pub status: String,
    /// Review state.
    pub review_state: String,
    /// Primary flag.
    pub is_primary: bool,
    /// Health status.
    pub health_status: String,
    /// Summary.
    pub summary: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating a work product.
#[derive(Debug, Clone)]
pub struct NewWorkProduct {
    /// Issue id.
    pub issue_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Type.
    pub r#type: String,
    /// Provider.
    pub provider: String,
    /// External id.
    pub external_id: Option<String>,
    /// Title.
    pub title: String,
    /// URL.
    pub url: Option<String>,
    /// Status.
    pub status: String,
    /// Review state.
    pub review_state: String,
    /// Primary flag.
    pub is_primary: bool,
    /// Health status.
    pub health_status: String,
    /// Summary.
    pub summary: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<String>,
}

/// Partial work product update.
#[derive(Debug, Default)]
pub struct WorkProductPatch {
    /// New title.
    pub title: Option<String>,
    /// New status.
    pub status: Option<String>,
    /// New review state.
    pub review_state: Option<String>,
    /// New health status.
    pub health_status: Option<String>,
    /// New summary.
    pub summary: Option<Option<String>>,
}

/// Work product repository errors.
#[derive(Debug, Error)]
pub enum WorkProductError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The issue does not exist.
    #[error("issue not found")]
    IssueNotFound,
    /// The project belongs to a different company.
    #[error("project belongs to a different company")]
    ProjectInDifferentCompany,
}

/// Work product persistence contract.
#[async_trait]
pub trait WorkProductRepository: Send + Sync {
    /// Creates a work product for an issue.
    ///
    /// # Errors
    ///
    /// Returns [`WorkProductError`] when the issue is missing.
    async fn create(&self, input: NewWorkProduct) -> Result<WorkProductRecord, WorkProductError>;

    /// Lists the work products of an issue.
    ///
    /// # Errors
    ///
    /// Returns [`WorkProductError`] on database failure.
    async fn list_for_issue(
        &self,
        issue_id: &str,
    ) -> Result<Vec<WorkProductRecord>, WorkProductError>;

    /// Lists all work products for a company, newest first.
    ///
    /// # Errors
    ///
    /// Returns [`WorkProductError`] on database failure.
    async fn list_for_company(
        &self,
        company_id: &str,
    ) -> Result<Vec<WorkProductRecord>, WorkProductError>;

    /// Fetches one work product by id.
    ///
    /// # Errors
    ///
    /// Returns [`WorkProductError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<WorkProductRecord>, WorkProductError>;

    /// Applies a partial update.
    ///
    /// # Errors
    ///
    /// Returns [`WorkProductError`] on database failure.
    async fn update(
        &self,
        id: &str,
        patch: WorkProductPatch,
    ) -> Result<Option<WorkProductRecord>, WorkProductError>;

    /// Deletes a work product.
    ///
    /// # Errors
    ///
    /// Returns [`WorkProductError`] on database failure.
    async fn delete(&self, id: &str) -> Result<Option<WorkProductRecord>, WorkProductError>;
}

/// Turso/libSQL implementation of [`WorkProductRepository`].
#[derive(Debug)]
pub struct TursoWorkProductRepository {
    db: Database,
}

impl TursoWorkProductRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const COLUMNS: &str = "id, company_id, project_id, issue_id, type, provider, external_id,
    title, url, status, review_state, is_primary, health_status, summary, metadata,
    created_at, updated_at";

fn row_to_work_product(row: &libsql::Row) -> Result<WorkProductRecord, libsql::Error> {
    Ok(WorkProductRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        project_id: helpers::row_text(row, 2)?,
        issue_id: helpers::row_text(row, 3)?.expect("issue_id is NOT NULL"),
        r#type: helpers::row_text(row, 4)?.expect("type is NOT NULL"),
        provider: helpers::row_text(row, 5)?.expect("provider is NOT NULL"),
        external_id: helpers::row_text(row, 6)?,
        title: helpers::row_text(row, 7)?.expect("title is NOT NULL"),
        url: helpers::row_text(row, 8)?,
        status: helpers::row_text(row, 9)?.expect("status is NOT NULL"),
        review_state: helpers::row_text(row, 10)?.expect("review_state is NOT NULL"),
        is_primary: helpers::row_i64(row, 11)? != 0,
        health_status: helpers::row_text(row, 12)?.expect("health_status is NOT NULL"),
        summary: helpers::row_text(row, 13)?,
        metadata: helpers::row_text(row, 14)?,
        created_at: helpers::row_text(row, 15)?.expect("created_at is NOT NULL"),
        updated_at: helpers::row_text(row, 16)?.expect("updated_at is NOT NULL"),
    })
}

#[async_trait]
impl WorkProductRepository for TursoWorkProductRepository {
    async fn create(&self, input: NewWorkProduct) -> Result<WorkProductRecord, WorkProductError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(company_id) = helpers::row_company(&conn, "issues", &input.issue_id).await? else {
            return Err(WorkProductError::IssueNotFound);
        };
        if let Some(project_id) = &input.project_id
            && !helpers::row_belongs_to_company(&conn, "projects", project_id, &company_id).await?
        {
            return Err(WorkProductError::ProjectInDifferentCompany);
        }

        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO issue_work_products (id, company_id, project_id, issue_id, type,
                                              provider, external_id, title, url, status,
                                              review_state, is_primary, health_status, summary,
                                              metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                company_id,
                input.project_id,
                input.issue_id,
                input.r#type,
                input.provider,
                input.external_id,
                input.title,
                input.url,
                input.status,
                input.review_state,
                i64::from(input.is_primary),
                input.health_status,
                input.summary,
                input.metadata
            ],
        )
        .await?;
        Ok(self
            .get(&id)
            .await?
            .expect("work product was just inserted"))
    }

    async fn list_for_issue(
        &self,
        issue_id: &str,
    ) -> Result<Vec<WorkProductRecord>, WorkProductError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {COLUMNS} FROM issue_work_products WHERE issue_id = ?1 ORDER BY created_at"
        );
        let mut rows = conn.query(&sql, libsql::params![issue_id]).await?;
        let mut products = Vec::new();
        while let Some(row) = rows.next().await? {
            products.push(row_to_work_product(&row)?);
        }
        Ok(products)
    }

    async fn list_for_company(
        &self,
        company_id: &str,
    ) -> Result<Vec<WorkProductRecord>, WorkProductError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {COLUMNS} FROM issue_work_products WHERE company_id = ?1 \
             ORDER BY created_at DESC"
        );
        let mut rows = conn.query(&sql, libsql::params![company_id]).await?;
        let mut products = Vec::new();
        while let Some(row) = rows.next().await? {
            products.push(row_to_work_product(&row)?);
        }
        Ok(products)
    }

    async fn get(&self, id: &str) -> Result<Option<WorkProductRecord>, WorkProductError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {COLUMNS} FROM issue_work_products WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_work_product(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        id: &str,
        patch: WorkProductPatch,
    ) -> Result<Option<WorkProductRecord>, WorkProductError> {
        let conn = crate::connection::connect(&self.db).await?;
        let (sets, values) = helpers::build_update(&[
            ("title", patch.title.map(|value| Some(value.into()))),
            ("status", patch.status.map(|value| Some(value.into()))),
            (
                "review_state",
                patch.review_state.map(|value| Some(value.into())),
            ),
            (
                "health_status",
                patch.health_status.map(|value| Some(value.into())),
            ),
            ("summary", patch.summary.map(|value| value.map(Into::into))),
        ]);
        if sets.is_empty() {
            return self.get(id).await;
        }
        let updated =
            helpers::execute_update(&conn, "issue_work_products", id, sets, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        self.get(id).await
    }

    async fn delete(&self, id: &str) -> Result<Option<WorkProductRecord>, WorkProductError> {
        let conn = crate::connection::connect(&self.db).await?;
        let product = self.get(id).await?;
        let Some(product) = product else {
            return Ok(None);
        };
        conn.execute(
            "DELETE FROM issue_work_products WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(product))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoWorkProductRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoWorkProductRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn create_list_get_update_delete_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'T', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();

        let created = repo
            .create(NewWorkProduct {
                issue_id: "i1".to_owned(),
                project_id: None,
                r#type: "artifact".to_owned(),
                provider: "paperclip".to_owned(),
                external_id: None,
                title: "Report".to_owned(),
                url: None,
                status: "active".to_owned(),
                review_state: "none".to_owned(),
                is_primary: true,
                health_status: "unknown".to_owned(),
                summary: Some("summary".to_owned()),
                metadata: Some(r#"{"kind":"workspace_file"}"#.to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(created.title, "Report");
        assert!(created.is_primary);

        let list = repo.list_for_issue("i1").await.unwrap();
        assert_eq!(list.len(), 1);

        let updated = repo
            .update(
                &created.id,
                WorkProductPatch {
                    status: Some("archived".to_owned()),
                    summary: Some(None),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "archived");
        assert_eq!(updated.summary, None);

        let deleted = repo.delete(&created.id).await.unwrap().unwrap();
        assert_eq!(deleted.id, created.id);
    }

    #[tokio::test]
    async fn list_for_company_scopes_to_company() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024),
                    ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'T1', 1, 'ALPHA-1'),
                    ('i2', 'c2', 'T2', 1, 'BETA-1')",
            (),
        )
        .await
        .unwrap();
        for (issue_id, title) in [
            ("i1", "Alpha artifact"),
            ("i1", "Alpha second"),
            ("i2", "Beta artifact"),
        ] {
            repo.create(NewWorkProduct {
                issue_id: issue_id.to_owned(),
                project_id: None,
                r#type: "artifact".to_owned(),
                provider: "paperclip".to_owned(),
                external_id: None,
                title: title.to_owned(),
                url: None,
                status: "active".to_owned(),
                review_state: "none".to_owned(),
                is_primary: false,
                health_status: "unknown".to_owned(),
                summary: None,
                metadata: None,
            })
            .await
            .unwrap();
        }

        let alpha = repo.list_for_company("c1").await.unwrap();
        assert_eq!(alpha.len(), 2);
        assert!(alpha.iter().all(|product| product.company_id == "c1"));
        assert!(
            alpha
                .iter()
                .any(|product| product.title == "Alpha artifact")
        );
        let beta = repo.list_for_company("c2").await.unwrap();
        assert_eq!(beta.len(), 1);
        assert_eq!(beta[0].title, "Beta artifact");
        assert!(repo.list_for_company("missing").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn create_requires_issue() {
        let (_dir, repo, _conn) = repo().await;
        let error = repo
            .create(NewWorkProduct {
                issue_id: "missing".to_owned(),
                project_id: None,
                r#type: "artifact".to_owned(),
                provider: "paperclip".to_owned(),
                external_id: None,
                title: "T".to_owned(),
                url: None,
                status: "active".to_owned(),
                review_state: "none".to_owned(),
                is_primary: false,
                health_status: "unknown".to_owned(),
                summary: None,
                metadata: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, WorkProductError::IssueNotFound));
    }
}
