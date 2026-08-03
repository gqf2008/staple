//! Goals repository: trait plus Turso/libSQL implementation.
//!
//! Enforces the service-level hierarchy rules: parents and owner agents must
//! exist and belong to the same company as the goal.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `goals` table.
#[derive(Debug, Clone)]
pub struct GoalRecord {
    /// Goal id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// `company | team | agent | task`.
    pub level: String,
    /// Parent goal id, when nested.
    pub parent_id: Option<String>,
    /// Owning agent id, when agent-owned.
    pub owner_agent_id: Option<String>,
    /// `planned | active | achieved | cancelled`.
    pub status: String,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating a goal.
#[derive(Debug, Clone)]
pub struct NewGoal {
    /// Owning company id.
    pub company_id: String,
    /// Title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// `company | team | agent | task`.
    pub level: String,
    /// Parent goal id.
    pub parent_id: Option<String>,
    /// Owning agent id.
    pub owner_agent_id: Option<String>,
    /// `planned | active | achieved | cancelled`.
    pub status: String,
}

/// Partial goal update. `Option<Option<T>>` distinguishes "leave unchanged"
/// from "set to NULL".
#[derive(Debug, Default)]
pub struct GoalPatch {
    /// New title.
    pub title: Option<String>,
    /// New description.
    pub description: Option<Option<String>>,
    /// New level.
    pub level: Option<String>,
    /// New parent goal id.
    pub parent_id: Option<Option<String>>,
    /// New owner agent id.
    pub owner_agent_id: Option<Option<String>>,
    /// New status.
    pub status: Option<String>,
}

/// Goals repository errors.
#[derive(Debug, Error)]
pub enum GoalError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The owning company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The parent goal does not exist.
    #[error("parent goal not found")]
    ParentNotFound,
    /// The parent goal belongs to a different company.
    #[error("parent goal belongs to a different company")]
    ParentInDifferentCompany,
    /// The owner agent does not exist.
    #[error("owner agent not found")]
    OwnerAgentNotFound,
    /// The owner agent belongs to a different company.
    #[error("owner agent belongs to a different company")]
    OwnerAgentInDifferentCompany,
    /// The row is referenced by other records and cannot be deleted.
    #[error("resource is referenced by other records")]
    InUse,
}

/// Goal persistence contract.
#[async_trait]
pub trait GoalRepository: Send + Sync {
    /// Creates a goal, validating company, parent, and owner references.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError`] when a referenced row is missing or belongs to a
    /// different company.
    async fn create(&self, input: NewGoal) -> Result<GoalRecord, GoalError>;

    /// Lists all goals of one company.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError`] on database failure.
    async fn list(&self, company_id: &str) -> Result<Vec<GoalRecord>, GoalError>;

    /// Fetches one goal by id.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<GoalRecord>, GoalError>;

    /// Applies a partial update.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError`] on database failure or invalid references.
    async fn update(&self, id: &str, patch: GoalPatch) -> Result<Option<GoalRecord>, GoalError>;

    /// Deletes a goal, returning the deleted row.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError`] on database failure.
    async fn delete(&self, id: &str) -> Result<Option<GoalRecord>, GoalError>;
}

/// Turso/libSQL implementation of [`GoalRepository`].
#[derive(Debug)]
pub struct TursoGoalRepository {
    db: Database,
}

impl TursoGoalRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const GOAL_COLUMNS: &str = "id, company_id, title, description, level, parent_id,
    owner_agent_id, status, created_at, updated_at";

fn row_to_goal(row: &libsql::Row) -> Result<GoalRecord, libsql::Error> {
    Ok(GoalRecord {
        id: helpers::row_text(row, 0)?.expect("id is NOT NULL"),
        company_id: helpers::row_text(row, 1)?.expect("company_id is NOT NULL"),
        title: helpers::row_text(row, 2)?.expect("title is NOT NULL"),
        description: helpers::row_text(row, 3)?,
        level: helpers::row_text(row, 4)?.expect("level is NOT NULL"),
        parent_id: helpers::row_text(row, 5)?,
        owner_agent_id: helpers::row_text(row, 6)?,
        status: helpers::row_text(row, 7)?.expect("status is NOT NULL"),
        created_at: helpers::row_text(row, 8)?.expect("created_at is NOT NULL"),
        updated_at: helpers::row_text(row, 9)?.expect("updated_at is NOT NULL"),
    })
}

#[async_trait]
impl GoalRepository for TursoGoalRepository {
    async fn create(&self, input: NewGoal) -> Result<GoalRecord, GoalError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(GoalError::CompanyNotFound);
        }
        if let Some(parent_id) = &input.parent_id {
            let parent = helpers::find_row(&conn, "goals", parent_id).await?;
            if !parent {
                return Err(GoalError::ParentNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "goals", parent_id, &input.company_id)
                .await?
            {
                return Err(GoalError::ParentInDifferentCompany);
            }
        }
        if let Some(owner_agent_id) = &input.owner_agent_id {
            if !helpers::find_row(&conn, "agents", owner_agent_id).await? {
                return Err(GoalError::OwnerAgentNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "agents", owner_agent_id, &input.company_id)
                .await?
            {
                return Err(GoalError::OwnerAgentInDifferentCompany);
            }
        }

        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO goals (id, company_id, title, description, level, parent_id,
                                owner_agent_id, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.title,
                input.description,
                input.level,
                input.parent_id,
                input.owner_agent_id,
                input.status
            ],
        )
        .await?;
        Ok(self.get(&id).await?.expect("goal was just inserted"))
    }

    async fn list(&self, company_id: &str) -> Result<Vec<GoalRecord>, GoalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql =
            format!("SELECT {GOAL_COLUMNS} FROM goals WHERE company_id = ?1 ORDER BY created_at");
        let mut rows = conn.query(&sql, libsql::params![company_id]).await?;
        let mut goals = Vec::new();
        while let Some(row) = rows.next().await? {
            goals.push(row_to_goal(&row)?);
        }
        Ok(goals)
    }

    async fn get(&self, id: &str) -> Result<Option<GoalRecord>, GoalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let sql = format!("SELECT {GOAL_COLUMNS} FROM goals WHERE id = ?1");
        let mut rows = conn.query(&sql, libsql::params![id]).await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_goal(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, id: &str, patch: GoalPatch) -> Result<Option<GoalRecord>, GoalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let company_id = helpers::row_company(&conn, "goals", id).await?;
        let Some(company_id) = company_id else {
            return Ok(None);
        };

        if let Some(Some(parent_id)) = &patch.parent_id {
            if !helpers::find_row(&conn, "goals", parent_id).await? {
                return Err(GoalError::ParentNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "goals", parent_id, &company_id).await? {
                return Err(GoalError::ParentInDifferentCompany);
            }
        }
        if let Some(Some(owner_agent_id)) = &patch.owner_agent_id {
            if !helpers::find_row(&conn, "agents", owner_agent_id).await? {
                return Err(GoalError::OwnerAgentNotFound);
            }
            if !helpers::row_belongs_to_company(&conn, "agents", owner_agent_id, &company_id)
                .await?
            {
                return Err(GoalError::OwnerAgentInDifferentCompany);
            }
        }

        let (sets, values) = helpers::build_update(&[
            ("title", patch.title.map(|value| Some(value.into()))),
            (
                "description",
                patch.description.map(|value| value.map(Into::into)),
            ),
            ("level", patch.level.map(|value| Some(value.into()))),
            (
                "parent_id",
                patch.parent_id.map(|value| value.map(Into::into)),
            ),
            (
                "owner_agent_id",
                patch.owner_agent_id.map(|value| value.map(Into::into)),
            ),
            ("status", patch.status.map(|value| Some(value.into()))),
        ]);
        if sets.is_empty() {
            return self.get(id).await;
        }
        let updated = helpers::execute_update(&conn, "goals", id, sets, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        Ok(self.get(id).await?)
    }

    async fn delete(&self, id: &str) -> Result<Option<GoalRecord>, GoalError> {
        let conn = crate::connection::connect(&self.db).await?;
        let goal = self.get(id).await?;
        let Some(goal) = goal else {
            return Ok(None);
        };
        match conn
            .execute("DELETE FROM goals WHERE id = ?1", libsql::params![id])
            .await
        {
            Ok(_) => Ok(Some(goal)),
            Err(error) if error.to_string().contains("FOREIGN KEY constraint failed") => {
                Err(GoalError::InUse)
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

    async fn repo() -> (TempDir, TursoGoalRepository, Connection) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let conn = crate::connect(&db).await.unwrap();
        let repo = TursoGoalRepository::new(db);
        (dir, repo, conn)
    }

    async fn agent(conn: &Connection, id: &str, company_id: &str) {
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES (?1, ?2, 'a', 'engineer', 'codex_local')",
            (id, company_id),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn create_validates_references() {
        let (_dir, repo, conn) = repo().await;
        let company_id = "c1";
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        agent(&conn, "a1", company_id).await;

        // Parent in a different company is rejected.
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO goals (id, company_id, title, level)
             VALUES ('g2', 'c2', 'Other goal', 'company')",
            (),
        )
        .await
        .unwrap();
        let error = repo
            .create(NewGoal {
                company_id: "c1".to_owned(),
                title: "x".to_owned(),
                description: None,
                level: "team".to_owned(),
                parent_id: Some("g2".to_owned()),
                owner_agent_id: None,
                status: "planned".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, GoalError::ParentInDifferentCompany));

        // Owner agent in a different company is rejected.
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a2', 'c2', 'other', 'engineer', 'codex_local')",
            (),
        )
        .await
        .unwrap();
        let error = repo
            .create(NewGoal {
                company_id: "c1".to_owned(),
                title: "x".to_owned(),
                description: None,
                level: "team".to_owned(),
                parent_id: None,
                owner_agent_id: Some("a2".to_owned()),
                status: "planned".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, GoalError::OwnerAgentInDifferentCompany));

        // Valid creation succeeds.
        let goal = repo
            .create(NewGoal {
                company_id: "c1".to_owned(),
                title: "Growth".to_owned(),
                description: Some("d".to_owned()),
                level: "team".to_owned(),
                parent_id: None,
                owner_agent_id: Some("a1".to_owned()),
                status: "active".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(goal.title, "Growth");
        assert_eq!(goal.level, "team");
        assert_eq!(goal.status, "active");
        assert_eq!(goal.owner_agent_id.as_deref(), Some("a1"));
    }

    #[tokio::test]
    async fn create_requires_company() {
        let (_dir, repo, _conn) = repo().await;
        let error = repo
            .create(NewGoal {
                company_id: "missing".to_owned(),
                title: "x".to_owned(),
                description: None,
                level: "company".to_owned(),
                parent_id: None,
                owner_agent_id: None,
                status: "planned".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, GoalError::CompanyNotFound));
    }

    #[tokio::test]
    async fn list_get_update_delete_roundtrip() {
        let (_dir, repo, conn) = repo().await;
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024)",
            (),
        )
        .await
        .unwrap();
        agent(&conn, "a1", "c1").await;
        let created = repo
            .create(NewGoal {
                company_id: "c1".to_owned(),
                title: "G".to_owned(),
                description: None,
                level: "company".to_owned(),
                parent_id: None,
                owner_agent_id: None,
                status: "planned".to_owned(),
            })
            .await
            .unwrap();

        let list = repo.list("c1").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);

        let fetched = repo.get(&created.id).await.unwrap().unwrap();
        assert_eq!(fetched.title, "G");

        let updated = repo
            .update(
                &created.id,
                GoalPatch {
                    title: Some("G2".to_owned()),
                    status: Some("achieved".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.title, "G2");
        assert_eq!(updated.status, "achieved");

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
                GoalPatch {
                    title: Some("x".to_owned()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
