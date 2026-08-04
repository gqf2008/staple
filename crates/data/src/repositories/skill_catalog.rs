//! Skill catalog repository: versions, policies, comments, stars, test
//! inputs, test run templates, and test runs (upstream company_skills.ts +
//! company_skill_policies.ts).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `company_skill_versions` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillVersionRecord {
    pub id: String,
    pub company_id: String,
    pub company_skill_id: String,
    pub revision_number: i64,
    pub label: Option<String>,
    pub release_id: Option<String>,
    pub release_name: Option<String>,
    pub released_at: Option<String>,
    pub file_inventory: serde_json::Value,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub created_at: String,
}

/// Input for publishing a skill version.
#[derive(Debug, Clone)]
pub struct NewSkillVersion {
    pub company_id: String,
    pub company_skill_id: String,
    pub label: Option<String>,
    pub release_id: Option<String>,
    pub release_name: Option<String>,
    pub released_at: Option<String>,
    pub file_inventory: serde_json::Value,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
}

/// A row of the `company_skill_policies` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPolicyRecord {
    pub company_id: String,
    pub schema_version: i64,
    pub revision: i64,
    pub default_effect: String,
    pub rules: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for setting a company skill policy (revision auto-increments).
#[derive(Debug, Clone)]
pub struct SetSkillPolicy {
    pub company_id: String,
    pub schema_version: i64,
    pub default_effect: String,
    pub rules: serde_json::Value,
}

/// A row of the `company_skill_comments` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCommentRecord {
    pub id: String,
    pub company_id: String,
    pub company_skill_id: String,
    pub parent_comment_id: Option<String>,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub body: String,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a skill comment.
#[derive(Debug, Clone)]
pub struct NewSkillComment {
    pub company_id: String,
    pub company_skill_id: String,
    pub parent_comment_id: Option<String>,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
    pub body: String,
}

/// A row of the `company_skill_stars` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillStarRecord {
    pub id: String,
    pub company_id: String,
    pub company_skill_id: String,
    pub agent_id: Option<String>,
    pub user_id: Option<String>,
    pub created_at: String,
}

/// Input for starring a skill.
#[derive(Debug, Clone)]
pub struct NewSkillStar {
    pub company_id: String,
    pub company_skill_id: String,
    pub agent_id: Option<String>,
    pub user_id: Option<String>,
}

/// A row of the `company_skill_test_inputs` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillTestInputRecord {
    pub id: String,
    pub company_id: String,
    pub skill_id: String,
    pub name: String,
    pub content: String,
    pub created_by: Option<String>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a skill test input.
#[derive(Debug, Clone)]
pub struct NewSkillTestInput {
    pub company_id: String,
    pub skill_id: String,
    pub name: String,
    pub content: String,
    pub created_by: Option<String>,
}

/// A row of the `company_skill_test_run_templates` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillTestRunTemplateRecord {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
    pub body: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub updated_by_agent_id: Option<String>,
    pub updated_by_user_id: Option<String>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a skill test run template.
#[derive(Debug, Clone)]
pub struct NewSkillTestRunTemplate {
    pub company_id: String,
    pub name: String,
    pub description: Option<String>,
    pub body: String,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub updated_by_agent_id: Option<String>,
    pub updated_by_user_id: Option<String>,
}

/// A row of the `company_skill_test_runs` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillTestRunRecord {
    pub id: String,
    pub company_id: String,
    pub skill_id: String,
    pub input_id: Option<String>,
    pub input_snapshot: String,
    pub skill_version_id: String,
    pub agent_id: String,
    pub agent_config_snapshot: serde_json::Value,
    pub issue_id: String,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub template_body: Option<String>,
    pub rendered_template_body: Option<String>,
    pub harness_issue_description: String,
    pub status: String,
    pub output_document_key: String,
    pub output_snapshot: String,
    pub error: Option<String>,
    pub deleted_at: Option<String>,
    pub superseded_at: Option<String>,
    pub harness_issue_expires_at: Option<String>,
    pub harness_issue_deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a skill test run.
#[derive(Debug, Clone)]
pub struct NewSkillTestRun {
    pub company_id: String,
    pub skill_id: String,
    pub input_id: Option<String>,
    pub input_snapshot: String,
    pub skill_version_id: String,
    pub agent_id: String,
    pub agent_config_snapshot: serde_json::Value,
    pub issue_id: String,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub template_body: Option<String>,
    pub rendered_template_body: Option<String>,
    pub harness_issue_description: String,
    pub status: String,
    pub output_document_key: String,
    pub output_snapshot: String,
    pub error: Option<String>,
}

/// Skill catalog repository errors.
#[derive(Debug, Error)]
pub enum SkillCatalogError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The skill does not exist in this company.
    #[error("skill not found")]
    SkillNotFound,
    /// A referenced record is missing or belongs to another company.
    #[error("reference not found")]
    ReferenceNotFound,
    /// A unique constraint rejected the insert.
    #[error("record already exists")]
    AlreadyExists,
}

/// Skill catalog persistence contract.
#[async_trait]
pub trait SkillCatalogRepository: Send + Sync {
    /// Publishes a new skill version (revision auto-increments).
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on invalid references or duplicates.
    async fn publish_version(
        &self,
        input: NewSkillVersion,
    ) -> Result<SkillVersionRecord, SkillCatalogError>;

    /// Lists skill versions for a skill.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on database failure.
    async fn list_versions(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillVersionRecord>, SkillCatalogError>;

    /// Sets (upserts) the company skill policy, bumping the revision.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on invalid references.
    async fn set_policy(
        &self,
        input: SetSkillPolicy,
    ) -> Result<SkillPolicyRecord, SkillCatalogError>;

    /// Fetches the company skill policy.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on database failure.
    async fn get_policy(
        &self,
        company_id: &str,
    ) -> Result<Option<SkillPolicyRecord>, SkillCatalogError>;

    /// Creates a skill comment.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on invalid references.
    async fn create_comment(
        &self,
        input: NewSkillComment,
    ) -> Result<SkillCommentRecord, SkillCatalogError>;

    /// Lists comments for a skill.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on database failure.
    async fn list_comments(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillCommentRecord>, SkillCatalogError>;

    /// Stars a skill for an agent or user.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on invalid references or duplicates.
    async fn create_star(&self, input: NewSkillStar) -> Result<SkillStarRecord, SkillCatalogError>;

    /// Lists stars for a skill.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on database failure.
    async fn list_stars(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillStarRecord>, SkillCatalogError>;

    /// Creates a skill test input.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on invalid references.
    async fn create_test_input(
        &self,
        input: NewSkillTestInput,
    ) -> Result<SkillTestInputRecord, SkillCatalogError>;

    /// Lists test inputs for a skill.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on database failure.
    async fn list_test_inputs(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillTestInputRecord>, SkillCatalogError>;

    /// Creates a skill test run template.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on invalid references.
    async fn create_test_run_template(
        &self,
        input: NewSkillTestRunTemplate,
    ) -> Result<SkillTestRunTemplateRecord, SkillCatalogError>;

    /// Lists skill test run templates for a company.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on database failure.
    async fn list_test_run_templates(
        &self,
        company_id: &str,
    ) -> Result<Vec<SkillTestRunTemplateRecord>, SkillCatalogError>;

    /// Creates a skill test run.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on invalid references or duplicates.
    async fn create_test_run(
        &self,
        input: NewSkillTestRun,
    ) -> Result<SkillTestRunRecord, SkillCatalogError>;

    /// Lists test runs for a skill.
    ///
    /// # Errors
    ///
    /// Returns [`SkillCatalogError`] on database failure.
    async fn list_test_runs(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillTestRunRecord>, SkillCatalogError>;
}

/// Turso/libSQL implementation of [`SkillCatalogRepository`].
#[derive(Debug)]
pub struct TursoSkillCatalogRepository {
    db: Database,
}

impl TursoSkillCatalogRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const VERSION_COLUMNS: &str = "id, company_id, company_skill_id, revision_number, label,
                               release_id, release_name, released_at, file_inventory,
                               author_agent_id, author_user_id, created_at";
const POLICY_COLUMNS: &str = "company_id, schema_version, revision, default_effect, rules,
                              created_at, updated_at";
const COMMENT_COLUMNS: &str = "id, company_id, company_skill_id, parent_comment_id,
                               author_agent_id, author_user_id, body, deleted_at, created_at,
                               updated_at";
const STAR_COLUMNS: &str = "id, company_id, company_skill_id, agent_id, user_id, created_at";
const TEST_INPUT_COLUMNS: &str = "id, company_id, skill_id, name, content, created_by,
                                  deleted_at, created_at, updated_at";
const TEMPLATE_COLUMNS: &str = "id, company_id, name, description, body, created_by_agent_id,
                                created_by_user_id, updated_by_agent_id, updated_by_user_id,
                                deleted_at, created_at, updated_at";
const TEST_RUN_COLUMNS: &str = "id, company_id, skill_id, input_id, input_snapshot,
                                skill_version_id, agent_id, agent_config_snapshot, issue_id,
                                template_id, template_name, template_body,
                                rendered_template_body, harness_issue_description, status,
                                output_document_key, output_snapshot, error, deleted_at,
                                superseded_at, harness_issue_expires_at,
                                harness_issue_deleted_at, created_at, updated_at";

fn parse_json(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw).unwrap_or_default()
}

fn row_to_version(row: &libsql::Row) -> Result<SkillVersionRecord, libsql::Error> {
    Ok(SkillVersionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        company_skill_id: helpers::row_text(row, 2)?.expect("company_skill_id"),
        revision_number: helpers::row_i64(row, 3)?,
        label: helpers::row_text(row, 4)?,
        release_id: helpers::row_text(row, 5)?,
        release_name: helpers::row_text(row, 6)?,
        released_at: helpers::row_text(row, 7)?,
        file_inventory: parse_json(&helpers::row_text(row, 8)?.expect("file_inventory")),
        author_agent_id: helpers::row_text(row, 9)?,
        author_user_id: helpers::row_text(row, 10)?,
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
    })
}

fn row_to_policy(row: &libsql::Row) -> Result<SkillPolicyRecord, libsql::Error> {
    Ok(SkillPolicyRecord {
        company_id: helpers::row_text(row, 0)?.expect("company_id"),
        schema_version: helpers::row_i64(row, 1)?,
        revision: helpers::row_i64(row, 2)?,
        default_effect: helpers::row_text(row, 3)?.expect("default_effect"),
        rules: parse_json(&helpers::row_text(row, 4)?.expect("rules")),
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
        updated_at: helpers::row_text(row, 6)?.expect("updated_at"),
    })
}

fn row_to_comment(row: &libsql::Row) -> Result<SkillCommentRecord, libsql::Error> {
    Ok(SkillCommentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        company_skill_id: helpers::row_text(row, 2)?.expect("company_skill_id"),
        parent_comment_id: helpers::row_text(row, 3)?,
        author_agent_id: helpers::row_text(row, 4)?,
        author_user_id: helpers::row_text(row, 5)?,
        body: helpers::row_text(row, 6)?.expect("body"),
        deleted_at: helpers::row_text(row, 7)?,
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
        updated_at: helpers::row_text(row, 9)?.expect("updated_at"),
    })
}

fn row_to_star(row: &libsql::Row) -> Result<SkillStarRecord, libsql::Error> {
    Ok(SkillStarRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        company_skill_id: helpers::row_text(row, 2)?.expect("company_skill_id"),
        agent_id: helpers::row_text(row, 3)?,
        user_id: helpers::row_text(row, 4)?,
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
    })
}

fn row_to_test_input(row: &libsql::Row) -> Result<SkillTestInputRecord, libsql::Error> {
    Ok(SkillTestInputRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        skill_id: helpers::row_text(row, 2)?.expect("skill_id"),
        name: helpers::row_text(row, 3)?.expect("name"),
        content: helpers::row_text(row, 4)?.expect("content"),
        created_by: helpers::row_text(row, 5)?,
        deleted_at: helpers::row_text(row, 6)?,
        created_at: helpers::row_text(row, 7)?.expect("created_at"),
        updated_at: helpers::row_text(row, 8)?.expect("updated_at"),
    })
}

fn row_to_template(row: &libsql::Row) -> Result<SkillTestRunTemplateRecord, libsql::Error> {
    Ok(SkillTestRunTemplateRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        name: helpers::row_text(row, 2)?.expect("name"),
        description: helpers::row_text(row, 3)?,
        body: helpers::row_text(row, 4)?.expect("body"),
        created_by_agent_id: helpers::row_text(row, 5)?,
        created_by_user_id: helpers::row_text(row, 6)?,
        updated_by_agent_id: helpers::row_text(row, 7)?,
        updated_by_user_id: helpers::row_text(row, 8)?,
        deleted_at: helpers::row_text(row, 9)?,
        created_at: helpers::row_text(row, 10)?.expect("created_at"),
        updated_at: helpers::row_text(row, 11)?.expect("updated_at"),
    })
}

fn row_to_test_run(row: &libsql::Row) -> Result<SkillTestRunRecord, libsql::Error> {
    Ok(SkillTestRunRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        skill_id: helpers::row_text(row, 2)?.expect("skill_id"),
        input_id: helpers::row_text(row, 3)?,
        input_snapshot: helpers::row_text(row, 4)?.expect("input_snapshot"),
        skill_version_id: helpers::row_text(row, 5)?.expect("skill_version_id"),
        agent_id: helpers::row_text(row, 6)?.expect("agent_id"),
        agent_config_snapshot: parse_json(
            &helpers::row_text(row, 7)?.expect("agent_config_snapshot"),
        ),
        issue_id: helpers::row_text(row, 8)?.expect("issue_id"),
        template_id: helpers::row_text(row, 9)?,
        template_name: helpers::row_text(row, 10)?,
        template_body: helpers::row_text(row, 11)?,
        rendered_template_body: helpers::row_text(row, 12)?,
        harness_issue_description: helpers::row_text(row, 13)?.expect("harness_issue_description"),
        status: helpers::row_text(row, 14)?.expect("status"),
        output_document_key: helpers::row_text(row, 15)?.expect("output_document_key"),
        output_snapshot: helpers::row_text(row, 16)?.expect("output_snapshot"),
        error: helpers::row_text(row, 17)?,
        deleted_at: helpers::row_text(row, 18)?,
        superseded_at: helpers::row_text(row, 19)?,
        harness_issue_expires_at: helpers::row_text(row, 20)?,
        harness_issue_deleted_at: helpers::row_text(row, 21)?,
        created_at: helpers::row_text(row, 22)?.expect("created_at"),
        updated_at: helpers::row_text(row, 23)?.expect("updated_at"),
    })
}

fn map_insert_error(error: libsql::Error) -> SkillCatalogError {
    let message = error.to_string();
    if message.contains("UNIQUE constraint failed") {
        SkillCatalogError::AlreadyExists
    } else if message.contains("FOREIGN KEY constraint failed") {
        SkillCatalogError::ReferenceNotFound
    } else {
        SkillCatalogError::Db(error)
    }
}

#[async_trait]
impl SkillCatalogRepository for TursoSkillCatalogRepository {
    async fn publish_version(
        &self,
        input: NewSkillVersion,
    ) -> Result<SkillVersionRecord, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillCatalogError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "company_skills",
            &input.company_skill_id,
            &input.company_id,
        )
        .await?
        {
            return Err(SkillCatalogError::SkillNotFound);
        }
        let mut rows = conn
            .query(
                "SELECT COALESCE(MAX(revision_number), 0) + 1 FROM company_skill_versions
                 WHERE company_skill_id = ?1",
                libsql::params![input.company_skill_id.clone()],
            )
            .await?;
        let revision = helpers::row_i64(&rows.next().await?.expect("aggregate row"), 0)?;
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO company_skill_versions
                   (id, company_id, company_skill_id, revision_number, label, release_id,
                    release_name, released_at, file_inventory, author_agent_id, author_user_id,
                    created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.company_skill_id,
                    revision,
                    input.label,
                    input.release_id,
                    input.release_name,
                    input.released_at,
                    input.file_inventory.to_string(),
                    input.author_agent_id,
                    input.author_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {VERSION_COLUMNS} FROM company_skill_versions WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("version was just inserted");
                Ok(row_to_version(&row)?)
            }
            Err(error) => Err(map_insert_error(error)),
        }
    }

    async fn list_versions(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillVersionRecord>, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {VERSION_COLUMNS} FROM company_skill_versions
                     WHERE company_id = ?1 AND company_skill_id = ?2
                     ORDER BY revision_number"
                ),
                libsql::params![company_id, skill_id],
            )
            .await?;
        let mut versions = Vec::new();
        while let Some(row) = rows.next().await? {
            versions.push(row_to_version(&row)?);
        }
        Ok(versions)
    }

    async fn set_policy(
        &self,
        input: SetSkillPolicy,
    ) -> Result<SkillPolicyRecord, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillCatalogError::CompanyNotFound);
        }
        let mut rows = conn
            .query(
                "SELECT COALESCE(MAX(revision), 0) + 1 FROM company_skill_policies
                 WHERE company_id = ?1",
                libsql::params![input.company_id.clone()],
            )
            .await?;
        let revision = helpers::row_i64(&rows.next().await?.expect("aggregate row"), 0)?;
        conn.execute(
            "INSERT INTO company_skill_policies
               (company_id, schema_version, revision, default_effect, rules, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id) DO UPDATE SET
               schema_version = excluded.schema_version,
               revision = excluded.revision,
               default_effect = excluded.default_effect,
               rules = excluded.rules,
               updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                input.company_id.clone(),
                input.schema_version,
                revision,
                input.default_effect,
                input.rules.to_string()
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {POLICY_COLUMNS} FROM company_skill_policies WHERE company_id = ?1"
                ),
                libsql::params![input.company_id],
            )
            .await?;
        let row = rows.next().await?.expect("policy was just upserted");
        Ok(row_to_policy(&row)?)
    }

    async fn get_policy(
        &self,
        company_id: &str,
    ) -> Result<Option<SkillPolicyRecord>, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {POLICY_COLUMNS} FROM company_skill_policies WHERE company_id = ?1"
                ),
                libsql::params![company_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_policy(&row)?)),
            None => Ok(None),
        }
    }

    async fn create_comment(
        &self,
        input: NewSkillComment,
    ) -> Result<SkillCommentRecord, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillCatalogError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "company_skills",
            &input.company_skill_id,
            &input.company_id,
        )
        .await?
        {
            return Err(SkillCatalogError::SkillNotFound);
        }
        if let Some(parent_id) = &input.parent_comment_id
            && !helpers::row_belongs_to_company(
                &conn,
                "company_skill_comments",
                parent_id,
                &input.company_id,
            )
            .await?
        {
            return Err(SkillCatalogError::ReferenceNotFound);
        }
        if let Some(agent_id) = &input.author_agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", agent_id, &input.company_id)
                .await?
        {
            return Err(SkillCatalogError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO company_skill_comments
                   (id, company_id, company_skill_id, parent_comment_id, author_agent_id,
                    author_user_id, body, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.company_skill_id,
                    input.parent_comment_id,
                    input.author_agent_id,
                    input.author_user_id,
                    input.body
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {COMMENT_COLUMNS} FROM company_skill_comments WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("comment was just inserted");
                Ok(row_to_comment(&row)?)
            }
            Err(error) => Err(map_insert_error(error)),
        }
    }

    async fn list_comments(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillCommentRecord>, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {COMMENT_COLUMNS} FROM company_skill_comments
                     WHERE company_id = ?1 AND company_skill_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, skill_id],
            )
            .await?;
        let mut comments = Vec::new();
        while let Some(row) = rows.next().await? {
            comments.push(row_to_comment(&row)?);
        }
        Ok(comments)
    }

    async fn create_star(&self, input: NewSkillStar) -> Result<SkillStarRecord, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillCatalogError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "company_skills",
            &input.company_skill_id,
            &input.company_id,
        )
        .await?
        {
            return Err(SkillCatalogError::SkillNotFound);
        }
        if let Some(agent_id) = &input.agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", agent_id, &input.company_id)
                .await?
        {
            return Err(SkillCatalogError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO company_skill_stars
                   (id, company_id, company_skill_id, agent_id, user_id, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.company_skill_id,
                    input.agent_id,
                    input.user_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {STAR_COLUMNS} FROM company_skill_stars WHERE id = ?1"),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("star was just inserted");
                Ok(row_to_star(&row)?)
            }
            Err(error) => Err(map_insert_error(error)),
        }
    }

    async fn list_stars(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillStarRecord>, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {STAR_COLUMNS} FROM company_skill_stars
                     WHERE company_id = ?1 AND company_skill_id = ?2 ORDER BY created_at"
                ),
                libsql::params![company_id, skill_id],
            )
            .await?;
        let mut stars = Vec::new();
        while let Some(row) = rows.next().await? {
            stars.push(row_to_star(&row)?);
        }
        Ok(stars)
    }

    async fn create_test_input(
        &self,
        input: NewSkillTestInput,
    ) -> Result<SkillTestInputRecord, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillCatalogError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "company_skills",
            &input.skill_id,
            &input.company_id,
        )
        .await?
        {
            return Err(SkillCatalogError::SkillNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO company_skill_test_inputs
               (id, company_id, skill_id, name, content, created_by, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.skill_id,
                input.name,
                input.content,
                input.created_by
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {TEST_INPUT_COLUMNS} FROM company_skill_test_inputs WHERE id = ?1"
                ),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("test input was just inserted");
        Ok(row_to_test_input(&row)?)
    }

    async fn list_test_inputs(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillTestInputRecord>, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {TEST_INPUT_COLUMNS} FROM company_skill_test_inputs
                     WHERE company_id = ?1 AND skill_id = ?2 ORDER BY name"
                ),
                libsql::params![company_id, skill_id],
            )
            .await?;
        let mut inputs = Vec::new();
        while let Some(row) = rows.next().await? {
            inputs.push(row_to_test_input(&row)?);
        }
        Ok(inputs)
    }

    async fn create_test_run_template(
        &self,
        input: NewSkillTestRunTemplate,
    ) -> Result<SkillTestRunTemplateRecord, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillCatalogError::CompanyNotFound);
        }
        for agent_id in [&input.created_by_agent_id, &input.updated_by_agent_id]
            .into_iter()
            .flatten()
        {
            if !helpers::row_belongs_to_company(&conn, "agents", agent_id, &input.company_id)
                .await?
            {
                return Err(SkillCatalogError::ReferenceNotFound);
            }
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO company_skill_test_run_templates
               (id, company_id, name, description, body, created_by_agent_id,
                created_by_user_id, updated_by_agent_id, updated_by_user_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.name,
                input.description,
                input.body,
                input.created_by_agent_id,
                input.created_by_user_id,
                input.updated_by_agent_id,
                input.updated_by_user_id
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {TEMPLATE_COLUMNS} FROM company_skill_test_run_templates WHERE id = ?1"
                ),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("template was just inserted");
        Ok(row_to_template(&row)?)
    }

    async fn list_test_run_templates(
        &self,
        company_id: &str,
    ) -> Result<Vec<SkillTestRunTemplateRecord>, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {TEMPLATE_COLUMNS} FROM company_skill_test_run_templates
                     WHERE company_id = ?1 AND deleted_at IS NULL ORDER BY name"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut templates = Vec::new();
        while let Some(row) = rows.next().await? {
            templates.push(row_to_template(&row)?);
        }
        Ok(templates)
    }

    async fn create_test_run(
        &self,
        input: NewSkillTestRun,
    ) -> Result<SkillTestRunRecord, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SkillCatalogError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "company_skills",
            &input.skill_id,
            &input.company_id,
        )
        .await?
        {
            return Err(SkillCatalogError::SkillNotFound);
        }
        for (table, id) in [
            ("company_skill_test_inputs", input.input_id.as_ref()),
            ("company_skill_versions", Some(&input.skill_version_id)),
            ("agents", Some(&input.agent_id)),
            ("issues", Some(&input.issue_id)),
        ] {
            if let Some(id) = id
                && !helpers::row_belongs_to_company(&conn, table, id, &input.company_id).await?
            {
                return Err(SkillCatalogError::ReferenceNotFound);
            }
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO company_skill_test_runs
                   (id, company_id, skill_id, input_id, input_snapshot, skill_version_id,
                    agent_id, agent_config_snapshot, issue_id, template_id, template_name,
                    template_body, rendered_template_body, harness_issue_description, status,
                    output_document_key, output_snapshot, error, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.skill_id,
                    input.input_id,
                    input.input_snapshot,
                    input.skill_version_id,
                    input.agent_id,
                    input.agent_config_snapshot.to_string(),
                    input.issue_id,
                    input.template_id,
                    input.template_name,
                    input.template_body,
                    input.rendered_template_body,
                    input.harness_issue_description,
                    input.status,
                    input.output_document_key,
                    input.output_snapshot,
                    input.error
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {TEST_RUN_COLUMNS} FROM company_skill_test_runs WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("test run was just inserted");
                Ok(row_to_test_run(&row)?)
            }
            Err(error) => Err(map_insert_error(error)),
        }
    }

    async fn list_test_runs(
        &self,
        company_id: &str,
        skill_id: &str,
    ) -> Result<Vec<SkillTestRunRecord>, SkillCatalogError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {TEST_RUN_COLUMNS} FROM company_skill_test_runs
                     WHERE company_id = ?1 AND skill_id = ?2 ORDER BY created_at DESC"
                ),
                libsql::params![company_id, skill_id],
            )
            .await?;
        let mut runs = Vec::new();
        while let Some(row) = rows.next().await? {
            runs.push(row_to_test_run(&row)?);
        }
        Ok(runs)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoSkillCatalogRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let repo = TursoSkillCatalogRepository::new(db);
        (dir, repo)
    }

    async fn seed(conn: &crate::Connection) -> (String, String, String) {
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO company_skills (id, company_id, name) VALUES ('s1', 'c1', 'code_review')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'Reviewer', 'senior', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'Review all', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        ("s1".to_owned(), "a1".to_owned(), "i1".to_owned())
    }

    #[tokio::test]
    async fn skill_catalog_lifecycle_and_dedupe() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();
        let (skill_id, agent_id, issue_id) = seed(&conn).await;

        // Publish two versions; revisions auto-increment.
        let v1 = repo
            .publish_version(NewSkillVersion {
                company_id: "c1".to_owned(),
                company_skill_id: skill_id.clone(),
                label: Some("v1".to_owned()),
                release_id: Some("rel-1".to_owned()),
                release_name: Some("First".to_owned()),
                released_at: Some("2026-08-01T00:00:00.000Z".to_owned()),
                file_inventory: serde_json::json!([{ "path": "SKILL.md" }]),
                author_agent_id: Some(agent_id.clone()),
                author_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(v1.revision_number, 1);
        let v2 = repo
            .publish_version(NewSkillVersion {
                company_id: "c1".to_owned(),
                company_skill_id: skill_id.clone(),
                label: Some("v2".to_owned()),
                release_id: Some("rel-2".to_owned()),
                release_name: None,
                released_at: None,
                file_inventory: serde_json::json!([]),
                author_agent_id: None,
                author_user_id: Some("u1".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(v2.revision_number, 2);

        // Same release id rejected (partial unique index).
        assert!(matches!(
            repo.publish_version(NewSkillVersion {
                company_id: "c1".to_owned(),
                company_skill_id: skill_id.clone(),
                label: None,
                release_id: Some("rel-1".to_owned()),
                release_name: None,
                released_at: None,
                file_inventory: serde_json::json!([]),
                author_agent_id: None,
                author_user_id: None,
            })
            .await
            .unwrap_err(),
            SkillCatalogError::AlreadyExists
        ));

        let versions = repo.list_versions("c1", &skill_id).await.unwrap();
        assert_eq!(versions.len(), 2);
        assert!(
            repo.list_versions("c2", &skill_id)
                .await
                .unwrap()
                .is_empty()
        );

        // Policy upsert bumps revision.
        let policy = repo
            .set_policy(SetSkillPolicy {
                company_id: "c1".to_owned(),
                schema_version: 1,
                default_effect: "allow".to_owned(),
                rules: serde_json::json!([{ "effect": "allow" }]),
            })
            .await
            .unwrap();
        assert_eq!(policy.revision, 1);
        let policy2 = repo
            .set_policy(SetSkillPolicy {
                company_id: "c1".to_owned(),
                schema_version: 1,
                default_effect: "deny".to_owned(),
                rules: serde_json::json!([]),
            })
            .await
            .unwrap();
        assert_eq!(policy2.revision, 2);
        assert_eq!(policy2.default_effect, "deny");
        assert_eq!(repo.get_policy("c1").await.unwrap().unwrap().revision, 2);
        assert!(repo.get_policy("c2").await.unwrap().is_none());

        // Comment + parent comment.
        let comment = repo
            .create_comment(NewSkillComment {
                company_id: "c1".to_owned(),
                company_skill_id: skill_id.clone(),
                parent_comment_id: None,
                author_agent_id: Some(agent_id.clone()),
                author_user_id: None,
                body: "Looks good".to_owned(),
            })
            .await
            .unwrap();
        let reply = repo
            .create_comment(NewSkillComment {
                company_id: "c1".to_owned(),
                company_skill_id: skill_id.clone(),
                parent_comment_id: Some(comment.id.clone()),
                author_agent_id: None,
                author_user_id: Some("u1".to_owned()),
                body: "Agreed".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(
            reply.parent_comment_id.as_deref(),
            Some(comment.id.as_str())
        );
        assert_eq!(repo.list_comments("c1", &skill_id).await.unwrap().len(), 2);

        // Star + dedupe (same agent).
        let star = repo
            .create_star(NewSkillStar {
                company_id: "c1".to_owned(),
                company_skill_id: skill_id.clone(),
                agent_id: Some(agent_id.clone()),
                user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(star.agent_id.as_deref(), Some("a1"));
        assert!(matches!(
            repo.create_star(NewSkillStar {
                company_id: "c1".to_owned(),
                company_skill_id: skill_id.clone(),
                agent_id: Some(agent_id.clone()),
                user_id: None,
            })
            .await
            .unwrap_err(),
            SkillCatalogError::AlreadyExists
        ));
        assert_eq!(repo.list_stars("c1", &skill_id).await.unwrap().len(), 1);

        // Test input + template + run.
        let input = repo
            .create_test_input(NewSkillTestInput {
                company_id: "c1".to_owned(),
                skill_id: skill_id.clone(),
                name: "sample".to_owned(),
                content: "hello".to_owned(),
                created_by: Some("u1".to_owned()),
            })
            .await
            .unwrap();
        let template = repo
            .create_test_run_template(NewSkillTestRunTemplate {
                company_id: "c1".to_owned(),
                name: "default".to_owned(),
                description: Some("default template".to_owned()),
                body: "Run against {{input}}".to_owned(),
                created_by_agent_id: Some(agent_id.clone()),
                created_by_user_id: None,
                updated_by_agent_id: None,
                updated_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(template.name, "default");
        let run = repo
            .create_test_run(NewSkillTestRun {
                company_id: "c1".to_owned(),
                skill_id: skill_id.clone(),
                input_id: Some(input.id.clone()),
                input_snapshot: "hello".to_owned(),
                skill_version_id: v2.id.clone(),
                agent_id: agent_id.clone(),
                agent_config_snapshot: serde_json::json!({ "model": "x" }),
                issue_id: issue_id.clone(),
                template_id: Some(template.id.clone()),
                template_name: Some("default".to_owned()),
                template_body: None,
                rendered_template_body: None,
                harness_issue_description: String::new(),
                status: "queued".to_owned(),
                output_document_key: "output".to_owned(),
                output_snapshot: String::new(),
                error: None,
            })
            .await
            .unwrap();
        assert_eq!(run.status, "queued");
        assert_eq!(run.agent_config_snapshot["model"], "x");

        // Same issue cannot have a second run in the company.
        assert!(matches!(
            repo.create_test_run(NewSkillTestRun {
                company_id: "c1".to_owned(),
                skill_id: skill_id.clone(),
                input_id: None,
                input_snapshot: "hello".to_owned(),
                skill_version_id: v2.id.clone(),
                agent_id: agent_id.clone(),
                agent_config_snapshot: serde_json::json!({}),
                issue_id: issue_id.clone(),
                template_id: None,
                template_name: None,
                template_body: None,
                rendered_template_body: None,
                harness_issue_description: String::new(),
                status: "queued".to_owned(),
                output_document_key: "output".to_owned(),
                output_snapshot: String::new(),
                error: None,
            })
            .await
            .unwrap_err(),
            SkillCatalogError::AlreadyExists
        ));
        assert_eq!(repo.list_test_runs("c1", &skill_id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn cross_company_rejection() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();
        let (skill_id, agent_id, issue_id) = seed(&conn).await;

        // Publishing a version for a c1 skill from c2 is rejected.
        assert!(matches!(
            repo.publish_version(NewSkillVersion {
                company_id: "c2".to_owned(),
                company_skill_id: skill_id.clone(),
                label: None,
                release_id: None,
                release_name: None,
                released_at: None,
                file_inventory: serde_json::json!([]),
                author_agent_id: None,
                author_user_id: None,
            })
            .await
            .unwrap_err(),
            SkillCatalogError::SkillNotFound
        ));

        // A test run in c2 referencing c1's agent is rejected.
        assert!(matches!(
            repo.create_test_run(NewSkillTestRun {
                company_id: "c2".to_owned(),
                skill_id: skill_id.clone(),
                input_id: None,
                input_snapshot: "x".to_owned(),
                skill_version_id: "missing".to_owned(),
                agent_id: agent_id.clone(),
                agent_config_snapshot: serde_json::json!({}),
                issue_id: issue_id.clone(),
                template_id: None,
                template_name: None,
                template_body: None,
                rendered_template_body: None,
                harness_issue_description: String::new(),
                status: "queued".to_owned(),
                output_document_key: "output".to_owned(),
                output_snapshot: String::new(),
                error: None,
            })
            .await
            .unwrap_err(),
            SkillCatalogError::SkillNotFound
        ));

        // A c2 comment on a c1 skill is rejected.
        assert!(matches!(
            repo.create_comment(NewSkillComment {
                company_id: "c2".to_owned(),
                company_skill_id: skill_id.clone(),
                parent_comment_id: None,
                author_agent_id: None,
                author_user_id: Some("u2".to_owned()),
                body: "hi".to_owned(),
            })
            .await
            .unwrap_err(),
            SkillCatalogError::SkillNotFound
        ));
    }
}
