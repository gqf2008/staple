//! Decision action domain: bundles, decisions, target issues, effect
//! executions, and training examples (upstream decisions.ts +
//! decision_training_examples.ts).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A decision bundle.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionBundleRecord {
    pub id: String,
    pub company_id: String,
    pub title: String,
    pub summary: String,
    pub origin_agent_id: String,
    pub origin_issue_id: String,
    pub origin_run_id: String,
    pub created_at: String,
}

/// Input for creating a decision bundle.
#[derive(Debug, Clone)]
pub struct NewDecisionBundle {
    pub company_id: String,
    pub title: String,
    pub summary: String,
    pub origin_agent_id: String,
    pub origin_issue_id: String,
    pub origin_run_id: String,
}

/// A decision.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionRecord {
    pub id: String,
    pub company_id: String,
    pub bundle_id: Option<String>,
    pub origin_agent_id: String,
    pub origin_issue_id: String,
    pub origin_run_id: String,
    pub rule_key: Option<String>,
    pub title: String,
    pub body: String,
    pub options: serde_json::Value,
    pub inputs: Option<serde_json::Value>,
    pub status: String,
    pub execution_status: Option<String>,
    pub chosen_option_id: Option<String>,
    pub input_values: Option<serde_json::Value>,
    pub decided_by_user_id: Option<String>,
    pub decided_at: Option<String>,
    pub expires_at: String,
    pub idempotency_key: Option<String>,
    pub signed_spec: String,
    pub target_snapshots: serde_json::Value,
    pub continuation_policy: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a decision.
#[derive(Debug, Clone)]
pub struct NewDecision {
    pub company_id: String,
    pub bundle_id: Option<String>,
    pub origin_agent_id: String,
    pub origin_issue_id: String,
    pub origin_run_id: String,
    pub rule_key: Option<String>,
    pub title: String,
    pub body: String,
    pub options: serde_json::Value,
    pub inputs: Option<serde_json::Value>,
    pub status: String,
    pub execution_status: Option<String>,
    pub chosen_option_id: Option<String>,
    pub input_values: Option<serde_json::Value>,
    pub decided_by_user_id: Option<String>,
    pub decided_at: Option<String>,
    pub expires_at: String,
    pub idempotency_key: Option<String>,
    pub signed_spec: String,
    pub target_snapshots: serde_json::Value,
    pub continuation_policy: String,
    pub metadata: serde_json::Value,
}

/// A decision -> target issue link.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTargetIssueRecord {
    pub decision_id: String,
    pub issue_id: String,
    pub company_id: String,
}

/// A decision effect execution.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionEffectExecutionRecord {
    pub id: String,
    pub company_id: String,
    pub decision_id: String,
    pub effect_index: i64,
    pub effect_type: String,
    pub target_issue_id: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub activity_log_id: Option<String>,
    pub executed_at: Option<String>,
}

/// Input for creating a decision effect execution.
#[derive(Debug, Clone)]
pub struct NewDecisionEffectExecution {
    pub company_id: String,
    pub decision_id: String,
    pub effect_index: i64,
    pub effect_type: String,
    pub target_issue_id: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub activity_log_id: Option<String>,
    pub executed_at: Option<String>,
}

/// A decision training example.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTrainingExampleRecord {
    pub id: String,
    pub company_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub issue_id: String,
    pub cutoff_at: String,
    pub notes: String,
    pub notes_history: serde_json::Value,
    pub decision_outcome: Option<String>,
    pub retention_policy: String,
    pub snapshot: serde_json::Value,
    pub created_by_user_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a decision training example.
#[derive(Debug, Clone)]
pub struct NewDecisionTrainingExample {
    pub company_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub issue_id: String,
    pub cutoff_at: String,
    pub notes: String,
    pub notes_history: serde_json::Value,
    pub decision_outcome: Option<String>,
    pub retention_policy: String,
    pub snapshot: serde_json::Value,
    pub created_by_user_id: String,
}

/// Input for resolving a decision.
#[derive(Debug, Clone)]
pub struct ResolveDecision {
    /// Owning company id.
    pub company_id: String,
    /// Decision id.
    pub decision_id: String,
    /// New status.
    pub status: String,
    /// Execution status.
    pub execution_status: Option<String>,
    /// Chosen option id.
    pub chosen_option_id: Option<String>,
    /// Deciding user id.
    pub decided_by_user_id: Option<String>,
    /// ISO 8601 decision time.
    pub decided_at: Option<String>,
    /// Input values JSON.
    pub input_values: Option<serde_json::Value>,
}

/// Decision action repository errors.
#[derive(Debug, Error)]
pub enum DecisionActionError {
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    #[error("company not found")]
    CompanyNotFound,
    #[error("reference not found")]
    ReferenceNotFound,
    #[error("record already exists")]
    AlreadyExists,
    #[error("decision not found")]
    DecisionNotFound,
}

/// Decision action persistence contract.
#[async_trait]
pub trait DecisionActionRepository: Send + Sync {
    /// Creates a decision bundle.
    async fn create_bundle(
        &self,
        input: NewDecisionBundle,
    ) -> Result<DecisionBundleRecord, DecisionActionError>;

    /// Lists decision bundles for a company.
    async fn list_bundles(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionBundleRecord>, DecisionActionError>;

    /// Creates a decision.
    async fn create_decision(
        &self,
        input: NewDecision,
    ) -> Result<DecisionRecord, DecisionActionError>;

    /// Fetches one decision (company-scoped).
    async fn get_decision(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<DecisionRecord>, DecisionActionError>;

    /// Lists decisions for a company, optionally filtered by status.
    async fn list_decisions(
        &self,
        company_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<DecisionRecord>, DecisionActionError>;

    /// Resolves a decision (status/outcome/input values).
    async fn resolve_decision(
        &self,
        input: ResolveDecision,
    ) -> Result<Option<DecisionRecord>, DecisionActionError>;

    /// Links a target issue to a decision.
    async fn add_target_issue(
        &self,
        company_id: &str,
        decision_id: &str,
        issue_id: &str,
    ) -> Result<DecisionTargetIssueRecord, DecisionActionError>;

    /// Lists target issues for a decision.
    async fn list_target_issues(
        &self,
        company_id: &str,
        decision_id: &str,
    ) -> Result<Vec<DecisionTargetIssueRecord>, DecisionActionError>;

    /// Unlinks a target issue.
    async fn remove_target_issue(
        &self,
        company_id: &str,
        decision_id: &str,
        issue_id: &str,
    ) -> Result<bool, DecisionActionError>;

    /// Creates a decision effect execution.
    async fn create_effect_execution(
        &self,
        input: NewDecisionEffectExecution,
    ) -> Result<DecisionEffectExecutionRecord, DecisionActionError>;

    /// Lists effect executions for a decision.
    async fn list_effect_executions(
        &self,
        company_id: &str,
        decision_id: &str,
    ) -> Result<Vec<DecisionEffectExecutionRecord>, DecisionActionError>;

    /// Updates an effect execution status/result.
    async fn update_effect_execution(
        &self,
        company_id: &str,
        id: &str,
        status: Option<&str>,
        result: Option<serde_json::Value>,
        error: Option<String>,
        executed_at: Option<String>,
    ) -> Result<Option<DecisionEffectExecutionRecord>, DecisionActionError>;

    /// Creates a decision training example.
    async fn create_training_example(
        &self,
        input: NewDecisionTrainingExample,
    ) -> Result<DecisionTrainingExampleRecord, DecisionActionError>;

    /// Lists training examples for a company, optionally filtered by issue.
    async fn list_training_examples(
        &self,
        company_id: &str,
        issue_id: Option<&str>,
    ) -> Result<Vec<DecisionTrainingExampleRecord>, DecisionActionError>;
}

/// Turso/libSQL implementation of [`DecisionActionRepository`].
#[derive(Debug)]
pub struct TursoDecisionActionRepository {
    db: Database,
}

impl TursoDecisionActionRepository {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const BUNDLE_COLUMNS: &str = "id, company_id, title, summary, origin_agent_id, origin_issue_id,
                              origin_run_id, created_at";
const DECISION_COLUMNS: &str = "id, company_id, bundle_id, origin_agent_id, origin_issue_id,
                                origin_run_id, rule_key, title, body, options, inputs, status,
                                execution_status, chosen_option_id, input_values,
                                decided_by_user_id, decided_at, expires_at, idempotency_key,
                                signed_spec, target_snapshots, continuation_policy, metadata,
                                created_at, updated_at";
const EFFECT_COLUMNS: &str = "id, company_id, decision_id, effect_index, effect_type,
                              target_issue_id, status, result, error, activity_log_id,
                              executed_at";
const EXAMPLE_COLUMNS: &str = "id, company_id, source_kind, source_id, issue_id, cutoff_at,
                               notes, notes_history, decision_outcome, retention_policy,
                               snapshot, created_by_user_id, created_at, updated_at";

#[async_trait]
impl DecisionActionRepository for TursoDecisionActionRepository {
    async fn create_bundle(
        &self,
        input: NewDecisionBundle,
    ) -> Result<DecisionBundleRecord, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(DecisionActionError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO decision_bundles (id, company_id, title, summary, origin_agent_id,
                                           origin_issue_id, origin_run_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.title,
                input.summary,
                input.origin_agent_id,
                input.origin_issue_id,
                input.origin_run_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {BUNDLE_COLUMNS} FROM decision_bundles WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("bundle was just inserted");
        Ok(row_to_bundle(&row)?)
    }

    async fn list_bundles(
        &self,
        company_id: &str,
    ) -> Result<Vec<DecisionBundleRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {BUNDLE_COLUMNS} FROM decision_bundles
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut bundles = Vec::new();
        while let Some(row) = rows.next().await? {
            bundles.push(row_to_bundle(&row)?);
        }
        Ok(bundles)
    }

    async fn create_decision(
        &self,
        input: NewDecision,
    ) -> Result<DecisionRecord, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(DecisionActionError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO decisions (id, company_id, bundle_id, origin_agent_id,
                                        origin_issue_id, origin_run_id, rule_key, title, body,
                                        options, inputs, status, execution_status,
                                        chosen_option_id, input_values, decided_by_user_id,
                                        decided_at, expires_at, idempotency_key, signed_spec,
                                        target_snapshots, continuation_policy, metadata,
                                        created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.bundle_id,
                    input.origin_agent_id,
                    input.origin_issue_id,
                    input.origin_run_id,
                    input.rule_key,
                    input.title,
                    input.body,
                    input.options.to_string(),
                    input.inputs.map(|v| v.to_string()),
                    input.status,
                    input.execution_status,
                    input.chosen_option_id,
                    input.input_values.map(|v| v.to_string()),
                    input.decided_by_user_id,
                    input.decided_at,
                    input.expires_at,
                    input.idempotency_key,
                    input.signed_spec,
                    input.target_snapshots.to_string(),
                    input.continuation_policy,
                    input.metadata.to_string(),
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {DECISION_COLUMNS} FROM decisions WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("decision was just inserted");
                Ok(row_to_decision(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionActionError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn get_decision(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<DecisionRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {DECISION_COLUMNS} FROM decisions WHERE id = ?1 AND company_id = ?2"
                ),
                libsql::params![id, company_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_decision(&row)?)),
            None => Ok(None),
        }
    }

    async fn list_decisions(
        &self,
        company_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<DecisionRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let status_filter = status.map(|_| "AND status = ?2").unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = status {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {DECISION_COLUMNS} FROM decisions
                     WHERE company_id = ?1 {status_filter}
                     ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut decisions = Vec::new();
        while let Some(row) = rows.next().await? {
            decisions.push(row_to_decision(&row)?);
        }
        Ok(decisions)
    }

    async fn resolve_decision(
        &self,
        input: ResolveDecision,
    ) -> Result<Option<DecisionRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let decision_id = input.decision_id.clone();
        let company_id = input.company_id.clone();
        let updated = conn
            .execute(
                "UPDATE decisions
                 SET status = ?1, execution_status = ?2, chosen_option_id = ?3,
                     decided_by_user_id = ?4, decided_at = ?5, input_values = ?6,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?7 AND company_id = ?8",
                libsql::params![
                    input.status,
                    input.execution_status,
                    input.chosen_option_id,
                    input.decided_by_user_id,
                    input.decided_at,
                    input.input_values.map(|v| v.to_string()),
                    decision_id.clone(),
                    company_id.clone()
                ],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {DECISION_COLUMNS} FROM decisions WHERE id = ?1 AND company_id = ?2"
                ),
                libsql::params![decision_id.clone(), company_id.clone()],
            )
            .await?;
        let row = rows.next().await?.expect("decision exists");
        Ok(Some(row_to_decision(&row)?))
    }

    async fn add_target_issue(
        &self,
        company_id: &str,
        decision_id: &str,
        issue_id: &str,
    ) -> Result<DecisionTargetIssueRecord, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "decisions", decision_id, company_id).await? {
            return Err(DecisionActionError::ReferenceNotFound);
        }
        if helpers::row_company(&conn, "issues", issue_id).await? != Some(company_id.to_owned()) {
            return Err(DecisionActionError::ReferenceNotFound);
        }
        let result = conn
            .execute(
                "INSERT INTO decision_target_issues (decision_id, issue_id, company_id)
                 VALUES (?1, ?2, ?3)",
                libsql::params![decision_id, issue_id, company_id],
            )
            .await;
        match result {
            Ok(_) => Ok(DecisionTargetIssueRecord {
                decision_id: decision_id.to_owned(),
                issue_id: issue_id.to_owned(),
                company_id: company_id.to_owned(),
            }),
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionActionError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_target_issues(
        &self,
        company_id: &str,
        decision_id: &str,
    ) -> Result<Vec<DecisionTargetIssueRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT decision_id, issue_id, company_id FROM decision_target_issues
                 WHERE company_id = ?1 AND decision_id = ?2 ORDER BY issue_id",
                libsql::params![company_id, decision_id],
            )
            .await?;
        let mut links = Vec::new();
        while let Some(row) = rows.next().await? {
            links.push(DecisionTargetIssueRecord {
                decision_id: helpers::row_text(&row, 0)?.expect("decision_id"),
                issue_id: helpers::row_text(&row, 1)?.expect("issue_id"),
                company_id: helpers::row_text(&row, 2)?.expect("company_id"),
            });
        }
        Ok(links)
    }

    async fn remove_target_issue(
        &self,
        company_id: &str,
        decision_id: &str,
        issue_id: &str,
    ) -> Result<bool, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let deleted = conn
            .execute(
                "DELETE FROM decision_target_issues
                 WHERE company_id = ?1 AND decision_id = ?2 AND issue_id = ?3",
                libsql::params![company_id, decision_id, issue_id],
            )
            .await?;
        Ok(deleted > 0)
    }

    async fn create_effect_execution(
        &self,
        input: NewDecisionEffectExecution,
    ) -> Result<DecisionEffectExecutionRecord, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "decisions",
            &input.decision_id,
            &input.company_id,
        )
        .await?
        {
            return Err(DecisionActionError::ReferenceNotFound);
        }
        if helpers::row_company(&conn, "issues", &input.target_issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(DecisionActionError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO decision_effect_executions (id, company_id, decision_id,
                                                         effect_index, effect_type,
                                                         target_issue_id, status, result, error,
                                                         activity_log_id, executed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.decision_id,
                    input.effect_index,
                    input.effect_type,
                    input.target_issue_id,
                    input.status,
                    input.result.map(|v| v.to_string()),
                    input.error,
                    input.activity_log_id,
                    input.executed_at
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {EFFECT_COLUMNS} FROM decision_effect_executions WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("execution was just inserted");
                Ok(row_to_effect(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionActionError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_effect_executions(
        &self,
        company_id: &str,
        decision_id: &str,
    ) -> Result<Vec<DecisionEffectExecutionRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EFFECT_COLUMNS} FROM decision_effect_executions
                     WHERE company_id = ?1 AND decision_id = ?2 ORDER BY effect_index"
                ),
                libsql::params![company_id, decision_id],
            )
            .await?;
        let mut executions = Vec::new();
        while let Some(row) = rows.next().await? {
            executions.push(row_to_effect(&row)?);
        }
        Ok(executions)
    }

    async fn update_effect_execution(
        &self,
        company_id: &str,
        id: &str,
        status: Option<&str>,
        result: Option<serde_json::Value>,
        error: Option<String>,
        executed_at: Option<String>,
    ) -> Result<Option<DecisionEffectExecutionRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "decision_effect_executions", id, company_id)
            .await?
        {
            return Ok(None);
        }
        let mut sets = Vec::new();
        let mut values: Vec<libsql::Value> = Vec::new();
        let mut push = |column: &str, value: Option<libsql::Value>| {
            if let Some(value) = value {
                sets.push(format!("{column} = ?{}", values.len() + 1));
                values.push(value);
            }
        };
        push("status", status.map(libsql::Value::from));
        push("result", result.map(|v| libsql::Value::from(v.to_string())));
        push("error", error.map(libsql::Value::from));
        push("executed_at", executed_at.map(libsql::Value::from));
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EFFECT_COLUMNS} FROM decision_effect_executions
                     WHERE id = ?1 AND company_id = ?2"
                ),
                libsql::params![id, company_id],
            )
            .await?;
        let Some(rows) = rows.next().await? else {
            return Ok(None);
        };
        drop(rows);
        if sets.is_empty() {
            let mut rows = conn
                .query(
                    &format!(
                        "SELECT {EFFECT_COLUMNS} FROM decision_effect_executions
                         WHERE id = ?1 AND company_id = ?2"
                    ),
                    libsql::params![id, company_id],
                )
                .await?;
            return match rows.next().await? {
                Some(row) => Ok(Some(row_to_effect(&row)?)),
                None => Ok(None),
            };
        }
        let param = values.len() + 1;
        values.push(libsql::Value::from(id.to_owned()));
        let sql = format!(
            "UPDATE decision_effect_executions SET {} WHERE id = ?{param} AND company_id = ?{}",
            sets.join(", "),
            param + 1
        );
        values.push(libsql::Value::from(company_id.to_owned()));
        conn.execute(&sql, values).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EFFECT_COLUMNS} FROM decision_effect_executions
                     WHERE id = ?1 AND company_id = ?2"
                ),
                libsql::params![id, company_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_effect(&row)?)),
            None => Ok(None),
        }
    }

    async fn create_training_example(
        &self,
        input: NewDecisionTrainingExample,
    ) -> Result<DecisionTrainingExampleRecord, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(DecisionActionError::CompanyNotFound);
        }
        if helpers::row_company(&conn, "issues", &input.issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(DecisionActionError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO decision_training_examples (id, company_id, source_kind, source_id,
                                                         issue_id, cutoff_at, notes,
                                                         notes_history, decision_outcome,
                                                         retention_policy, snapshot,
                                                         created_by_user_id, created_at,
                                                         updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.source_kind,
                    input.source_id,
                    input.issue_id,
                    input.cutoff_at,
                    input.notes,
                    input.notes_history.to_string(),
                    input.decision_outcome,
                    input.retention_policy,
                    input.snapshot.to_string(),
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {EXAMPLE_COLUMNS} FROM decision_training_examples WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("example was just inserted");
                Ok(row_to_example(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(DecisionActionError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_training_examples(
        &self,
        company_id: &str,
        issue_id: Option<&str>,
    ) -> Result<Vec<DecisionTrainingExampleRecord>, DecisionActionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let issue_filter = issue_id.map(|_| "AND issue_id = ?2").unwrap_or_default();
        let mut params: Vec<libsql::Value> = vec![company_id.into()];
        if let Some(value) = issue_id {
            params.push(value.into());
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EXAMPLE_COLUMNS} FROM decision_training_examples
                     WHERE company_id = ?1 {issue_filter}
                     ORDER BY created_at DESC"
                ),
                params,
            )
            .await?;
        let mut examples = Vec::new();
        while let Some(row) = rows.next().await? {
            examples.push(row_to_example(&row)?);
        }
        Ok(examples)
    }
}

fn row_to_bundle(row: &libsql::Row) -> Result<DecisionBundleRecord, libsql::Error> {
    Ok(DecisionBundleRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        title: helpers::row_text(row, 2)?.expect("title"),
        summary: helpers::row_text(row, 3)?.expect("summary"),
        origin_agent_id: helpers::row_text(row, 4)?.expect("origin_agent_id"),
        origin_issue_id: helpers::row_text(row, 5)?.expect("origin_issue_id"),
        origin_run_id: helpers::row_text(row, 6)?.expect("origin_run_id"),
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
    })
}

fn json_or_default(value: Option<String>) -> serde_json::Value {
    value
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

fn row_to_decision(row: &libsql::Row) -> Result<DecisionRecord, libsql::Error> {
    Ok(DecisionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        bundle_id: helpers::row_text(row, 2)?,
        origin_agent_id: helpers::row_text(row, 3)?.expect("origin_agent_id"),
        origin_issue_id: helpers::row_text(row, 4)?.expect("origin_issue_id"),
        origin_run_id: helpers::row_text(row, 5)?.expect("origin_run_id"),
        rule_key: helpers::row_text(row, 6)?,
        title: helpers::row_text(row, 7)?.expect("title"),
        body: helpers::row_text(row, 8)?.expect("body"),
        options: json_or_default(helpers::row_text(row, 9)?),
        inputs: helpers::row_text(row, 10)?.and_then(|v| serde_json::from_str(&v).ok()),
        status: helpers::row_text(row, 11)?.expect("status"),
        execution_status: helpers::row_text(row, 12)?,
        chosen_option_id: helpers::row_text(row, 13)?,
        input_values: helpers::row_text(row, 14)?.and_then(|v| serde_json::from_str(&v).ok()),
        decided_by_user_id: helpers::row_text(row, 15)?,
        decided_at: helpers::row_text(row, 16)?,
        expires_at: helpers::row_text(row, 17)?.expect("expires_at"),
        idempotency_key: helpers::row_text(row, 18)?,
        signed_spec: helpers::row_text(row, 19)?.expect("signed_spec"),
        target_snapshots: json_or_default(helpers::row_text(row, 20)?),
        continuation_policy: helpers::row_text(row, 21)?.expect("continuation_policy"),
        metadata: json_or_default(helpers::row_text(row, 22)?),
        created_at: helpers::row_text(row, 23)?.expect("created_at"),
        updated_at: helpers::row_text(row, 24)?.expect("updated_at"),
    })
}

fn row_to_effect(row: &libsql::Row) -> Result<DecisionEffectExecutionRecord, libsql::Error> {
    Ok(DecisionEffectExecutionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        decision_id: helpers::row_text(row, 2)?.expect("decision_id"),
        effect_index: helpers::row_i64(row, 3)?,
        effect_type: helpers::row_text(row, 4)?.expect("effect_type"),
        target_issue_id: helpers::row_text(row, 5)?.expect("target_issue_id"),
        status: helpers::row_text(row, 6)?.expect("status"),
        result: helpers::row_text(row, 7)?.and_then(|v| serde_json::from_str(&v).ok()),
        error: helpers::row_text(row, 8)?,
        activity_log_id: helpers::row_text(row, 9)?,
        executed_at: helpers::row_text(row, 10)?,
    })
}

fn row_to_example(row: &libsql::Row) -> Result<DecisionTrainingExampleRecord, libsql::Error> {
    Ok(DecisionTrainingExampleRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        source_kind: helpers::row_text(row, 2)?.expect("source_kind"),
        source_id: helpers::row_text(row, 3)?.expect("source_id"),
        issue_id: helpers::row_text(row, 4)?.expect("issue_id"),
        cutoff_at: helpers::row_text(row, 5)?.expect("cutoff_at"),
        notes: helpers::row_text(row, 6)?.expect("notes"),
        notes_history: json_or_default(helpers::row_text(row, 7)?),
        decision_outcome: helpers::row_text(row, 8)?,
        retention_policy: helpers::row_text(row, 9)?.expect("retention_policy"),
        snapshot: json_or_default(helpers::row_text(row, 10)?),
        created_by_user_id: helpers::row_text(row, 11)?.expect("created_by_user_id"),
        created_at: helpers::row_text(row, 12)?.expect("created_at"),
        updated_at: helpers::row_text(row, 13)?.expect("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoDecisionActionRepository) {
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
             VALUES ('a1', 'c1', 'Agent', 'engineer', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'Issue 1', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO heartbeat_runs (id, company_id, agent_id, invocation_source)
             VALUES ('r1', 'c1', 'a1', 'manual')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoDecisionActionRepository::new(db);
        (dir, repo)
    }

    fn bundle_input() -> NewDecisionBundle {
        NewDecisionBundle {
            company_id: "c1".to_owned(),
            title: "Bundle".to_owned(),
            summary: "Summary".to_owned(),
            origin_agent_id: "a1".to_owned(),
            origin_issue_id: "i1".to_owned(),
            origin_run_id: "r1".to_owned(),
        }
    }

    fn decision_input() -> NewDecision {
        NewDecision {
            company_id: "c1".to_owned(),
            bundle_id: None,
            origin_agent_id: "a1".to_owned(),
            origin_issue_id: "i1".to_owned(),
            origin_run_id: "r1".to_owned(),
            rule_key: Some("rule-1".to_owned()),
            title: "Decision".to_owned(),
            body: "Body".to_owned(),
            options: serde_json::json!([{ "id": "opt-1", "label": "Option 1" }]),
            inputs: Some(serde_json::json!({ "severity": "high" })),
            status: "open".to_owned(),
            execution_status: None,
            chosen_option_id: None,
            input_values: None,
            decided_by_user_id: None,
            decided_at: None,
            expires_at: "2026-12-31T00:00:00.000Z".to_owned(),
            idempotency_key: Some("idem-1".to_owned()),
            signed_spec: "spec".to_owned(),
            target_snapshots: serde_json::json!({ "issue": { "title": "Issue 1" } }),
            continuation_policy: "none".to_owned(),
            metadata: serde_json::json!({ "origin": "test" }),
        }
    }

    #[tokio::test]
    async fn bundle_decision_target_effect_roundtrip() {
        let (_dir, repo) = repo().await;
        let bundle = repo.create_bundle(bundle_input()).await.unwrap();
        assert_eq!(bundle.title, "Bundle");
        assert_eq!(repo.list_bundles("c1").await.unwrap().len(), 1);

        let decision = repo.create_decision(decision_input()).await.unwrap();
        assert_eq!(decision.status, "open");
        assert_eq!(decision.options[0]["id"].as_str(), Some("opt-1"));

        // Duplicate idempotency key rejected.
        assert!(matches!(
            repo.create_decision(decision_input()).await.unwrap_err(),
            DecisionActionError::AlreadyExists
        ));

        // Target issues.
        let link = repo
            .add_target_issue("c1", &decision.id, "i1")
            .await
            .unwrap();
        assert_eq!(link.issue_id, "i1");
        assert!(matches!(
            repo.add_target_issue("c1", &decision.id, "i1")
                .await
                .unwrap_err(),
            DecisionActionError::AlreadyExists
        ));
        assert_eq!(
            repo.list_target_issues("c1", &decision.id)
                .await
                .unwrap()
                .len(),
            1
        );

        // Resolve.
        let resolved = repo
            .resolve_decision(ResolveDecision {
                company_id: "c1".to_owned(),
                decision_id: decision.id.clone(),
                status: "decided".to_owned(),
                execution_status: Some("succeeded".to_owned()),
                chosen_option_id: Some("opt-1".to_owned()),
                decided_by_user_id: Some("u1".to_owned()),
                decided_at: Some("2026-08-04T00:00:00.000Z".to_owned()),
                input_values: Some(serde_json::json!({ "opt-1": "yes" })),
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resolved.status, "decided");
        assert_eq!(resolved.chosen_option_id.as_deref(), Some("opt-1"));
        assert_eq!(resolved.input_values.as_ref().unwrap()["opt-1"], "yes");

        // Effect executions.
        let effect = repo
            .create_effect_execution(NewDecisionEffectExecution {
                company_id: "c1".to_owned(),
                decision_id: decision.id.clone(),
                effect_index: 0,
                effect_type: "apply_label".to_owned(),
                target_issue_id: "i1".to_owned(),
                status: "claimed".to_owned(),
                result: None,
                error: None,
                activity_log_id: None,
                executed_at: None,
            })
            .await
            .unwrap();
        assert_eq!(effect.effect_type, "apply_label");
        // Duplicate effect index rejected.
        assert!(matches!(
            repo.create_effect_execution(NewDecisionEffectExecution {
                company_id: "c1".to_owned(),
                decision_id: decision.id.clone(),
                effect_index: 0,
                effect_type: "other".to_owned(),
                target_issue_id: "i1".to_owned(),
                status: "claimed".to_owned(),
                result: None,
                error: None,
                activity_log_id: None,
                executed_at: None,
            })
            .await
            .unwrap_err(),
            DecisionActionError::AlreadyExists
        ));
        let updated = repo
            .update_effect_execution(
                "c1",
                &effect.id,
                Some("succeeded"),
                Some(serde_json::json!({ "ok": true })),
                None,
                Some("2026-08-04T00:00:00.000Z".to_owned()),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "succeeded");
        assert_eq!(updated.result.as_ref().unwrap()["ok"], true);

        // Cross-company lists are empty.
        assert!(
            repo.get_decision("c2", &decision.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(repo.list_bundles("c2").await.unwrap().is_empty());
        assert!(
            repo.list_effect_executions("c2", &decision.id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn training_example_roundtrip_and_dedupe() {
        let (_dir, repo) = repo().await;
        let example = repo
            .create_training_example(NewDecisionTrainingExample {
                company_id: "c1".to_owned(),
                source_kind: "decision".to_owned(),
                source_id: "d1".to_owned(),
                issue_id: "i1".to_owned(),
                cutoff_at: "2026-08-01T00:00:00.000Z".to_owned(),
                notes: "note".to_owned(),
                notes_history: serde_json::json!([]),
                decision_outcome: Some("accepted".to_owned()),
                retention_policy: "scrub_deleted_comments_v1".to_owned(),
                snapshot: serde_json::json!({ "title": "Decision" }),
                created_by_user_id: "u1".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(example.source_kind, "decision");
        assert_eq!(example.snapshot["title"].as_str(), Some("Decision"));

        // Same source + author dedupes.
        assert!(matches!(
            repo.create_training_example(NewDecisionTrainingExample {
                company_id: "c1".to_owned(),
                source_kind: "decision".to_owned(),
                source_id: "d1".to_owned(),
                issue_id: "i1".to_owned(),
                cutoff_at: "2026-08-02T00:00:00.000Z".to_owned(),
                notes: "other".to_owned(),
                notes_history: serde_json::json!([]),
                decision_outcome: None,
                retention_policy: "scrub_deleted_comments_v1".to_owned(),
                snapshot: serde_json::json!({}),
                created_by_user_id: "u1".to_owned(),
            })
            .await
            .unwrap_err(),
            DecisionActionError::AlreadyExists
        ));

        let list = repo.list_training_examples("c1", Some("i1")).await.unwrap();
        assert_eq!(list.len(), 1);
        assert!(
            repo.list_training_examples("c2", None)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
