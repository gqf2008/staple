//! Company skills repository.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;
use crate::skills::{
    AgentFacts, SkillEvaluation, SkillFacts, SkillRestrictionPolicy, evaluate_skill,
};

/// A row of the `company_skills` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRecord {
    /// Skill id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Restriction policy.
    pub restriction_policy: SkillRestrictionPolicy,
    /// Status.
    pub status: String,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for creating a skill.
#[derive(Debug, Clone)]
pub struct NewSkill {
    /// Owning company id.
    pub company_id: String,
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Restriction policy.
    pub restriction_policy: SkillRestrictionPolicy,
}

/// Skill repository errors.
#[derive(Debug, Error)]
pub enum SkillError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The skill name already exists.
    #[error("skill already exists")]
    AlreadyExists,
}

/// Skill persistence contract.
#[async_trait]
pub trait SkillRepository: Send + Sync {
    /// Creates a skill.
    ///
    /// # Errors
    ///
    /// Returns [`SkillError`] on invalid references or duplicates.
    async fn create(&self, input: NewSkill) -> Result<SkillRecord, SkillError>;

    /// Lists skills.
    ///
    /// # Errors
    ///
    /// Returns [`SkillError`] on database failure.
    async fn list(&self, company_id: &str) -> Result<Vec<SkillRecord>, SkillError>;

    /// Fetches one skill by name.
    ///
    /// # Errors
    ///
    /// Returns [`SkillError`] on database failure.
    async fn get(&self, company_id: &str, name: &str) -> Result<Option<SkillRecord>, SkillError>;

    /// Evaluates the skill policy for an agent (both must belong to the
    /// company). Returns `None` when the skill or agent does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`SkillError`] on database failure.
    async fn evaluate(
        &self,
        company_id: &str,
        agent_id: &str,
        skill_name: &str,
    ) -> Result<Option<SkillEvaluation>, SkillError>;
}

/// Turso/libSQL implementation of [`SkillRepository`].
#[derive(Debug)]
pub struct TursoSkillRepository {
    db: Database,
}

impl TursoSkillRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn parse_policy(raw: &str) -> SkillRestrictionPolicy {
    serde_json::from_str(raw).unwrap_or_default()
}

fn row_to_skill(row: &libsql::Row) -> Result<SkillRecord, libsql::Error> {
    Ok(SkillRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        name: helpers::row_text(row, 2)?.expect("name"),
        description: helpers::row_text(row, 3)?,
        restriction_policy: parse_policy(&helpers::row_text(row, 4)?.expect("restriction_policy")),
        status: helpers::row_text(row, 5)?.expect("status"),
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
    })
}

#[async_trait]
impl SkillRepository for TursoSkillRepository {
    async fn create(&self, input: NewSkill) -> Result<SkillRecord, SkillError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let policy_json =
            serde_json::to_string(&input.restriction_policy).unwrap_or_else(|_| "{}".to_owned());
        let result = conn
            .execute(
                "INSERT INTO company_skills (id, company_id, name, description,
                                             restriction_policy, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active',
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.name,
                    input.description,
                    policy_json
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, name, description, restriction_policy, status, created_at
                         FROM company_skills WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("skill was just inserted");
                Ok(row_to_skill(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(SkillError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list(&self, company_id: &str) -> Result<Vec<SkillRecord>, SkillError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, description, restriction_policy, status, created_at
                 FROM company_skills WHERE company_id = ?1 ORDER BY name",
                libsql::params![company_id],
            )
            .await?;
        let mut skills = Vec::new();
        while let Some(row) = rows.next().await? {
            skills.push(row_to_skill(&row)?);
        }
        Ok(skills)
    }

    async fn get(&self, company_id: &str, name: &str) -> Result<Option<SkillRecord>, SkillError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, description, restriction_policy, status, created_at
                 FROM company_skills WHERE company_id = ?1 AND name = ?2",
                libsql::params![company_id, name],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_skill(&row)?)),
            None => Ok(None),
        }
    }

    async fn evaluate(
        &self,
        company_id: &str,
        agent_id: &str,
        skill_name: &str,
    ) -> Result<Option<SkillEvaluation>, SkillError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(skill) = self.get(company_id, skill_name).await? else {
            return Ok(None);
        };
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, role, status FROM agents
                 WHERE id = ?1 AND company_id = ?2",
                libsql::params![agent_id, company_id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let agent = AgentFacts {
            agent_id: helpers::row_text(&row, 0)?.expect("id"),
            company_id: helpers::row_text(&row, 1)?.expect("company_id"),
            role: helpers::row_text(&row, 3)?.expect("role"),
            status: helpers::row_text(&row, 4)?.expect("status"),
        };
        Ok(Some(evaluate_skill(
            &agent,
            &SkillFacts {
                company_id: skill.company_id,
                name: skill.name,
                status: skill.status,
                policy: skill.restriction_policy,
            },
        )))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    #[tokio::test]
    async fn create_list_get_roundtrip() {
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
        let repo = TursoSkillRepository::new(db);

        let created = repo
            .create(NewSkill {
                company_id: "c1".to_owned(),
                name: "code_review".to_owned(),
                description: Some("review code".to_owned()),
                restriction_policy: SkillRestrictionPolicy {
                    allowed_roles: vec!["senior".to_owned()],
                    ..Default::default()
                },
            })
            .await
            .unwrap();
        assert_eq!(created.status, "active");
        assert_eq!(
            created.restriction_policy.allowed_roles,
            vec!["senior".to_owned()]
        );

        let error = repo
            .create(NewSkill {
                company_id: "c1".to_owned(),
                name: "code_review".to_owned(),
                description: None,
                restriction_policy: Default::default(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, SkillError::AlreadyExists));

        let list = repo.list("c1").await.unwrap();
        assert_eq!(list.len(), 1);
        let fetched = repo.get("c1", "code_review").await.unwrap().unwrap();
        assert_eq!(fetched.id, created.id);
    }
}
