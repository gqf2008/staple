//! Routines repository: definitions with append-only revisions, triggers,
//! and runs.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A routine definition.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineRecord {
    /// Routine id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Goal id.
    pub goal_id: Option<String>,
    /// Parent issue id.
    pub parent_issue_id: Option<String>,
    /// Title.
    pub title: String,
    /// Description.
    pub description: Option<String>,
    /// Assignee agent id.
    pub assignee_agent_id: Option<String>,
    /// Priority.
    pub priority: String,
    /// Status.
    pub status: String,
    /// Concurrency policy.
    pub concurrency_policy: String,
    /// Catch-up policy.
    pub catch_up_policy: String,
    /// Variables JSON.
    pub variables: String,
    /// Latest revision number.
    pub latest_revision_number: i64,
    /// Latest revision id.
    pub latest_revision_id: Option<String>,
    /// ISO 8601 last triggered time.
    pub last_triggered_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A routine run.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineRunRecord {
    /// Run id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Routine id.
    pub routine_id: String,
    /// Revision id.
    pub revision_id: Option<String>,
    /// Status.
    pub status: String,
    /// Triggered by.
    pub triggered_by: Option<String>,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Error.
    pub error: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating a routine (revision 1).
#[derive(Debug, Clone)]
pub struct NewRoutine {
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Goal id.
    pub goal_id: Option<String>,
    /// Parent issue id.
    pub parent_issue_id: Option<String>,
    /// Title.
    pub title: String,
    /// Description.
    pub description: Option<String>,
    /// Assignee agent id.
    pub assignee_agent_id: Option<String>,
    /// Priority.
    pub priority: String,
    /// Variables JSON.
    pub variables: Option<String>,
}

/// Input for updating a routine (appends a revision).
#[derive(Debug, Clone)]
pub struct UpdateRoutine {
    /// Owning company id.
    pub company_id: String,
    /// Routine id.
    pub routine_id: String,
    /// Title.
    pub title: String,
    /// Description.
    pub description: Option<String>,
    /// Variables JSON.
    pub variables: Option<String>,
}

/// Input for creating a trigger.
#[derive(Debug, Clone)]
pub struct NewTrigger {
    /// Owning company id.
    pub company_id: String,
    /// Routine id.
    pub routine_id: String,
    /// Schedule kind.
    pub schedule_kind: String,
    /// Schedule expression.
    pub schedule_expr: Option<String>,
}

/// Routines repository errors.
#[derive(Debug, Error)]
pub enum RoutineError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// A referenced record does not exist in this company.
    #[error("referenced record not found: {0}")]
    ReferenceNotFound(&'static str),
    /// The routine does not exist.
    #[error("routine not found")]
    RoutineNotFound,
}

/// Routines persistence contract.
#[async_trait]
pub trait RoutineRepository: Send + Sync {
    /// Creates a routine with revision 1.
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] on invalid references.
    async fn create(&self, input: NewRoutine) -> Result<RoutineRecord, RoutineError>;

    /// Lists routines for a company.
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] on database failure.
    async fn list(&self, company_id: &str) -> Result<Vec<RoutineRecord>, RoutineError>;

    /// Fetches one routine by id.
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<RoutineRecord>, RoutineError>;

    /// Updates a routine (appends a revision).
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] when the routine is missing.
    async fn update(&self, input: UpdateRoutine) -> Result<RoutineRecord, RoutineError>;

    /// Triggers a routine: creates a run and stamps last-triggered times.
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] when the routine is missing.
    async fn trigger(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<RoutineRunRecord, RoutineError>;

    /// Lists runs for a routine.
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] on database failure.
    async fn list_runs(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<Vec<RoutineRunRecord>, RoutineError>;

    /// Creates a trigger for a routine.
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] on invalid references.
    async fn create_trigger(&self, input: NewTrigger) -> Result<serde_json::Value, RoutineError>;

    /// Lists triggers for a routine.
    ///
    /// # Errors
    ///
    /// Returns [`RoutineError`] on database failure.
    async fn list_triggers(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<Vec<serde_json::Value>, RoutineError>;
}

/// Turso/libSQL implementation of [`RoutineRepository`].
#[derive(Debug)]
pub struct TursoRoutineRepository {
    db: Database,
}

impl TursoRoutineRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RoutineRepository for TursoRoutineRepository {
    async fn create(&self, input: NewRoutine) -> Result<RoutineRecord, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        for (reference, value) in [
            ("project", input.project_id.as_deref()),
            ("goal", input.goal_id.as_deref()),
            ("parent_issue", input.parent_issue_id.as_deref()),
            ("assignee_agent", input.assignee_agent_id.as_deref()),
        ] {
            if let Some(value) = value {
                let table = match reference {
                    "project" => "projects",
                    "goal" => "goals",
                    "parent_issue" => "issues",
                    _ => "agents",
                };
                if !helpers::row_belongs_to_company(&conn, table, value, &input.company_id).await? {
                    return Err(RoutineError::ReferenceNotFound(reference));
                }
            }
        }
        let routine_id = Uuid::new_v4().to_string();
        let revision_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO routines (id, company_id, project_id, goal_id, parent_issue_id, title,
                                   description, assignee_agent_id, priority, status, variables,
                                   latest_revision_id, latest_revision_number, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'active', ?10, ?11, 1,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                routine_id.clone(),
                input.company_id.clone(),
                input.project_id,
                input.goal_id,
                input.parent_issue_id,
                input.title.clone(),
                input.description.clone(),
                input.assignee_agent_id,
                input.priority.clone(),
                input.variables.clone().unwrap_or_else(|| "[]".to_owned()),
                revision_id.clone()
            ],
        )
        .await?;
        conn.execute(
            "INSERT INTO routine_revisions (id, company_id, routine_id, revision_number, title,
                                            description, priority, variables, created_at)
             VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                revision_id,
                input.company_id,
                routine_id.clone(),
                input.title,
                input.description,
                input.priority,
                input.variables.unwrap_or_else(|| "[]".to_owned())
            ],
        )
        .await?;
        Ok(self
            .get(&routine_id)
            .await?
            .expect("routine was just inserted"))
    }

    async fn list(&self, company_id: &str) -> Result<Vec<RoutineRecord>, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, project_id, goal_id, parent_issue_id, title, description,
                        assignee_agent_id, priority, status, concurrency_policy, catch_up_policy,
                        variables, latest_revision_number, latest_revision_id, last_triggered_at,
                        created_at
                 FROM routines WHERE company_id = ?1 ORDER BY created_at",
                libsql::params![company_id],
            )
            .await?;
        let mut routines = Vec::new();
        while let Some(row) = rows.next().await? {
            routines.push(row_to_routine(&row)?);
        }
        Ok(routines)
    }

    async fn get(&self, id: &str) -> Result<Option<RoutineRecord>, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, project_id, goal_id, parent_issue_id, title, description,
                        assignee_agent_id, priority, status, concurrency_policy, catch_up_policy,
                        variables, latest_revision_number, latest_revision_id, last_triggered_at,
                        created_at
                 FROM routines WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_routine(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, input: UpdateRoutine) -> Result<RoutineRecord, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(existing) = self.get(&input.routine_id).await? else {
            return Err(RoutineError::RoutineNotFound);
        };
        if existing.company_id != input.company_id {
            return Err(RoutineError::RoutineNotFound);
        }
        let revision_number = existing.latest_revision_number + 1;
        let revision_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO routine_revisions (id, company_id, routine_id, revision_number, title,
                                            description, priority, variables, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                revision_id.clone(),
                input.company_id.clone(),
                input.routine_id.clone(),
                revision_number,
                input.title.clone(),
                input.description.clone(),
                existing.priority,
                input.variables.clone().unwrap_or(existing.variables)
            ],
        )
        .await?;
        conn.execute(
            "UPDATE routines
             SET title = ?1, description = ?2, latest_revision_id = ?3,
                 latest_revision_number = ?4, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?5",
            libsql::params![
                input.title,
                input.description,
                revision_id,
                revision_number,
                input.routine_id.clone()
            ],
        )
        .await?;
        Ok(self.get(&input.routine_id).await?.expect("routine exists"))
    }

    async fn trigger(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<RoutineRunRecord, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(routine) = self.get(routine_id).await? else {
            return Err(RoutineError::RoutineNotFound);
        };
        if routine.company_id != company_id {
            return Err(RoutineError::RoutineNotFound);
        }
        let run_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO routine_runs (id, company_id, routine_id, revision_id, status,
                                       triggered_by, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'queued', 'manual',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                run_id.clone(),
                company_id,
                routine_id,
                routine.latest_revision_id
            ],
        )
        .await?;
        conn.execute(
            "UPDATE routines
             SET last_triggered_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 last_enqueued_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?1",
            libsql::params![routine_id],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, routine_id, revision_id, status, triggered_by, issue_id,
                        error, created_at FROM routine_runs WHERE id = ?1",
                libsql::params![run_id],
            )
            .await?;
        let row = rows.next().await?.expect("run was just inserted");
        Ok(row_to_run(&row)?)
    }

    async fn list_runs(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<Vec<RoutineRunRecord>, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, routine_id, revision_id, status, triggered_by, issue_id,
                        error, created_at FROM routine_runs
                 WHERE company_id = ?1 AND routine_id = ?2 ORDER BY created_at DESC",
                libsql::params![company_id, routine_id],
            )
            .await?;
        let mut runs = Vec::new();
        while let Some(row) = rows.next().await? {
            runs.push(row_to_run(&row)?);
        }
        Ok(runs)
    }

    async fn create_trigger(&self, input: NewTrigger) -> Result<serde_json::Value, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "routines", &input.routine_id, &input.company_id)
            .await?
        {
            return Err(RoutineError::RoutineNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO routine_triggers (id, company_id, routine_id, schedule_kind,
                                           schedule_expr, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.routine_id,
                input.schedule_kind,
                input.schedule_expr
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, routine_id, schedule_kind, schedule_expr, enabled, created_at
                 FROM routine_triggers WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("trigger was just inserted");
        Ok(serde_json::json!({
            "id": helpers::row_text(&row, 0)?.expect("id"),
            "companyId": helpers::row_text(&row, 1)?.expect("company_id"),
            "routineId": helpers::row_text(&row, 2)?.expect("routine_id"),
            "scheduleKind": helpers::row_text(&row, 3)?.expect("schedule_kind"),
            "scheduleExpr": helpers::row_text(&row, 4)?,
            "enabled": helpers::row_i64(&row, 5)? != 0,
        }))
    }

    async fn list_triggers(
        &self,
        company_id: &str,
        routine_id: &str,
    ) -> Result<Vec<serde_json::Value>, RoutineError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, routine_id, schedule_kind, schedule_expr, enabled, created_at
                 FROM routine_triggers WHERE company_id = ?1 AND routine_id = ?2 ORDER BY created_at",
                libsql::params![company_id, routine_id],
            )
            .await?;
        let mut triggers = Vec::new();
        while let Some(row) = rows.next().await? {
            triggers.push(serde_json::json!({
                "id": helpers::row_text(&row, 0)?.expect("id"),
                "companyId": helpers::row_text(&row, 1)?.expect("company_id"),
                "routineId": helpers::row_text(&row, 2)?.expect("routine_id"),
                "scheduleKind": helpers::row_text(&row, 3)?.expect("schedule_kind"),
                "scheduleExpr": helpers::row_text(&row, 4)?,
                "enabled": helpers::row_i64(&row, 5)? != 0,
            }));
        }
        Ok(triggers)
    }
}

fn row_to_routine(row: &libsql::Row) -> Result<RoutineRecord, libsql::Error> {
    Ok(RoutineRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        project_id: helpers::row_text(row, 2)?,
        goal_id: helpers::row_text(row, 3)?,
        parent_issue_id: helpers::row_text(row, 4)?,
        title: helpers::row_text(row, 5)?.expect("title"),
        description: helpers::row_text(row, 6)?,
        assignee_agent_id: helpers::row_text(row, 7)?,
        priority: helpers::row_text(row, 8)?.expect("priority"),
        status: helpers::row_text(row, 9)?.expect("status"),
        concurrency_policy: helpers::row_text(row, 10)?.expect("concurrency_policy"),
        catch_up_policy: helpers::row_text(row, 11)?.expect("catch_up_policy"),
        variables: helpers::row_text(row, 12)?.expect("variables"),
        latest_revision_number: helpers::row_i64(row, 13)?,
        latest_revision_id: helpers::row_text(row, 14)?,
        last_triggered_at: helpers::row_text(row, 15)?,
        created_at: helpers::row_text(row, 16)?.expect("created_at"),
    })
}

fn row_to_run(row: &libsql::Row) -> Result<RoutineRunRecord, libsql::Error> {
    Ok(RoutineRunRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        routine_id: helpers::row_text(row, 2)?.expect("routine_id"),
        revision_id: helpers::row_text(row, 3)?,
        status: helpers::row_text(row, 4)?.expect("status"),
        triggered_by: helpers::row_text(row, 5)?,
        issue_id: helpers::row_text(row, 6)?,
        error: helpers::row_text(row, 7)?,
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoRoutineRepository) {
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
             VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoRoutineRepository::new(db);
        (dir, repo)
    }

    fn new_routine() -> NewRoutine {
        NewRoutine {
            company_id: "c1".to_owned(),
            project_id: None,
            goal_id: None,
            parent_issue_id: None,
            title: "Daily report".to_owned(),
            description: Some("generate daily report".to_owned()),
            assignee_agent_id: Some("a1".to_owned()),
            priority: "medium".to_owned(),
            variables: Some(r#"[{"name":"fmt","value":"md"}]"#.to_owned()),
        }
    }

    #[tokio::test]
    async fn create_update_trigger_roundtrip() {
        let (_dir, repo) = repo().await;
        let routine = repo.create(new_routine()).await.unwrap();
        assert_eq!(routine.latest_revision_number, 1);
        assert_eq!(routine.status, "active");

        let updated = repo
            .update(UpdateRoutine {
                company_id: "c1".to_owned(),
                routine_id: routine.id.clone(),
                title: "Weekly report".to_owned(),
                description: None,
                variables: None,
            })
            .await
            .unwrap();
        assert_eq!(updated.latest_revision_number, 2);
        assert_eq!(updated.title, "Weekly report");

        let run = repo.trigger("c1", &routine.id).await.unwrap();
        assert_eq!(run.status, "queued");
        let runs = repo.list_runs("c1", &routine.id).await.unwrap();
        assert_eq!(runs.len(), 1);

        let trigger = repo
            .create_trigger(NewTrigger {
                company_id: "c1".to_owned(),
                routine_id: routine.id.clone(),
                schedule_kind: "cron".to_owned(),
                schedule_expr: Some("0 9 * * *".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(trigger["scheduleKind"], "cron");
        let triggers = repo.list_triggers("c1", &routine.id).await.unwrap();
        assert_eq!(triggers.len(), 1);

        // Cross-company trigger rejected.
        let error = repo
            .create_trigger(NewTrigger {
                company_id: "c2".to_owned(),
                routine_id: routine.id,
                schedule_kind: "manual".to_owned(),
                schedule_expr: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, RoutineError::RoutineNotFound));
    }
}
