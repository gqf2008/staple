//! Companies repository: trait plus Turso/libSQL implementation.
//!
//! Mirrors the upstream company row shape (SPEC §7.1) and the issue-prefix
//! allocation rules from `server/src/services/companies.ts`.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

/// A row of the `companies` table.
#[derive(Debug, Clone)]
pub struct CompanyRecord {
    /// Company id (UUID string).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// `active | paused | archived`.
    pub status: String,
    /// Reason for pausing, when paused.
    pub pause_reason: Option<String>,
    /// ISO 8601 timestamp when paused, if paused.
    pub paused_at: Option<String>,
    /// Issue identifier prefix (unique per company).
    pub issue_prefix: String,
    /// Next issue number for this company.
    pub issue_counter: i64,
    /// Monthly budget in cents.
    pub budget_monthly_cents: i64,
    /// Spent this month in cents.
    pub spent_monthly_cents: i64,
    /// Largest allowed attachment size in bytes.
    pub attachment_max_bytes: i64,
    /// Default responsible user, if any.
    pub default_responsible_user_id: Option<String>,
    /// Whether new agents need board approval.
    pub require_board_approval_for_new_agents: bool,
    /// Feedback data sharing consent state.
    pub feedback_data_sharing_enabled: bool,
    /// ISO 8601 timestamp of consent, if given.
    pub feedback_data_sharing_consent_at: Option<String>,
    /// User id that gave consent.
    pub feedback_data_sharing_consent_by_user_id: Option<String>,
    /// Consent terms version.
    pub feedback_data_sharing_terms_version: Option<String>,
    /// Brand color.
    pub brand_color: Option<String>,
    /// Logo asset id.
    pub logo_asset_id: Option<String>,
    /// Logo URL.
    pub logo_url: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating a company.
#[derive(Debug, Clone)]
pub struct NewCompany {
    /// Display name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Monthly budget in cents.
    pub budget_monthly_cents: i64,
    /// Largest allowed attachment size in bytes.
    pub attachment_max_bytes: i64,
}

/// Partial update for a company. `Option<Option<T>>` distinguishes "leave
/// unchanged" from "set to NULL".
#[derive(Debug, Default)]
pub struct CompanyPatch {
    /// New name.
    pub name: Option<String>,
    /// New description (`Some(None)` clears it).
    pub description: Option<Option<String>>,
    /// New status.
    pub status: Option<String>,
    /// New monthly budget in cents.
    pub budget_monthly_cents: Option<i64>,
    /// New spent-this-month amount in cents.
    pub spent_monthly_cents: Option<i64>,
    /// New attachment size limit in bytes.
    pub attachment_max_bytes: Option<i64>,
    /// New require-board-approval flag.
    pub require_board_approval_for_new_agents: Option<bool>,
    /// New brand color (`Some(None)` clears it).
    pub brand_color: Option<Option<String>>,
}

/// Repository errors.
#[derive(Debug, Error)]
pub enum RepoError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The generated issue prefix collided with an existing company.
    #[error("a company with this issue prefix already exists")]
    IssuePrefixConflict,
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
}

/// Company persistence contract.
#[async_trait]
pub trait CompanyRepository: Send + Sync {
    /// Creates a company with a derived unique issue prefix.
    ///
    /// # Errors
    ///
    /// Returns [`RepoError`] on database failure.
    async fn create(&self, input: NewCompany) -> Result<CompanyRecord, RepoError>;

    /// Lists all companies.
    ///
    /// # Errors
    ///
    /// Returns [`RepoError`] on database failure.
    async fn list(&self) -> Result<Vec<CompanyRecord>, RepoError>;

    /// Fetches one company by id, or `None` when it does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`RepoError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<CompanyRecord>, RepoError>;

    /// Applies a partial update, or returns `None` when the company does not
    /// exist.
    ///
    /// # Errors
    ///
    /// Returns [`RepoError`] on database failure.
    async fn update(
        &self,
        id: &str,
        patch: CompanyPatch,
    ) -> Result<Option<CompanyRecord>, RepoError>;
}

/// Turso/libSQL implementation of [`CompanyRepository`].
#[derive(Debug)]
pub struct TursoCompanyRepository {
    db: Database,
}

impl TursoCompanyRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const COMPANY_COLUMNS: &str = "id, name, description, status, pause_reason, paused_at,
    issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents,
    attachment_max_bytes, default_responsible_user_id,
    require_board_approval_for_new_agents, feedback_data_sharing_enabled,
    feedback_data_sharing_consent_at, feedback_data_sharing_consent_by_user_id,
    feedback_data_sharing_terms_version, brand_color, logo_asset_id, logo_url,
    created_at, updated_at";

fn row_text(row: &libsql::Row, idx: i32) -> Result<Option<String>, libsql::Error> {
    let value = row.get_value(idx)?;
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value.as_text().expect("TEXT column").clone()))
    }
}

fn row_i64(row: &libsql::Row, idx: i32) -> Result<i64, libsql::Error> {
    let value = row.get_value(idx)?;
    Ok(*value.as_integer().expect("INTEGER column"))
}

fn row_bool(row: &libsql::Row, idx: i32) -> Result<bool, libsql::Error> {
    Ok(row_i64(row, idx)? != 0)
}

fn row_to_company(row: &libsql::Row) -> Result<CompanyRecord, libsql::Error> {
    Ok(CompanyRecord {
        id: row_text(row, 0)?.expect("id is NOT NULL"),
        name: row_text(row, 1)?.expect("name is NOT NULL"),
        description: row_text(row, 2)?,
        status: row_text(row, 3)?.expect("status is NOT NULL"),
        pause_reason: row_text(row, 4)?,
        paused_at: row_text(row, 5)?,
        issue_prefix: row_text(row, 6)?.expect("issue_prefix is NOT NULL"),
        issue_counter: row_i64(row, 7)?,
        budget_monthly_cents: row_i64(row, 8)?,
        spent_monthly_cents: row_i64(row, 9)?,
        attachment_max_bytes: row_i64(row, 10)?,
        default_responsible_user_id: row_text(row, 11)?,
        require_board_approval_for_new_agents: row_bool(row, 12)?,
        feedback_data_sharing_enabled: row_bool(row, 13)?,
        feedback_data_sharing_consent_at: row_text(row, 14)?,
        feedback_data_sharing_consent_by_user_id: row_text(row, 15)?,
        feedback_data_sharing_terms_version: row_text(row, 16)?,
        brand_color: row_text(row, 17)?,
        logo_asset_id: row_text(row, 18)?,
        logo_url: row_text(row, 19)?,
        created_at: row_text(row, 20)?.expect("created_at is NOT NULL"),
        updated_at: row_text(row, 21)?.expect("updated_at is NOT NULL"),
    })
}

/// Derives the base issue prefix from a company name, matching upstream:
/// uppercase, keep A-Z only, take the first three letters.
fn derive_issue_prefix_base(name: &str) -> String {
    let normalized = name
        .to_uppercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect::<String>();
    if normalized.is_empty() {
        "CMP".to_owned()
    } else {
        normalized.chars().take(3).collect()
    }
}

#[async_trait]
impl CompanyRepository for TursoCompanyRepository {
    async fn create(&self, input: NewCompany) -> Result<CompanyRecord, RepoError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let base = derive_issue_prefix_base(&input.name);

        // Try progressively longer suffixes ("" , "A", "AA", ...) on prefix
        // collisions, mirroring the upstream allocator.
        for attempt in 1..100 {
            let suffix = if attempt == 1 {
                String::new()
            } else {
                "A".repeat(attempt - 1)
            };
            let candidate = format!("{base}{suffix}");
            let result = conn
                .execute(
                    "INSERT INTO companies (
                        id, name, description, status, issue_prefix, issue_counter,
                        budget_monthly_cents, spent_monthly_cents, attachment_max_bytes,
                        require_board_approval_for_new_agents, created_at, updated_at
                     ) VALUES (
                        ?1, ?2, ?3, 'active', ?4, 0, ?5, 0, ?6, 0,
                        strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                        strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     )",
                    libsql::params![
                        id.clone(),
                        input.name.clone(),
                        input.description.clone(),
                        candidate.clone(),
                        input.budget_monthly_cents,
                        input.attachment_max_bytes
                    ],
                )
                .await;
            match result {
                Ok(_) => {
                    tracing::debug!(company_id = %id, prefix = %candidate, "created company");
                    return Ok(self.get(&id).await?.expect("company was just inserted"));
                }
                Err(error) if is_unique_violation(&error) => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err(RepoError::IssuePrefixConflict)
    }

    async fn list(&self) -> Result<Vec<CompanyRecord>, RepoError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {COMPANY_COLUMNS} FROM companies ORDER BY created_at");
        let mut rows = conn.query(&sql, ()).await?;
        let mut companies = Vec::new();
        while let Some(row) = rows.next().await? {
            companies.push(row_to_company(&row)?);
        }
        Ok(companies)
    }

    async fn get(&self, id: &str) -> Result<Option<CompanyRecord>, RepoError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {COMPANY_COLUMNS} FROM companies WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_company(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        id: &str,
        patch: CompanyPatch,
    ) -> Result<Option<CompanyRecord>, RepoError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut param = 0usize;
        // `Some(Some(v))` binds a value; `Some(None)` sets the column to NULL;
        // `None` leaves the column untouched.
        let mut push = |column: &str, value: Option<Option<libsql::Value>>| match value {
            Some(Some(value)) => {
                param += 1;
                sets.push(format!("{column} = ?{param}"));
                values.push(value);
            }
            Some(None) => sets.push(format!("{column} = NULL")),
            None => {}
        };

        push("name", patch.name.map(|value| Some(value.into())));
        push(
            "description",
            patch.description.map(|value| value.map(Into::into)),
        );
        push("status", patch.status.map(|value| Some(value.into())));
        push(
            "budget_monthly_cents",
            patch.budget_monthly_cents.map(|value| Some(value.into())),
        );
        push(
            "spent_monthly_cents",
            patch.spent_monthly_cents.map(|value| Some(value.into())),
        );
        push(
            "attachment_max_bytes",
            patch.attachment_max_bytes.map(|value| Some(value.into())),
        );
        push(
            "require_board_approval_for_new_agents",
            patch
                .require_board_approval_for_new_agents
                .map(|value| Some(libsql::Value::from(i64::from(value)))),
        );
        push(
            "brand_color",
            patch.brand_color.map(|value| value.map(Into::into)),
        );

        if sets.is_empty() {
            return self.get(id).await;
        }

        param += 1;
        values.push(libsql::Value::from(id.to_owned()));
        let sql = format!(
            "UPDATE companies SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?{param}",
            sets.join(", "),
        );
        let updated = conn.execute(&sql, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        Ok(self.get(id).await?)
    }
}

fn is_unique_violation(error: &libsql::Error) -> bool {
    error.to_string().contains("UNIQUE constraint failed")
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoCompanyRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoCompanyRepository::new(db);
        (dir, repo, conn)
    }

    #[tokio::test]
    async fn create_assigns_id_prefix_and_defaults() {
        let (_dir, repo, _conn) = repo().await;
        let company = repo
            .create(NewCompany {
                name: "Acme Corp".to_owned(),
                description: Some("d".to_owned()),
                budget_monthly_cents: 1000,
                attachment_max_bytes: 1024,
            })
            .await
            .unwrap();
        assert_eq!(company.name, "Acme Corp");
        assert_eq!(company.status, "active");
        assert_eq!(company.issue_prefix, "ACM");
        assert_eq!(company.issue_counter, 0);
        assert_eq!(company.budget_monthly_cents, 1000);
        assert_eq!(company.spent_monthly_cents, 0);
        assert!(!company.id.is_empty());
    }

    #[tokio::test]
    async fn prefix_is_derived_and_unique() {
        let (_dir, repo, _conn) = repo().await;
        let a = repo
            .create(NewCompany {
                name: "Acme".to_owned(),
                description: None,
                budget_monthly_cents: 0,
                attachment_max_bytes: 1024,
            })
            .await
            .unwrap();
        let b = repo
            .create(NewCompany {
                name: "Acme Again".to_owned(),
                description: None,
                budget_monthly_cents: 0,
                attachment_max_bytes: 1024,
            })
            .await
            .unwrap();
        assert_eq!(a.issue_prefix, "ACM");
        assert_eq!(b.issue_prefix, "ACMA");
        assert_ne!(a.id, b.id);
    }

    #[tokio::test]
    async fn get_returns_none_for_missing() {
        let (_dir, repo, _conn) = repo().await;
        assert!(repo.get("missing").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update_applies_fields_and_clears_optionals() {
        let (_dir, repo, _conn) = repo().await;
        let created = repo
            .create(NewCompany {
                name: "Acme".to_owned(),
                description: Some("hello".to_owned()),
                budget_monthly_cents: 0,
                attachment_max_bytes: 1024,
            })
            .await
            .unwrap();

        let updated = repo
            .update(
                &created.id,
                CompanyPatch {
                    name: Some("Acme 2".to_owned()),
                    description: Some(None),
                    status: Some("paused".to_owned()),
                    budget_monthly_cents: Some(500),
                    spent_monthly_cents: Some(75),
                    attachment_max_bytes: Some(2048),
                    require_board_approval_for_new_agents: Some(true),
                    brand_color: Some(Some("#ff0000".to_owned())),
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "Acme 2");
        assert_eq!(updated.description, None);
        assert_eq!(updated.status, "paused");
        assert_eq!(updated.budget_monthly_cents, 500);
        assert!(updated.require_board_approval_for_new_agents);
        assert_eq!(updated.brand_color.as_deref(), Some("#ff0000"));
        assert_eq!(updated.issue_prefix, created.issue_prefix);
    }

    #[tokio::test]
    async fn update_missing_returns_none() {
        let (_dir, repo, _conn) = repo().await;
        let result = repo
            .update(
                "missing",
                CompanyPatch {
                    name: Some("x".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
