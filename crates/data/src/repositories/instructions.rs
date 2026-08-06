//! Instruction documents and per-agent instruction file mounts.
//!
//! Mirrors the upstream agent-instructions service's data surface: a
//! company-scoped document library plus a managed file bundle per agent with
//! an entry-file flag (`AGENTS.md` by default).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A company instruction document.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionDocumentRecord {
    /// Document id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Document name (unique per company).
    pub name: String,
    /// Document content.
    pub content: String,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// An instruction file mounted on an agent (managed bundle semantics).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstructionFileRecord {
    /// File id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Normalized relative path inside the bundle (e.g. `AGENTS.md`).
    pub path: String,
    /// File content.
    pub content: String,
    /// Whether this file is the bundle entry file.
    pub is_entry: bool,
    /// ISO 8601 creation time.
    pub created_at: String,
    /// ISO 8601 last update time.
    pub updated_at: String,
}

/// Input for creating a company instruction document.
#[derive(Debug, Clone)]
pub struct NewInstructionDocument {
    /// Owning company id.
    pub company_id: String,
    /// Document name.
    pub name: String,
    /// Document content.
    pub content: String,
}

/// Input for updating a company instruction document.
#[derive(Debug, Clone)]
pub struct UpdateInstructionDocument {
    /// Document id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// New document name.
    pub name: String,
    /// New document content.
    pub content: String,
}

/// Input for creating/updating an agent instruction file.
#[derive(Debug, Clone)]
pub struct NewAgentInstructionFile {
    /// Owning company id.
    pub company_id: String,
    /// Agent id.
    pub agent_id: String,
    /// Normalized relative path inside the bundle.
    pub path: String,
    /// File content.
    pub content: String,
    /// Whether this file is the bundle entry file.
    pub is_entry: bool,
}

/// Instruction repository errors.
#[derive(Debug, Error)]
pub enum InstructionError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// A referenced record does not exist in this company.
    #[error("referenced record not found: {0}")]
    ReferenceNotFound(&'static str),
    /// The instruction record does not exist.
    #[error("instruction not found")]
    NotFound,
}

/// Instruction persistence contract.
#[async_trait]
pub trait InstructionRepository: Send + Sync {
    /// Creates a company instruction document.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError`] when the company does not exist.
    async fn create_document(
        &self,
        input: NewInstructionDocument,
    ) -> Result<InstructionDocumentRecord, InstructionError>;

    /// Lists instruction documents for a company.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError`] on database failure.
    async fn list_documents(
        &self,
        company_id: &str,
    ) -> Result<Vec<InstructionDocumentRecord>, InstructionError>;

    /// Fetches one instruction document by id.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError`] on database failure.
    async fn get_document(
        &self,
        id: &str,
    ) -> Result<Option<InstructionDocumentRecord>, InstructionError>;

    /// Updates an instruction document, scoped to its owning company.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError::NotFound`] when the document is missing or
    /// belongs to another company.
    async fn update_document(
        &self,
        input: UpdateInstructionDocument,
    ) -> Result<InstructionDocumentRecord, InstructionError>;

    /// Deletes an instruction document owned by `company_id`.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError`] on database failure.
    async fn delete_document(&self, id: &str, company_id: &str) -> Result<bool, InstructionError>;

    /// Lists the instruction files mounted on an agent.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError`] on database failure.
    async fn list_agent_files(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Vec<AgentInstructionFileRecord>, InstructionError>;

    /// Creates or replaces an agent instruction file (upsert on
    /// `(company_id, agent_id, path)`). Setting `is_entry` clears the entry
    /// flag on the agent's other files so the bundle keeps a single entry.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError::ReferenceNotFound`] when the agent does not
    /// belong to the company.
    async fn upsert_agent_file(
        &self,
        input: NewAgentInstructionFile,
    ) -> Result<AgentInstructionFileRecord, InstructionError>;

    /// Deletes an agent instruction file.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError`] on database failure.
    async fn delete_agent_file(
        &self,
        company_id: &str,
        agent_id: &str,
        path: &str,
    ) -> Result<bool, InstructionError>;

    /// Fetches one agent instruction file.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionError`] on database failure.
    async fn get_agent_file(
        &self,
        company_id: &str,
        agent_id: &str,
        path: &str,
    ) -> Result<Option<AgentInstructionFileRecord>, InstructionError>;
}

/// Turso/libSQL implementation of [`InstructionRepository`].
#[derive(Debug)]
pub struct TursoInstructionRepository {
    db: Database,
}

impl TursoInstructionRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InstructionRepository for TursoInstructionRepository {
    async fn create_document(
        &self,
        input: NewInstructionDocument,
    ) -> Result<InstructionDocumentRecord, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(InstructionError::ReferenceNotFound("company"));
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO instruction_documents (id, company_id, name, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.name.clone(),
                input.content.clone()
            ],
        )
        .await?;
        Ok(self
            .get_document(&id)
            .await?
            .expect("document was just inserted"))
    }

    async fn list_documents(
        &self,
        company_id: &str,
    ) -> Result<Vec<InstructionDocumentRecord>, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, content, created_at, updated_at
                 FROM instruction_documents WHERE company_id = ?1 ORDER BY name",
                libsql::params![company_id],
            )
            .await?;
        let mut documents = Vec::new();
        while let Some(row) = rows.next().await? {
            documents.push(row_to_document(&row)?);
        }
        Ok(documents)
    }

    async fn get_document(
        &self,
        id: &str,
    ) -> Result<Option<InstructionDocumentRecord>, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, name, content, created_at, updated_at
                 FROM instruction_documents WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_document(&row)?)),
            None => Ok(None),
        }
    }

    async fn update_document(
        &self,
        input: UpdateInstructionDocument,
    ) -> Result<InstructionDocumentRecord, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(
            &conn,
            "instruction_documents",
            &input.id,
            &input.company_id,
        )
        .await?
        {
            return Err(InstructionError::NotFound);
        }
        conn.execute(
            "UPDATE instruction_documents SET name = ?1, content = ?2,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?3",
            libsql::params![input.name.clone(), input.content.clone(), input.id.clone()],
        )
        .await?;
        Ok(self
            .get_document(&input.id)
            .await?
            .expect("document exists after update"))
    }

    async fn delete_document(&self, id: &str, company_id: &str) -> Result<bool, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let result = conn
            .execute(
                "DELETE FROM instruction_documents WHERE id = ?1 AND company_id = ?2",
                libsql::params![id, company_id],
            )
            .await?;
        Ok(result > 0)
    }

    async fn list_agent_files(
        &self,
        company_id: &str,
        agent_id: &str,
    ) -> Result<Vec<AgentInstructionFileRecord>, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, agent_id, path, content, is_entry, created_at, updated_at
                 FROM agent_instruction_files
                 WHERE company_id = ?1 AND agent_id = ?2 ORDER BY path",
                libsql::params![company_id, agent_id],
            )
            .await?;
        let mut files = Vec::new();
        while let Some(row) = rows.next().await? {
            files.push(row_to_file(&row)?);
        }
        Ok(files)
    }

    async fn upsert_agent_file(
        &self,
        input: NewAgentInstructionFile,
    ) -> Result<AgentInstructionFileRecord, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::row_belongs_to_company(&conn, "agents", &input.agent_id, &input.company_id)
            .await?
        {
            return Err(InstructionError::ReferenceNotFound("agent"));
        }
        if input.is_entry {
            conn.execute(
                "UPDATE agent_instruction_files SET is_entry = 0
                 WHERE company_id = ?1 AND agent_id = ?2 AND path != ?3",
                libsql::params![
                    input.company_id.clone(),
                    input.agent_id.clone(),
                    input.path.clone()
                ],
            )
            .await?;
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO agent_instruction_files
               (id, company_id, agent_id, path, content, is_entry, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, agent_id, path)
             DO UPDATE SET content = excluded.content,
                           is_entry = excluded.is_entry,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.agent_id.clone(),
                input.path.clone(),
                input.content.clone(),
                i64::from(input.is_entry)
            ],
        )
        .await?;
        Ok(self
            .get_agent_file(&input.company_id, &input.agent_id, &input.path)
            .await?
            .expect("file was just upserted"))
    }

    async fn delete_agent_file(
        &self,
        company_id: &str,
        agent_id: &str,
        path: &str,
    ) -> Result<bool, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let result = conn
            .execute(
                "DELETE FROM agent_instruction_files
                 WHERE company_id = ?1 AND agent_id = ?2 AND path = ?3",
                libsql::params![company_id, agent_id, path],
            )
            .await?;
        Ok(result > 0)
    }

    async fn get_agent_file(
        &self,
        company_id: &str,
        agent_id: &str,
        path: &str,
    ) -> Result<Option<AgentInstructionFileRecord>, InstructionError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, agent_id, path, content, is_entry, created_at, updated_at
                 FROM agent_instruction_files
                 WHERE company_id = ?1 AND agent_id = ?2 AND path = ?3",
                libsql::params![company_id, agent_id, path],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_file(&row)?)),
            None => Ok(None),
        }
    }
}

fn row_to_document(row: &libsql::Row) -> Result<InstructionDocumentRecord, libsql::Error> {
    Ok(InstructionDocumentRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        name: helpers::row_text(row, 2)?.expect("name"),
        content: helpers::row_text(row, 3)?.expect("content"),
        created_at: helpers::row_text(row, 4)?.expect("created_at"),
        updated_at: helpers::row_text(row, 5)?.expect("updated_at"),
    })
}

fn row_to_file(row: &libsql::Row) -> Result<AgentInstructionFileRecord, libsql::Error> {
    Ok(AgentInstructionFileRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        agent_id: helpers::row_text(row, 2)?.expect("agent_id"),
        path: helpers::row_text(row, 3)?.expect("path"),
        content: helpers::row_text(row, 4)?.expect("content"),
        is_entry: helpers::row_i64(row, 5)? != 0,
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
        updated_at: helpers::row_text(row, 7)?.expect("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoInstructionRepository) {
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
        let repo = TursoInstructionRepository::new(db);
        (dir, repo)
    }

    fn new_document() -> NewInstructionDocument {
        NewInstructionDocument {
            company_id: "c1".to_owned(),
            name: "AGENTS.md".to_owned(),
            content: "# Instructions".to_owned(),
        }
    }

    fn new_file(is_entry: bool) -> NewAgentInstructionFile {
        NewAgentInstructionFile {
            company_id: "c1".to_owned(),
            agent_id: "a1".to_owned(),
            path: "AGENTS.md".to_owned(),
            content: "# Agent instructions".to_owned(),
            is_entry,
        }
    }

    #[tokio::test]
    async fn document_crud_roundtrip() {
        let (_dir, repo) = repo().await;
        let created = repo.create_document(new_document()).await.unwrap();
        assert_eq!(created.name, "AGENTS.md");
        assert_eq!(created.content, "# Instructions");

        let listed = repo.list_documents("c1").await.unwrap();
        assert_eq!(listed.len(), 1);

        let updated = repo
            .update_document(UpdateInstructionDocument {
                id: created.id.clone(),
                company_id: "c1".to_owned(),
                name: "SOUL.md".to_owned(),
                content: "# Soul".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(updated.name, "SOUL.md");

        // Cross-company update rejected.
        let error = repo
            .update_document(UpdateInstructionDocument {
                id: created.id.clone(),
                company_id: "c2".to_owned(),
                name: "AGENTS.md".to_owned(),
                content: "x".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(error, InstructionError::NotFound));

        assert!(repo.delete_document(&created.id, "c1").await.unwrap());
        assert!(!repo.delete_document(&created.id, "c1").await.unwrap());
        assert!(repo.list_documents("c1").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn create_document_rejects_missing_company() {
        let (_dir, repo) = repo().await;
        let error = repo
            .create_document(NewInstructionDocument {
                company_id: "missing".to_owned(),
                name: "AGENTS.md".to_owned(),
                content: "x".to_owned(),
            })
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            InstructionError::ReferenceNotFound("company")
        ));
    }

    #[tokio::test]
    async fn agent_file_upsert_delete_roundtrip() {
        let (_dir, repo) = repo().await;
        let file = repo.upsert_agent_file(new_file(true)).await.unwrap();
        assert!(file.is_entry);
        assert_eq!(file.path, "AGENTS.md");

        // Upsert on the same path replaces content and keeps the entry flag.
        let replaced = repo
            .upsert_agent_file(NewAgentInstructionFile {
                content: "# v2".to_owned(),
                ..new_file(true)
            })
            .await
            .unwrap();
        assert_eq!(replaced.content, "# v2");
        assert!(replaced.is_entry);

        // A second non-entry file does not steal the entry flag.
        let extra = repo
            .upsert_agent_file(NewAgentInstructionFile {
                path: "HEARTBEAT.md".to_owned(),
                content: "# Heartbeat".to_owned(),
                is_entry: false,
                ..new_file(false)
            })
            .await
            .unwrap();
        assert!(!extra.is_entry);

        let files = repo.list_agent_files("c1", "a1").await.unwrap();
        assert_eq!(files.len(), 2);

        // Setting a new entry file clears the previous one.
        let new_entry = repo
            .upsert_agent_file(NewAgentInstructionFile {
                path: "HEARTBEAT.md".to_owned(),
                content: "# Heartbeat".to_owned(),
                is_entry: true,
                ..new_file(false)
            })
            .await
            .unwrap();
        assert!(new_entry.is_entry);
        let files = repo.list_agent_files("c1", "a1").await.unwrap();
        let agents_entry = files.iter().find(|file| file.path == "AGENTS.md").unwrap();
        assert!(!agents_entry.is_entry);

        assert!(
            repo.delete_agent_file("c1", "a1", "AGENTS.md")
                .await
                .unwrap()
        );
        assert!(
            !repo
                .delete_agent_file("c1", "a1", "AGENTS.md")
                .await
                .unwrap()
        );
        assert!(
            repo.get_agent_file("c1", "a1", "AGENTS.md")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn agent_file_rejects_cross_company_agent() {
        let (_dir, repo) = repo().await;
        let error = repo
            .upsert_agent_file(NewAgentInstructionFile {
                company_id: "c2".to_owned(),
                agent_id: "a1".to_owned(),
                path: "AGENTS.md".to_owned(),
                content: "x".to_owned(),
                is_entry: true,
            })
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            InstructionError::ReferenceNotFound("agent")
        ));
    }
}
