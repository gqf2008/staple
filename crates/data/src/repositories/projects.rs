//! Projects repository: trait plus Turso/libSQL implementation.
//!
//! Enforces the service-level hierarchy rules: linked goals and lead agents
//! must exist and belong to the same company as the project.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `projects` table.
#[derive(Debug, Clone)]
pub struct ProjectRecord {
    /// Project id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Linked goal id.
    pub goal_id: Option<String>,
    /// Name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// `backlog | planned | in_progress | completed | cancelled`.
    pub status: String,
    /// Lead agent id.
    pub lead_agent_id: Option<String>,
    /// Target date.
    pub target_date: Option<String>,
    /// Secret-aware environment bindings (JSON).
    pub env: Option<String>,
    /// Brand color.
    pub color: Option<String>,
    /// Icon.
    pub icon: Option<String>,
    /// Pause reason.
    pub pause_reason: Option<String>,
    /// ISO 8601 pause time.
    pub paused_at: Option<String>,
    /// Execution workspace policy JSON.
    pub execution_workspace_policy: Option<serde_json::Value>,
    /// ISO 8601 archive time.
    pub archived_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating a project.
#[derive(Debug, Clone)]
pub struct NewProject {
    /// Owning company id.
    pub company_id: String,
    /// Linked goal id.
    pub goal_id: Option<String>,
    /// Name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// `backlog | planned | in_progress | completed | cancelled`.
    pub status: String,
    /// Lead agent id.
    pub lead_agent_id: Option<String>,
    /// Target date.
    pub target_date: Option<String>,
    /// Environment bindings (JSON).
    pub env: Option<String>,
    /// Execution workspace policy JSON.
    pub execution_workspace_policy: Option<serde_json::Value>,
}

/// Partial project update.
#[derive(Debug, Default)]
pub struct ProjectPatch {
    /// New linked goal id.
    pub goal_id: Option<Option<String>>,
    /// New name.
    pub name: Option<String>,
    /// New description.
    pub description: Option<Option<String>>,
    /// New status.
    pub status: Option<String>,
    /// New lead agent id.
    pub lead_agent_id: Option<Option<String>>,
    /// New target date.
    pub target_date: Option<Option<String>>,
    /// New execution workspace policy JSON (`null` clears).
    pub execution_workspace_policy: Option<Option<serde_json::Value>>,
}

/// Projects repository errors.
#[derive(Debug, Error)]
pub enum ProjectError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The owning company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The linked goal does not exist.
    #[error("goal not found")]
    GoalNotFound,
    /// The linked goal belongs to a different company.
    #[error("goal belongs to a different company")]
    GoalInDifferentCompany,
    /// The lead agent does not exist.
    #[error("lead agent not found")]
    LeadAgentNotFound,
    /// The lead agent belongs to a different company.
    #[error("lead agent belongs to a different company")]
    LeadAgentInDifferentCompany,
    /// The row is referenced by other records and cannot be deleted.
    #[error("resource is referenced by other records")]
    InUse,
}

/// Project persistence contract.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Creates a project, validating company, goal, and lead references.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError`] when a referenced row is missing or belongs to
    /// a different company.
    async fn create(&self, input: NewProject) -> Result<ProjectRecord, ProjectError>;

    /// Lists all projects of one company.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError`] on database failure.
    async fn list(&self, company_id: &str) -> Result<Vec<ProjectRecord>, ProjectError>;

    /// Fetches one project by id.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<ProjectRecord>, ProjectError>;

    /// Applies a partial update.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError`] on database failure or invalid references.
    async fn update(
        &self,
        id: &str,
        patch: ProjectPatch,
    ) -> Result<Option<ProjectRecord>, ProjectError>;

    /// Deletes a project, returning the deleted row.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError`] on database failure.
    async fn delete(&self, id: &str) -> Result<Option<ProjectRecord>, ProjectError>;
}

/// Turso/libSQL implementation of [`ProjectRepository`].
#[derive(Debug)]
pub struct TursoProjectRepository {
    db: Database,
}

impl TursoProjectRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const PROJECT_COLUMNS: &str = "id, company_id, goal_id, name, description, status,
    lead_agent_id, target_date, env, color, icon, pause_reason, paused_at,
    execution_workspace_policy, archived_at, created_at, updated_at";

fn row_to_project(row: &libsql::Row) -> Result<ProjectRecord, libsql::Error> {
    Ok(ProjectRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        goal_id: helpers::row_text(row, 2)?,
        name: helpers::row_text(row, 3)?.expect("name is NOT NULL"),
        description: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status is NOT NULL"),
        lead_agent_id: helpers::row_text(row, 6)?,
        target_date: helpers::row_text(row, 7)?,
        env: helpers::row_text(row, 8)?,
        color: helpers::row_text(row, 9)?,
        icon: helpers::row_text(row, 10)?,
        pause_reason: helpers::row_text(row, 11)?,
        paused_at: helpers::row_text(row, 12)?,
        execution_workspace_policy: helpers::row_text(row, 13)?
            .and_then(|raw| serde_json::from_str(&raw).ok()),
        archived_at: helpers::row_text(row, 14)?,
        created_at: helpers::row_text(row, 15)?.expect("created_at is NOT NULL"),
        updated_at: helpers::row_text(row, 16)?.expect("updated_at is NOT NULL"),
    })
}

#[async_trait]
impl ProjectRepository for TursoProjectRepository {
    async fn create(&self, input: NewProject) -> Result<ProjectRecord, ProjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ProjectError::CompanyNotFound);
        }
        if let Some(goal_id) = &input.goal_id {
            if !helpers::find_row(&conn, "goals", goal_id).await? {
                return Err(ProjectError::GoalNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "goals", goal_id, &input.company_id).await? {
                return Err(ProjectError::GoalInDifferentCompany);
            }
        }
        if let Some(lead_agent_id) = &input.lead_agent_id {
            if !helpers::find_row(&conn, "agents", lead_agent_id).await? {
                return Err(ProjectError::LeadAgentNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "agents", lead_agent_id, &input.company_id)
                .await?
            {
                return Err(ProjectError::LeadAgentInDifferentCompany);
            }
        }

        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO projects (id, company_id, goal_id, name, description, status,
                                   lead_agent_id, target_date, env, execution_workspace_policy,
                                   created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.goal_id,
                input.name,
                input.description,
                input.status,
                input.lead_agent_id,
                input.target_date,
                input.env,
                input
                    .execution_workspace_policy
                    .map(|value| value.to_string())
            ],
        )
        .await?;
        Ok(self.get(&id).await?.expect("project was just inserted"))
    }

    async fn list(&self, company_id: &str) -> Result<Vec<ProjectRecord>, ProjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!(
            "SELECT {PROJECT_COLUMNS} FROM projects WHERE company_id = ?1 ORDER BY created_at"
        );
        let mut rows = conn.query(&sql, libsql::params![company_id]).await?;
        let mut projects = Vec::new();
        while let Some(row) = rows.next().await? {
            projects.push(row_to_project(&row)?);
        }
        Ok(projects)
    }

    async fn get(&self, id: &str) -> Result<Option<ProjectRecord>, ProjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {PROJECT_COLUMNS} FROM projects WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_project(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        id: &str,
        patch: ProjectPatch,
    ) -> Result<Option<ProjectRecord>, ProjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let company_id = helpers::row_company(&conn, "projects", id).await?;
        let Some(company_id) = company_id else {
            return Ok(None);
        };

        if let Some(Some(goal_id)) = &patch.goal_id {
            if !helpers::find_row(&conn, "goals", goal_id).await? {
                return Err(ProjectError::GoalNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "goals", goal_id, &company_id).await? {
                return Err(ProjectError::GoalInDifferentCompany);
            }
        }
        if let Some(Some(lead_agent_id)) = &patch.lead_agent_id {
            if !helpers::find_row(&conn, "agents", lead_agent_id).await? {
                return Err(ProjectError::LeadAgentNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "agents", lead_agent_id, &company_id).await?
            {
                return Err(ProjectError::LeadAgentInDifferentCompany);
            }
        }

        let (sets, values) = helpers::build_update(&[
            ("goal_id", patch.goal_id.map(|value| value.map(Into::into))),
            ("name", patch.name.map(|value| Some(value.into()))),
            (
                "description",
                patch.description.map(|value| value.map(Into::into)),
            ),
            ("status", patch.status.map(|value| Some(value.into()))),
            (
                "lead_agent_id",
                patch.lead_agent_id.map(|value| value.map(Into::into)),
            ),
            (
                "target_date",
                patch.target_date.map(|value| value.map(Into::into)),
            ),
            (
                "execution_workspace_policy",
                patch
                    .execution_workspace_policy
                    .map(|value| value.map(|json| json.to_string().into())),
            ),
        ]);
        if sets.is_empty() {
            return self.get(id).await;
        }
        let updated = helpers::execute_update(&conn, "projects", id, sets, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        Ok(self.get(id).await?)
    }

    async fn delete(&self, id: &str) -> Result<Option<ProjectRecord>, ProjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let project = self.get(id).await?;
        let Some(project) = project else {
            return Ok(None);
        };
        match conn
            .execute("DELETE FROM projects WHERE id = ?1", libsql::params![id])
            .await
        {
            Ok(_) => Ok(Some(project)),
            Err(error) if error.to_string().contains("FOREIGN KEY constraint failed") => {
                Err(ProjectError::InUse)
            }
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{Connection, migrate, open};

    async fn repo() -> (TempDir, TursoProjectRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoProjectRepository::new(db);
        (dir, repo, conn)
    }

    async fn seed(conn: &Connection) {
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024),
                    ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'one', 'engineer', 'codex_local'),
                    ('a2', 'c2', 'two', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO goals (id, company_id, title, level)
             VALUES ('g1', 'c1', 'Goal One', 'company'),
                    ('g2', 'c2', 'Goal Two', 'company')",
            (),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn create_validates_references() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;

        // Cross-company goal rejected.
        let error = repo
            .create(NewProject {
                company_id: "c1".to_owned(),
                goal_id: Some("g2".to_owned()),
                name: "P".to_owned(),
                description: None,
                status: "backlog".to_owned(),
                lead_agent_id: None,
                target_date: None,
                env: None,
                execution_workspace_policy: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, ProjectError::GoalInDifferentCompany));

        // Cross-company lead rejected.
        let error = repo
            .create(NewProject {
                company_id: "c1".to_owned(),
                goal_id: None,
                name: "P".to_owned(),
                description: None,
                status: "backlog".to_owned(),
                lead_agent_id: Some("a2".to_owned()),
                target_date: None,
                env: None,
                execution_workspace_policy: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, ProjectError::LeadAgentInDifferentCompany));

        // Valid creation succeeds.
        let project = repo
            .create(NewProject {
                company_id: "c1".to_owned(),
                goal_id: Some("g1".to_owned()),
                name: "Ship".to_owned(),
                description: Some("d".to_owned()),
                status: "in_progress".to_owned(),
                lead_agent_id: Some("a1".to_owned()),
                target_date: Some("2026-09-01".to_owned()),
                env: None,
                execution_workspace_policy: None,
            })
            .await
            .unwrap();
        assert_eq!(project.name, "Ship");
        assert_eq!(project.status, "in_progress");
        assert_eq!(project.goal_id.as_deref(), Some("g1"));
    }

    #[tokio::test]
    async fn create_requires_company() {
        let (_dir, repo, _conn) = repo().await;
        let error = repo
            .create(NewProject {
                company_id: "missing".to_owned(),
                goal_id: None,
                name: "P".to_owned(),
                description: None,
                status: "backlog".to_owned(),
                lead_agent_id: None,
                target_date: None,
                env: None,
                execution_workspace_policy: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, ProjectError::CompanyNotFound));
    }

    #[tokio::test]
    async fn list_get_update_delete_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let created = repo
            .create(NewProject {
                company_id: "c1".to_owned(),
                goal_id: None,
                name: "P".to_owned(),
                description: None,
                status: "backlog".to_owned(),
                lead_agent_id: None,
                target_date: None,
                env: None,
                execution_workspace_policy: None,
            })
            .await
            .unwrap();

        let list = repo.list("c1").await.unwrap();
        assert_eq!(list.len(), 1);

        let fetched = repo.get(&created.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "P");

        let updated = repo
            .update(
                &created.id,
                ProjectPatch {
                    name: Some("P2".to_owned()),
                    status: Some("completed".to_owned()),
                    goal_id: Some(Some("g1".to_owned())),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "P2");
        assert_eq!(updated.status, "completed");
        assert_eq!(updated.goal_id.as_deref(), Some("g1"));

        let deleted = repo.delete(&created.id).await.unwrap().unwrap();
        assert_eq!(deleted.id, created.id);
        assert!(repo.get(&created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update_missing_returns_none() {
        let (_dir, repo, _conn) = repo().await;
        let result = repo
            .update(
                "missing",
                ProjectPatch {
                    name: Some("x".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn execution_workspace_policy_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        seed(&conn).await;
        let policy = serde_json::json!({
            "enabled": true,
            "sharedWorkspaceConcurrency": "serialize"
        });
        let created = repo
            .create(NewProject {
                company_id: "c1".to_owned(),
                goal_id: None,
                name: "Policy".to_owned(),
                description: None,
                status: "backlog".to_owned(),
                lead_agent_id: None,
                target_date: None,
                env: None,
                execution_workspace_policy: Some(policy.clone()),
            })
            .await
            .unwrap();
        assert_eq!(created.execution_workspace_policy, Some(policy.clone()));

        // Read-back through get/list.
        let fetched = repo.get(&created.id).await.unwrap().unwrap();
        assert_eq!(fetched.execution_workspace_policy, Some(policy.clone()));
        let listed = repo.list("c1").await.unwrap();
        assert_eq!(listed[0].execution_workspace_policy, Some(policy.clone()));

        // Update to a different value.
        let updated = repo
            .update(
                &created.id,
                ProjectPatch {
                    execution_workspace_policy: Some(Some(serde_json::json!({
                        "enabled": true,
                        "sharedWorkspaceConcurrency": "allow"
                    }))),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            updated.execution_workspace_policy,
            Some(serde_json::json!({ "enabled": true, "sharedWorkspaceConcurrency": "allow" }))
        );

        // Clear with explicit null.
        let cleared = repo
            .update(
                &created.id,
                ProjectPatch {
                    execution_workspace_policy: Some(None),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cleared.execution_workspace_policy, None);

        // Omitting the field leaves the stored value untouched (still NULL).
        let untouched = repo
            .update(
                &created.id,
                ProjectPatch {
                    name: Some("Untouched".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(untouched.execution_workspace_policy, None);

        // Raw column round-trip: stored as TEXT JSON.
        let mut rows = conn
            .query(
                "SELECT execution_workspace_policy FROM projects WHERE id = ?1",
                libsql::params![created.id],
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert!(super::helpers::row_text(&row, 0).unwrap().is_none());
    }
}
