//! Minimal agents repository: org hierarchy and subordinate budgets.
//!
//! Agent CRUD arrives with the access/operations milestone (#56); this module
//! only carries the pieces the permission matrix needs today.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;

use super::helpers;

/// One org-graph row used for subtree evaluation (`reports_to` chain).
#[derive(Debug, Clone)]
pub struct AgentHierarchyRow {
    /// Agent id.
    pub id: String,
    /// Manager agent id, or `None` for a root.
    pub reports_to: Option<String>,
}

/// A budget row of the `agents` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBudgetRecord {
    /// Agent id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Name.
    pub name: String,
    /// Role.
    pub role: String,
    /// Status.
    pub status: String,
    /// Monthly budget in cents (0 = unlimited).
    pub budget_monthly_cents: i64,
    /// Monthly spending in cents.
    pub spent_monthly_cents: i64,
    /// Whether the agent is paused.
    pub paused: bool,
    /// Manager agent id.
    pub reports_to: Option<String>,
}

/// A full agent row (list/detail surface).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRecord {
    /// Agent id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Name.
    pub name: String,
    /// Role.
    pub role: String,
    /// Title.
    pub title: Option<String>,
    /// Icon.
    pub icon: Option<String>,
    /// Status.
    pub status: String,
    /// Manager agent id.
    pub reports_to: Option<String>,
    /// Adapter type.
    pub adapter_type: String,
    /// Monthly budget in cents.
    pub budget_monthly_cents: i64,
    /// Monthly spending in cents.
    pub spent_monthly_cents: i64,
    /// Pause reason.
    pub pause_reason: Option<String>,
    /// ISO 8601 last heartbeat.
    pub last_heartbeat_at: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Agent repository errors.
#[derive(Debug, Error)]
pub enum AgentError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The agent does not exist in this company.
    #[error("agent not found")]
    AgentNotFound,
}

/// Agent persistence contract.
#[async_trait]
pub trait AgentRepository: Send + Sync {
    /// Lists the org hierarchy for a company (id + manager).
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] on database failure.
    async fn hierarchy(&self, company_id: &str) -> Result<Vec<AgentHierarchyRow>, AgentError>;

    /// Resolves the owning company of an agent.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] on database failure.
    async fn company_of(&self, agent_id: &str) -> Result<Option<String>, AgentError>;

    /// Lists agents for a company.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] on database failure.
    async fn list(&self, company_id: &str) -> Result<Vec<AgentRecord>, AgentError>;

    /// Gets one agent (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] on database failure.
    async fn get(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Option<AgentRecord>, AgentError>;

    /// Updates an agent's status and pause reason.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] on database failure.
    async fn update_status(
        &self,
        company_id: &str,
        agent_id: &str,
        status: &str,
        pause_reason: Option<Option<String>>,
    ) -> Result<Option<AgentRecord>, AgentError>;

    /// Sets an agent's monthly budget (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when the company or agent does not exist.
    async fn set_budget(
        &self,
        company_id: &str,
        agent_id: &str,
        budget_monthly_cents: i64,
    ) -> Result<Option<AgentBudgetRecord>, AgentError>;
}

/// Turso/libSQL implementation of [`AgentRepository`].
#[derive(Debug)]
pub struct TursoAgentRepository {
    db: Database,
}

impl TursoAgentRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_budget(row: &libsql::Row) -> Result<AgentBudgetRecord, libsql::Error> {
    Ok(AgentBudgetRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        name: helpers::row_text(row, 2)?.expect("name"),
        role: helpers::row_text(row, 3)?.expect("role"),
        status: helpers::row_text(row, 4)?.expect("status"),
        budget_monthly_cents: helpers::row_i64(row, 5)?,
        spent_monthly_cents: helpers::row_i64(row, 6)?,
        paused: helpers::row_i64(row, 7)? != 0,
        reports_to: helpers::row_text(row, 8)?,
    })
}

#[async_trait]
impl AgentRepository for TursoAgentRepository {
    async fn hierarchy(&self, company_id: &str) -> Result<Vec<AgentHierarchyRow>, AgentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, reports_to FROM agents WHERE company_id = ?1",
                libsql::params![company_id],
            )
            .await?;
        let mut hierarchy = Vec::new();
        while let Some(row) = rows.next().await? {
            hierarchy.push(AgentHierarchyRow {
                id: helpers::row_text(&row, 0)?.expect("id"),
                reports_to: helpers::row_text(&row, 1)?,
            });
        }
        Ok(hierarchy)
    }

    async fn company_of(&self, agent_id: &str) -> Result<Option<String>, AgentError> {
        let conn = crate::connection::connect(&self.db).await?;
        Ok(helpers::row_company(&conn, "agents", agent_id).await?)
    }

    async fn list(&self, company_id: &str) -> Result<Vec<AgentRecord>, AgentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, role, title, icon, status, reports_to,
                        adapter_type, budget_monthly_cents, spent_monthly_cents, pause_reason,
                        last_heartbeat_at, created_at
                 FROM agents WHERE company_id = ?1 ORDER BY name",
                libsql::params![company_id],
            )
            .await?;
        let mut agents = Vec::new();
        while let Some(row) = rows.next().await? {
            agents.push(row_to_agent(&row)?);
        }
        Ok(agents)
    }

    async fn get(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Option<AgentRecord>, AgentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, role, title, icon, status, reports_to,
                        adapter_type, budget_monthly_cents, spent_monthly_cents, pause_reason,
                        last_heartbeat_at, created_at
                 FROM agents WHERE company_id = ?1 AND id = ?2",
                libsql::params![company_id, agent_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_agent(&row)?)),
            None => Ok(None),
        }
    }

    async fn update_status(
        &self,
        company_id: &str,
        agent_id: &str,
        status: &str,
        pause_reason: Option<Option<String>>,
    ) -> Result<Option<AgentRecord>, AgentError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut values: Vec<libsql::Value> = Vec::new();
        values.push(libsql::Value::from(status));
        let mut sets = vec!["status = ?1".to_owned()];
        if let Some(reason) = pause_reason {
            sets.push("pause_reason = ?2".to_owned());
            values.push(
                reason
                    .map(libsql::Value::from)
                    .unwrap_or(libsql::Value::Null),
            );
        }
        sets.push("paused_at = CASE WHEN ?1 IN ('paused', 'terminated') THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END".to_owned());
        sets.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')".to_owned());
        let company_param = values.len() + 1;
        let id_param = values.len() + 2;
        values.push(libsql::Value::from(company_id.to_owned()));
        values.push(libsql::Value::from(agent_id.to_owned()));
        let sql = format!(
            "UPDATE agents SET {} WHERE company_id = ?{company_param} AND id = ?{id_param}",
            sets.join(", ")
        );
        let updated = conn.execute(&sql, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, role, title, icon, status, reports_to,
                        adapter_type, budget_monthly_cents, spent_monthly_cents, pause_reason,
                        last_heartbeat_at, created_at
                 FROM agents WHERE company_id = ?1 AND id = ?2",
                libsql::params![company_id, agent_id],
            )
            .await?;
        let row = rows.next().await?.expect("agent exists");
        Ok(Some(row_to_agent(&row)?))
    }

    async fn set_budget(
        &self,
        company_id: &str,
        agent_id: &str,
        budget_monthly_cents: i64,
    ) -> Result<Option<AgentBudgetRecord>, AgentError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, company_id).await? {
            return Err(AgentError::CompanyNotFound);
        }
        let updated = conn
            .execute(
                "UPDATE agents
                 SET budget_monthly_cents = ?1,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?2 AND company_id = ?3",
                libsql::params![budget_monthly_cents, agent_id, company_id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, role, status, budget_monthly_cents,
                        spent_monthly_cents, pause_reason IS NOT NULL AS paused, reports_to
                 FROM agents WHERE id = ?1",
                libsql::params![agent_id],
            )
            .await?;
        let row = rows.next().await?.expect("agent was just updated");
        Ok(Some(row_to_budget(&row)?))
    }
}

fn row_to_agent(row: &libsql::Row) -> Result<AgentRecord, libsql::Error> {
    Ok(AgentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        name: helpers::row_text(row, 2)?.expect("name"),
        role: helpers::row_text(row, 3)?.expect("role"),
        title: helpers::row_text(row, 4)?,
        icon: helpers::row_text(row, 5)?,
        status: helpers::row_text(row, 6)?.expect("status"),
        reports_to: helpers::row_text(row, 7)?,
        adapter_type: helpers::row_text(row, 8)?.expect("adapter_type"),
        budget_monthly_cents: helpers::row_i64(row, 9)?,
        spent_monthly_cents: helpers::row_i64(row, 10)?,
        pause_reason: helpers::row_text(row, 11)?,
        last_heartbeat_at: helpers::row_text(row, 12)?,
        created_at: helpers::row_text(row, 13)?.expect("created_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoAgentRepository) {
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
            "INSERT INTO agents (id, company_id, name, role, adapter_type, reports_to)
             VALUES ('a1', 'c1', 'Root', 'manager', 'cli', NULL),
                    ('a2', 'c1', 'Mid', 'manager', 'cli', 'a1'),
                    ('a3', 'c1', 'Leaf', 'worker', 'cli', 'a2')",
            (),
        )
        .await
        .unwrap();
        (dir, TursoAgentRepository::new(db))
    }

    #[tokio::test]
    async fn hierarchy_returns_reports_to_chain() {
        let (_dir, repo) = repo().await;
        let rows = repo.hierarchy("c1").await.unwrap();
        let by_id: std::collections::HashMap<_, _> = rows
            .into_iter()
            .map(|row| (row.id.clone(), row.reports_to))
            .collect();
        assert_eq!(by_id.get("a1").cloned().flatten(), None);
        assert_eq!(by_id.get("a2").cloned().flatten().as_deref(), Some("a1"));
        assert_eq!(by_id.get("a3").cloned().flatten().as_deref(), Some("a2"));
    }

    #[tokio::test]
    async fn set_budget_updates_agent() {
        let (_dir, repo) = repo().await;
        let budget = repo
            .set_budget("c1", "a3", 50_000)
            .await
            .unwrap()
            .expect("agent");
        assert_eq!(budget.budget_monthly_cents, 50_000);
        assert_eq!(budget.reports_to.as_deref(), Some("a2"));

        assert_eq!(repo.company_of("a3").await.unwrap().as_deref(), Some("c1"));
        assert!(repo.company_of("missing").await.unwrap().is_none());
        assert!(repo.set_budget("c1", "missing", 1).await.unwrap().is_none());
        let err = repo.set_budget("c2", "a1", 1).await.unwrap_err();
        assert!(matches!(err, AgentError::CompanyNotFound));
    }

    #[tokio::test]
    async fn list_get_update_status() {
        let (_dir, repo) = repo().await;
        let agents = repo.list("c1").await.unwrap();
        assert_eq!(agents.len(), 3);
        let leaf = repo.get("c1", "a3").await.unwrap().unwrap();
        assert_eq!(leaf.name, "Leaf");

        let paused = repo
            .update_status("c1", "a3", "paused", Some(Some("budget".to_owned())))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(paused.status, "paused");
        assert_eq!(paused.pause_reason.as_deref(), Some("budget"));

        let resumed = repo
            .update_status("c1", "a3", "active", Some(None))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resumed.status, "active");
        assert!(resumed.pause_reason.is_none());

        // Cross-company get is not found.
        assert!(repo.get("c2", "a1").await.unwrap().is_none());
    }
}
