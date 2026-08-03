//! Board API keys and CLI auth challenges repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `board_api_keys` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardApiKeyRecord {
    /// Key id.
    pub id: String,
    /// Owning user id.
    pub user_id: String,
    /// Display name.
    pub name: String,
    /// ISO 8601 last use.
    pub last_used_at: Option<String>,
    /// ISO 8601 revocation.
    pub revoked_at: Option<String>,
    /// ISO 8601 expiry.
    pub expires_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `cli_auth_challenges` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliAuthChallengeRecord {
    /// Challenge id.
    pub id: String,
    /// Requested access (`board`).
    pub requested_access: String,
    /// Requested company id (optional scope).
    pub requested_company_id: Option<String>,
    /// Pending key name.
    pub pending_key_name: String,
    /// Approving user id.
    pub approved_by_user_id: Option<String>,
    /// Created board key id.
    pub board_api_key_id: Option<String>,
    /// ISO 8601 approval.
    pub approved_at: Option<String>,
    /// ISO 8601 cancellation.
    pub cancelled_at: Option<String>,
    /// ISO 8601 expiry.
    pub expires_at: String,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Input for creating a board API key.
#[derive(Debug, Clone)]
pub struct NewBoardApiKey {
    /// Owning user id.
    pub user_id: String,
    /// Display name.
    pub name: String,
    /// ISO 8601 expiry (optional).
    pub expires_at: Option<String>,
}

/// Input for creating a CLI auth challenge.
#[derive(Debug, Clone)]
pub struct NewCliAuthChallenge {
    /// Command text.
    pub command: String,
    /// Client name.
    pub client_name: Option<String>,
    /// Requested access.
    pub requested_access: String,
    /// Requested company id.
    pub requested_company_id: Option<String>,
    /// Pending key name.
    pub pending_key_name: String,
    /// ISO 8601 expiry.
    pub expires_at: String,
}

/// Board key repository errors.
#[derive(Debug, Error)]
pub enum BoardKeyError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The key is invalid, revoked, or expired.
    #[error("invalid board API key")]
    InvalidKey,
    /// The challenge is missing or not pending.
    #[error("challenge not found or not pending")]
    ChallengeNotFound,
}

/// Board key persistence contract.
#[async_trait]
pub trait BoardKeyRepository: Send + Sync {
    /// Creates a board API key and returns the record plus the plaintext key.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError`] on database failure.
    async fn create_key(
        &self,
        input: NewBoardApiKey,
    ) -> Result<(BoardApiKeyRecord, String), BoardKeyError>;

    /// Authenticates a plaintext board key; updates `last_used_at`.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError::InvalidKey`] when unknown/revoked/expired.
    async fn authenticate(&self, plaintext: &str) -> Result<BoardApiKeyRecord, BoardKeyError>;

    /// Lists board API keys.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError`] on database failure.
    async fn list_keys(&self) -> Result<Vec<BoardApiKeyRecord>, BoardKeyError>;

    /// Revokes a board API key.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError`] on database failure.
    async fn revoke_key(&self, id: &str) -> Result<Option<BoardApiKeyRecord>, BoardKeyError>;

    /// Creates a CLI auth challenge and returns the record plus the
    /// plaintext secret the CLI presents to claim the pending key.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError`] on database failure.
    async fn create_challenge(
        &self,
        input: NewCliAuthChallenge,
    ) -> Result<(CliAuthChallengeRecord, String), BoardKeyError>;

    /// Lists CLI auth challenges.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError`] on database failure.
    async fn list_challenges(&self) -> Result<Vec<CliAuthChallengeRecord>, BoardKeyError>;

    /// Approves a pending challenge: creates the board API key from the
    /// pending key hash and stamps the challenge.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError`] when the challenge is missing/not pending.
    async fn approve_challenge(
        &self,
        id: &str,
        approved_by_user_id: Option<String>,
    ) -> Result<CliAuthChallengeRecord, BoardKeyError>;

    /// Cancels a pending challenge.
    ///
    /// # Errors
    ///
    /// Returns [`BoardKeyError`] when the challenge is missing/not pending.
    async fn cancel_challenge(&self, id: &str) -> Result<CliAuthChallengeRecord, BoardKeyError>;
}

/// Turso/libSQL implementation of [`BoardKeyRepository`].
#[derive(Debug)]
pub struct TursoBoardKeyRepository {
    db: Database,
}

impl TursoBoardKeyRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_key(row: &libsql::Row) -> Result<BoardApiKeyRecord, libsql::Error> {
    Ok(BoardApiKeyRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        user_id: helpers::row_text(row, 1)?.expect("user_id"),
        name: helpers::row_text(row, 2)?.expect("name"),
        last_used_at: helpers::row_text(row, 3)?,
        revoked_at: helpers::row_text(row, 4)?,
        expires_at: helpers::row_text(row, 5)?,
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
    })
}

fn row_to_challenge(row: &libsql::Row) -> Result<CliAuthChallengeRecord, libsql::Error> {
    Ok(CliAuthChallengeRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        requested_access: helpers::row_text(row, 1)?.expect("requested_access"),
        requested_company_id: helpers::row_text(row, 2)?,
        pending_key_name: helpers::row_text(row, 3)?.expect("pending_key_name"),
        approved_by_user_id: helpers::row_text(row, 4)?,
        board_api_key_id: helpers::row_text(row, 5)?,
        approved_at: helpers::row_text(row, 6)?,
        cancelled_at: helpers::row_text(row, 7)?,
        expires_at: helpers::row_text(row, 8)?.expect("expires_at"),
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
    })
}

const KEY_COLUMNS: &str = "id, user_id, name, last_used_at, revoked_at, expires_at, created_at";
const CHALLENGE_COLUMNS: &str =
    "id, requested_access, requested_company_id, pending_key_name, approved_by_user_id,
     board_api_key_id, approved_at, cancelled_at, expires_at, created_at";

#[async_trait]
impl BoardKeyRepository for TursoBoardKeyRepository {
    async fn create_key(
        &self,
        input: NewBoardApiKey,
    ) -> Result<(BoardApiKeyRecord, String), BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let plaintext = format!("bk-{}", Uuid::new_v4());
        let key_hash = helpers::sha256_hex(&plaintext);
        conn.execute(
            "INSERT INTO board_api_keys (id, user_id, name, key_hash, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.user_id,
                input.name,
                key_hash,
                input.expires_at
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM board_api_keys WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("key was just inserted");
        Ok((row_to_key(&row)?, plaintext))
    }

    async fn authenticate(&self, plaintext: &str) -> Result<BoardApiKeyRecord, BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let key_hash = helpers::sha256_hex(plaintext);
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM board_api_keys WHERE key_hash = ?1"),
                libsql::params![key_hash],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(BoardKeyError::InvalidKey);
        };
        let key = row_to_key(&row)?;
        if key.revoked_at.is_some() {
            return Err(BoardKeyError::InvalidKey);
        }
        let mut now_rows = conn
            .query(
                "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![],
            )
            .await?;
        let now = helpers::row_text(&now_rows.next().await?.expect("now"), 0)?.expect("now");
        if key
            .expires_at
            .as_deref()
            .is_some_and(|expires| expires <= now.as_str())
        {
            return Err(BoardKeyError::InvalidKey);
        }
        conn.execute(
            "UPDATE board_api_keys SET last_used_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![key.id.clone()],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM board_api_keys WHERE id = ?1"),
                libsql::params![key.id],
            )
            .await?;
        let row = rows.next().await?.expect("key exists");
        Ok(row_to_key(&row)?)
    }

    async fn list_keys(&self) -> Result<Vec<BoardApiKeyRecord>, BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM board_api_keys ORDER BY created_at DESC"),
                libsql::params![],
            )
            .await?;
        let mut keys = Vec::new();
        while let Some(row) = rows.next().await? {
            keys.push(row_to_key(&row)?);
        }
        Ok(keys)
    }

    async fn revoke_key(&self, id: &str) -> Result<Option<BoardApiKeyRecord>, BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE board_api_keys SET revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?1 AND revoked_at IS NULL",
                libsql::params![id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {KEY_COLUMNS} FROM board_api_keys WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("key exists");
        Ok(Some(row_to_key(&row)?))
    }

    async fn create_challenge(
        &self,
        input: NewCliAuthChallenge,
    ) -> Result<(CliAuthChallengeRecord, String), BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let secret = format!("chal-{}", Uuid::new_v4());
        let secret_hash = helpers::sha256_hex(&secret);
        let pending_plaintext = format!("bk-{}", Uuid::new_v4());
        let pending_key_hash = helpers::sha256_hex(&pending_plaintext);
        conn.execute(
            "INSERT INTO cli_auth_challenges
               (id, secret_hash, command, client_name, requested_access, requested_company_id,
                pending_key_hash, pending_key_name, expires_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                secret_hash,
                input.command,
                input.client_name,
                input.requested_access,
                input.requested_company_id,
                pending_key_hash,
                input.pending_key_name,
                input.expires_at
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CHALLENGE_COLUMNS} FROM cli_auth_challenges WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("challenge was just inserted");
        Ok((row_to_challenge(&row)?, secret))
    }

    async fn list_challenges(&self) -> Result<Vec<CliAuthChallengeRecord>, BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CHALLENGE_COLUMNS} FROM cli_auth_challenges ORDER BY created_at DESC"
                ),
                libsql::params![],
            )
            .await?;
        let mut challenges = Vec::new();
        while let Some(row) = rows.next().await? {
            challenges.push(row_to_challenge(&row)?);
        }
        Ok(challenges)
    }

    async fn approve_challenge(
        &self,
        id: &str,
        approved_by_user_id: Option<String>,
    ) -> Result<CliAuthChallengeRecord, BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CHALLENGE_COLUMNS} FROM cli_auth_challenges WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(BoardKeyError::ChallengeNotFound);
        };
        let challenge = row_to_challenge(&row)?;
        if challenge.approved_at.is_some() || challenge.cancelled_at.is_some() {
            return Err(BoardKeyError::ChallengeNotFound);
        }
        let key_id = Uuid::new_v4().to_string();
        let (pending_hash, pending_name) = {
            let mut pending_rows = conn
                .query(
                    "SELECT pending_key_hash, pending_key_name FROM cli_auth_challenges WHERE id = ?1",
                    libsql::params![id],
                )
                .await?;
            let pending = pending_rows.next().await?.expect("challenge exists");
            (
                helpers::row_text(&pending, 0)?.expect("pending_key_hash"),
                helpers::row_text(&pending, 1)?.expect("pending_key_name"),
            )
        };
        conn.execute(
            "INSERT INTO board_api_keys (id, user_id, name, key_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                key_id.clone(),
                approved_by_user_id.clone().unwrap_or_default(),
                pending_name,
                pending_hash
            ],
        )
        .await?;
        conn.execute(
            "UPDATE cli_auth_challenges SET approved_by_user_id = ?1, board_api_key_id = ?2,
                    approved_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?3",
            libsql::params![approved_by_user_id, key_id, id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CHALLENGE_COLUMNS} FROM cli_auth_challenges WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("challenge exists");
        Ok(row_to_challenge(&row)?)
    }

    async fn cancel_challenge(&self, id: &str) -> Result<CliAuthChallengeRecord, BoardKeyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CHALLENGE_COLUMNS} FROM cli_auth_challenges WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(BoardKeyError::ChallengeNotFound);
        };
        let challenge = row_to_challenge(&row)?;
        if challenge.approved_at.is_some() || challenge.cancelled_at.is_some() {
            return Err(BoardKeyError::ChallengeNotFound);
        }
        conn.execute(
            "UPDATE cli_auth_challenges SET cancelled_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CHALLENGE_COLUMNS} FROM cli_auth_challenges WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("challenge exists");
        Ok(row_to_challenge(&row)?)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoBoardKeyRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        (dir, TursoBoardKeyRepository::new(db))
    }

    fn future() -> String {
        "2999-01-01T00:00:00.000Z".to_owned()
    }

    #[tokio::test]
    async fn board_key_create_authenticate_revoke() {
        let (_dir, repo) = repo().await;
        let (key, plaintext) = repo
            .create_key(NewBoardApiKey {
                user_id: "u1".to_owned(),
                name: "ci".to_owned(),
                expires_at: Some(future()),
            })
            .await
            .unwrap();
        assert!(plaintext.starts_with("bk-"));

        let authed = repo.authenticate(&plaintext).await.unwrap();
        assert_eq!(authed.id, key.id);
        assert!(authed.last_used_at.is_some());

        repo.revoke_key(&key.id).await.unwrap();
        assert!(matches!(
            repo.authenticate(&plaintext).await.unwrap_err(),
            BoardKeyError::InvalidKey
        ));
        assert!(matches!(
            repo.authenticate("bk-bogus").await.unwrap_err(),
            BoardKeyError::InvalidKey
        ));
    }

    #[tokio::test]
    async fn cli_challenge_approve_creates_key_and_cancel() {
        let (_dir, repo) = repo().await;
        let (challenge, secret) = repo
            .create_challenge(NewCliAuthChallenge {
                command: "paperclip login".to_owned(),
                client_name: Some("test-cli".to_owned()),
                requested_access: "board".to_owned(),
                requested_company_id: None,
                pending_key_name: "cli-session".to_owned(),
                expires_at: future(),
            })
            .await
            .unwrap();
        assert!(secret.starts_with("chal-"));
        assert_eq!(repo.list_challenges().await.unwrap().len(), 1);

        let approved = repo
            .approve_challenge(&challenge.id, Some("u-board".to_owned()))
            .await
            .unwrap();
        assert!(approved.approved_at.is_some());
        assert!(approved.board_api_key_id.is_some());

        // The created board key exists.
        let keys = repo.list_keys().await.unwrap();
        assert!(
            keys.iter()
                .any(|key| key.id == approved.board_api_key_id.as_deref().unwrap())
        );

        // Approving again fails.
        assert!(matches!(
            repo.approve_challenge(&challenge.id, None)
                .await
                .unwrap_err(),
            BoardKeyError::ChallengeNotFound
        ));

        // Cancel flow.
        let (second, _) = repo
            .create_challenge(NewCliAuthChallenge {
                command: "paperclip login 2".to_owned(),
                client_name: None,
                requested_access: "board".to_owned(),
                requested_company_id: None,
                pending_key_name: "cli-2".to_owned(),
                expires_at: future(),
            })
            .await
            .unwrap();
        let cancelled = repo.cancel_challenge(&second.id).await.unwrap();
        assert!(cancelled.cancelled_at.is_some());
    }
}
