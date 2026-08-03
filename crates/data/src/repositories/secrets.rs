//! Company secrets repository: versioned, encrypted secret values.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;
use crate::secrets::SecretCipher;

/// A row of the `company_secrets` table (no value material).
#[derive(Debug, Clone)]
pub struct CompanySecretRecord {
    /// Secret id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Name (unique per company).
    pub name: String,
    /// Scope (`company`).
    pub scope: String,
    /// Provider (`local_encrypted`).
    pub provider: String,
    /// Latest version number.
    pub latest_version: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A secret version row (value material stays encrypted).
#[derive(Debug, Clone)]
pub struct SecretVersionRecord {
    /// Version id.
    pub id: String,
    /// Secret id.
    pub secret_id: String,
    /// Version number.
    pub version: i64,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating a secret.
#[derive(Debug, Clone)]
pub struct NewSecret {
    /// Owning company id.
    pub company_id: String,
    /// Name.
    pub name: String,
    /// Plaintext value (encrypted before storage).
    pub value: String,
}

/// Secret repository errors.
#[derive(Debug, Error)]
pub enum SecretError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// A secret with this name already exists.
    #[error("secret already exists")]
    AlreadyExists,
    /// The secret does not exist.
    #[error("secret not found")]
    SecretNotFound,
    /// The requested version does not exist.
    #[error("secret version not found")]
    VersionNotFound,
    /// A value could not be encrypted/decrypted.
    #[error("cipher error: {0}")]
    Cipher(#[from] crate::secrets::CipherError),
}

/// Secret persistence contract.
#[async_trait]
pub trait SecretRepository: Send + Sync {
    /// Creates a secret with version 1.
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] on invalid references or duplicate names.
    async fn create_secret(&self, input: NewSecret) -> Result<CompanySecretRecord, SecretError>;

    /// Lists secrets (no values).
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] on database failure.
    async fn list_secrets(&self, company_id: &str)
    -> Result<Vec<CompanySecretRecord>, SecretError>;

    /// Fetches secret metadata by name.
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] on database failure.
    async fn get_secret(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Option<CompanySecretRecord>, SecretError>;

    /// Reads and decrypts the current value.
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] when the secret is missing or undecryptable.
    async fn get_secret_value(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Option<String>, SecretError>;

    /// Rotates a secret: stores a new value as the next version.
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] when the secret is missing.
    async fn rotate_secret(
        &self,
        company_id: &str,
        name: &str,
        new_value: String,
    ) -> Result<CompanySecretRecord, SecretError>;

    /// Rolls back to a previous version by storing its value as the newest
    /// version (append-only history).
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] when the secret or version is missing.
    async fn rollback_secret(
        &self,
        company_id: &str,
        name: &str,
        version: i64,
    ) -> Result<CompanySecretRecord, SecretError>;

    /// Lists the versions of a secret.
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] on database failure.
    async fn list_versions(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Vec<SecretVersionRecord>, SecretError>;

    /// Deletes a secret and all versions.
    ///
    /// # Errors
    ///
    /// Returns [`SecretError`] on database failure.
    async fn delete_secret(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Option<CompanySecretRecord>, SecretError>;
}

/// Turso/libSQL implementation of [`SecretRepository`].
#[derive(Debug)]
pub struct TursoSecretRepository {
    db: Database,
    cipher: SecretCipher,
}

impl TursoSecretRepository {
    /// Creates a repository over the given database with the given cipher.
    #[must_use]
    pub fn new(db: Database, cipher: SecretCipher) -> Self {
        Self { db, cipher }
    }
}

const SECRET_COLUMNS: &str = "id, company_id, name, scope, provider, created_at";

fn row_to_secret(row: &libsql::Row) -> Result<CompanySecretRecord, libsql::Error> {
    Ok(CompanySecretRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        name: helpers::row_text(row, 2)?.expect("name is NOT NULL"),
        scope: helpers::row_text(row, 3)?.expect("scope is NOT NULL"),
        provider: helpers::row_text(row, 4)?.expect("provider is NOT NULL"),
        latest_version: 0,
        created_at: helpers::row_text(row, 5)?.expect("created_at is NOT NULL"),
    })
}

/// Resolves the secret id for a company/name pair.
async fn secret_id(
    conn: &libsql::Connection,
    company_id: &str,
    name: &str,
) -> Result<Option<String>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT id FROM company_secrets WHERE company_id = ?1 AND name = ?2",
            libsql::params![company_id, name],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(helpers::row_text(&row, 0)?),
        None => Ok(None),
    }
}

/// The newest version number of a secret.
async fn latest_version(conn: &libsql::Connection, secret_id: &str) -> Result<i64, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT COALESCE(MAX(version), 0) FROM company_secret_versions WHERE secret_id = ?1",
            libsql::params![secret_id],
        )
        .await?;
    let row = rows.next().await?.expect("aggregate row");
    helpers::row_i64(&row, 0)
}

async fn insert_version(
    conn: &libsql::Connection,
    company_id: &str,
    secret_id: &str,
    version: i64,
    encrypted_value: &str,
) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT INTO company_secret_versions (id, company_id, secret_id, version,
                                              encrypted_value, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5,
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        libsql::params![
            Uuid::new_v4().to_string(),
            company_id,
            secret_id,
            version,
            encrypted_value
        ],
    )
    .await?;
    Ok(())
}

#[async_trait]
impl SecretRepository for TursoSecretRepository {
    async fn create_secret(&self, input: NewSecret) -> Result<CompanySecretRecord, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SecretError::CompanyNotFound);
        }
        if secret_id(&conn, &input.company_id, &input.name)
            .await?
            .is_some()
        {
            return Err(SecretError::AlreadyExists);
        }
        let id = Uuid::new_v4().to_string();
        let encrypted = self.cipher.encrypt(input.value.as_bytes())?;
        conn.execute(
            "INSERT INTO company_secrets (id, company_id, name, scope, provider, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'company', 'local_encrypted',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![id.clone(), input.company_id.clone(), input.name.clone()],
        )
        .await?;
        insert_version(&conn, &input.company_id, &id, 1, &encrypted).await?;
        Ok(self
            .get_secret(&input.company_id, &input.name)
            .await?
            .expect("secret was just inserted"))
    }

    async fn list_secrets(
        &self,
        company_id: &str,
    ) -> Result<Vec<CompanySecretRecord>, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {SECRET_COLUMNS} FROM company_secrets WHERE company_id = ?1 ORDER BY name"),
                libsql::params![company_id],
            )
            .await?;
        let mut secrets = Vec::new();
        while let Some(row) = rows.next().await? {
            let mut secret = row_to_secret(&row)?;
            secret.latest_version = latest_version(&conn, &secret.id).await?;
            secrets.push(secret);
        }
        Ok(secrets)
    }

    async fn get_secret(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Option<CompanySecretRecord>, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {SECRET_COLUMNS} FROM company_secrets WHERE company_id = ?1 AND name = ?2"),
                libsql::params![company_id, name],
            )
            .await?;
        match rows.next().await? {
            Some(row) => {
                let mut secret = row_to_secret(&row)?;
                secret.latest_version = latest_version(&conn, &secret.id).await?;
                Ok(Some(secret))
            }
            None => Ok(None),
        }
    }

    async fn get_secret_value(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Option<String>, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(id) = secret_id(&conn, company_id, name).await? else {
            return Ok(None);
        };
        let version = latest_version(&conn, &id).await?;
        let mut rows = conn
            .query(
                "SELECT encrypted_value FROM company_secret_versions
                 WHERE secret_id = ?1 AND version = ?2",
                libsql::params![id, version],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let encrypted = helpers::row_text(&row, 0)?.expect("encrypted_value is NOT NULL");
        let plaintext = self.cipher.decrypt(&encrypted)?;
        Ok(Some(String::from_utf8_lossy(&plaintext).into_owned()))
    }

    async fn rotate_secret(
        &self,
        company_id: &str,
        name: &str,
        new_value: String,
    ) -> Result<CompanySecretRecord, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(id) = secret_id(&conn, company_id, name).await? else {
            return Err(SecretError::SecretNotFound);
        };
        let version = latest_version(&conn, &id).await? + 1;
        let encrypted = self.cipher.encrypt(new_value.as_bytes())?;
        insert_version(&conn, company_id, &id, version, &encrypted).await?;
        Ok(self
            .get_secret(company_id, name)
            .await?
            .expect("secret exists"))
    }

    async fn rollback_secret(
        &self,
        company_id: &str,
        name: &str,
        version: i64,
    ) -> Result<CompanySecretRecord, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(id) = secret_id(&conn, company_id, name).await? else {
            return Err(SecretError::SecretNotFound);
        };
        let mut rows = conn
            .query(
                "SELECT encrypted_value FROM company_secret_versions
                 WHERE secret_id = ?1 AND version = ?2",
                libsql::params![id.clone(), version],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(SecretError::VersionNotFound);
        };
        let encrypted = helpers::row_text(&row, 0)?.expect("encrypted_value is NOT NULL");
        let current = latest_version(&conn, &id).await?;
        insert_version(&conn, company_id, &id, current + 1, &encrypted).await?;
        Ok(self
            .get_secret(company_id, name)
            .await?
            .expect("secret exists"))
    }

    async fn list_versions(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Vec<SecretVersionRecord>, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(id) = secret_id(&conn, company_id, name).await? else {
            return Ok(Vec::new());
        };
        let mut rows = conn
            .query(
                "SELECT id, secret_id, version, created_at FROM company_secret_versions
                 WHERE secret_id = ?1 ORDER BY version",
                libsql::params![id.clone()],
            )
            .await?;
        let mut versions = Vec::new();
        while let Some(row) = rows.next().await? {
            versions.push(SecretVersionRecord {
                id: helpers::row_text(&row, 0)?.expect("id is NOT NULL"),
                secret_id: helpers::row_text(&row, 1)?.expect("secret_id is NOT NULL"),
                version: helpers::row_i64(&row, 2)?,
                created_at: helpers::row_text(&row, 3)?.expect("created_at is NOT NULL"),
            });
        }
        Ok(versions)
    }

    async fn delete_secret(
        &self,
        company_id: &str,
        name: &str,
    ) -> Result<Option<CompanySecretRecord>, SecretError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(secret) = self.get_secret(company_id, name).await? else {
            return Ok(None);
        };
        let id = secret_id(&conn, company_id, name)
            .await?
            .expect("secret exists");
        conn.execute(
            "DELETE FROM company_secret_versions WHERE secret_id = ?1",
            libsql::params![id.clone()],
        )
        .await?;
        conn.execute(
            "DELETE FROM company_secrets WHERE id = ?1",
            libsql::params![id.clone()],
        )
        .await?;
        Ok(Some(secret))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoSecretRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let cipher = SecretCipher::load_or_create(dir.path().join("key")).unwrap();
        let repo = TursoSecretRepository::new(db, cipher);
        (dir, repo)
    }

    #[tokio::test]
    async fn create_rotate_rollback_roundtrip() {
        let (_dir, repo) = repo().await;
        // company needed for FK
        let conn = crate::connect(&repo.db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();

        let created = repo
            .create_secret(NewSecret {
                company_id: "c1".to_owned(),
                name: "github_token".to_owned(),
                value: "v1-secret".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(created.latest_version, 1);

        // Value roundtrip.
        let value = repo
            .get_secret_value("c1", "github_token")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(value, "v1-secret");

        // Duplicate name rejected.
        let error = repo
            .create_secret(NewSecret {
                company_id: "c1".to_owned(),
                name: "github_token".to_owned(),
                value: "x".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, SecretError::AlreadyExists));

        // Rotate -> v2.
        let rotated = repo
            .rotate_secret("c1", "github_token", "v2-secret".to_owned())
            .await
            .unwrap();
        assert_eq!(rotated.latest_version, 2);
        assert_eq!(
            repo.get_secret_value("c1", "github_token")
                .await
                .unwrap()
                .unwrap(),
            "v2-secret"
        );

        // Versions list has 1 and 2.
        let versions = repo.list_versions("c1", "github_token").await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[1].version, 2);

        // Rollback to v1 -> v3 with v1's value.
        let rolled = repo.rollback_secret("c1", "github_token", 1).await.unwrap();
        assert_eq!(rolled.latest_version, 3);
        assert_eq!(
            repo.get_secret_value("c1", "github_token")
                .await
                .unwrap()
                .unwrap(),
            "v1-secret"
        );

        // Encrypted at rest: raw column must not contain the plaintext
        // (scope the read so its statement releases any read lock).
        {
            let mut rows = conn
                .query(
                    "SELECT encrypted_value FROM company_secret_versions WHERE version = 1",
                    (),
                )
                .await
                .unwrap();
            let row = rows.next().await.unwrap().unwrap();
            let stored = helpers::row_text(&row, 0).unwrap().unwrap();
            assert!(!stored.contains("v1-secret"));
        }

        // Delete.
        let deleted = repo
            .delete_secret("c1", "github_token")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(deleted.name, "github_token");
        assert!(
            repo.get_secret("c1", "github_token")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn rollback_missing_version_is_rejected() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        repo.create_secret(NewSecret {
            company_id: "c1".to_owned(),
            name: "s".to_owned(),
            value: "v".to_owned(),
        })
        .await
        .unwrap();
        let error = repo.rollback_secret("c1", "s", 99).await.unwrap_err();
        assert!(matches!(error, SecretError::VersionNotFound));
    }
}
