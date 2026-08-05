//! Agent API keys repository: generated once, hashed at rest, revocable.

use async_trait::async_trait;
use libsql::Database;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `agent_api_keys` table (hash only, never the plaintext).
#[derive(Debug, Clone)]
pub struct AgentApiKeyRecord {
    /// Key id.
    pub id: String,
    /// Agent id.
    pub agent_id: String,
    /// Owning company id.
    pub company_id: String,
    /// Display name.
    pub name: String,
    /// SHA-256 hex of the plaintext key.
    pub key_hash: String,
    /// ISO 8601 last use time.
    pub last_used_at: Option<String>,
    /// ISO 8601 revocation time.
    pub revoked_at: Option<String>,
    /// Responsible user id.
    pub responsible_user_id: Option<String>,
    /// Scope config JSON.
    pub scope_config: Option<serde_json::Value>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A successfully authenticated agent principal.
#[derive(Debug, Clone)]
pub struct AgentPrincipal {
    /// Agent id.
    pub agent_id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent name.
    pub agent_name: String,
    /// Agent role.
    pub agent_role: String,
    /// API key id used.
    pub api_key_id: String,
}

/// Input for creating an API key.
#[derive(Debug, Clone)]
pub struct NewAgentApiKey {
    /// Owning company id.
    pub company_id: String,
    /// Agent id (must belong to the company).
    pub agent_id: String,
    /// Display name.
    pub name: String,
}

/// API key repository errors.
#[derive(Debug, Error)]
pub enum ApiKeyError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The agent does not exist or belongs to another company.
    #[error("agent not found")]
    AgentNotFound,
    /// The plaintext key is invalid (unknown, revoked, or malformed).
    #[error("invalid API key")]
    InvalidKey,
}

/// API key persistence contract.
#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Creates a key, returning the record plus the one-time plaintext.
    ///
    /// # Errors
    ///
    /// Returns [`ApiKeyError`] when the agent is missing.
    async fn create_key(
        &self,
        input: NewAgentApiKey,
    ) -> Result<(AgentApiKeyRecord, String), ApiKeyError>;

    /// Lists keys for a company.
    ///
    /// # Errors
    ///
    /// Returns [`ApiKeyError`] on database failure.
    async fn list_keys(&self, company_id: &str) -> Result<Vec<AgentApiKeyRecord>, ApiKeyError>;

    /// Revokes a key.
    ///
    /// # Errors
    ///
    /// Returns [`ApiKeyError`] on database failure.
    async fn revoke_key(&self, key_id: &str) -> Result<Option<AgentApiKeyRecord>, ApiKeyError>;

    /// Authenticates a plaintext key, returning the agent principal.
    ///
    /// # Errors
    ///
    /// Returns [`ApiKeyError::InvalidKey`] for unknown/revoked keys.
    async fn authenticate(&self, plaintext: &str) -> Result<AgentPrincipal, ApiKeyError>;
}

/// Turso/libSQL implementation of [`ApiKeyRepository`].
#[derive(Debug)]
pub struct TursoApiKeyRepository {
    db: Database,
}

impl TursoApiKeyRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const KEY_COLUMNS: &str = "id, agent_id, company_id, name, key_hash, last_used_at,
    revoked_at, responsible_user_id, scope_config, created_at";

fn row_to_key(row: &libsql::Row) -> Result<AgentApiKeyRecord, libsql::Error> {
    Ok(AgentApiKeyRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        agent_id: helpers::row_text(row, 1)?.expect("agent_id is NOT NULL"),
        company_id: helpers::row_text(row, 2)?.expect("company_id is NOT NULL"),
        name: helpers::row_text(row, 3)?.expect("name is NOT NULL"),
        key_hash: helpers::row_text(row, 4)?.expect("key_hash is NOT NULL"),
        last_used_at: helpers::row_text(row, 5)?,
        revoked_at: helpers::row_text(row, 6)?,
        responsible_user_id: helpers::row_text(row, 7)?,
        scope_config: helpers::row_text(row, 8)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        created_at: helpers::row_text(row, 9)?.expect("created_at is NOT NULL"),
    })
}

fn hash_key(plaintext: &str) -> String {
    let digest = Sha256::digest(plaintext.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Generates a random plaintext key.
fn generate_key() -> String {
    let mut bytes = [0u8; 32];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    format!(
        "sk-{}",
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

#[async_trait]
impl ApiKeyRepository for TursoApiKeyRepository {
    async fn create_key(
        &self,
        input: NewAgentApiKey,
    ) -> Result<(AgentApiKeyRecord, String), ApiKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "agents", &input.agent_id, &input.company_id)
            .await?
        {
            return Err(ApiKeyError::AgentNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let plaintext = generate_key();
        let key_hash = hash_key(&plaintext);
        conn.execute(
            "INSERT INTO agent_api_keys (id, agent_id, company_id, name, key_hash,
                                         created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.agent_id,
                input.company_id,
                input.name,
                key_hash
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM agent_api_keys WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("key was just inserted");
        Ok((row_to_key(&row)?, plaintext))
    }

    async fn list_keys(&self, company_id: &str) -> Result<Vec<AgentApiKeyRecord>, ApiKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM agent_api_keys WHERE company_id = ?1 ORDER BY created_at"),
                libsql::params![company_id],
            )
            .await?;
        let mut keys = Vec::new();
        while let Some(row) = rows.next().await? {
            keys.push(row_to_key(&row)?);
        }
        Ok(keys)
    }

    async fn revoke_key(&self, key_id: &str) -> Result<Option<AgentApiKeyRecord>, ApiKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM agent_api_keys WHERE id = ?1"),
                libsql::params![key_id],
            )
            .await?;
        let Some(_) = rows.next().await? else {
            return Ok(None);
        };
        conn.execute(
            "UPDATE agent_api_keys
             SET revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![key_id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM agent_api_keys WHERE id = ?1"),
                libsql::params![key_id],
            )
            .await?;
        let row = rows.next().await?.expect("key exists");
        Ok(Some(row_to_key(&row)?))
    }

    async fn authenticate(&self, plaintext: &str) -> Result<AgentPrincipal, ApiKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let key_hash = hash_key(plaintext);
        let mut rows = conn
            .query(
                "SELECT ak.id, ak.agent_id, ak.company_id, a.name, a.role
                 FROM agent_api_keys ak
                 JOIN agents a ON a.id = ak.agent_id
                 WHERE ak.key_hash = ?1 AND ak.revoked_at IS NULL",
                libsql::params![key_hash],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(ApiKeyError::InvalidKey);
        };
        let key_id = helpers::row_text(&row, 0)?.expect("id is NOT NULL");
        let agent_id = helpers::row_text(&row, 1)?.expect("agent_id is NOT NULL");
        let company_id = helpers::row_text(&row, 2)?.expect("company_id is NOT NULL");
        let agent_name = helpers::row_text(&row, 3)?.expect("name is NOT NULL");
        let agent_role = helpers::row_text(&row, 4)?.expect("role is NOT NULL");

        // Record last use.
        conn.execute(
            "UPDATE agent_api_keys
             SET last_used_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![key_id.clone()],
        )
        .await?;
        Ok(AgentPrincipal {
            agent_id,
            company_id,
            agent_name,
            agent_role,
            api_key_id: key_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoApiKeyRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoApiKeyRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn create_authenticate_revoke_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();

        let (record, plaintext) = repo
            .create_key(NewAgentApiKey {
                company_id: "c1".to_owned(),
                agent_id: "a1".to_owned(),
                name: "dev".to_owned(),
            })
            .await
            .unwrap();
        assert!(plaintext.starts_with("sk-"));
        assert_ne!(record.key_hash, plaintext); // hashed at rest

        // Cross-company agent rejected.
        let error = repo
            .create_key(NewAgentApiKey {
                company_id: "c2".to_owned(),
                agent_id: "a1".to_owned(),
                name: "x".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, ApiKeyError::AgentNotFound));

        // Authenticate with the plaintext.
        let principal = repo.authenticate(&plaintext).await.unwrap();
        assert_eq!(principal.company_id, "c1");
        assert_eq!(principal.agent_id, "a1");

        // Revoke -> authentication fails.
        repo.revoke_key(&record.id).await.unwrap();
        let error = repo.authenticate(&plaintext).await.unwrap_err();
        assert!(matches!(error, ApiKeyError::InvalidKey));

        // Unknown key rejected.
        let error = repo.authenticate("sk-nope").await.unwrap_err();
        assert!(matches!(error, ApiKeyError::InvalidKey));
    }
}
