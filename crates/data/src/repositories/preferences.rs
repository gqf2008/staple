//! Sidebar preferences and company logos repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `company_user_sidebar_preferences` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarPreferenceRecord {
    /// Preference id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// User id.
    pub user_id: String,
    /// Project order (JSON array of project ids).
    pub project_order: Vec<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `company_logos` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyLogoRecord {
    /// Logo id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Asset id.
    pub asset_id: String,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Preference repository errors.
#[derive(Debug, Error)]
pub enum PreferenceError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The asset does not exist in this company.
    #[error("asset not found")]
    AssetNotFound,
    /// The logo does not exist.
    #[error("logo not found")]
    LogoNotFound,
}

/// Preference persistence contract.
#[async_trait]
pub trait PreferenceRepository: Send + Sync {
    /// Gets sidebar preferences for a company + user.
    ///
    /// # Errors
    ///
    /// Returns [`PreferenceError`] on database failure.
    async fn sidebar_prefs(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Option<SidebarPreferenceRecord>, PreferenceError>;

    /// Upserts sidebar preferences for a company + user.
    ///
    /// # Errors
    ///
    /// Returns [`PreferenceError`] on invalid references.
    async fn upsert_sidebar_prefs(
        &self,
        company_id: &str,
        user_id: &str,
        project_order: Vec<String>,
    ) -> Result<SidebarPreferenceRecord, PreferenceError>;

    /// Gets the company logo.
    ///
    /// # Errors
    ///
    /// Returns [`PreferenceError`] on database failure.
    async fn logo(&self, company_id: &str) -> Result<Option<CompanyLogoRecord>, PreferenceError>;

    /// Sets the company logo (upsert on company).
    ///
    /// # Errors
    ///
    /// Returns [`PreferenceError`] when the asset is missing.
    async fn set_logo(
        &self,
        company_id: &str,
        asset_id: &str,
    ) -> Result<CompanyLogoRecord, PreferenceError>;

    /// Deletes the company logo.
    ///
    /// # Errors
    ///
    /// Returns [`PreferenceError`] on database failure.
    async fn delete_logo(
        &self,
        company_id: &str,
    ) -> Result<Option<CompanyLogoRecord>, PreferenceError>;
}

/// Turso/libSQL implementation of [`PreferenceRepository`].
#[derive(Debug)]
pub struct TursoPreferenceRepository {
    db: Database,
}

impl TursoPreferenceRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_prefs(row: &libsql::Row) -> Result<SidebarPreferenceRecord, libsql::Error> {
    let project_order = helpers::row_text(row, 3)?
        .map(|raw| serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default())
        .unwrap_or_default();
    Ok(SidebarPreferenceRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        user_id: helpers::row_text(row, 2)?.expect("user_id"),
        project_order,
        created_at: helpers::row_text(row, 4)?.expect("created_at"),
    })
}

fn row_to_logo(row: &libsql::Row) -> Result<CompanyLogoRecord, libsql::Error> {
    Ok(CompanyLogoRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        asset_id: helpers::row_text(row, 2)?.expect("asset_id"),
        created_at: helpers::row_text(row, 3)?.expect("created_at"),
    })
}

const PREFS_COLUMNS: &str = "id, company_id, user_id, project_order, created_at";
const LOGO_COLUMNS: &str = "id, company_id, asset_id, created_at";

#[async_trait]
impl PreferenceRepository for TursoPreferenceRepository {
    async fn sidebar_prefs(
        &self,
        company_id: &str,
        user_id: &str,
    ) -> Result<Option<SidebarPreferenceRecord>, PreferenceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PREFS_COLUMNS} FROM company_user_sidebar_preferences
                     WHERE company_id = ?1 AND user_id = ?2"
                ),
                libsql::params![company_id, user_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_prefs(&row)?)),
            None => Ok(None),
        }
    }

    async fn upsert_sidebar_prefs(
        &self,
        company_id: &str,
        user_id: &str,
        project_order: Vec<String>,
    ) -> Result<SidebarPreferenceRecord, PreferenceError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(PreferenceError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let order = serde_json::to_string(&project_order).unwrap_or_else(|_| "[]".to_owned());
        conn.execute(
            "INSERT INTO company_user_sidebar_preferences
               (id, company_id, user_id, project_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, user_id)
             DO UPDATE SET project_order = excluded.project_order,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![id.clone(), company_id, user_id, order],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PREFS_COLUMNS} FROM company_user_sidebar_preferences
                     WHERE company_id = ?1 AND user_id = ?2"
                ),
                libsql::params![company_id, user_id],
            )
            .await?;
        let row = rows.next().await?.expect("prefs were just upserted");
        Ok(row_to_prefs(&row)?)
    }

    async fn logo(&self, company_id: &str) -> Result<Option<CompanyLogoRecord>, PreferenceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {LOGO_COLUMNS} FROM company_logos WHERE company_id = ?1"),
                libsql::params![company_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_logo(&row)?)),
            None => Ok(None),
        }
    }

    async fn set_logo(
        &self,
        company_id: &str,
        asset_id: &str,
    ) -> Result<CompanyLogoRecord, PreferenceError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(PreferenceError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(&conn, "assets", asset_id, company_id).await? {
            return Err(PreferenceError::AssetNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO company_logos (id, company_id, asset_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id)
             DO UPDATE SET asset_id = excluded.asset_id,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![id.clone(), company_id, asset_id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {LOGO_COLUMNS} FROM company_logos WHERE company_id = ?1"),
                libsql::params![company_id],
            )
            .await?;
        let row = rows.next().await?.expect("logo was just upserted");
        Ok(row_to_logo(&row)?)
    }

    async fn delete_logo(
        &self,
        company_id: &str,
    ) -> Result<Option<CompanyLogoRecord>, PreferenceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {LOGO_COLUMNS} FROM company_logos WHERE company_id = ?1"),
                libsql::params![company_id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_logo(&row)?;
        conn.execute(
            "DELETE FROM company_logos WHERE company_id = ?1",
            libsql::params![company_id],
        )
        .await?;
        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoPreferenceRepository) {
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
            "INSERT INTO assets (id, company_id, provider, object_key, content_type, byte_size, sha256)
             VALUES ('a1', 'c1', 'local', 'logo.png', 'image/png', 100, 'abc')",
            (),
        )
        .await
        .unwrap();
        (dir, TursoPreferenceRepository::new(db))
    }

    #[tokio::test]
    async fn sidebar_prefs_upsert_roundtrip() {
        let (_dir, repo) = repo().await;
        let prefs = repo
            .upsert_sidebar_prefs("c1", "u1", vec!["p2".to_owned(), "p1".to_owned()])
            .await
            .unwrap();
        assert_eq!(prefs.project_order, vec!["p2", "p1"]);
        let again = repo
            .upsert_sidebar_prefs("c1", "u1", vec!["p3".to_owned()])
            .await
            .unwrap();
        assert_eq!(again.id, prefs.id);
        assert_eq!(again.project_order, vec!["p3"]);
        assert!(repo.sidebar_prefs("c1", "u1").await.unwrap().is_some());
        assert!(repo.sidebar_prefs("c1", "u2").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn logo_set_get_delete_with_company_boundary() {
        let (_dir, repo) = repo().await;
        let logo = repo.set_logo("c1", "a1").await.unwrap();
        assert_eq!(logo.asset_id, "a1");
        assert!(repo.logo("c1").await.unwrap().is_some());

        let err = repo.set_logo("c1", "foreign-asset").await.unwrap_err();
        assert!(matches!(err, PreferenceError::AssetNotFound));

        assert!(repo.delete_logo("c1").await.unwrap().is_some());
        assert!(repo.logo("c1").await.unwrap().is_none());
    }
}
