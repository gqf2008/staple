//! Invites and join requests repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `invites` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteRecord {
    /// Invite id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Invite type (`company_join` | `bootstrap_ceo`).
    pub invite_type: String,
    /// Allowed join types (`human` | `agent` | `both`).
    pub allowed_join_types: String,
    /// Defaults payload JSON.
    pub defaults_payload: Option<serde_json::Value>,
    /// ISO 8601 expiry.
    pub expires_at: String,
    /// Inviting board user id.
    pub invited_by_user_id: Option<String>,
    /// ISO 8601 revocation time.
    pub revoked_at: Option<String>,
    /// ISO 8601 acceptance time.
    pub accepted_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A row of the `join_requests` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRequestRecord {
    /// Join request id.
    pub id: String,
    /// Invite id.
    pub invite_id: String,
    /// Owning company id.
    pub company_id: String,
    /// Request type (`human` | `agent`).
    pub request_type: String,
    /// Status.
    pub status: String,
    /// Request IP.
    pub request_ip: String,
    /// Requesting user id.
    pub requesting_user_id: Option<String>,
    /// Request email snapshot.
    pub request_email_snapshot: Option<String>,
    /// Agent name (agent requests).
    pub agent_name: Option<String>,
    /// Adapter type (agent requests).
    pub adapter_type: Option<String>,
    /// Capabilities (agent requests).
    pub capabilities: Option<String>,
    /// Agent defaults payload JSON.
    pub agent_defaults_payload: Option<serde_json::Value>,
    /// Claim secret hash.
    pub claim_secret_hash: Option<String>,
    /// ISO 8601 claim secret expiry.
    pub claim_secret_expires_at: Option<String>,
    /// ISO 8601 claim secret consumption.
    pub claim_secret_consumed_at: Option<String>,
    /// Created agent id.
    pub created_agent_id: Option<String>,
    /// Approving board user id.
    pub approved_by_user_id: Option<String>,
    /// ISO 8601 approval time.
    pub approved_at: Option<String>,
    /// Rejecting board user id.
    pub rejected_by_user_id: Option<String>,
    /// ISO 8601 rejection time.
    pub rejected_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating an invite.
#[derive(Debug, Clone)]
pub struct NewInvite {
    /// Owning company id.
    pub company_id: String,
    /// Invite type.
    pub invite_type: String,
    /// Allowed join types.
    pub allowed_join_types: String,
    /// Defaults payload JSON.
    pub defaults_payload: Option<serde_json::Value>,
    /// ISO 8601 expiry.
    pub expires_at: String,
    /// Inviting board user id.
    pub invited_by_user_id: Option<String>,
}

/// Input for creating a join request.
#[derive(Debug, Clone)]
pub struct NewJoinRequest {
    /// Owning company id.
    pub company_id: String,
    /// Invite id.
    pub invite_id: String,
    /// Request type (`human` | `agent`).
    pub request_type: String,
    /// Request IP.
    pub request_ip: String,
    /// Requesting user id.
    pub requesting_user_id: Option<String>,
    /// Request email snapshot.
    pub request_email_snapshot: Option<String>,
    /// Agent name.
    pub agent_name: Option<String>,
    /// Adapter type.
    pub adapter_type: Option<String>,
    /// Capabilities.
    pub capabilities: Option<String>,
    /// Agent defaults payload JSON.
    pub agent_defaults_payload: Option<serde_json::Value>,
    /// Claim secret hash (agent requests).
    pub claim_secret_hash: Option<String>,
    /// ISO 8601 claim secret expiry.
    pub claim_secret_expires_at: Option<String>,
}

/// Invite repository errors.
#[derive(Debug, Error)]
pub enum InviteError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The invite does not exist.
    #[error("invite not found")]
    InviteNotFound,
    /// The invite is revoked or expired.
    #[error("invite revoked or expired")]
    InviteRevokedOrExpired,
    /// The join request does not exist.
    #[error("join request not found")]
    JoinRequestNotFound,
    /// The join request is not pending approval.
    #[error("join request not pending")]
    NotPending,
    /// A join request already exists for this invite.
    #[error("join request already exists")]
    AlreadyExists,
}

/// Invite persistence contract.
#[async_trait]
pub trait InviteRepository: Send + Sync {
    /// Creates an invite and returns the record plus the plaintext token.
    ///
    /// # Errors
    ///
    /// Returns [`InviteError`] on invalid references.
    async fn create_invite(&self, input: NewInvite) -> Result<(InviteRecord, String), InviteError>;

    /// Lists invites for a company.
    ///
    /// # Errors
    ///
    /// Returns [`InviteError`] on database failure.
    async fn list_invites(&self, company_id: &str) -> Result<Vec<InviteRecord>, InviteError>;

    /// Revokes an invite (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`InviteError`] on database failure.
    async fn revoke_invite(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<InviteRecord>, InviteError>;

    /// Creates a join request for an invite (one per invite).
    ///
    /// # Errors
    ///
    /// Returns [`InviteError`] when the invite is missing/revoked/expired or a
    /// request already exists.
    async fn create_join_request(
        &self,
        input: NewJoinRequest,
    ) -> Result<JoinRequestRecord, InviteError>;

    /// Lists join requests for a company.
    ///
    /// # Errors
    ///
    /// Returns [`InviteError`] on database failure.
    async fn list_join_requests(
        &self,
        company_id: &str,
    ) -> Result<Vec<JoinRequestRecord>, InviteError>;

    /// Approves a join request: creates the agent (agent requests) or
    /// membership (human requests) and stamps the request.
    ///
    /// # Errors
    ///
    /// Returns [`InviteError`] on invalid state.
    async fn approve(
        &self,
        company_id: &str,
        id: &str,
        approved_by_user_id: Option<String>,
    ) -> Result<JoinRequestRecord, InviteError>;

    /// Rejects a join request.
    ///
    /// # Errors
    ///
    /// Returns [`InviteError`] on invalid state.
    async fn reject(
        &self,
        company_id: &str,
        id: &str,
        rejected_by_user_id: Option<String>,
    ) -> Result<JoinRequestRecord, InviteError>;
}

/// Turso/libSQL implementation of [`InviteRepository`].
#[derive(Debug)]
pub struct TursoInviteRepository {
    db: Database,
}

impl TursoInviteRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_invite(row: &libsql::Row) -> Result<InviteRecord, libsql::Error> {
    Ok(InviteRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        invite_type: helpers::row_text(row, 2)?.expect("invite_type"),
        allowed_join_types: helpers::row_text(row, 3)?.expect("allowed_join_types"),
        defaults_payload: helpers::row_text(row, 4)?
            .and_then(|raw| serde_json::from_str(&raw).ok()),
        expires_at: helpers::row_text(row, 5)?.expect("expires_at"),
        invited_by_user_id: helpers::row_text(row, 6)?,
        revoked_at: helpers::row_text(row, 7)?,
        accepted_at: helpers::row_text(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
    })
}

fn row_to_join_request(row: &libsql::Row) -> Result<JoinRequestRecord, libsql::Error> {
    Ok(JoinRequestRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        invite_id: helpers::row_text(row, 1)?.expect("invite_id"),
        company_id: helpers::row_text(row, 2)?.expect("company_id"),
        request_type: helpers::row_text(row, 3)?.expect("request_type"),
        status: helpers::row_text(row, 4)?.expect("status"),
        request_ip: helpers::row_text(row, 5)?.expect("request_ip"),
        requesting_user_id: helpers::row_text(row, 6)?,
        request_email_snapshot: helpers::row_text(row, 7)?,
        agent_name: helpers::row_text(row, 8)?,
        adapter_type: helpers::row_text(row, 9)?,
        capabilities: helpers::row_text(row, 10)?,
        agent_defaults_payload: helpers::row_text(row, 11)?
            .and_then(|raw| serde_json::from_str(&raw).ok()),
        claim_secret_hash: helpers::row_text(row, 12)?,
        claim_secret_expires_at: helpers::row_text(row, 13)?,
        claim_secret_consumed_at: helpers::row_text(row, 14)?,
        created_agent_id: helpers::row_text(row, 15)?,
        approved_by_user_id: helpers::row_text(row, 16)?,
        approved_at: helpers::row_text(row, 17)?,
        rejected_by_user_id: helpers::row_text(row, 18)?,
        rejected_at: helpers::row_text(row, 19)?,
        created_at: helpers::row_text(row, 20)?.expect("created_at"),
    })
}

const INVITE_COLUMNS: &str = "id, company_id, invite_type, allowed_join_types, defaults_payload,
    expires_at, invited_by_user_id, revoked_at, accepted_at, created_at";

const JOIN_REQUEST_COLUMNS: &str = "id, invite_id, company_id, request_type, status, request_ip,
    requesting_user_id, request_email_snapshot, agent_name, adapter_type, capabilities,
    agent_defaults_payload, claim_secret_hash, claim_secret_expires_at, claim_secret_consumed_at,
    created_agent_id, approved_by_user_id, approved_at, rejected_by_user_id, rejected_at, created_at";

#[async_trait]
impl InviteRepository for TursoInviteRepository {
    async fn create_invite(&self, input: NewInvite) -> Result<(InviteRecord, String), InviteError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(InviteError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let token = format!("inv-{}", Uuid::new_v4());
        let token_hash = helpers::sha256_hex(&token);
        let defaults = input.defaults_payload.map(|value| value.to_string());
        conn.execute(
            "INSERT INTO invites
               (id, company_id, invite_type, token_hash, allowed_join_types, defaults_payload,
                expires_at, invited_by_user_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.invite_type,
                token_hash,
                input.allowed_join_types,
                defaults,
                input.expires_at,
                input.invited_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {INVITE_COLUMNS} FROM invites WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("invite was just inserted");
        Ok((row_to_invite(&row)?, token))
    }

    async fn list_invites(&self, company_id: &str) -> Result<Vec<InviteRecord>, InviteError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {INVITE_COLUMNS} FROM invites WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut invites = Vec::new();
        while let Some(row) = rows.next().await? {
            invites.push(row_to_invite(&row)?);
        }
        Ok(invites)
    }

    async fn revoke_invite(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<InviteRecord>, InviteError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE invites SET revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                                   updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?1 AND id = ?2 AND revoked_at IS NULL",
                libsql::params![company_id, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {INVITE_COLUMNS} FROM invites WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("invite exists");
        Ok(Some(row_to_invite(&row)?))
    }

    async fn create_join_request(
        &self,
        input: NewJoinRequest,
    ) -> Result<JoinRequestRecord, InviteError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {INVITE_COLUMNS} FROM invites WHERE company_id = ?1 AND id = ?2"),
                libsql::params![input.company_id.clone(), input.invite_id.clone()],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(InviteError::InviteNotFound);
        };
        let invite = row_to_invite(&row)?;
        let mut now_rows = conn
            .query(
                "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![],
            )
            .await?;
        let now = helpers::row_text(&now_rows.next().await?.expect("now"), 0)?.expect("now");
        if invite.revoked_at.is_some() || invite.expires_at.as_str() <= now.as_str() {
            return Err(InviteError::InviteRevokedOrExpired);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO join_requests
                   (id, invite_id, company_id, request_type, status, request_ip,
                    requesting_user_id, request_email_snapshot, agent_name, adapter_type,
                    capabilities, agent_defaults_payload, claim_secret_hash,
                    claim_secret_expires_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'pending_approval', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.invite_id,
                    input.company_id,
                    input.request_type,
                    input.request_ip,
                    input.requesting_user_id,
                    input.request_email_snapshot,
                    input.agent_name,
                    input.adapter_type,
                    input.capabilities,
                    input.agent_defaults_payload.map(|value| value.to_string()),
                    input.claim_secret_hash,
                    input.claim_secret_expires_at
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {JOIN_REQUEST_COLUMNS} FROM join_requests WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("join request was just inserted");
                Ok(row_to_join_request(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(InviteError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_join_requests(
        &self,
        company_id: &str,
    ) -> Result<Vec<JoinRequestRecord>, InviteError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOIN_REQUEST_COLUMNS} FROM join_requests
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut requests = Vec::new();
        while let Some(row) = rows.next().await? {
            requests.push(row_to_join_request(&row)?);
        }
        Ok(requests)
    }

    async fn approve(
        &self,
        company_id: &str,
        id: &str,
        approved_by_user_id: Option<String>,
    ) -> Result<JoinRequestRecord, InviteError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOIN_REQUEST_COLUMNS} FROM join_requests WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(InviteError::JoinRequestNotFound);
        };
        let request = row_to_join_request(&row)?;
        if request.status != "pending_approval" {
            return Err(InviteError::NotPending);
        }
        let mut created_agent_id = request.created_agent_id.clone();
        if request.request_type == "agent" && created_agent_id.is_none() {
            let agent_id = Uuid::new_v4().to_string();
            let adapter = request
                .adapter_type
                .clone()
                .unwrap_or_else(|| "cli".to_owned());
            conn.execute(
                "INSERT INTO agents (id, company_id, name, role, adapter_type, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'worker', ?4,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    agent_id.clone(),
                    company_id,
                    request.agent_name.clone().unwrap_or_else(|| "Agent".to_owned()),
                    adapter
                ],
            )
            .await?;
            conn.execute(
                "INSERT INTO company_memberships
                   (id, company_id, principal_type, principal_id, status, membership_role,
                    created_at, updated_at)
                 VALUES (?1, ?2, 'agent', ?3, 'active', 'operator',
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![Uuid::new_v4().to_string(), company_id, agent_id.clone()],
            )
            .await?;
            created_agent_id = Some(agent_id);
        } else if request.request_type == "human" {
            conn.execute(
                "INSERT INTO company_memberships
                   (id, company_id, principal_type, principal_id, status, membership_role,
                    created_at, updated_at)
                 VALUES (?1, ?2, 'user', ?3, 'active', 'operator',
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT (company_id, principal_type, principal_id)
                 DO UPDATE SET status = 'active', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![
                    Uuid::new_v4().to_string(),
                    company_id,
                    request.requesting_user_id.clone().unwrap_or_default()
                ],
            )
            .await?;
        }
        conn.execute(
            "UPDATE join_requests SET status = 'approved', created_agent_id = ?1,
                    approved_by_user_id = ?2,
                    approved_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?3",
            libsql::params![created_agent_id, approved_by_user_id, id],
        )
        .await?;
        conn.execute(
            "UPDATE invites SET accepted_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![request.invite_id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {JOIN_REQUEST_COLUMNS} FROM join_requests WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("join request exists");
        Ok(row_to_join_request(&row)?)
    }

    async fn reject(
        &self,
        company_id: &str,
        id: &str,
        rejected_by_user_id: Option<String>,
    ) -> Result<JoinRequestRecord, InviteError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {JOIN_REQUEST_COLUMNS} FROM join_requests WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Err(InviteError::JoinRequestNotFound);
        };
        let request = row_to_join_request(&row)?;
        if request.status != "pending_approval" {
            return Err(InviteError::NotPending);
        }
        conn.execute(
            "UPDATE join_requests SET status = 'rejected', rejected_by_user_id = ?1,
                    rejected_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?2",
            libsql::params![rejected_by_user_id, id],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {JOIN_REQUEST_COLUMNS} FROM join_requests WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("join request exists");
        Ok(row_to_join_request(&row)?)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoInviteRepository) {
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
        (dir, TursoInviteRepository::new(db))
    }

    fn future() -> String {
        "2999-01-01T00:00:00.000Z".to_owned()
    }

    #[tokio::test]
    async fn invite_join_request_approve_agent_flow() {
        let (_dir, repo) = repo().await;
        let (invite, token) = repo
            .create_invite(NewInvite {
                company_id: "c1".to_owned(),
                invite_type: "company_join".to_owned(),
                allowed_join_types: "both".to_owned(),
                defaults_payload: None,
                expires_at: future(),
                invited_by_user_id: None,
            })
            .await
            .unwrap();
        assert!(token.starts_with("inv-"));
        assert!(repo.list_invites("c1").await.unwrap().len() == 1);

        let request = repo
            .create_join_request(NewJoinRequest {
                company_id: "c1".to_owned(),
                invite_id: invite.id.clone(),
                request_type: "agent".to_owned(),
                request_ip: "127.0.0.1".to_owned(),
                requesting_user_id: None,
                request_email_snapshot: None,
                agent_name: Some("Helper".to_owned()),
                adapter_type: Some("cli".to_owned()),
                capabilities: None,
                agent_defaults_payload: None,
                claim_secret_hash: Some("abc".to_owned()),
                claim_secret_expires_at: Some(future()),
            })
            .await
            .unwrap();
        assert_eq!(request.status, "pending_approval");

        let err = repo
            .create_join_request(NewJoinRequest {
                company_id: "c1".to_owned(),
                invite_id: invite.id.clone(),
                request_type: "agent".to_owned(),
                request_ip: "127.0.0.1".to_owned(),
                requesting_user_id: None,
                request_email_snapshot: None,
                agent_name: None,
                adapter_type: None,
                capabilities: None,
                agent_defaults_payload: None,
                claim_secret_hash: None,
                claim_secret_expires_at: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, InviteError::AlreadyExists));

        let approved = repo.approve("c1", &request.id, None).await.unwrap();
        assert_eq!(approved.status, "approved");
        assert!(approved.created_agent_id.is_some());
        // Approving again fails.
        assert!(matches!(
            repo.approve("c1", &request.id, None).await.unwrap_err(),
            InviteError::NotPending
        ));

        // The created agent has a membership.
        let memberships = crate::connect(&repo.db).await.unwrap();
        let mut rows = memberships
            .query(
                "SELECT COUNT(*) FROM company_memberships WHERE company_id = 'c1'",
                libsql::params![],
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(helpers::row_i64(&row, 0).unwrap(), 1);
    }

    #[tokio::test]
    async fn revoked_or_expired_invite_rejects_join_request() {
        let (_dir, repo) = repo().await;
        let (invite, _) = repo
            .create_invite(NewInvite {
                company_id: "c1".to_owned(),
                invite_type: "company_join".to_owned(),
                allowed_join_types: "human".to_owned(),
                defaults_payload: None,
                expires_at: future(),
                invited_by_user_id: None,
            })
            .await
            .unwrap();
        repo.revoke_invite("c1", &invite.id).await.unwrap().unwrap();
        let err = repo
            .create_join_request(NewJoinRequest {
                company_id: "c1".to_owned(),
                invite_id: invite.id.clone(),
                request_type: "human".to_owned(),
                request_ip: "127.0.0.1".to_owned(),
                requesting_user_id: Some("u1".to_owned()),
                request_email_snapshot: None,
                agent_name: None,
                adapter_type: None,
                capabilities: None,
                agent_defaults_payload: None,
                claim_secret_hash: None,
                claim_secret_expires_at: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, InviteError::InviteRevokedOrExpired));
    }

    #[tokio::test]
    async fn human_join_request_approval_creates_membership() {
        let (_dir, repo) = repo().await;
        let (invite, _) = repo
            .create_invite(NewInvite {
                company_id: "c1".to_owned(),
                invite_type: "company_join".to_owned(),
                allowed_join_types: "human".to_owned(),
                defaults_payload: None,
                expires_at: future(),
                invited_by_user_id: None,
            })
            .await
            .unwrap();
        let request = repo
            .create_join_request(NewJoinRequest {
                company_id: "c1".to_owned(),
                invite_id: invite.id,
                request_type: "human".to_owned(),
                request_ip: "10.0.0.1".to_owned(),
                requesting_user_id: Some("u-human".to_owned()),
                request_email_snapshot: Some("human@example.com".to_owned()),
                agent_name: None,
                adapter_type: None,
                capabilities: None,
                agent_defaults_payload: None,
                claim_secret_hash: None,
                claim_secret_expires_at: None,
            })
            .await
            .unwrap();
        let approved = repo
            .approve("c1", &request.id, Some("board".to_owned()))
            .await
            .unwrap();
        assert_eq!(approved.status, "approved");

        let conn = crate::connect(&repo.db).await.unwrap();
        let mut rows = conn
            .query(
                "SELECT principal_type, principal_id, membership_role FROM company_memberships
                 WHERE company_id = 'c1'",
                libsql::params![],
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(helpers::row_text(&row, 0).unwrap().as_deref(), Some("user"));
        assert_eq!(
            helpers::row_text(&row, 1).unwrap().as_deref(),
            Some("u-human")
        );
        assert_eq!(
            helpers::row_text(&row, 2).unwrap().as_deref(),
            Some("operator")
        );
    }
}
