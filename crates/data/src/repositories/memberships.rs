//! Company memberships and instance user roles repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `company_memberships` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyMembershipRecord {
    /// Membership id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Principal type (`agent` or `user`).
    pub principal_type: String,
    /// Principal id.
    pub principal_id: String,
    /// Status (`active` | `inactive` | `pending` | `removed`).
    pub status: String,
    /// Membership role (`owner` | `admin` | `operator` | `viewer`).
    pub membership_role: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// A row of the `instance_user_roles` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceUserRoleRecord {
    /// Role id.
    pub id: String,
    /// User id.
    pub user_id: String,
    /// Role (`instance_admin`).
    pub role: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// User-level access summary for the instance access page.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccessSummary {
    /// User id.
    pub user_id: String,
    /// Number of companies the user is a member of (any status).
    pub company_count: i64,
    /// Whether the user holds an instance admin role.
    pub instance_admin: bool,
    /// Ids of companies where the user is an active member.
    pub active_company_ids: Vec<String>,
}

/// Company access row for one user.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyAccessRow {
    /// Membership id.
    pub membership_id: String,
    /// Company id.
    pub company_id: String,
    /// Company name.
    pub company_name: String,
    /// Membership status.
    pub status: String,
    /// Membership role.
    pub membership_role: Option<String>,
}

/// Public user profile (no auth table; principal-id based).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileRecord {
    /// User id (principal id).
    pub user_id: String,
    /// Whether the user holds an instance admin role.
    pub instance_admin: bool,
    /// Company memberships with names/roles/statuses.
    pub memberships: Vec<CompanyAccessRow>,
    /// Issues created by the user.
    pub created_issues: i64,
    /// Open issues assigned to the user (not done/cancelled).
    pub assigned_open_issues: i64,
    /// Comments authored by the user.
    pub comment_count: i64,
}

/// Input for creating a company membership.
#[derive(Debug, Clone)]
pub struct NewCompanyMembership {
    /// Owning company id.
    pub company_id: String,
    /// Principal type (`agent` or `user`).
    pub principal_type: String,
    /// Principal id.
    pub principal_id: String,
    /// Membership role.
    pub membership_role: Option<String>,
}

/// Input for creating an instance user role.
#[derive(Debug, Clone)]
pub struct NewInstanceUserRole {
    /// User id.
    pub user_id: String,
    /// Role (`instance_admin`).
    pub role: String,
}

/// Membership repository errors.
#[derive(Debug, Error)]
pub enum MembershipError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The referenced agent principal does not exist in this company.
    #[error("principal not found")]
    PrincipalNotFound,
    /// The membership does not exist.
    #[error("membership not found")]
    NotFound,
}

/// Membership persistence contract.
#[async_trait]
pub trait MembershipRepository: Send + Sync {
    /// Creates or replaces a membership (upsert on company + principal).
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on invalid references.
    async fn upsert(
        &self,
        input: NewCompanyMembership,
    ) -> Result<CompanyMembershipRecord, MembershipError>;

    /// Lists memberships for a company.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn list(&self, company_id: &str)
    -> Result<Vec<CompanyMembershipRecord>, MembershipError>;

    /// Resolves the owning company of a membership.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn company_of(&self, id: &str) -> Result<Option<String>, MembershipError>;

    /// Updates status/role (company-scoped by id).
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn update(
        &self,
        company_id: &str,
        id: &str,
        status: Option<String>,
        membership_role: Option<Option<String>>,
    ) -> Result<Option<CompanyMembershipRecord>, MembershipError>;

    /// Deletes a membership (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn delete(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<CompanyMembershipRecord>, MembershipError>;

    /// Creates or replaces an instance user role.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn upsert_role(
        &self,
        input: NewInstanceUserRole,
    ) -> Result<InstanceUserRoleRecord, MembershipError>;

    /// Lists instance user roles.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn list_roles(&self) -> Result<Vec<InstanceUserRoleRecord>, MembershipError>;

    /// Deletes an instance user role.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn delete_role(
        &self,
        id: &str,
    ) -> Result<Option<InstanceUserRoleRecord>, MembershipError>;

    /// Lists all users known to the instance (user principals + role rows).
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn list_users(&self) -> Result<Vec<UserAccessSummary>, MembershipError>;

    /// Lists a user's access across companies.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn user_company_access(
        &self,
        user_id: &str,
    ) -> Result<Vec<CompanyAccessRow>, MembershipError>;

    /// Sets the set of companies a user can access: activates (or creates)
    /// memberships for `company_ids` and marks all other user memberships
    /// inactive. Returns the number of active memberships after the change.
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] when a target company does not exist.
    async fn set_user_company_access(
        &self,
        user_id: &str,
        company_ids: &[String],
    ) -> Result<usize, MembershipError>;

    /// Builds a public user profile (memberships + roles + issue stats).
    ///
    /// # Errors
    ///
    /// Returns [`MembershipError`] on database failure.
    async fn user_profile(&self, user_id: &str) -> Result<UserProfileRecord, MembershipError>;
}

/// Turso/libSQL implementation of [`MembershipRepository`].
#[derive(Debug)]
pub struct TursoMembershipRepository {
    db: Database,
}

impl TursoMembershipRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_membership(row: &libsql::Row) -> Result<CompanyMembershipRecord, libsql::Error> {
    Ok(CompanyMembershipRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        principal_type: helpers::row_text(row, 2)?.expect("principal_type"),
        principal_id: helpers::row_text(row, 3)?.expect("principal_id"),
        status: helpers::row_text(row, 4)?.expect("status"),
        membership_role: helpers::row_text(row, 5)?,
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
        updated_at: helpers::row_text(row, 7)?.expect("updated_at"),
    })
}

fn row_to_role(row: &libsql::Row) -> Result<InstanceUserRoleRecord, libsql::Error> {
    Ok(InstanceUserRoleRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        user_id: helpers::row_text(row, 1)?.expect("user_id"),
        role: helpers::row_text(row, 2)?.expect("role"),
        created_at: helpers::row_text(row, 3)?.expect("created_at"),
    })
}

const MEMBERSHIP_COLUMNS: &str = "id, company_id, principal_type, principal_id, status,
    membership_role, created_at, updated_at";

#[async_trait]
impl MembershipRepository for TursoMembershipRepository {
    async fn upsert(
        &self,
        input: NewCompanyMembership,
    ) -> Result<CompanyMembershipRecord, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(MembershipError::CompanyNotFound);
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
            return Err(MembershipError::PrincipalNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO company_memberships
               (id, company_id, principal_type, principal_id, status, membership_role,
                created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, principal_type, principal_id)
             DO UPDATE SET status = 'active',
                           membership_role = excluded.membership_role,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.principal_type.clone(),
                input.principal_id.clone(),
                input.membership_role
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MEMBERSHIP_COLUMNS} FROM company_memberships
                     WHERE company_id = ?1 AND principal_type = ?2 AND principal_id = ?3"
                ),
                libsql::params![input.company_id, input.principal_type, input.principal_id],
            )
            .await?;
        let row = rows.next().await?.expect("membership was just upserted");
        Ok(row_to_membership(&row)?)
    }

    async fn list(
        &self,
        company_id: &str,
    ) -> Result<Vec<CompanyMembershipRecord>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MEMBERSHIP_COLUMNS} FROM company_memberships
                     WHERE company_id = ?1 ORDER BY principal_type, principal_id"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut memberships = Vec::new();
        while let Some(row) = rows.next().await? {
            memberships.push(row_to_membership(&row)?);
        }
        Ok(memberships)
    }

    async fn company_of(&self, id: &str) -> Result<Option<String>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(helpers::row_company(&conn, "company_memberships", id).await?)
    }

    async fn update(
        &self,
        company_id: &str,
        id: &str,
        status: Option<String>,
        membership_role: Option<Option<String>>,
    ) -> Result<Option<CompanyMembershipRecord>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut param = 0usize;
        if let Some(status) = status {
            param += 1;
            sets.push(format!("status = ?{param}"));
            values.push(libsql::Value::from(status));
        }
        if let Some(role) = membership_role {
            match role {
                Some(role) => {
                    param += 1;
                    sets.push(format!("membership_role = ?{param}"));
                    values.push(libsql::Value::from(role));
                }
                None => sets.push("membership_role = NULL".to_owned()),
            }
        }
        if sets.is_empty() {
            return Err(MembershipError::NotFound);
        }
        let company_param = param + 1;
        let id_param = param + 2;
        values.push(libsql::Value::from(company_id.to_owned()));
        values.push(libsql::Value::from(id.to_owned()));
        let updated = conn
            .execute(
                &format!(
                    "UPDATE company_memberships SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE company_id = ?{company_param} AND id = ?{id_param}",
                    sets.join(", ")
                ),
                values,
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MEMBERSHIP_COLUMNS} FROM company_memberships WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_membership(&row)?)),
            None => Ok(None),
        }
    }

    async fn delete(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<CompanyMembershipRecord>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MEMBERSHIP_COLUMNS} FROM company_memberships WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_membership(&row)?;
        conn.execute(
            "DELETE FROM company_memberships WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(record))
    }

    async fn upsert_role(
        &self,
        input: NewInstanceUserRole,
    ) -> Result<InstanceUserRoleRecord, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO instance_user_roles (id, user_id, role, created_at, updated_at)
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (user_id, role)
             DO UPDATE SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![id.clone(), input.user_id.clone(), input.role.clone()],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, user_id, role, created_at FROM instance_user_roles
                 WHERE user_id = ?1 AND role = ?2",
                libsql::params![input.user_id, input.role],
            )
            .await?;
        let row = rows.next().await?.expect("role was just upserted");
        Ok(row_to_role(&row)?)
    }

    async fn list_roles(&self) -> Result<Vec<InstanceUserRoleRecord>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, user_id, role, created_at FROM instance_user_roles ORDER BY user_id",
                libsql::params![],
            )
            .await?;
        let mut roles = Vec::new();
        while let Some(row) = rows.next().await? {
            roles.push(row_to_role(&row)?);
        }
        Ok(roles)
    }

    async fn delete_role(
        &self,
        id: &str,
    ) -> Result<Option<InstanceUserRoleRecord>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, user_id, role, created_at FROM instance_user_roles WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_role(&row)?;
        conn.execute(
            "DELETE FROM instance_user_roles WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(record))
    }

    async fn list_users(&self) -> Result<Vec<UserAccessSummary>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut user_ids = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for table in ["company_memberships", "instance_user_roles"] {
            let column = if table == "company_memberships" {
                "principal_id"
            } else {
                "user_id"
            };
            let filter = if table == "company_memberships" {
                " AND principal_type = 'user'"
            } else {
                ""
            };
            let mut rows = conn
                .query(
                    &format!("SELECT DISTINCT {column} FROM {table} WHERE 1=1{filter}"),
                    libsql::params![],
                )
                .await?;
            while let Some(row) = rows.next().await? {
                if let Some(id) = helpers::row_text(&row, 0)?
                    && seen.insert(id.clone())
                {
                    user_ids.push(id);
                }
            }
        }
        let mut users = Vec::new();
        for user_id in user_ids {
            let mut memberships = conn
                .query(
                    "SELECT m.company_id, m.status
                     FROM company_memberships m
                     WHERE m.principal_type = 'user' AND m.principal_id = ?1",
                    libsql::params![user_id.clone()],
                )
                .await?;
            let mut active_company_ids = Vec::new();
            let mut company_count = 0i64;
            while let Some(row) = memberships.next().await? {
                company_count += 1;
                if helpers::row_text(&row, 1)?.as_deref() == Some("active") {
                    active_company_ids.push(helpers::row_text(&row, 0)?.expect("company_id"));
                }
            }
            let mut roles = conn
            .query(
                "SELECT 1 FROM instance_user_roles WHERE user_id = ?1 AND role = 'instance_admin'",
                libsql::params![user_id.clone()],
            )
            .await?;
            let instance_admin = roles.next().await?.is_some();
            users.push(UserAccessSummary {
                user_id,
                company_count,
                instance_admin,
                active_company_ids,
            });
        }
        users.sort_by(|left, right| left.user_id.cmp(&right.user_id));
        Ok(users)
    }

    async fn user_company_access(
        &self,
        user_id: &str,
    ) -> Result<Vec<CompanyAccessRow>, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT m.id, m.company_id, c.name, m.status, m.membership_role
                 FROM company_memberships m
                 JOIN companies c ON c.id = m.company_id
                 WHERE m.principal_type = 'user' AND m.principal_id = ?1
                 ORDER BY c.name",
                libsql::params![user_id],
            )
            .await?;
        let mut access = Vec::new();
        while let Some(row) = rows.next().await? {
            access.push(CompanyAccessRow {
                membership_id: helpers::row_text(&row, 0)?.expect("membership id"),
                company_id: helpers::row_text(&row, 1)?.expect("company id"),
                company_name: helpers::row_text(&row, 2)?.expect("company name"),
                status: helpers::row_text(&row, 3)?.expect("status"),
                membership_role: helpers::row_text(&row, 4)?,
            });
        }
        Ok(access)
    }

    async fn set_user_company_access(
        &self,
        user_id: &str,
        company_ids: &[String],
    ) -> Result<usize, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        for company_id in company_ids {
            if !helpers::company_exists(&conn, company_id).await? {
                return Err(MembershipError::CompanyNotFound);
            }
        }
        for company_id in company_ids {
            let mut rows = conn
                .query(
                    "SELECT membership_role FROM company_memberships
                     WHERE company_id = ?1 AND principal_type = 'user' AND principal_id = ?2",
                    libsql::params![company_id.as_str(), user_id],
                )
                .await?;
            let role = match rows.next().await? {
                Some(row) => helpers::row_text(&row, 0)?,
                None => None,
            };
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO company_memberships
                   (id, company_id, principal_type, principal_id, status, membership_role,
                    created_at, updated_at)
                 VALUES (?1, ?2, 'user', ?3, 'active', ?4,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT (company_id, principal_type, principal_id)
                 DO UPDATE SET status = 'active', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![id, company_id.as_str(), user_id, role],
            )
            .await?;
        }
        if company_ids.is_empty() {
            conn.execute(
                "UPDATE company_memberships SET status = 'inactive',
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE principal_type = 'user' AND principal_id = ?1",
                libsql::params![user_id],
            )
            .await?;
        } else {
            let placeholders = company_ids
                .iter()
                .enumerate()
                .map(|(idx, _)| format!("?{}", idx + 2))
                .collect::<Vec<_>>()
                .join(", ");
            let mut params: Vec<libsql::Value> = vec![libsql::Value::Text(user_id.to_owned())];
            params.extend(company_ids.iter().cloned().map(libsql::Value::Text));
            conn.execute(
                &format!(
                    "UPDATE company_memberships SET status = 'inactive',
                            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE principal_type = 'user' AND principal_id = ?1
                       AND company_id NOT IN ({placeholders})"
                ),
                libsql::params_from_iter(params),
            )
            .await?;
        }
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM company_memberships
                 WHERE principal_type = 'user' AND principal_id = ?1 AND status = 'active'",
                libsql::params![user_id],
            )
            .await?;
        let row = rows.next().await?.expect("count");
        Ok(helpers::row_i64(&row, 0)? as usize)
    }

    async fn user_profile(&self, user_id: &str) -> Result<UserProfileRecord, MembershipError> {
        let conn = crate::connection::connect(&self.db).await?;
        let memberships = self.user_company_access(user_id).await?;
        let mut roles = conn
            .query(
                "SELECT 1 FROM instance_user_roles WHERE user_id = ?1 AND role = 'instance_admin'",
                libsql::params![user_id],
            )
            .await?;
        let instance_admin = roles.next().await?.is_some();
        let mut created = conn
            .query(
                "SELECT COUNT(*) FROM issues WHERE created_by_user_id = ?1",
                libsql::params![user_id],
            )
            .await?;
        let created_issues = helpers::row_i64(&created.next().await?.expect("count"), 0)?;
        let mut assigned = conn
            .query(
                "SELECT COUNT(*) FROM issues
                 WHERE assignee_user_id = ?1 AND status NOT IN ('done', 'cancelled')",
                libsql::params![user_id],
            )
            .await?;
        let assigned_open_issues = helpers::row_i64(&assigned.next().await?.expect("count"), 0)?;
        let mut comments = conn
            .query(
                "SELECT COUNT(*) FROM issue_comments WHERE author_user_id = ?1",
                libsql::params![user_id],
            )
            .await?;
        let comment_count = helpers::row_i64(&comments.next().await?.expect("count"), 0)?;
        Ok(UserProfileRecord {
            user_id: user_id.to_owned(),
            instance_admin,
            memberships,
            created_issues,
            assigned_open_issues,
            comment_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoMembershipRepository) {
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
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'One', 'worker', 'cli')",
            (),
        )
        .await
        .unwrap();
        (dir, TursoMembershipRepository::new(db))
    }

    #[tokio::test]
    async fn membership_lifecycle_and_upsert() {
        let (_dir, repo) = repo().await;
        let first = repo
            .upsert(NewCompanyMembership {
                company_id: "c1".to_owned(),
                principal_type: "agent".to_owned(),
                principal_id: "a1".to_owned(),
                membership_role: Some("operator".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(first.status, "active");
        let second = repo
            .upsert(NewCompanyMembership {
                company_id: "c1".to_owned(),
                principal_type: "agent".to_owned(),
                principal_id: "a1".to_owned(),
                membership_role: Some("admin".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(second.id, first.id);
        assert_eq!(second.membership_role.as_deref(), Some("admin"));

        let list = repo.list("c1").await.unwrap();
        assert_eq!(list.len(), 1);

        let updated = repo
            .update("c1", &first.id, Some("inactive".to_owned()), Some(None))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "inactive");
        assert!(updated.membership_role.is_none());

        assert!(repo.delete("c1", &first.id).await.unwrap().is_some());
        assert!(repo.list("c1").await.unwrap().is_empty());
        // Cross-company delete is not found.
        repo.upsert(NewCompanyMembership {
            company_id: "c1".to_owned(),
            principal_type: "user".to_owned(),
            principal_id: "u1".to_owned(),
            membership_role: None,
        })
        .await
        .unwrap();
        let created = repo.list("c1").await.unwrap();
        assert!(repo.delete("c2", &created[0].id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn agent_principal_must_belong_to_company() {
        let (_dir, repo) = repo().await;
        let err = repo
            .upsert(NewCompanyMembership {
                company_id: "c1".to_owned(),
                principal_type: "agent".to_owned(),
                principal_id: "missing".to_owned(),
                membership_role: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, MembershipError::PrincipalNotFound));
    }

    #[tokio::test]
    async fn instance_roles_upsert_list_delete() {
        let (_dir, repo) = repo().await;
        let role = repo
            .upsert_role(NewInstanceUserRole {
                user_id: "u1".to_owned(),
                role: "instance_admin".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(role.role, "instance_admin");
        let again = repo
            .upsert_role(NewInstanceUserRole {
                user_id: "u1".to_owned(),
                role: "instance_admin".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(again.id, role.id);
        assert_eq!(repo.list_roles().await.unwrap().len(), 1);
        assert!(repo.delete_role(&role.id).await.unwrap().is_some());
        assert!(repo.list_roles().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn user_company_access_management() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();

        // Seed user memberships: u1 active in c1, inactive in c2; u2 active in c1.
        repo.upsert(NewCompanyMembership {
            company_id: "c1".to_owned(),
            principal_type: "user".to_owned(),
            principal_id: "u1".to_owned(),
            membership_role: Some("operator".to_owned()),
        })
        .await
        .unwrap();
        repo.upsert(NewCompanyMembership {
            company_id: "c2".to_owned(),
            principal_type: "user".to_owned(),
            principal_id: "u1".to_owned(),
            membership_role: None,
        })
        .await
        .unwrap();
        repo.upsert(NewCompanyMembership {
            company_id: "c1".to_owned(),
            principal_type: "user".to_owned(),
            principal_id: "u2".to_owned(),
            membership_role: Some("viewer".to_owned()),
        })
        .await
        .unwrap();
        repo.upsert_role(NewInstanceUserRole {
            user_id: "u2".to_owned(),
            role: "instance_admin".to_owned(),
        })
        .await
        .unwrap();

        // list_users aggregates both users with role/company counts.
        let users = repo.list_users().await.unwrap();
        assert_eq!(users.len(), 2);
        let u1 = users.iter().find(|user| user.user_id == "u1").unwrap();
        assert_eq!(u1.company_count, 2);
        assert_eq!(
            u1.active_company_ids,
            vec!["c1".to_owned(), "c2".to_owned()]
        );
        assert!(!u1.instance_admin);
        let u2 = users.iter().find(|user| user.user_id == "u2").unwrap();
        assert!(u2.instance_admin);
        assert_eq!(u2.active_company_ids, vec!["c1".to_owned()]);

        // user_company_access includes company names and statuses.
        let access = repo.user_company_access("u1").await.unwrap();
        assert_eq!(access.len(), 2);
        let c1 = access.iter().find(|row| row.company_id == "c1").unwrap();
        assert_eq!(c1.company_name, "Alpha");
        assert_eq!(c1.status, "active");
        assert_eq!(c1.membership_role.as_deref(), Some("operator"));

        // set_user_company_access activates c2, deactivates c1.
        let active = repo
            .set_user_company_access("u1", &["c2".to_owned()])
            .await
            .unwrap();
        assert_eq!(active, 1);
        let access = repo.user_company_access("u1").await.unwrap();
        let c1 = access.iter().find(|row| row.company_id == "c1").unwrap();
        let c2 = access.iter().find(|row| row.company_id == "c2").unwrap();
        assert_eq!(c1.status, "inactive");
        assert_eq!(c2.status, "active");
        // Role preserved on re-activation.
        let active = repo
            .set_user_company_access("u1", &["c1".to_owned(), "c2".to_owned()])
            .await
            .unwrap();
        assert_eq!(active, 2);
        let access = repo.user_company_access("u1").await.unwrap();
        let c1 = access.iter().find(|row| row.company_id == "c1").unwrap();
        assert_eq!(c1.status, "active");
        assert_eq!(c1.membership_role.as_deref(), Some("operator"));

        // Unknown company is rejected.
        let error = repo
            .set_user_company_access("u1", &["c9".to_owned()])
            .await
            .unwrap_err();
        assert!(matches!(error, MembershipError::CompanyNotFound));
    }

    #[tokio::test]
    async fn user_profile_aggregates_memberships_and_stats() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();
        repo.upsert(NewCompanyMembership {
            company_id: "c1".to_owned(),
            principal_type: "user".to_owned(),
            principal_id: "u1".to_owned(),
            membership_role: Some("operator".to_owned()),
        })
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier, status,
                                 created_by_user_id, assignee_user_id)
             VALUES ('i1', 'c1', 'One', 1, 'ALPHA-1', 'in_progress', 'u1', 'u1'),
                    ('i2', 'c1', 'Two', 2, 'ALPHA-2', 'done', 'u1', NULL),
                    ('i3', 'c1', 'Three', 3, 'ALPHA-3', 'todo', NULL, 'u1')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issue_comments (id, company_id, issue_id, body, author_user_id)
             VALUES ('c1', 'c1', 'i1', 'hello', 'u1')",
            (),
        )
        .await
        .unwrap();

        let profile = repo.user_profile("u1").await.unwrap();
        assert_eq!(profile.user_id, "u1");
        assert!(!profile.instance_admin);
        assert_eq!(profile.memberships.len(), 1);
        assert_eq!(profile.memberships[0].company_name, "Alpha");
        assert_eq!(profile.created_issues, 2);
        assert_eq!(profile.assigned_open_issues, 2); // i1 in_progress + i3 todo
        assert_eq!(profile.comment_count, 1);

        repo.upsert_role(NewInstanceUserRole {
            user_id: "u1".to_owned(),
            role: "instance_admin".to_owned(),
        })
        .await
        .unwrap();
        let profile = repo.user_profile("u1").await.unwrap();
        assert!(profile.instance_admin);
    }
}
