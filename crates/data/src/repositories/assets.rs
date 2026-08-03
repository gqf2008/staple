//! Assets and issue attachments repository (provider-backed object metadata).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `assets` table.
#[derive(Debug, Clone)]
pub struct AssetRecord {
    /// Asset id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Storage provider (`local_disk | s3`).
    pub provider: String,
    /// Unique object key within the company.
    pub object_key: String,
    /// Content type.
    pub content_type: String,
    /// Size in bytes.
    pub byte_size: i64,
    /// SHA-256 of the content.
    pub sha256: String,
    /// Original filename.
    pub original_filename: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A row of the `issue_attachments` table.
#[derive(Debug, Clone)]
pub struct IssueAttachmentRecord {
    /// Attachment id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Asset id.
    pub asset_id: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for registering an uploaded asset.
#[derive(Debug, Clone)]
pub struct NewAsset {
    /// Owning company id.
    pub company_id: String,
    /// Storage provider.
    pub provider: String,
    /// Object key (unique per company).
    pub object_key: String,
    /// Content type.
    pub content_type: String,
    /// Size in bytes.
    pub byte_size: i64,
    /// SHA-256 of the content.
    pub sha256: String,
    /// Original filename.
    pub original_filename: Option<String>,
}

/// Input for linking an asset to an issue.
#[derive(Debug, Clone)]
pub struct NewIssueAttachment {
    /// Issue id.
    pub issue_id: String,
    /// Asset id.
    pub asset_id: String,
}

/// Asset repository errors.
#[derive(Debug, Error)]
pub enum AssetError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The issue does not exist or belongs to another company.
    #[error("issue not found")]
    IssueNotFound,
    /// The asset does not exist or belongs to another company.
    #[error("asset not found")]
    AssetNotFound,
    /// The asset was already attached to the issue.
    #[error("attachment already exists")]
    AttachmentExists,
}

/// Asset persistence contract.
#[async_trait]
pub trait AssetRepository: Send + Sync {
    /// Registers an asset.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] on database failure.
    async fn create_asset(&self, input: NewAsset) -> Result<AssetRecord, AssetError>;

    /// Fetches an asset by id.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] on database failure.
    async fn get_asset(&self, id: &str) -> Result<Option<AssetRecord>, AssetError>;

    /// Links an asset to an issue.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] when the issue or asset is missing or belongs
    /// to a different company.
    async fn create_issue_attachment(
        &self,
        input: NewIssueAttachment,
    ) -> Result<IssueAttachmentRecord, AssetError>;

    /// Lists the attachments of an issue.
    ///
    /// # Errors
    ///
    /// Returns [`AssetError`] on database failure.
    async fn list_issue_attachments(
        &self,
        issue_id: &str,
    ) -> Result<Vec<IssueAttachmentRecord>, AssetError>;
}

/// Turso/libSQL implementation of [`AssetRepository`].
#[derive(Debug)]
pub struct TursoAssetRepository {
    db: Database,
}

impl TursoAssetRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const ASSET_COLUMNS: &str = "id, company_id, provider, object_key, content_type, byte_size,
    sha256, original_filename, created_at";

fn row_to_asset(row: &libsql::Row) -> Result<AssetRecord, libsql::Error> {
    Ok(AssetRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        provider: helpers::row_text(row, 2)?.expect("provider is NOT NULL"),
        object_key: helpers::row_text(row, 3)?.expect("object_key is NOT NULL"),
        content_type: helpers::row_text(row, 4)?.expect("content_type is NOT NULL"),
        byte_size: helpers::row_i64(row, 5)?,
        sha256: helpers::row_text(row, 6)?.expect("sha256 is NOT NULL"),
        original_filename: helpers::row_text(row, 7)?,
        created_at: helpers::row_text(row, 8)?.expect("created_at is NOT NULL"),
    })
}

const ATTACHMENT_COLUMNS: &str = "id, company_id, issue_id, asset_id, created_at";

fn row_to_attachment(row: &libsql::Row) -> Result<IssueAttachmentRecord, libsql::Error> {
    Ok(IssueAttachmentRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id is NOT NULL"),
        asset_id: helpers::row_text(row, 3)?.expect("asset_id is NOT NULL"),
        created_at: helpers::row_text(row, 4)?.expect("created_at is NOT NULL"),
    })
}

#[async_trait]
impl AssetRepository for TursoAssetRepository {
    async fn create_asset(&self, input: NewAsset) -> Result<AssetRecord, AssetError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO assets (id, company_id, provider, object_key, content_type,
                                 byte_size, sha256, original_filename, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.provider,
                input.object_key,
                input.content_type,
                input.byte_size,
                input.sha256,
                input.original_filename
            ],
        )
        .await?;
        Ok(self.get_asset(&id).await?.expect("asset was just inserted"))
    }

    async fn get_asset(&self, id: &str) -> Result<Option<AssetRecord>, AssetError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {ASSET_COLUMNS} FROM assets WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_asset(&row)?)),
            None => Ok(None),
        }
    }

    async fn create_issue_attachment(
        &self,
        input: NewIssueAttachment,
    ) -> Result<IssueAttachmentRecord, AssetError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(company_id) = helpers::row_company(&conn, "issues", &input.issue_id).await? else {
            return Err(AssetError::IssueNotFound);
        };
        if !helpers::row_belongs_to_company(&conn, "assets", &input.asset_id, &company_id).await? {
            return Err(AssetError::AssetNotFound);
        }

        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_attachments (id, company_id, issue_id, asset_id,
                                                created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![id.clone(), company_id, input.issue_id, input.asset_id],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {ATTACHMENT_COLUMNS} FROM issue_attachments WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("attachment was just inserted");
                Ok(row_to_attachment(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(AssetError::AttachmentExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_issue_attachments(
        &self,
        issue_id: &str,
    ) -> Result<Vec<IssueAttachmentRecord>, AssetError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {ATTACHMENT_COLUMNS} FROM issue_attachments WHERE issue_id = ?1 ORDER BY created_at"
        );
        let mut rows = conn.query(&sql, libsql::params![issue_id]).await?;
        let mut attachments = Vec::new();
        while let Some(row) = rows.next().await? {
            attachments.push(row_to_attachment(&row)?);
        }
        Ok(attachments)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoAssetRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoAssetRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn asset_and_attachment_roundtrip() {
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

        let asset = repo
            .create_asset(NewAsset {
                company_id: "c1".to_owned(),
                provider: "local_disk".to_owned(),
                object_key: "abc.txt".to_owned(),
                content_type: "text/plain".to_owned(),
                byte_size: 4,
                sha256: "deadbeef".to_owned(),
                original_filename: Some("abc.txt".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(asset.object_key, "abc.txt");

        let attachment = repo
            .create_issue_attachment(NewIssueAttachment {
                issue_id: "i1".to_owned(),
                asset_id: asset.id.clone(),
            })
            .await
            .unwrap();
        assert_eq!(attachment.asset_id, asset.id);

        let list = repo.list_issue_attachments("i1").await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn attachment_requires_same_company() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
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
        // Asset belongs to a different company than the issue.
        let asset = repo
            .create_asset(NewAsset {
                company_id: "c2".to_owned(),
                provider: "local_disk".to_owned(),
                object_key: "k".to_owned(),
                content_type: "text/plain".to_owned(),
                byte_size: 1,
                sha256: "s".to_owned(),
                original_filename: None,
            })
            .await
            .unwrap();
        let error = repo
            .create_issue_attachment(NewIssueAttachment {
                issue_id: "i1".to_owned(),
                asset_id: asset.id.clone(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, AssetError::AssetNotFound));
    }
}
