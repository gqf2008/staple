//! Principal permission grants repository (upstream §9.8).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `principal_permission_grants` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGrantRecord {
    /// Grant id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Principal type (`agent` or `user`).
    pub principal_type: String,
    /// Principal id.
    pub principal_id: String,
    /// Permission key (e.g. `tasks:assign_scope`, `inbox:manage`).
    pub permission_key: String,
    /// JSON scope object, or `None` for an unscoped grant.
    pub scope: Option<serde_json::Value>,
    /// Board user id that created/updated the grant.
    pub granted_by_user_id: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating or replacing a permission grant.
#[derive(Debug, Clone)]
pub struct NewPermissionGrant {
    /// Owning company id.
    pub company_id: String,
    /// Principal type (`agent` or `user`).
    pub principal_type: String,
    /// Principal id.
    pub principal_id: String,
    /// Permission key.
    pub permission_key: String,
    /// JSON scope object (validated by the route), or `None` for unscoped.
    pub scope: Option<serde_json::Value>,
    /// Board user id that created/updated the grant.
    pub granted_by_user_id: Option<String>,
}

/// Permission grant repository errors.
#[derive(Debug, Error)]
pub enum PermissionGrantError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The referenced principal (agent) does not exist in this company.
    #[error("principal not found")]
    PrincipalNotFound,
    /// The grant does not exist.
    #[error("grant not found")]
    NotFound,
}

/// Permission grant persistence contract.
#[async_trait]
pub trait PermissionGrantRepository: Send + Sync {
    /// Creates or replaces a grant (upsert on the company-scoped unique key).
    ///
    /// # Errors
    ///
    /// Returns [`PermissionGrantError`] on invalid references.
    async fn upsert(
        &self,
        input: NewPermissionGrant,
    ) -> Result<PermissionGrantRecord, PermissionGrantError>;

    /// Lists all grants for a company.
    ///
    /// # Errors
    ///
    /// Returns [`PermissionGrantError`] on database failure.
    async fn list(
        &self,
        company_id: &str,
    ) -> Result<Vec<PermissionGrantRecord>, PermissionGrantError>;

    /// Finds one grant by its natural key.
    ///
    /// # Errors
    ///
    /// Returns [`PermissionGrantError`] on database failure.
    async fn find(
        &self,
        company_id: &str,
        principal_type: &str,
        principal_id: &str,
        permission_key: &str,
    ) -> Result<Option<PermissionGrantRecord>, PermissionGrantError>;

    /// Deletes a grant (company-scoped; cross-company id returns not found).
    ///
    /// # Errors
    ///
    /// Returns [`PermissionGrantError`] on database failure.
    async fn delete(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PermissionGrantRecord>, PermissionGrantError>;
}

/// Turso/libSQL implementation of [`PermissionGrantRepository`].
#[derive(Debug)]
pub struct TursoPermissionGrantRepository {
    db: Database,
}

impl TursoPermissionGrantRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_record(row: &libsql::Row) -> Result<PermissionGrantRecord, libsql::Error> {
    Ok(PermissionGrantRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        principal_type: helpers::row_text(row, 2)?.expect("principal_type"),
        principal_id: helpers::row_text(row, 3)?.expect("principal_id"),
        permission_key: helpers::row_text(row, 4)?.expect("permission_key"),
        scope: helpers::row_text(row, 5)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        granted_by_user_id: helpers::row_text(row, 6)?,
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
        updated_at: helpers::row_text(row, 8)?.expect("updated_at"),
    })
}

#[async_trait]
impl PermissionGrantRepository for TursoPermissionGrantRepository {
    async fn upsert(
        &self,
        input: NewPermissionGrant,
    ) -> Result<PermissionGrantRecord, PermissionGrantError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(PermissionGrantError::CompanyNotFound);
        }
        if input.principal_type == "agent"
            && !helpers::row_belongs_to_company(
                &conn,
                "agents",
                &input.principal_id,
                &input.company_id,
            )
            .await?
        {
            return Err(PermissionGrantError::PrincipalNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let scope = input.scope.map(|value| value.to_string());
        conn.execute(
            "INSERT INTO principal_permission_grants
               (id, company_id, principal_type, principal_id, permission_key, scope,
                granted_by_user_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, principal_type, principal_id, permission_key)
             DO UPDATE SET scope = excluded.scope,
                           granted_by_user_id = excluded.granted_by_user_id,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.principal_type.clone(),
                input.principal_id.clone(),
                input.permission_key.clone(),
                scope,
                input.granted_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, principal_type, principal_id, permission_key, scope,
                        granted_by_user_id, created_at, updated_at
                 FROM principal_permission_grants
                 WHERE company_id = ?1 AND principal_type = ?2 AND principal_id = ?3
                   AND permission_key = ?4",
                libsql::params![
                    input.company_id,
                    input.principal_type,
                    input.principal_id,
                    input.permission_key
                ],
            )
            .await?;
        let row = rows.next().await?.expect("grant was just upserted");
        Ok(row_to_record(&row)?)
    }

    async fn list(
        &self,
        company_id: &str,
    ) -> Result<Vec<PermissionGrantRecord>, PermissionGrantError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, principal_type, principal_id, permission_key, scope,
                        granted_by_user_id, created_at, updated_at
                 FROM principal_permission_grants
                 WHERE company_id = ?1 ORDER BY permission_key, principal_id",
                libsql::params![company_id],
            )
            .await?;
        let mut grants = Vec::new();
        while let Some(row) = rows.next().await? {
            grants.push(row_to_record(&row)?);
        }
        Ok(grants)
    }

    async fn find(
        &self,
        company_id: &str,
        principal_type: &str,
        principal_id: &str,
        permission_key: &str,
    ) -> Result<Option<PermissionGrantRecord>, PermissionGrantError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, principal_type, principal_id, permission_key, scope,
                        granted_by_user_id, created_at, updated_at
                 FROM principal_permission_grants
                 WHERE company_id = ?1 AND principal_type = ?2 AND principal_id = ?3
                   AND permission_key = ?4",
                libsql::params![company_id, principal_type, principal_id, permission_key],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_record(&row)?)),
            None => Ok(None),
        }
    }

    async fn delete(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PermissionGrantRecord>, PermissionGrantError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, principal_type, principal_id, permission_key, scope,
                        granted_by_user_id, created_at, updated_at
                 FROM principal_permission_grants WHERE company_id = ?1 AND id = ?2",
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_record(&row)?;
        conn.execute(
            "DELETE FROM principal_permission_grants WHERE id = ?1",
            libsql::params![id],
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

    async fn repo() -> (TempDir, TursoPermissionGrantRepository) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .expect("open");
        migrate(&db).await.expect("migrate");
        let conn = crate::connection::connect(&db).await.expect("connect");
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('11111111-1111-1111-1111-111111111111', 'Acme', 'ACME', 1048576)",
            libsql::params![],
        )
        .await
        .expect("seed company");
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type, reports_to,
                                 created_at, updated_at)
             VALUES ('22222222-2222-2222-2222-222222222222', '11111111-1111-1111-1111-111111111111',
                     'Worker', 'worker', 'cli', NULL,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![],
        )
        .await
        .expect("seed agent");
        (dir, TursoPermissionGrantRepository::new(db))
    }

    #[tokio::test]
    async fn upsert_find_list_delete_roundtrip() {
        let (_dir, repo) = repo().await;
        let company = "11111111-1111-1111-1111-111111111111";
        let agent = "22222222-2222-2222-2222-222222222222";
        let created = repo
            .upsert(NewPermissionGrant {
                company_id: company.to_owned(),
                principal_type: "agent".to_owned(),
                principal_id: agent.to_owned(),
                permission_key: "tasks:assign_scope".to_owned(),
                scope: Some(serde_json::json!({ "projectId": "p-1" })),
                granted_by_user_id: None,
            })
            .await
            .expect("upsert");
        assert_eq!(
            created
                .scope
                .as_ref()
                .and_then(|v| v.get("projectId"))
                .and_then(|v| v.as_str()),
            Some("p-1")
        );

        // Upsert replaces the scope and keeps one row.
        let replaced = repo
            .upsert(NewPermissionGrant {
                company_id: company.to_owned(),
                principal_type: "agent".to_owned(),
                principal_id: agent.to_owned(),
                permission_key: "tasks:assign_scope".to_owned(),
                scope: Some(serde_json::json!({ "agentIds": [agent] })),
                granted_by_user_id: None,
            })
            .await
            .expect("re-upsert");
        assert_eq!(replaced.id, created.id);
        assert!(
            replaced
                .scope
                .as_ref()
                .and_then(|v| v.get("agentIds"))
                .is_some()
        );

        let found = repo
            .find(company, "agent", agent, "tasks:assign_scope")
            .await
            .expect("find")
            .expect("exists");
        assert_eq!(found.id, created.id);

        let list = repo.list(company).await.expect("list");
        assert_eq!(list.len(), 1);

        let deleted = repo.delete(company, &created.id).await.expect("delete");
        assert!(deleted.is_some());
        assert!(
            repo.find(company, "agent", agent, "tasks:assign_scope")
                .await
                .expect("find")
                .is_none()
        );
    }

    #[tokio::test]
    async fn foreign_principal_is_rejected() {
        let (_dir, repo) = repo().await;
        let company = "11111111-1111-1111-1111-111111111111";
        let err = repo
            .upsert(NewPermissionGrant {
                company_id: company.to_owned(),
                principal_type: "agent".to_owned(),
                principal_id: "33333333-3333-3333-3333-333333333333".to_owned(),
                permission_key: "tasks:assign_scope".to_owned(),
                scope: None,
                granted_by_user_id: None,
            })
            .await
            .expect_err("unknown agent");
        assert!(matches!(err, PermissionGrantError::PrincipalNotFound));
    }

    #[tokio::test]
    async fn cross_company_delete_is_not_found() {
        let (_dir, repo) = repo().await;
        let company = "11111111-1111-1111-1111-111111111111";
        let agent = "22222222-2222-2222-2222-222222222222";
        let created = repo
            .upsert(NewPermissionGrant {
                company_id: company.to_owned(),
                principal_type: "agent".to_owned(),
                principal_id: agent.to_owned(),
                permission_key: "inbox:manage".to_owned(),
                scope: None,
                granted_by_user_id: None,
            })
            .await
            .expect("upsert");
        assert!(
            repo.delete("99999999-9999-9999-9999-999999999999", &created.id)
                .await
                .expect("cross-company delete")
                .is_none()
        );
        assert!(
            repo.find(company, "agent", agent, "inbox:manage")
                .await
                .expect("find")
                .is_some()
        );
    }
}
