//! Workspace repository: project workspaces, execution workspaces, runtime
//! services, and workspace operations (SPEC §7.16 addenda).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `project_workspaces` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceRecord {
    /// Workspace id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: String,
    /// Name.
    pub name: String,
    /// Source type.
    pub source_type: String,
    /// Working directory.
    pub cwd: Option<String>,
    /// Repo URL.
    pub repo_url: Option<String>,
    /// Repo ref.
    pub repo_ref: Option<String>,
    /// Default ref.
    pub default_ref: Option<String>,
    /// Visibility.
    pub visibility: String,
    /// Primary flag.
    pub is_primary: bool,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A row of the `execution_workspaces` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionWorkspaceRecord {
    /// Workspace id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: String,
    /// Project workspace id.
    pub project_workspace_id: Option<String>,
    /// Source issue id.
    pub source_issue_id: Option<String>,
    /// Mode.
    pub mode: String,
    /// Strategy type.
    pub strategy_type: String,
    /// Name.
    pub name: String,
    /// Status.
    pub status: String,
    /// Working directory.
    pub cwd: Option<String>,
    /// Repo URL.
    pub repo_url: Option<String>,
    /// Provider type.
    pub provider_type: String,
    /// Whether the repo has been materialized server-side.
    pub materialized: bool,
    /// ISO 8601 materialization time.
    pub materialized_at: Option<String>,
    /// Last materialization error.
    pub materialize_error: Option<String>,
    /// Company secret name used for credential injection.
    pub credential_secret_name: Option<String>,
    /// ISO 8601 cleanup eligibility.
    pub cleanup_eligible_at: Option<String>,
    /// Cleanup reason.
    pub cleanup_reason: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A row of the `workspace_runtime_services` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeServiceRecord {
    /// Service id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: Option<String>,
    /// Execution workspace id.
    pub execution_workspace_id: Option<String>,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Scope type.
    pub scope_type: String,
    /// Scope id.
    pub scope_id: Option<String>,
    /// Service name.
    pub service_name: String,
    /// Status.
    pub status: String,
    /// Lifecycle.
    pub lifecycle: String,
    /// Command.
    pub command: Option<String>,
    /// Port.
    pub port: Option<i64>,
    /// URL.
    pub url: Option<String>,
    /// Provider.
    pub provider: String,
    /// ISO 8601 last used time.
    pub last_used_at: Option<String>,
    /// ISO 8601 started time.
    pub started_at: Option<String>,
    /// ISO 8601 stopped time.
    pub stopped_at: Option<String>,
    /// Stop policy JSON.
    pub stop_policy: Option<serde_json::Value>,
    /// Health status.
    pub health_status: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// A row of the `workspace_operations` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceOperationRecord {
    /// Operation id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Execution workspace id.
    pub execution_workspace_id: Option<String>,
    /// Heartbeat run id.
    pub heartbeat_run_id: Option<String>,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Phase.
    pub phase: String,
    /// Command.
    pub command: Option<String>,
    /// Status.
    pub status: String,
    /// Exit code.
    pub exit_code: Option<i64>,
    /// Log ref.
    pub log_ref: Option<String>,
    /// Log bytes.
    pub log_bytes: Option<i64>,
    /// Log sha256.
    pub log_sha256: Option<String>,
    /// Whether logs are compressed.
    pub log_compressed: bool,
    /// Stdout excerpt.
    pub stdout_excerpt: Option<String>,
    /// Stderr excerpt.
    pub stderr_excerpt: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 started time.
    pub started_at: Option<String>,
    /// ISO 8601 finished time.
    pub finished_at: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating a project workspace.
#[derive(Debug, Clone)]
pub struct NewProjectWorkspace {
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: String,
    /// Name.
    pub name: String,
    /// Working directory.
    pub cwd: Option<String>,
    /// Repo URL.
    pub repo_url: Option<String>,
    /// Primary flag.
    pub is_primary: bool,
}

/// Input for creating an execution workspace.
#[derive(Debug, Clone)]
pub struct NewExecutionWorkspace {
    /// Owning company id.
    pub company_id: String,
    /// Project id.
    pub project_id: String,
    /// Project workspace id.
    pub project_workspace_id: Option<String>,
    /// Source issue id.
    pub source_issue_id: Option<String>,
    /// Mode.
    pub mode: String,
    /// Strategy type.
    pub strategy_type: String,
    /// Name.
    pub name: String,
    /// Working directory.
    pub cwd: Option<String>,
    /// Repo URL.
    pub repo_url: Option<String>,
}

/// Input for creating a runtime service.
#[derive(Debug, Clone)]
pub struct NewRuntimeService {
    /// Owning company id.
    pub company_id: String,
    /// Execution workspace id.
    pub execution_workspace_id: Option<String>,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Scope type.
    pub scope_type: String,
    /// Scope id.
    pub scope_id: Option<String>,
    /// Service name.
    pub service_name: String,
    /// Lifecycle.
    pub lifecycle: String,
    /// Command.
    pub command: Option<String>,
    /// Port.
    pub port: Option<i64>,
    /// URL.
    pub url: Option<String>,
    /// Provider.
    pub provider: String,
}

/// Input for recording a workspace operation.
#[derive(Debug, Clone)]
pub struct NewWorkspaceOperation {
    /// Owning company id.
    pub company_id: String,
    /// Execution workspace id.
    pub execution_workspace_id: Option<String>,
    /// Heartbeat run id.
    pub heartbeat_run_id: Option<String>,
    /// Issue id.
    pub issue_id: Option<String>,
    /// Phase.
    pub phase: String,
    /// Command.
    pub command: Option<String>,
    /// Log ref.
    pub log_ref: Option<String>,
}

/// Workspace repository errors.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// A referenced parent (project/workspace/issue/run) does not exist in
    /// this company.
    #[error("referenced record not found: {0}")]
    ReferenceNotFound(&'static str),
}

/// Workspace persistence contract.
#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    /// Creates a project workspace.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on invalid references.
    async fn create_project_workspace(
        &self,
        input: NewProjectWorkspace,
    ) -> Result<ProjectWorkspaceRecord, WorkspaceError>;

    /// Lists project workspaces for a company (optionally a project).
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on database failure.
    async fn list_project_workspaces(
        &self,
        company_id: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<ProjectWorkspaceRecord>, WorkspaceError>;

    /// Creates an execution workspace.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on invalid references.
    async fn create_execution_workspace(
        &self,
        input: NewExecutionWorkspace,
    ) -> Result<ExecutionWorkspaceRecord, WorkspaceError>;

    /// Lists execution workspaces for a company (optionally a project).
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on database failure.
    async fn list_execution_workspaces(
        &self,
        company_id: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<ExecutionWorkspaceRecord>, WorkspaceError>;

    /// Gets one execution workspace (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on database failure.
    async fn get_execution_workspace(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ExecutionWorkspaceRecord>, WorkspaceError>;

    /// Updates materialization state for an execution workspace.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on database failure.
    async fn set_materialization(
        &self,
        company_id: &str,
        id: &str,
        materialized: bool,
        materialize_error: Option<String>,
        credential_secret_name: Option<String>,
    ) -> Result<Option<ExecutionWorkspaceRecord>, WorkspaceError>;

    /// Registers a runtime service.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on invalid references.
    async fn create_runtime_service(
        &self,
        input: NewRuntimeService,
    ) -> Result<RuntimeServiceRecord, WorkspaceError>;

    /// Lists runtime services for a company.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on database failure.
    async fn list_runtime_services(
        &self,
        company_id: &str,
    ) -> Result<Vec<RuntimeServiceRecord>, WorkspaceError>;

    /// Records a workspace operation.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on invalid references.
    async fn create_operation(
        &self,
        input: NewWorkspaceOperation,
    ) -> Result<WorkspaceOperationRecord, WorkspaceError>;

    /// Lists operations for a company.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] on database failure.
    async fn list_operations(
        &self,
        company_id: &str,
    ) -> Result<Vec<WorkspaceOperationRecord>, WorkspaceError>;
}

/// Turso/libSQL implementation of [`WorkspaceRepository`].
#[derive(Debug)]
pub struct TursoWorkspaceRepository {
    db: Database,
}

impl TursoWorkspaceRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_opt_i64(row: &libsql::Row, idx: i32) -> Result<Option<i64>, libsql::Error> {
    let value = row.get_value(idx)?;
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(*value.as_integer().expect("INTEGER column")))
    }
}

const EXEC_COLUMNS: &str = "id, company_id, project_id, project_workspace_id, source_issue_id,
    mode, strategy_type, name, status, cwd, repo_url, provider_type, materialized,
    materialized_at, materialize_error, credential_secret_name, cleanup_eligible_at,
    cleanup_reason, metadata, created_at";

fn row_to_execution(row: &libsql::Row) -> Result<ExecutionWorkspaceRecord, libsql::Error> {
    Ok(ExecutionWorkspaceRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        project_id: helpers::row_text(row, 2)?.expect("project_id"),
        project_workspace_id: helpers::row_text(row, 3)?,
        source_issue_id: helpers::row_text(row, 4)?,
        mode: helpers::row_text(row, 5)?.expect("mode"),
        strategy_type: helpers::row_text(row, 6)?.expect("strategy_type"),
        name: helpers::row_text(row, 7)?.expect("name"),
        status: helpers::row_text(row, 8)?.expect("status"),
        cwd: helpers::row_text(row, 9)?,
        repo_url: helpers::row_text(row, 10)?,
        provider_type: helpers::row_text(row, 11)?.expect("provider_type"),
        materialized: helpers::row_i64(row, 12)? != 0,
        materialized_at: helpers::row_text(row, 13)?,
        materialize_error: helpers::row_text(row, 14)?,
        credential_secret_name: helpers::row_text(row, 15)?,
        cleanup_eligible_at: helpers::row_text(row, 16)?,
        cleanup_reason: helpers::row_text(row, 17)?,
        metadata: helpers::row_text(row, 18)?.and_then(|raw| serde_json::from_str(&raw).ok()),
        created_at: helpers::row_text(row, 19)?.expect("created_at"),
    })
}

#[async_trait]
impl WorkspaceRepository for TursoWorkspaceRepository {
    async fn create_project_workspace(
        &self,
        input: NewProjectWorkspace,
    ) -> Result<ProjectWorkspaceRecord, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "projects", &input.project_id, &input.company_id)
            .await?
        {
            return Err(WorkspaceError::ReferenceNotFound("project"));
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO project_workspaces (id, company_id, project_id, name, source_type,
                                             cwd, repo_url, is_primary, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'local_path', ?5, ?6, ?7,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.project_id,
                input.name,
                input.cwd,
                input.repo_url,
                i64::from(input.is_primary)
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, project_id, name, source_type, cwd, repo_url,
                        repo_ref, default_ref, visibility, is_primary, created_at
                 FROM project_workspaces WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("workspace was just inserted");
        Ok(ProjectWorkspaceRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            project_id: helpers::row_text(&row, 2)?.expect("project_id"),
            name: helpers::row_text(&row, 3)?.expect("name"),
            source_type: helpers::row_text(&row, 4)?.expect("source_type"),
            cwd: helpers::row_text(&row, 5)?,
            repo_url: helpers::row_text(&row, 6)?,
            repo_ref: helpers::row_text(&row, 7)?,
            default_ref: helpers::row_text(&row, 8)?,
            visibility: helpers::row_text(&row, 9)?.expect("visibility"),
            is_primary: helpers::row_i64(&row, 10)? != 0,
            created_at: helpers::row_text(&row, 11)?.expect("created_at"),
        })
    }

    async fn list_project_workspaces(
        &self,
        company_id: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<ProjectWorkspaceRecord>, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = match project_id {
            Some(_) => "SELECT id, company_id, project_id, name, source_type, cwd, repo_url,
                        repo_ref, default_ref, visibility, is_primary, created_at
                 FROM project_workspaces WHERE company_id = ?1 AND project_id = ?2 ORDER BY created_at",
            None => "SELECT id, company_id, project_id, name, source_type, cwd, repo_url,
                        repo_ref, default_ref, visibility, is_primary, created_at
                 FROM project_workspaces WHERE company_id = ?1 ORDER BY created_at",
        };
        let params: Vec<libsql::Value> = match project_id {
            Some(project_id) => vec![company_id.into(), project_id.into()],
            None => vec![company_id.into()],
        };
        let mut rows = conn.query(sql, params).await?;
        let mut workspaces = Vec::new();
        while let Some(row) = rows.next().await? {
            workspaces.push(ProjectWorkspaceRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                project_id: helpers::row_text(&row, 2)?.expect("project_id"),
                name: helpers::row_text(&row, 3)?.expect("name"),
                source_type: helpers::row_text(&row, 4)?.expect("source_type"),
                cwd: helpers::row_text(&row, 5)?,
                repo_url: helpers::row_text(&row, 6)?,
                repo_ref: helpers::row_text(&row, 7)?,
                default_ref: helpers::row_text(&row, 8)?,
                visibility: helpers::row_text(&row, 9)?.expect("visibility"),
                is_primary: helpers::row_i64(&row, 10)? != 0,
                created_at: helpers::row_text(&row, 11)?.expect("created_at"),
            });
        }
        Ok(workspaces)
    }

    async fn create_execution_workspace(
        &self,
        input: NewExecutionWorkspace,
    ) -> Result<ExecutionWorkspaceRecord, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "projects", &input.project_id, &input.company_id)
            .await?
        {
            return Err(WorkspaceError::ReferenceNotFound("project"));
        }
        if let Some(pw_id) = &input.project_workspace_id
            && !helpers::row_belongs_to_company(
                &conn,
                "project_workspaces",
                pw_id,
                &input.company_id,
            )
            .await?
        {
            return Err(WorkspaceError::ReferenceNotFound("project_workspace"));
        }
        if let Some(issue_id) = &input.source_issue_id
            && !helpers::row_belongs_to_company(&conn, "issues", issue_id, &input.company_id)
                .await?
        {
            return Err(WorkspaceError::ReferenceNotFound("issue"));
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO execution_workspaces (id, company_id, project_id, project_workspace_id,
                                               source_issue_id, mode, strategy_type, name, status,
                                               cwd, repo_url, provider_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, ?10, 'local_fs',
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.project_id,
                input.project_workspace_id,
                input.source_issue_id,
                input.mode,
                input.strategy_type,
                input.name,
                input.cwd,
                input.repo_url
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {EXEC_COLUMNS} FROM execution_workspaces WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("workspace was just inserted");
        Ok(row_to_execution(&row)?)
    }

    async fn list_execution_workspaces(
        &self,
        company_id: &str,
        project_id: Option<&str>,
    ) -> Result<Vec<ExecutionWorkspaceRecord>, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = match project_id {
            Some(_) => format!(
                "SELECT {EXEC_COLUMNS}
                 FROM execution_workspaces WHERE company_id = ?1 AND project_id = ?2 ORDER BY created_at"
            ),
            None => format!(
                "SELECT {EXEC_COLUMNS}
                 FROM execution_workspaces WHERE company_id = ?1 ORDER BY created_at"
            ),
        };
        let params: Vec<libsql::Value> = match project_id {
            Some(project_id) => vec![company_id.into(), project_id.into()],
            None => vec![company_id.into()],
        };
        let mut rows = conn.query(&sql, params).await?;
        let mut workspaces = Vec::new();
        while let Some(row) = rows.next().await? {
            workspaces.push(row_to_execution(&row)?);
        }
        Ok(workspaces)
    }

    async fn get_execution_workspace(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ExecutionWorkspaceRecord>, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EXEC_COLUMNS} FROM execution_workspaces WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_execution(&row)?)),
            None => Ok(None),
        }
    }

    async fn set_materialization(
        &self,
        company_id: &str,
        id: &str,
        materialized: bool,
        materialize_error: Option<String>,
        credential_secret_name: Option<String>,
    ) -> Result<Option<ExecutionWorkspaceRecord>, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE execution_workspaces
                 SET materialized = ?1,
                     materialized_at = CASE WHEN ?1 = 1
                                            THEN strftime('%Y-%m-%dT%H:%M:%fZ','now')
                                            ELSE materialized_at END,
                     materialize_error = ?2,
                     credential_secret_name = COALESCE(?3, credential_secret_name),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE company_id = ?4 AND id = ?5",
                libsql::params![
                    i64::from(materialized),
                    materialize_error,
                    credential_secret_name,
                    company_id,
                    id
                ],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EXEC_COLUMNS} FROM execution_workspaces WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let row = rows.next().await?.expect("workspace exists");
        Ok(Some(row_to_execution(&row)?))
    }

    async fn create_runtime_service(
        &self,
        input: NewRuntimeService,
    ) -> Result<RuntimeServiceRecord, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        if let Some(ws_id) = &input.execution_workspace_id
            && !helpers::row_belongs_to_company(
                &conn,
                "execution_workspaces",
                ws_id,
                &input.company_id,
            )
            .await?
        {
            return Err(WorkspaceError::ReferenceNotFound("execution_workspace"));
        }
        if let Some(issue_id) = &input.issue_id
            && !helpers::row_belongs_to_company(&conn, "issues", issue_id, &input.company_id)
                .await?
        {
            return Err(WorkspaceError::ReferenceNotFound("issue"));
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO workspace_runtime_services (id, company_id, execution_workspace_id,
                                                     issue_id, scope_type, scope_id, service_name,
                                                     status, lifecycle, command, port, url,
                                                     provider, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'running', ?8, ?9, ?10, ?11, ?12,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.execution_workspace_id,
                input.issue_id,
                input.scope_type,
                input.scope_id,
                input.service_name,
                input.lifecycle,
                input.command,
                input.port,
                input.url,
                input.provider
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, project_id, execution_workspace_id, issue_id, scope_type,
                        scope_id, service_name, status, lifecycle, command, port, url, provider,
                        last_used_at, started_at, stopped_at, stop_policy, health_status,
                        created_at
                 FROM workspace_runtime_services WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("service was just inserted");
        Ok(RuntimeServiceRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            project_id: helpers::row_text(&row, 2)?,
            execution_workspace_id: helpers::row_text(&row, 3)?,
            issue_id: helpers::row_text(&row, 4)?,
            scope_type: helpers::row_text(&row, 5)?.expect("scope_type"),
            scope_id: helpers::row_text(&row, 6)?,
            service_name: helpers::row_text(&row, 7)?.expect("service_name"),
            status: helpers::row_text(&row, 8)?.expect("status"),
            lifecycle: helpers::row_text(&row, 9)?.expect("lifecycle"),
            command: helpers::row_text(&row, 10)?,
            port: row_opt_i64(&row, 11)?,
            url: helpers::row_text(&row, 12)?,
            provider: helpers::row_text(&row, 13)?.expect("provider"),
            last_used_at: helpers::row_text(&row, 14)?,
            started_at: helpers::row_text(&row, 15)?,
            stopped_at: helpers::row_text(&row, 16)?,
            stop_policy: helpers::row_text(&row, 17)?
                .and_then(|raw| serde_json::from_str(&raw).ok()),
            health_status: helpers::row_text(&row, 18)?.unwrap_or_else(|| "unknown".to_owned()),
            created_at: helpers::row_text(&row, 19)?.expect("created_at"),
        })
    }

    async fn list_runtime_services(
        &self,
        company_id: &str,
    ) -> Result<Vec<RuntimeServiceRecord>, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, project_id, execution_workspace_id, issue_id, scope_type,
                        scope_id, service_name, status, lifecycle, command, port, url, provider,
                        last_used_at, started_at, stopped_at, stop_policy, health_status,
                        created_at
                 FROM workspace_runtime_services WHERE company_id = ?1 ORDER BY created_at",
                libsql::params![company_id],
            )
            .await?;
        let mut services = Vec::new();
        while let Some(row) = rows.next().await? {
            services.push(RuntimeServiceRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                project_id: helpers::row_text(&row, 2)?,
                execution_workspace_id: helpers::row_text(&row, 3)?,
                issue_id: helpers::row_text(&row, 4)?,
                scope_type: helpers::row_text(&row, 5)?.expect("scope_type"),
                scope_id: helpers::row_text(&row, 6)?,
                service_name: helpers::row_text(&row, 7)?.expect("service_name"),
                status: helpers::row_text(&row, 8)?.expect("status"),
                lifecycle: helpers::row_text(&row, 9)?.expect("lifecycle"),
                command: helpers::row_text(&row, 10)?,
                port: row_opt_i64(&row, 11)?,
                url: helpers::row_text(&row, 12)?,
                provider: helpers::row_text(&row, 13)?.expect("provider"),
                last_used_at: helpers::row_text(&row, 14)?,
                started_at: helpers::row_text(&row, 15)?,
                stopped_at: helpers::row_text(&row, 16)?,
                stop_policy: helpers::row_text(&row, 17)?
                    .and_then(|raw| serde_json::from_str(&raw).ok()),
                health_status: helpers::row_text(&row, 18)?.unwrap_or_else(|| "unknown".to_owned()),
                created_at: helpers::row_text(&row, 19)?.expect("created_at"),
            });
        }
        Ok(services)
    }

    async fn create_operation(
        &self,
        input: NewWorkspaceOperation,
    ) -> Result<WorkspaceOperationRecord, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        if let Some(ws_id) = &input.execution_workspace_id
            && !helpers::row_belongs_to_company(
                &conn,
                "execution_workspaces",
                ws_id,
                &input.company_id,
            )
            .await?
        {
            return Err(WorkspaceError::ReferenceNotFound("execution_workspace"));
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO workspace_operations (id, company_id, execution_workspace_id,
                                               heartbeat_run_id, issue_id, phase, command,
                                               status, log_ref, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'running', ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.execution_workspace_id,
                input.heartbeat_run_id,
                input.issue_id,
                input.phase,
                input.command,
                input.log_ref
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, execution_workspace_id, heartbeat_run_id, issue_id, phase,
                        command, status, exit_code, log_ref, log_bytes, log_sha256,
                        log_compressed, stdout_excerpt, stderr_excerpt, metadata, started_at,
                        finished_at, created_at
                 FROM workspace_operations WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("operation was just inserted");
        Ok(WorkspaceOperationRecord {
            id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            execution_workspace_id: helpers::row_text(&row, 2)?,
            heartbeat_run_id: helpers::row_text(&row, 3)?,
            issue_id: helpers::row_text(&row, 4)?,
            phase: helpers::row_text(&row, 5)?.expect("phase"),
            command: helpers::row_text(&row, 6)?,
            status: helpers::row_text(&row, 7)?.expect("status"),
            exit_code: row_opt_i64(&row, 8)?,
            log_ref: helpers::row_text(&row, 9)?,
            log_bytes: row_opt_i64(&row, 10)?,
            log_sha256: helpers::row_text(&row, 11)?,
            log_compressed: helpers::row_i64(&row, 12)? != 0,
            stdout_excerpt: helpers::row_text(&row, 13)?,
            stderr_excerpt: helpers::row_text(&row, 14)?,
            metadata: helpers::row_text(&row, 15)?.and_then(|raw| serde_json::from_str(&raw).ok()),
            started_at: helpers::row_text(&row, 16)?,
            finished_at: helpers::row_text(&row, 17)?,
            created_at: helpers::row_text(&row, 18)?.expect("created_at"),
        })
    }

    async fn list_operations(
        &self,
        company_id: &str,
    ) -> Result<Vec<WorkspaceOperationRecord>, WorkspaceError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, execution_workspace_id, heartbeat_run_id, issue_id, phase,
                        command, status, exit_code, log_ref, log_bytes, log_sha256,
                        log_compressed, stdout_excerpt, stderr_excerpt, metadata, started_at,
                        finished_at, created_at
                 FROM workspace_operations WHERE company_id = ?1 ORDER BY created_at DESC",
                libsql::params![company_id],
            )
            .await?;
        let mut operations = Vec::new();
        while let Some(row) = rows.next().await? {
            operations.push(WorkspaceOperationRecord {
                id: helpers::row_text(&row, 0)?.expect("id"),
                company_id: helpers::row_text(&row, 1)?.expect("company_id"),
                execution_workspace_id: helpers::row_text(&row, 2)?,
                heartbeat_run_id: helpers::row_text(&row, 3)?,
                issue_id: helpers::row_text(&row, 4)?,
                phase: helpers::row_text(&row, 5)?.expect("phase"),
                command: helpers::row_text(&row, 6)?,
                status: helpers::row_text(&row, 7)?.expect("status"),
                exit_code: row_opt_i64(&row, 8)?,
                log_ref: helpers::row_text(&row, 9)?,
                log_bytes: row_opt_i64(&row, 10)?,
                log_sha256: helpers::row_text(&row, 11)?,
                log_compressed: helpers::row_i64(&row, 12)? != 0,
                stdout_excerpt: helpers::row_text(&row, 13)?,
                stderr_excerpt: helpers::row_text(&row, 14)?,
                metadata: helpers::row_text(&row, 15)?
                    .and_then(|raw| serde_json::from_str(&raw).ok()),
                started_at: helpers::row_text(&row, 16)?,
                finished_at: helpers::row_text(&row, 17)?,
                created_at: helpers::row_text(&row, 18)?.expect("created_at"),
            });
        }
        Ok(operations)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoWorkspaceRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
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
        conn.execute(
            "INSERT INTO projects (id, company_id, name)
             VALUES ('p1', 'c1', 'Ship'), ('p2', 'c2', 'Other')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'T', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoWorkspaceRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn project_and_execution_workspaces() {
        let (_dir, repo) = repo().await;

        let pw = repo
            .create_project_workspace(NewProjectWorkspace {
                company_id: "c1".to_owned(),
                project_id: "p1".to_owned(),
                name: "main".to_owned(),
                cwd: Some("/ws/main".to_owned()),
                repo_url: None,
                is_primary: true,
            })
            .await
            .unwrap();
        assert_eq!(pw.name, "main");
        assert!(pw.is_primary);

        // Cross-company project rejected.
        let error = repo
            .create_project_workspace(NewProjectWorkspace {
                company_id: "c1".to_owned(),
                project_id: "p2".to_owned(),
                name: "x".to_owned(),
                cwd: None,
                repo_url: None,
                is_primary: false,
            })
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            WorkspaceError::ReferenceNotFound("project")
        ));

        let ew = repo
            .create_execution_workspace(NewExecutionWorkspace {
                company_id: "c1".to_owned(),
                project_id: "p1".to_owned(),
                project_workspace_id: Some(pw.id.clone()),
                source_issue_id: Some("i1".to_owned()),
                mode: "ephemeral".to_owned(),
                strategy_type: "checkout".to_owned(),
                name: "run-ws".to_owned(),
                cwd: Some("/ws/run".to_owned()),
                repo_url: None,
            })
            .await
            .unwrap();
        assert_eq!(ew.status, "active");
        assert_eq!(ew.mode, "ephemeral");

        let list = repo.list_execution_workspaces("c1", None).await.unwrap();
        assert_eq!(list.len(), 1);
        let list_pw = repo
            .list_project_workspaces("c1", Some("p1"))
            .await
            .unwrap();
        assert_eq!(list_pw.len(), 1);
    }

    #[tokio::test]
    async fn runtime_services_and_operations() {
        let (_dir, repo) = repo().await;
        let pw = repo
            .create_project_workspace(NewProjectWorkspace {
                company_id: "c1".to_owned(),
                project_id: "p1".to_owned(),
                name: "main".to_owned(),
                cwd: None,
                repo_url: None,
                is_primary: true,
            })
            .await
            .unwrap();
        let ew = repo
            .create_execution_workspace(NewExecutionWorkspace {
                company_id: "c1".to_owned(),
                project_id: "p1".to_owned(),
                project_workspace_id: Some(pw.id),
                source_issue_id: None,
                mode: "ephemeral".to_owned(),
                strategy_type: "checkout".to_owned(),
                name: "ws".to_owned(),
                cwd: None,
                repo_url: None,
            })
            .await
            .unwrap();

        let service = repo
            .create_runtime_service(NewRuntimeService {
                company_id: "c1".to_owned(),
                execution_workspace_id: Some(ew.id.clone()),
                issue_id: Some("i1".to_owned()),
                scope_type: "execution_workspace".to_owned(),
                scope_id: Some(ew.id.clone()),
                service_name: "vite".to_owned(),
                lifecycle: "ephemeral".to_owned(),
                command: Some("npm run dev".to_owned()),
                port: Some(5173),
                url: Some("http://localhost:5173".to_owned()),
                provider: "local".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(service.status, "running");
        assert_eq!(service.port, Some(5173));

        let operation = repo
            .create_operation(NewWorkspaceOperation {
                company_id: "c1".to_owned(),
                execution_workspace_id: Some(ew.id),
                heartbeat_run_id: None,
                issue_id: Some("i1".to_owned()),
                phase: "setup".to_owned(),
                command: Some("git clone".to_owned()),
                log_ref: Some("s3://logs/op1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(operation.status, "running");
        assert_eq!(operation.log_ref.as_deref(), Some("s3://logs/op1"));

        let services = repo.list_runtime_services("c1").await.unwrap();
        assert_eq!(services.len(), 1);
        let ops = repo.list_operations("c1").await.unwrap();
        assert_eq!(ops.len(), 1);
    }
}
