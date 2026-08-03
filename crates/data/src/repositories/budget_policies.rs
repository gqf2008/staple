//! Budget policies and incidents repository (upstream budget_policies.ts /
//! budget_incidents.ts).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `budget_policies` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetPolicyRecord {
    /// Policy id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Scope type (`company` | `agent` | `project`).
    pub scope_type: String,
    /// Scope id.
    pub scope_id: String,
    /// Metric (`billed_cents`).
    pub metric: String,
    /// Window kind (`calendar_month_utc` | `rolling_30d`).
    pub window_kind: String,
    /// Amount limit in cents.
    pub amount: i64,
    /// Warning threshold percent.
    pub warn_percent: i64,
    /// Whether the hard stop is enabled.
    pub hard_stop_enabled: bool,
    /// Whether notifications are enabled.
    pub notify_enabled: bool,
    /// Whether the policy is active.
    pub is_active: bool,
    /// Creating user id.
    pub created_by_user_id: Option<String>,
    /// Updating user id.
    pub updated_by_user_id: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `budget_incidents` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetIncidentRecord {
    /// Incident id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Policy id.
    pub policy_id: String,
    /// Scope type.
    pub scope_type: String,
    /// Scope id.
    pub scope_id: String,
    /// Metric.
    pub metric: String,
    /// Window kind.
    pub window_kind: String,
    /// Window start (ISO 8601).
    pub window_start: String,
    /// Window end (ISO 8601).
    pub window_end: String,
    /// Threshold type (`warn` | `hard_stop`).
    pub threshold_type: String,
    /// Amount limit.
    pub amount_limit: i64,
    /// Amount observed.
    pub amount_observed: i64,
    /// Status (`open` | `resolved` | `dismissed`).
    pub status: String,
    /// Approval id (optional).
    pub approval_id: Option<String>,
    /// ISO 8601 resolution.
    pub resolved_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Input for creating a budget policy.
#[derive(Debug, Clone)]
pub struct NewBudgetPolicy {
    /// Owning company id.
    pub company_id: String,
    /// Scope type.
    pub scope_type: String,
    /// Scope id.
    pub scope_id: String,
    /// Metric.
    pub metric: String,
    /// Window kind.
    pub window_kind: String,
    /// Amount limit.
    pub amount: i64,
    /// Warning percent.
    pub warn_percent: i64,
    /// Hard stop enabled.
    pub hard_stop_enabled: bool,
    /// Notify enabled.
    pub notify_enabled: bool,
    /// Created by user id.
    pub created_by_user_id: Option<String>,
}

/// Input for creating a budget incident.
#[derive(Debug, Clone)]
pub struct NewBudgetIncident {
    /// Owning company id.
    pub company_id: String,
    /// Policy id.
    pub policy_id: String,
    /// Scope type.
    pub scope_type: String,
    /// Scope id.
    pub scope_id: String,
    /// Metric.
    pub metric: String,
    /// Window kind.
    pub window_kind: String,
    /// Window start.
    pub window_start: String,
    /// Window end.
    pub window_end: String,
    /// Threshold type.
    pub threshold_type: String,
    /// Amount limit.
    pub amount_limit: i64,
    /// Amount observed.
    pub amount_observed: i64,
}

/// Budget policy repository errors.
#[derive(Debug, Error)]
pub enum BudgetPolicyError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The policy does not exist.
    #[error("policy not found")]
    PolicyNotFound,
    /// The incident does not exist.
    #[error("incident not found")]
    IncidentNotFound,
    /// The policy is not open.
    #[error("incident not open")]
    NotOpen,
}

/// Budget policy persistence contract.
#[async_trait]
pub trait BudgetPolicyRepository: Send + Sync {
    /// Creates or replaces a policy (upsert on company+scope+metric+window).
    ///
    /// # Errors
    ///
    /// Returns [`BudgetPolicyError`] on invalid references.
    async fn upsert_policy(
        &self,
        input: NewBudgetPolicy,
    ) -> Result<BudgetPolicyRecord, BudgetPolicyError>;

    /// Lists policies for a company.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetPolicyError`] on database failure.
    async fn list_policies(
        &self,
        company_id: &str,
    ) -> Result<Vec<BudgetPolicyRecord>, BudgetPolicyError>;

    /// Deletes a policy (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`BudgetPolicyError`] on database failure.
    async fn delete_policy(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<BudgetPolicyRecord>, BudgetPolicyError>;

    /// Creates an incident (deduped per policy+window+threshold).
    ///
    /// # Errors
    ///
    /// Returns [`BudgetPolicyError`] when the policy is missing.
    async fn create_incident(
        &self,
        input: NewBudgetIncident,
    ) -> Result<BudgetIncidentRecord, BudgetPolicyError>;

    /// Lists incidents for a company.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetPolicyError`] on database failure.
    async fn list_incidents(
        &self,
        company_id: &str,
    ) -> Result<Vec<BudgetIncidentRecord>, BudgetPolicyError>;

    /// Resolves or dismisses an incident.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetPolicyError`] on invalid state.
    async fn set_incident_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
    ) -> Result<Option<BudgetIncidentRecord>, BudgetPolicyError>;
}

/// Turso/libSQL implementation of [`BudgetPolicyRepository`].
#[derive(Debug)]
pub struct TursoBudgetPolicyRepository {
    db: Database,
}

impl TursoBudgetPolicyRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_policy(row: &libsql::Row) -> Result<BudgetPolicyRecord, libsql::Error> {
    Ok(BudgetPolicyRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        scope_type: helpers::row_text(row, 2)?.expect("scope_type"),
        scope_id: helpers::row_text(row, 3)?.expect("scope_id"),
        metric: helpers::row_text(row, 4)?.expect("metric"),
        window_kind: helpers::row_text(row, 5)?.expect("window_kind"),
        amount: helpers::row_i64(row, 6)?,
        warn_percent: helpers::row_i64(row, 7)?,
        hard_stop_enabled: helpers::row_i64(row, 8)? != 0,
        notify_enabled: helpers::row_i64(row, 9)? != 0,
        is_active: helpers::row_i64(row, 10)? != 0,
        created_by_user_id: helpers::row_text(row, 11)?,
        updated_by_user_id: helpers::row_text(row, 12)?,
        created_at: helpers::row_text(row, 13)?.expect("created_at"),
    })
}

fn row_to_incident(row: &libsql::Row) -> Result<BudgetIncidentRecord, libsql::Error> {
    Ok(BudgetIncidentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        policy_id: helpers::row_text(row, 2)?.expect("policy_id"),
        scope_type: helpers::row_text(row, 3)?.expect("scope_type"),
        scope_id: helpers::row_text(row, 4)?.expect("scope_id"),
        metric: helpers::row_text(row, 5)?.expect("metric"),
        window_kind: helpers::row_text(row, 6)?.expect("window_kind"),
        window_start: helpers::row_text(row, 7)?.expect("window_start"),
        window_end: helpers::row_text(row, 8)?.expect("window_end"),
        threshold_type: helpers::row_text(row, 9)?.expect("threshold_type"),
        amount_limit: helpers::row_i64(row, 10)?,
        amount_observed: helpers::row_i64(row, 11)?,
        status: helpers::row_text(row, 12)?.expect("status"),
        approval_id: helpers::row_text(row, 13)?,
        resolved_at: helpers::row_text(row, 14)?,
        created_at: helpers::row_text(row, 15)?.expect("created_at"),
    })
}

const POLICY_COLUMNS: &str = "id, company_id, scope_type, scope_id, metric, window_kind, amount,
    warn_percent, hard_stop_enabled, notify_enabled, is_active, created_by_user_id,
    updated_by_user_id, created_at";

const INCIDENT_COLUMNS: &str = "id, company_id, policy_id, scope_type, scope_id, metric,
    window_kind, window_start, window_end, threshold_type, amount_limit, amount_observed,
    status, approval_id, resolved_at, created_at";

#[async_trait]
impl BudgetPolicyRepository for TursoBudgetPolicyRepository {
    async fn upsert_policy(
        &self,
        input: NewBudgetPolicy,
    ) -> Result<BudgetPolicyRecord, BudgetPolicyError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(BudgetPolicyError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budget_policies
               (id, company_id, scope_type, scope_id, metric, window_kind, amount,
                warn_percent, hard_stop_enabled, notify_enabled, is_active,
                created_by_user_id, updated_by_user_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11, ?11,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, scope_type, scope_id, metric, window_kind)
             DO UPDATE SET amount = excluded.amount,
                           warn_percent = excluded.warn_percent,
                           hard_stop_enabled = excluded.hard_stop_enabled,
                           notify_enabled = excluded.notify_enabled,
                           is_active = 1,
                           updated_by_user_id = excluded.updated_by_user_id,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.scope_type.clone(),
                input.scope_id.clone(),
                input.metric.clone(),
                input.window_kind.clone(),
                input.amount,
                input.warn_percent,
                i64::from(input.hard_stop_enabled),
                i64::from(input.notify_enabled),
                input.created_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {POLICY_COLUMNS} FROM budget_policies
                     WHERE company_id = ?1 AND scope_type = ?2 AND scope_id = ?3
                       AND metric = ?4 AND window_kind = ?5"
                ),
                libsql::params![
                    input.company_id,
                    input.scope_type,
                    input.scope_id,
                    input.metric,
                    input.window_kind
                ],
            )
            .await?;
        let row = rows.next().await?.expect("policy was just upserted");
        Ok(row_to_policy(&row)?)
    }

    async fn list_policies(
        &self,
        company_id: &str,
    ) -> Result<Vec<BudgetPolicyRecord>, BudgetPolicyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {POLICY_COLUMNS} FROM budget_policies
                     WHERE company_id = ?1 ORDER BY scope_type, scope_id"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut policies = Vec::new();
        while let Some(row) = rows.next().await? {
            policies.push(row_to_policy(&row)?);
        }
        Ok(policies)
    }

    async fn delete_policy(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<BudgetPolicyRecord>, BudgetPolicyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {POLICY_COLUMNS} FROM budget_policies WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_policy(&row)?;
        conn.execute(
            "DELETE FROM budget_policies WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(record))
    }

    async fn create_incident(
        &self,
        input: NewBudgetIncident,
    ) -> Result<BudgetIncidentRecord, BudgetPolicyError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "budget_policies",
            &input.policy_id,
            &input.company_id,
        )
        .await?
        {
            return Err(BudgetPolicyError::PolicyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO budget_incidents
                   (id, company_id, policy_id, scope_type, scope_id, metric, window_kind,
                    window_start, window_end, threshold_type, amount_limit, amount_observed,
                    status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'open',
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.policy_id.clone(),
                    input.scope_type,
                    input.scope_id,
                    input.metric,
                    input.window_kind,
                    input.window_start.clone(),
                    input.window_end,
                    input.threshold_type.clone(),
                    input.amount_limit,
                    input.amount_observed
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {INCIDENT_COLUMNS} FROM budget_incidents WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("incident was just inserted");
                Ok(row_to_incident(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                // Same policy + window + threshold already reported: return the
                // existing incident by re-querying.
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {INCIDENT_COLUMNS} FROM budget_incidents
                             WHERE policy_id = ?1 AND window_start = ?2 AND threshold_type = ?3"
                        ),
                        libsql::params![input.policy_id, input.window_start, input.threshold_type],
                    )
                    .await?;
                let row = rows.next().await?.expect("existing incident");
                Ok(row_to_incident(&row)?)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_incidents(
        &self,
        company_id: &str,
    ) -> Result<Vec<BudgetIncidentRecord>, BudgetPolicyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {INCIDENT_COLUMNS} FROM budget_incidents
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut incidents = Vec::new();
        while let Some(row) = rows.next().await? {
            incidents.push(row_to_incident(&row)?);
        }
        Ok(incidents)
    }

    async fn set_incident_status(
        &self,
        company_id: &str,
        id: &str,
        status: &str,
    ) -> Result<Option<BudgetIncidentRecord>, BudgetPolicyError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE budget_incidents SET status = ?1,
                        resolved_at = CASE WHEN ?1 IN ('resolved', 'dismissed')
                                           THEN strftime('%Y-%m-%dT%H:%M:%fZ','now')
                                           ELSE resolved_at END,
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?2 AND id = ?3 AND status = 'open'",
                libsql::params![status, company_id, id],
            )
            .await?;
        if updated == 0 {
            let mut rows = conn
                .query(
                    &format!(
                        "SELECT {INCIDENT_COLUMNS} FROM budget_incidents WHERE company_id = ?1 AND id = ?2"
                    ),
                    libsql::params![company_id, id],
                )
                .await?;
            return match rows.next().await? {
                Some(_) => Ok(None), // exists but not open
                None => Err(BudgetPolicyError::IncidentNotFound),
            };
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {INCIDENT_COLUMNS} FROM budget_incidents WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("incident exists");
        Ok(Some(row_to_incident(&row)?))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoBudgetPolicyRepository) {
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
        (dir, TursoBudgetPolicyRepository::new(db))
    }

    #[tokio::test]
    async fn policy_upsert_incident_lifecycle() {
        let (_dir, repo) = repo().await;
        let policy = repo
            .upsert_policy(NewBudgetPolicy {
                company_id: "c1".to_owned(),
                scope_type: "company".to_owned(),
                scope_id: "c1".to_owned(),
                metric: "billed_cents".to_owned(),
                window_kind: "calendar_month_utc".to_owned(),
                amount: 100_000,
                warn_percent: 80,
                hard_stop_enabled: true,
                notify_enabled: true,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(policy.amount, 100_000);
        let replaced = repo
            .upsert_policy(NewBudgetPolicy {
                company_id: "c1".to_owned(),
                scope_type: "company".to_owned(),
                scope_id: "c1".to_owned(),
                metric: "billed_cents".to_owned(),
                window_kind: "calendar_month_utc".to_owned(),
                amount: 120_000,
                warn_percent: 80,
                hard_stop_enabled: true,
                notify_enabled: true,
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(replaced.id, policy.id);
        assert_eq!(replaced.amount, 120_000);

        let incident = repo
            .create_incident(NewBudgetIncident {
                company_id: "c1".to_owned(),
                policy_id: policy.id.clone(),
                scope_type: "company".to_owned(),
                scope_id: "c1".to_owned(),
                metric: "billed_cents".to_owned(),
                window_kind: "calendar_month_utc".to_owned(),
                window_start: "2026-08-01T00:00:00.000Z".to_owned(),
                window_end: "2026-08-31T23:59:59.999Z".to_owned(),
                threshold_type: "hard_stop".to_owned(),
                amount_limit: 120_000,
                amount_observed: 125_000,
            })
            .await
            .unwrap();
        assert_eq!(incident.status, "open");

        // Dedupe returns the same incident.
        let again = repo
            .create_incident(NewBudgetIncident {
                company_id: "c1".to_owned(),
                policy_id: policy.id.clone(),
                scope_type: "company".to_owned(),
                scope_id: "c1".to_owned(),
                metric: "billed_cents".to_owned(),
                window_kind: "calendar_month_utc".to_owned(),
                window_start: "2026-08-01T00:00:00.000Z".to_owned(),
                window_end: "2026-08-31T23:59:59.999Z".to_owned(),
                threshold_type: "hard_stop".to_owned(),
                amount_limit: 120_000,
                amount_observed: 126_000,
            })
            .await
            .unwrap();
        assert_eq!(again.id, incident.id);

        let resolved = repo
            .set_incident_status("c1", &incident.id, "resolved")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resolved.status, "resolved");
        assert!(resolved.resolved_at.is_some());
        assert!(
            repo.set_incident_status("c1", &incident.id, "dismissed")
                .await
                .unwrap()
                .is_none()
        );

        assert!(repo.list_incidents("c1").await.unwrap().len() == 1);
        // Cross-company delete is not found.
        assert!(
            repo.delete_policy("c2", &policy.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            repo.delete_policy("c1", &policy.id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(repo.list_policies("c1").await.unwrap().is_empty());
    }
}
