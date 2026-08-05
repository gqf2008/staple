//! Cost events and budget repository.
//!
//! Records cost events, maintains company/agent monthly spending rollups, and
//! implements the hard-stop rule: when a budget (company or agent) is
//! exhausted, affected agents are paused automatically.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `cost_events` table.
#[derive(Debug, Clone)]
pub struct CostEventRecord {
    /// Event id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Billing code.
    pub billing_code: Option<String>,
    /// Provider.
    pub provider: String,
    /// Model.
    pub model: String,
    /// Input tokens.
    pub input_tokens: i64,
    /// Cached input tokens.
    pub cached_input_tokens: i64,
    /// Output tokens.
    pub output_tokens: i64,
    /// Cost in cents.
    pub cost_cents: i64,
    /// Biller.
    pub biller: String,
    /// Billing type.
    pub billing_type: String,
    /// Heartbeat run id.
    pub heartbeat_run_id: Option<String>,
    /// ISO 8601 occurrence time.
    pub occurred_at: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for recording a cost event.
#[derive(Debug, Clone)]
pub struct NewCostEvent {
    /// Owning company id.
    pub company_id: String,
    /// Agent id (must belong to the company).
    pub agent_id: String,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Billing code.
    pub billing_code: Option<String>,
    /// Provider.
    pub provider: String,
    /// Model.
    pub model: String,
    /// Input tokens.
    pub input_tokens: i64,
    /// Output tokens.
    pub output_tokens: i64,
    /// Cost in cents.
    pub cost_cents: i64,
    /// ISO 8601 occurrence time.
    pub occurred_at: String,
}

/// Outcome of recording a cost event, including any hard-stop pause.
#[derive(Debug, Clone)]
pub struct CostEventOutcome {
    /// The recorded event.
    pub event: CostEventRecord,
    /// Whether a hard stop triggered.
    pub hard_stop_triggered: bool,
    /// Agents paused by the hard stop.
    pub paused_agent_ids: Vec<String>,
}

/// Company budget summary.
#[derive(Debug, Clone)]
pub struct BudgetSummary {
    /// Company id.
    pub company_id: String,
    /// Monthly budget in cents (0 = unlimited).
    pub budget_monthly_cents: i64,
    /// Spent this month in cents.
    pub spent_monthly_cents: i64,
    /// Remaining cents (0 when unlimited budgets are treated as unlimited).
    pub remaining_cents: i64,
    /// Agents paused due to budget exhaustion.
    pub paused_agents: i64,
}

/// Per-agent spending row.
#[derive(Debug, Clone)]
pub struct AgentCostRow {
    /// Agent id.
    pub agent_id: String,
    /// Agent name.
    pub agent_name: String,
    /// Agent status.
    pub status: String,
    /// Agent monthly budget in cents.
    pub budget_monthly_cents: i64,
    /// Agent spent this month in cents.
    pub spent_monthly_cents: i64,
}

/// Cost repository errors.
#[derive(Debug, Error)]
pub enum CostError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The agent does not exist or belongs to another company.
    #[error("agent not found")]
    AgentNotFound,
    /// The referenced issue belongs to another company.
    #[error("issue belongs to a different company")]
    IssueInDifferentCompany,
}

/// Cost persistence contract.
#[async_trait]
pub trait CostRepository: Send + Sync {
    /// Records a cost event, updates spending rollups, and applies the
    /// hard-stop rule.
    ///
    /// # Errors
    ///
    /// Returns [`CostError`] on invalid references.
    async fn create_event(&self, input: NewCostEvent) -> Result<CostEventOutcome, CostError>;

    /// Company budget summary.
    ///
    /// # Errors
    ///
    /// Returns [`CostError`] on database failure.
    async fn summary(&self, company_id: &str) -> Result<Option<BudgetSummary>, CostError>;

    /// Per-agent spending.
    ///
    /// # Errors
    ///
    /// Returns [`CostError`] on database failure.
    async fn by_agent(&self, company_id: &str) -> Result<Vec<AgentCostRow>, CostError>;

    /// Sets the company monthly budget in cents.
    ///
    /// # Errors
    ///
    /// Returns [`CostError`] when the company is missing.
    async fn set_company_budget(
        &self,
        company_id: &str,
        budget_cents: i64,
    ) -> Result<Option<BudgetSummary>, CostError>;

    /// Resets company spending to zero and resumes agents paused by budget
    /// exhaustion. Returns the number of resumed agents.
    ///
    /// # Errors
    ///
    /// Returns [`CostError`] when the company is missing.
    async fn reset_company_spending(&self, company_id: &str) -> Result<Option<i64>, CostError>;
}

/// Turso/libSQL implementation of [`CostRepository`].
#[derive(Debug)]
pub struct TursoCostRepository {
    db: Database,
}

impl TursoCostRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const EVENT_COLUMNS: &str = "id, company_id, agent_id, issue_id, billing_code, provider,
    model, input_tokens, cached_input_tokens, output_tokens, cost_cents, biller, billing_type,
    heartbeat_run_id, occurred_at, created_at";

fn row_to_event(row: &libsql::Row) -> Result<CostEventRecord, libsql::Error> {
    Ok(CostEventRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        agent_id: helpers::row_text(row, 2)?.expect("agent_id is NOT NULL"),
        issue_id: helpers::row_text(row, 3)?,
        billing_code: helpers::row_text(row, 4)?,
        provider: helpers::row_text(row, 5)?.expect("provider is NOT NULL"),
        model: helpers::row_text(row, 6)?.expect("model is NOT NULL"),
        input_tokens: helpers::row_i64(row, 7)?,
        cached_input_tokens: helpers::row_i64(row, 8)?,
        output_tokens: helpers::row_i64(row, 9)?,
        cost_cents: helpers::row_i64(row, 10)?,
        biller: helpers::row_text(row, 11)?.expect("biller is NOT NULL"),
        billing_type: helpers::row_text(row, 12)?.expect("billing_type is NOT NULL"),
        heartbeat_run_id: helpers::row_text(row, 13)?,
        occurred_at: helpers::row_text(row, 14)?.expect("occurred_at is NOT NULL"),
        created_at: helpers::row_text(row, 15)?.expect("created_at is NOT NULL"),
    })
}

/// Reads a company's budget/spent columns.
async fn company_budget(
    conn: &libsql::Connection,
    company_id: &str,
) -> Result<Option<(i64, i64)>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT budget_monthly_cents, spent_monthly_cents FROM companies WHERE id = ?1",
            libsql::params![company_id],
        )
        .await?;
    match rows.next().await? {
        Some(row) => Ok(Some((
            helpers::row_i64(&row, 0)?,
            helpers::row_i64(&row, 1)?,
        ))),
        None => Ok(None),
    }
}

#[async_trait]
impl CostRepository for TursoCostRepository {
    async fn create_event(&self, input: NewCostEvent) -> Result<CostEventOutcome, CostError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;

        if !helpers::company_exists(&tx, &input.company_id).await? {
            return Err(CostError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(&tx, "agents", &input.agent_id, &input.company_id)
            .await?
        {
            return Err(CostError::AgentNotFound);
        }
        if let Some(issue_id) = &input.issue_id
            && !helpers::row_belongs_to_company(&tx, "issues", issue_id, &input.company_id).await?
        {
            return Err(CostError::IssueInDifferentCompany);
        }

        let id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO cost_events (id, company_id, agent_id, issue_id, billing_code,
                                      provider, model, input_tokens, output_tokens, cost_cents,
                                      occurred_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.agent_id.clone(),
                input.issue_id,
                input.billing_code,
                input.provider,
                input.model,
                input.input_tokens,
                input.output_tokens,
                input.cost_cents,
                input.occurred_at
            ],
        )
        .await?;

        // Rollups.
        tx.execute(
            "UPDATE companies
             SET spent_monthly_cents = spent_monthly_cents + ?1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?2",
            libsql::params![input.cost_cents, input.company_id.clone()],
        )
        .await?;
        tx.execute(
            "UPDATE agents
             SET spent_monthly_cents = spent_monthly_cents + ?1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?2 AND company_id = ?3",
            libsql::params![
                input.cost_cents,
                input.agent_id.clone(),
                input.company_id.clone()
            ],
        )
        .await?;

        // Hard stop: pause agents whose company or own budget is exhausted.
        let mut paused = Vec::new();
        let mut hard_stop = false;
        let (company_budget_cents, company_spent_cents) = company_budget(&tx, &input.company_id)
            .await?
            .expect("company exists");
        if company_budget_cents > 0 && company_spent_cents >= company_budget_cents {
            hard_stop = true;
            let mut rows = tx
                .query(
                    "SELECT id FROM agents
                     WHERE company_id = ?1 AND status = 'active'",
                    libsql::params![input.company_id.clone()],
                )
                .await?;
            let mut agent_ids = Vec::new();
            while let Some(row) = rows.next().await? {
                agent_ids.push(helpers::row_text(&row, 0)?.expect("id is NOT NULL"));
            }
            for agent_id in &agent_ids {
                tx.execute(
                    "UPDATE agents
                     SET status = 'paused', pause_reason = 'budget_exhausted',
                         paused_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE id = ?1",
                    libsql::params![agent_id.clone()],
                )
                .await?;
            }
            paused = agent_ids;
        } else {
            // Agent-level budget.
            let mut rows = tx
                .query(
                    "SELECT id FROM agents
                     WHERE id = ?1 AND company_id = ?2 AND budget_monthly_cents > 0
                       AND spent_monthly_cents >= budget_monthly_cents AND status = 'active'",
                    libsql::params![input.agent_id.clone(), input.company_id.clone()],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                let agent_id = helpers::row_text(&row, 0)?.expect("id is NOT NULL");
                tx.execute(
                    "UPDATE agents
                     SET status = 'paused', pause_reason = 'budget_exhausted',
                         paused_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE id = ?1",
                    libsql::params![agent_id.clone()],
                )
                .await?;
                paused.push(agent_id);
                hard_stop = true;
            }
        }
        tx.commit().await?;

        let mut rows = conn
            .query(
                &format!("SELECT {EVENT_COLUMNS} FROM cost_events WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let event = row_to_event(&rows.next().await?.expect("event was just inserted"))?;
        Ok(CostEventOutcome {
            event,
            hard_stop_triggered: hard_stop,
            paused_agent_ids: paused,
        })
    }

    async fn summary(&self, company_id: &str) -> Result<Option<BudgetSummary>, CostError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some((budget, spent)) = company_budget(&conn, company_id).await? else {
            return Ok(None);
        };
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM agents
                 WHERE company_id = ?1 AND status = 'paused' AND pause_reason = 'budget_exhausted'",
                libsql::params![company_id],
            )
            .await?;
        let paused_agents = match rows.next().await? {
            Some(row) => helpers::row_i64(&row, 0)?,
            None => 0,
        };
        Ok(Some(BudgetSummary {
            company_id: company_id.to_owned(),
            budget_monthly_cents: budget,
            spent_monthly_cents: spent,
            remaining_cents: if budget > 0 { budget - spent } else { 0 },
            paused_agents,
        }))
    }

    async fn by_agent(&self, company_id: &str) -> Result<Vec<AgentCostRow>, CostError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, name, status, budget_monthly_cents, spent_monthly_cents
                 FROM agents WHERE company_id = ?1 ORDER BY spent_monthly_cents DESC",
                libsql::params![company_id],
            )
            .await?;
        let mut agents = Vec::new();
        while let Some(row) = rows.next().await? {
            agents.push(AgentCostRow {
                agent_id: helpers::row_text(&row, 0)?.expect("id is NOT NULL"),
                agent_name: helpers::row_text(&row, 1)?.expect("name is NOT NULL"),
                status: helpers::row_text(&row, 2)?.expect("status is NOT NULL"),
                budget_monthly_cents: helpers::row_i64(&row, 3)?,
                spent_monthly_cents: helpers::row_i64(&row, 4)?,
            });
        }
        Ok(agents)
    }

    async fn set_company_budget(
        &self,
        company_id: &str,
        budget_cents: i64,
    ) -> Result<Option<BudgetSummary>, CostError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE companies
                 SET budget_monthly_cents = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?2",
                libsql::params![budget_cents, company_id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        self.summary(company_id).await
    }

    async fn reset_company_spending(&self, company_id: &str) -> Result<Option<i64>, CostError> {
        let conn = crate::connection::connect(&self.db).await?;
        let tx = conn.transaction().await?;
        if !helpers::company_exists(&tx, company_id).await? {
            return Ok(None);
        }
        tx.execute(
            "UPDATE companies
             SET spent_monthly_cents = 0, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![company_id],
        )
        .await?;
        tx.execute(
            "UPDATE agents
             SET spent_monthly_cents = 0, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE company_id = ?1",
            libsql::params![company_id],
        )
        .await?;
        let resumed = tx
            .execute(
                "UPDATE agents
                 SET status = 'active', pause_reason = NULL, paused_at = NULL,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?1 AND status = 'paused'
                   AND pause_reason = 'budget_exhausted'",
                libsql::params![company_id],
            )
            .await?;
        tx.commit().await?;
        Ok(Some(resumed as i64))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoCostRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoCostRepository::new(db);
        (dir, repo, conn)
    }

    async fn seed(conn: &Connection) {
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes,
                                    budget_monthly_cents)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024, 100)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type,
                                 budget_monthly_cents)
             VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local', 50),
                    ('a2', 'c1', 'two', 'engineer', 'codex_local', 0)",
            (),
        )
        .await
        .unwrap();
    }

    fn event(cost_cents: i64, agent_id: &str) -> NewCostEvent {
        NewCostEvent {
            company_id: "c1".to_owned(),
            agent_id: agent_id.to_owned(),
            issue_id: None,
            billing_code: None,
            provider: "anthropic".to_owned(),
            model: "claude".to_owned(),
            input_tokens: 10,
            output_tokens: 5,
            cost_cents,
            occurred_at: "2026-08-03T00:00:00.000Z".to_owned(),
        }
    }

    #[tokio::test]
    async fn company_budget_hard_stop_pauses_agents() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;

        // 60 cents: company budget is 100, agent budget 50 -> agent hard stop.
        let outcome = repo.create_event(event(60, "a1")).await.unwrap();
        assert!(outcome.hard_stop_triggered);
        assert_eq!(outcome.paused_agent_ids, vec!["a1".to_owned()]);

        // Summary reflects it.
        let summary = repo.summary("c1").await.unwrap().unwrap();
        assert_eq!(summary.spent_monthly_cents, 60);
        assert_eq!(summary.paused_agents, 1);

        // a2 remains active; another 60 on a2 exhausts the company budget and
        // pauses the still-active a2.
        let outcome = repo.create_event(event(60, "a2")).await.unwrap();
        assert!(outcome.hard_stop_triggered);
        assert_eq!(outcome.paused_agent_ids, vec!["a2".to_owned()]);

        let summary = repo.summary("c1").await.unwrap().unwrap();
        assert_eq!(summary.spent_monthly_cents, 120);
        assert_eq!(summary.paused_agents, 2);
    }

    #[tokio::test]
    async fn reset_restores_agents_and_spending() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        repo.create_event(event(60, "a1")).await.unwrap();

        let resumed = repo.reset_company_spending("c1").await.unwrap().unwrap();
        assert_eq!(resumed, 1);

        let summary = repo.summary("c1").await.unwrap().unwrap();
        assert_eq!(summary.spent_monthly_cents, 0);
        assert_eq!(summary.paused_agents, 0);

        let agents = repo.by_agent("c1").await.unwrap();
        assert!(agents.iter().all(|agent| agent.status == "active"));
    }

    #[tokio::test]
    async fn set_company_budget_updates_summary() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let summary = repo.set_company_budget("c1", 500).await.unwrap().unwrap();
        assert_eq!(summary.budget_monthly_cents, 500);
        assert!(
            repo.set_company_budget("missing", 1)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn event_requires_company_agent() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let error = repo
            .create_event(NewCostEvent {
                company_id: "c1".to_owned(),
                agent_id: "missing".to_owned(),
                issue_id: None,
                billing_code: None,
                provider: "p".to_owned(),
                model: "m".to_owned(),
                input_tokens: 0,
                output_tokens: 0,
                cost_cents: 1,
                occurred_at: "2026-08-03T00:00:00.000Z".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, CostError::AgentNotFound));
    }
}
