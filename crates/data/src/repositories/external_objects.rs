//! Issue external object links with refreshable status.

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `issue_external_objects` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalObjectRecord {
    /// Link id.
    pub id: String,
    /// Owning company id.
    pub company_id: String,
    /// Issue id.
    pub issue_id: String,
    /// Kind (e.g. `github_pr`).
    pub kind: String,
    /// External id.
    pub external_id: String,
    /// URL.
    pub url: Option<String>,
    /// Status.
    pub status: String,
    /// ISO 8601 last sync time.
    pub last_synced_at: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<String>,
    /// ISO 8601 creation time.
    pub created_at: String,
}

/// Input for linking an external object.
#[derive(Debug, Clone)]
pub struct NewExternalObject {
    /// Issue id.
    pub issue_id: String,
    /// Kind.
    pub kind: String,
    /// External id.
    pub external_id: String,
    /// URL.
    pub url: Option<String>,
    /// Metadata JSON.
    pub metadata: Option<String>,
}

/// External object repository errors.
#[derive(Debug, Error)]
pub enum ExternalObjectError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The issue does not exist.
    #[error("issue not found")]
    IssueNotFound,
    /// The link already exists.
    #[error("external object link already exists")]
    AlreadyExists,
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// A referenced record is missing or belongs to another company.
    #[error("reference not found")]
    ReferenceNotFound,
}

/// External object persistence contract.
#[async_trait]
pub trait ExternalObjectRepository: Send + Sync {
    /// Creates a link.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on invalid references or duplicates.
    async fn create(
        &self,
        input: NewExternalObject,
    ) -> Result<ExternalObjectRecord, ExternalObjectError>;

    /// Lists links for an issue.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn list_for_issue(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ExternalObjectRecord>, ExternalObjectError>;

    /// Refreshes a link's status and sync time.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn refresh(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Option<ExternalObjectRecord>, ExternalObjectError>;
}

/// Turso/libSQL implementation of [`ExternalObjectRepository`].
#[derive(Debug)]
pub struct TursoExternalObjectRepository {
    db: Database,
}

impl TursoExternalObjectRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ExternalObjectRepository for TursoExternalObjectRepository {
    async fn create(
        &self,
        input: NewExternalObject,
    ) -> Result<ExternalObjectRecord, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let Some(company_id) = helpers::row_company(&conn, "issues", &input.issue_id).await? else {
            return Err(ExternalObjectError::IssueNotFound);
        };
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO issue_external_objects (id, company_id, issue_id, kind, external_id,
                                                     url, status, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    company_id,
                    input.issue_id,
                    input.kind,
                    input.external_id,
                    input.url,
                    input.metadata
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        "SELECT id, company_id, issue_id, kind, external_id, url, status,
                                last_synced_at, metadata, created_at
                         FROM issue_external_objects WHERE id = ?1",
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("link was just inserted");
                Ok(row_to_object(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(ExternalObjectError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_for_issue(
        &self,
        issue_id: &str,
    ) -> Result<Vec<ExternalObjectRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, kind, external_id, url, status,
                        last_synced_at, metadata, created_at
                 FROM issue_external_objects WHERE issue_id = ?1 ORDER BY created_at",
                libsql::params![issue_id],
            )
            .await?;
        let mut objects = Vec::new();
        while let Some(row) = rows.next().await? {
            objects.push(row_to_object(&row)?);
        }
        Ok(objects)
    }

    async fn refresh(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Option<ExternalObjectRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let updated = conn
            .execute(
                "UPDATE issue_external_objects
                 SET status = ?1, last_synced_at = strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?2",
                libsql::params![status, id],
            )
            .await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                "SELECT id, company_id, issue_id, kind, external_id, url, status,
                        last_synced_at, metadata, created_at
                 FROM issue_external_objects WHERE id = ?1",
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("link exists");
        Ok(Some(row_to_object(&row)?))
    }
}

fn row_to_object(row: &libsql::Row) -> Result<ExternalObjectRecord, libsql::Error> {
    Ok(ExternalObjectRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        issue_id: helpers::row_text(row, 2)?.expect("issue_id"),
        kind: helpers::row_text(row, 3)?.expect("kind"),
        external_id: helpers::row_text(row, 4)?.expect("external_id"),
        url: helpers::row_text(row, 5)?,
        status: helpers::row_text(row, 6)?.expect("status"),
        last_synced_at: helpers::row_text(row, 7)?,
        metadata: helpers::row_text(row, 8)?,
        created_at: helpers::row_text(row, 9)?.expect("created_at"),
    })
}

/// A row of the `external_objects` catalog (upstream `external_objects.ts`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalObjectCatalogRecord {
    pub id: String,
    pub company_id: String,
    pub provider_key: String,
    pub plugin_id: Option<String>,
    pub object_type: String,
    pub external_id: String,
    pub sanitized_canonical_url: Option<String>,
    pub canonical_identity_hash: Option<String>,
    pub display_key: Option<String>,
    pub icon_key: Option<String>,
    pub display_title: Option<String>,
    pub status_key: Option<String>,
    pub status_label: Option<String>,
    pub status_icon_key: Option<String>,
    pub status_category: String,
    pub status_tone: String,
    pub liveness: String,
    pub is_terminal: bool,
    pub data: serde_json::Value,
    pub remote_version: Option<String>,
    pub etag: Option<String>,
    pub last_resolved_at: Option<String>,
    pub last_changed_at: Option<String>,
    pub last_error_at: Option<String>,
    pub next_refresh_at: Option<String>,
    pub refresh_started_at: Option<String>,
    pub refresh_token: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for upserting an external-object catalog entry.
#[derive(Debug, Clone)]
pub struct NewExternalObjectCatalog {
    pub company_id: String,
    pub provider_key: String,
    pub plugin_id: Option<String>,
    pub object_type: String,
    pub external_id: String,
    pub sanitized_canonical_url: Option<String>,
    pub canonical_identity_hash: Option<String>,
    pub display_key: Option<String>,
    pub icon_key: Option<String>,
    pub display_title: Option<String>,
    pub status_key: Option<String>,
    pub status_label: Option<String>,
    pub status_icon_key: Option<String>,
    pub status_category: String,
    pub status_tone: String,
    pub liveness: String,
    pub is_terminal: bool,
    pub data: serde_json::Value,
    pub remote_version: Option<String>,
    pub etag: Option<String>,
    pub last_resolved_at: Option<String>,
    pub last_changed_at: Option<String>,
    pub last_error_at: Option<String>,
    pub next_refresh_at: Option<String>,
    pub refresh_started_at: Option<String>,
    pub refresh_token: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
}

/// A row of the `external_object_mentions` table (upstream
/// `external_object_mentions.ts`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalObjectMentionRecord {
    pub id: String,
    pub company_id: String,
    pub source_issue_id: String,
    pub source_kind: String,
    pub source_record_id: Option<String>,
    pub document_key: Option<String>,
    pub property_key: Option<String>,
    pub matched_text_redacted: Option<String>,
    pub sanitized_display_url: Option<String>,
    pub canonical_identity_hash: Option<String>,
    pub canonical_identity: Option<serde_json::Value>,
    pub object_id: Option<String>,
    pub provider_key: Option<String>,
    pub detector_key: Option<String>,
    pub object_type: Option<String>,
    pub confidence: String,
    pub created_by_plugin_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating an external-object mention.
#[derive(Debug, Clone)]
pub struct NewExternalObjectMention {
    pub company_id: String,
    pub source_issue_id: String,
    pub source_kind: String,
    pub source_record_id: Option<String>,
    pub document_key: Option<String>,
    pub property_key: Option<String>,
    pub matched_text_redacted: Option<String>,
    pub sanitized_display_url: Option<String>,
    pub canonical_identity_hash: Option<String>,
    pub canonical_identity: Option<serde_json::Value>,
    pub object_id: Option<String>,
    pub provider_key: Option<String>,
    pub detector_key: Option<String>,
    pub object_type: Option<String>,
    pub confidence: String,
    pub created_by_plugin_id: Option<String>,
}

/// External-object catalog persistence contract (upstream
/// `external_objects.ts` / `external_object_mentions.ts`).
#[async_trait]
pub trait ExternalObjectCatalogRepository: Send + Sync {
    /// Upserts a catalog entry keyed on
    /// `(company_id, provider_key, object_type, external_id)`.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on invalid references or database
    /// failure.
    async fn upsert_catalog(
        &self,
        input: NewExternalObjectCatalog,
    ) -> Result<ExternalObjectCatalogRecord, ExternalObjectError>;

    /// Fetches one catalog entry by id (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn get_catalog(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ExternalObjectCatalogRecord>, ExternalObjectError>;

    /// Lists catalog entries for a company and provider/object type.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn list_catalog(
        &self,
        company_id: &str,
        provider_key: &str,
        object_type: &str,
    ) -> Result<Vec<ExternalObjectCatalogRecord>, ExternalObjectError>;

    /// Creates an external-object mention.
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on invalid references, duplicate
    /// mentions, or database failure.
    async fn create_mention(
        &self,
        input: NewExternalObjectMention,
    ) -> Result<ExternalObjectMentionRecord, ExternalObjectError>;

    /// Lists mentions for an issue (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn list_mentions_for_issue(
        &self,
        company_id: &str,
        source_issue_id: &str,
    ) -> Result<Vec<ExternalObjectMentionRecord>, ExternalObjectError>;

    /// Lists mentions that reference a catalog object (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`ExternalObjectError`] on database failure.
    async fn list_mentions_for_object(
        &self,
        company_id: &str,
        object_id: &str,
    ) -> Result<Vec<ExternalObjectMentionRecord>, ExternalObjectError>;
}

/// Turso/libSQL implementation of [`ExternalObjectCatalogRepository`].
#[derive(Debug)]
pub struct TursoExternalObjectCatalogRepository {
    db: Database,
}

impl TursoExternalObjectCatalogRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const CATALOG_COLUMNS: &str = "id, company_id, provider_key, plugin_id, object_type, external_id,
                               sanitized_canonical_url, canonical_identity_hash, display_key,
                               icon_key, display_title, status_key, status_label, status_icon_key,
                               status_category, status_tone, liveness, is_terminal, data,
                               remote_version, etag, last_resolved_at, last_changed_at,
                               last_error_at, next_refresh_at, refresh_started_at, refresh_token,
                               last_error_code, last_error_message, created_at, updated_at";

const MENTION_COLUMNS: &str = "id, company_id, source_issue_id, source_kind, source_record_id,
                               document_key, property_key, matched_text_redacted,
                               sanitized_display_url, canonical_identity_hash, canonical_identity,
                               object_id, provider_key, detector_key, object_type, confidence,
                               created_by_plugin_id, created_at, updated_at";

#[async_trait]
impl ExternalObjectCatalogRepository for TursoExternalObjectCatalogRepository {
    async fn upsert_catalog(
        &self,
        input: NewExternalObjectCatalog,
    ) -> Result<ExternalObjectCatalogRecord, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ExternalObjectError::CompanyNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let key_company_id = input.company_id.clone();
        let key_provider = input.provider_key.clone();
        let key_object_type = input.object_type.clone();
        let key_external_id = input.external_id.clone();
        conn.execute(
            &format!(
                "INSERT INTO external_objects (
                   id, company_id, provider_key, plugin_id, object_type, external_id,
                   sanitized_canonical_url, canonical_identity_hash, display_key, icon_key,
                   display_title, status_key, status_label, status_icon_key, status_category,
                   status_tone, liveness, is_terminal, data, remote_version, etag,
                   last_resolved_at, last_changed_at, last_error_at, next_refresh_at,
                   refresh_started_at, refresh_token, last_error_code, last_error_message,
                   created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                           ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29,
                           {now}, {now})
                 ON CONFLICT (company_id, provider_key, object_type, external_id) DO UPDATE SET
                   plugin_id = excluded.plugin_id,
                   sanitized_canonical_url = excluded.sanitized_canonical_url,
                   canonical_identity_hash = excluded.canonical_identity_hash,
                   display_key = excluded.display_key,
                   icon_key = excluded.icon_key,
                   display_title = excluded.display_title,
                   status_key = excluded.status_key,
                   status_label = excluded.status_label,
                   status_icon_key = excluded.status_icon_key,
                   status_category = excluded.status_category,
                   status_tone = excluded.status_tone,
                   liveness = excluded.liveness,
                   is_terminal = excluded.is_terminal,
                   data = excluded.data,
                   remote_version = excluded.remote_version,
                   etag = excluded.etag,
                   last_resolved_at = excluded.last_resolved_at,
                   last_changed_at = excluded.last_changed_at,
                   last_error_at = excluded.last_error_at,
                   next_refresh_at = excluded.next_refresh_at,
                   refresh_started_at = excluded.refresh_started_at,
                   refresh_token = excluded.refresh_token,
                   last_error_code = excluded.last_error_code,
                   last_error_message = excluded.last_error_message,
                   updated_at = {now}",
                now = now
            ),
            libsql::params![
                id.clone(),
                input.company_id,
                input.provider_key,
                input.plugin_id,
                input.object_type,
                input.external_id,
                input.sanitized_canonical_url,
                input.canonical_identity_hash,
                input.display_key,
                input.icon_key,
                input.display_title,
                input.status_key,
                input.status_label,
                input.status_icon_key,
                input.status_category,
                input.status_tone,
                input.liveness,
                i64::from(input.is_terminal),
                input.data.to_string(),
                input.remote_version,
                input.etag,
                input.last_resolved_at,
                input.last_changed_at,
                input.last_error_at,
                input.next_refresh_at,
                input.refresh_started_at,
                input.refresh_token,
                input.last_error_code,
                input.last_error_message,
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CATALOG_COLUMNS} FROM external_objects
                     WHERE company_id = ?1 AND provider_key = ?2 AND object_type = ?3
                       AND external_id = ?4
                     LIMIT 1"
                ),
                libsql::params![
                    key_company_id,
                    key_provider,
                    key_object_type,
                    key_external_id
                ],
            )
            .await?;
        let row = rows.next().await?.expect("catalog entry was just upserted");
        Ok(row_to_catalog(&row)?)
    }

    async fn get_catalog(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<ExternalObjectCatalogRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CATALOG_COLUMNS} FROM external_objects
                     WHERE id = ?1 AND company_id = ?2"
                ),
                libsql::params![id, company_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_catalog(&row)?)),
            None => Ok(None),
        }
    }

    async fn list_catalog(
        &self,
        company_id: &str,
        provider_key: &str,
        object_type: &str,
    ) -> Result<Vec<ExternalObjectCatalogRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CATALOG_COLUMNS} FROM external_objects
                     WHERE company_id = ?1 AND provider_key = ?2 AND object_type = ?3
                     ORDER BY updated_at DESC"
                ),
                libsql::params![company_id, provider_key, object_type],
            )
            .await?;
        let mut objects = Vec::new();
        while let Some(row) = rows.next().await? {
            objects.push(row_to_catalog(&row)?);
        }
        Ok(objects)
    }

    async fn create_mention(
        &self,
        input: NewExternalObjectMention,
    ) -> Result<ExternalObjectMentionRecord, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(ExternalObjectError::CompanyNotFound);
        }
        if helpers::row_company(&conn, "issues", &input.source_issue_id).await?
            != Some(input.company_id.clone())
        {
            return Err(ExternalObjectError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let now = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
        let result = conn
            .execute(
                &format!(
                    "INSERT INTO external_object_mentions (
                       id, company_id, source_issue_id, source_kind, source_record_id,
                       document_key, property_key, matched_text_redacted, sanitized_display_url,
                       canonical_identity_hash, canonical_identity, object_id, provider_key,
                       detector_key, object_type, confidence, created_by_plugin_id,
                       created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                               ?16, ?17, {now}, {now})",
                    now = now
                ),
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.source_issue_id,
                    input.source_kind,
                    input.source_record_id,
                    input.document_key,
                    input.property_key,
                    input.matched_text_redacted,
                    input.sanitized_display_url,
                    input.canonical_identity_hash,
                    input.canonical_identity.map(|v| v.to_string()),
                    input.object_id,
                    input.provider_key,
                    input.detector_key,
                    input.object_type,
                    input.confidence,
                    input.created_by_plugin_id,
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {MENTION_COLUMNS} FROM external_object_mentions WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("mention was just inserted");
                Ok(row_to_mention(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(ExternalObjectError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list_mentions_for_issue(
        &self,
        company_id: &str,
        source_issue_id: &str,
    ) -> Result<Vec<ExternalObjectMentionRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MENTION_COLUMNS} FROM external_object_mentions
                     WHERE company_id = ?1 AND source_issue_id = ?2
                     ORDER BY created_at"
                ),
                libsql::params![company_id, source_issue_id],
            )
            .await?;
        let mut mentions = Vec::new();
        while let Some(row) = rows.next().await? {
            mentions.push(row_to_mention(&row)?);
        }
        Ok(mentions)
    }

    async fn list_mentions_for_object(
        &self,
        company_id: &str,
        object_id: &str,
    ) -> Result<Vec<ExternalObjectMentionRecord>, ExternalObjectError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {MENTION_COLUMNS} FROM external_object_mentions
                     WHERE company_id = ?1 AND object_id = ?2
                     ORDER BY created_at"
                ),
                libsql::params![company_id, object_id],
            )
            .await?;
        let mut mentions = Vec::new();
        while let Some(row) = rows.next().await? {
            mentions.push(row_to_mention(&row)?);
        }
        Ok(mentions)
    }
}

fn row_to_catalog(row: &libsql::Row) -> Result<ExternalObjectCatalogRecord, libsql::Error> {
    Ok(ExternalObjectCatalogRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        provider_key: helpers::row_text(row, 2)?.expect("provider_key"),
        plugin_id: helpers::row_text(row, 3)?,
        object_type: helpers::row_text(row, 4)?.expect("object_type"),
        external_id: helpers::row_text(row, 5)?.expect("external_id"),
        sanitized_canonical_url: helpers::row_text(row, 6)?,
        canonical_identity_hash: helpers::row_text(row, 7)?,
        display_key: helpers::row_text(row, 8)?,
        icon_key: helpers::row_text(row, 9)?,
        display_title: helpers::row_text(row, 10)?,
        status_key: helpers::row_text(row, 11)?,
        status_label: helpers::row_text(row, 12)?,
        status_icon_key: helpers::row_text(row, 13)?,
        status_category: helpers::row_text(row, 14)?.expect("status_category"),
        status_tone: helpers::row_text(row, 15)?.expect("status_tone"),
        liveness: helpers::row_text(row, 16)?.expect("liveness"),
        is_terminal: helpers::row_i64(row, 17)? != 0,
        data: serde_json::from_str(&helpers::row_text(row, 18)?.unwrap_or_else(|| "{}".to_owned()))
            .unwrap_or_default(),
        remote_version: helpers::row_text(row, 19)?,
        etag: helpers::row_text(row, 20)?,
        last_resolved_at: helpers::row_text(row, 21)?,
        last_changed_at: helpers::row_text(row, 22)?,
        last_error_at: helpers::row_text(row, 23)?,
        next_refresh_at: helpers::row_text(row, 24)?,
        refresh_started_at: helpers::row_text(row, 25)?,
        refresh_token: helpers::row_text(row, 26)?,
        last_error_code: helpers::row_text(row, 27)?,
        last_error_message: helpers::row_text(row, 28)?,
        created_at: helpers::row_text(row, 29)?.expect("created_at"),
        updated_at: helpers::row_text(row, 30)?.expect("updated_at"),
    })
}

fn row_to_mention(row: &libsql::Row) -> Result<ExternalObjectMentionRecord, libsql::Error> {
    Ok(ExternalObjectMentionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        source_issue_id: helpers::row_text(row, 2)?.expect("source_issue_id"),
        source_kind: helpers::row_text(row, 3)?.expect("source_kind"),
        source_record_id: helpers::row_text(row, 4)?,
        document_key: helpers::row_text(row, 5)?,
        property_key: helpers::row_text(row, 6)?,
        matched_text_redacted: helpers::row_text(row, 7)?,
        sanitized_display_url: helpers::row_text(row, 8)?,
        canonical_identity_hash: helpers::row_text(row, 9)?,
        canonical_identity: helpers::row_text(row, 10)?
            .and_then(|value| serde_json::from_str(&value).ok()),
        object_id: helpers::row_text(row, 11)?,
        provider_key: helpers::row_text(row, 12)?,
        detector_key: helpers::row_text(row, 13)?,
        object_type: helpers::row_text(row, 14)?,
        confidence: helpers::row_text(row, 15)?.expect("confidence"),
        created_by_plugin_id: helpers::row_text(row, 16)?,
        created_at: helpers::row_text(row, 17)?.expect("created_at"),
        updated_at: helpers::row_text(row, 18)?.expect("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoExternalObjectRepository) {
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
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'T', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        let repo = TursoExternalObjectRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn create_list_refresh_roundtrip() {
        let (_dir, repo) = repo().await;
        let created = repo
            .create(NewExternalObject {
                issue_id: "i1".to_owned(),
                kind: "github_pr".to_owned(),
                external_id: "123".to_owned(),
                url: Some("https://github.com/x/y/pull/123".to_owned()),
                metadata: None,
            })
            .await
            .unwrap();
        assert_eq!(created.status, "pending");

        let error = repo
            .create(NewExternalObject {
                issue_id: "i1".to_owned(),
                kind: "github_pr".to_owned(),
                external_id: "123".to_owned(),
                url: None,
                metadata: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(error, ExternalObjectError::AlreadyExists));

        let refreshed = repo.refresh(&created.id, "merged").await.unwrap().unwrap();
        assert_eq!(refreshed.status, "merged");
        assert!(refreshed.last_synced_at.is_some());

        let list = repo.list_for_issue("i1").await.unwrap();
        assert_eq!(list.len(), 1);
    }
}

#[cfg(test)]
mod catalog_tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    fn catalog_input(company_id: &str) -> NewExternalObjectCatalog {
        NewExternalObjectCatalog {
            company_id: company_id.to_owned(),
            provider_key: "github".to_owned(),
            plugin_id: None,
            object_type: "pull_request".to_owned(),
            external_id: "repo/123".to_owned(),
            sanitized_canonical_url: Some("https://github.com/x/y/pull/123".to_owned()),
            canonical_identity_hash: Some("abc123".to_owned()),
            display_key: Some("x/y#123".to_owned()),
            icon_key: Some("pr".to_owned()),
            display_title: Some("Fix the thing".to_owned()),
            status_key: Some("open".to_owned()),
            status_label: Some("Open".to_owned()),
            status_icon_key: Some("circle".to_owned()),
            status_category: "open".to_owned(),
            status_tone: "neutral".to_owned(),
            liveness: "active".to_owned(),
            is_terminal: false,
            data: serde_json::json!({ "title": "Fix the thing" }),
            remote_version: Some("v2".to_owned()),
            etag: Some("\"etag-1\"".to_owned()),
            last_resolved_at: None,
            last_changed_at: None,
            last_error_at: None,
            next_refresh_at: None,
            refresh_started_at: None,
            refresh_token: None,
            last_error_code: None,
            last_error_message: None,
        }
    }

    async fn repo() -> (TempDir, TursoExternalObjectCatalogRepository) {
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
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c2', 'Beta', 'BETA', 1024)",
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
        let repo = TursoExternalObjectCatalogRepository::new(db);
        (dir, repo)
    }

    #[tokio::test]
    async fn upsert_get_list_catalog() {
        let (_dir, repo) = repo().await;
        let created = repo.upsert_catalog(catalog_input("c1")).await.unwrap();
        assert_eq!(created.provider_key, "github");
        assert_eq!(created.status_category, "open");
        assert!(!created.is_terminal);

        // Upsert with same key updates, does not duplicate.
        let mut updated_input = catalog_input("c1");
        updated_input.status_key = Some("merged".to_owned());
        updated_input.is_terminal = true;
        let updated = repo.upsert_catalog(updated_input).await.unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.status_key.as_deref(), Some("merged"));
        assert!(updated.is_terminal);

        let fetched = repo
            .get_catalog("c1", &created.id)
            .await
            .unwrap()
            .expect("catalog entry");
        assert_eq!(fetched.id, created.id);
        assert_eq!(
            fetched.data,
            serde_json::json!({ "title": "Fix the thing" })
        );

        let list = repo
            .list_catalog("c1", "github", "pull_request")
            .await
            .unwrap();
        assert_eq!(list.len(), 1);

        // Cross-company lookup is empty.
        assert!(repo.get_catalog("c2", &created.id).await.unwrap().is_none());
        assert!(
            repo.list_catalog("c2", "github", "pull_request")
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn upsert_rejects_unknown_company() {
        let (_dir, repo) = repo().await;
        let error = repo
            .upsert_catalog(catalog_input("nope"))
            .await
            .unwrap_err();
        assert!(matches!(error, ExternalObjectError::CompanyNotFound));
    }

    #[tokio::test]
    async fn mention_roundtrip_and_rejects_cross_company() {
        let (_dir, repo) = repo().await;
        let object = repo.upsert_catalog(catalog_input("c1")).await.unwrap();
        let mention = repo
            .create_mention(NewExternalObjectMention {
                company_id: "c1".to_owned(),
                source_issue_id: "i1".to_owned(),
                source_kind: "issue_body".to_owned(),
                source_record_id: None,
                document_key: Some("body".to_owned()),
                property_key: Some("links".to_owned()),
                matched_text_redacted: Some("x/y#123".to_owned()),
                sanitized_display_url: Some("https://github.com/x/y/pull/123".to_owned()),
                canonical_identity_hash: Some("abc123".to_owned()),
                canonical_identity: Some(serde_json::json!({ "repo": "x/y", "number": 123 })),
                object_id: Some(object.id.clone()),
                provider_key: Some("github".to_owned()),
                detector_key: Some("url".to_owned()),
                object_type: Some("pull_request".to_owned()),
                confidence: "exact".to_owned(),
                created_by_plugin_id: None,
            })
            .await
            .unwrap();
        assert_eq!(mention.object_id.as_deref(), Some(object.id.as_str()));
        assert_eq!(mention.confidence, "exact");

        // Duplicate mention (same identity, no source record) is rejected.
        let duplicate = repo
            .create_mention(NewExternalObjectMention {
                company_id: "c1".to_owned(),
                source_issue_id: "i1".to_owned(),
                source_kind: "issue_body".to_owned(),
                source_record_id: None,
                document_key: Some("body".to_owned()),
                property_key: Some("links".to_owned()),
                matched_text_redacted: Some("x/y#123".to_owned()),
                sanitized_display_url: Some("https://github.com/x/y/pull/123".to_owned()),
                canonical_identity_hash: Some("abc123".to_owned()),
                canonical_identity: None,
                object_id: Some(object.id.clone()),
                provider_key: Some("github".to_owned()),
                detector_key: Some("url".to_owned()),
                object_type: Some("pull_request".to_owned()),
                confidence: "exact".to_owned(),
                created_by_plugin_id: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(duplicate, ExternalObjectError::AlreadyExists));

        // Cross-company source issue is rejected.
        let cross = repo
            .create_mention(NewExternalObjectMention {
                company_id: "c2".to_owned(),
                source_issue_id: "i1".to_owned(),
                source_kind: "issue_body".to_owned(),
                source_record_id: None,
                document_key: None,
                property_key: None,
                matched_text_redacted: None,
                sanitized_display_url: None,
                canonical_identity_hash: None,
                canonical_identity: None,
                object_id: None,
                provider_key: Some("github".to_owned()),
                detector_key: Some("url".to_owned()),
                object_type: Some("pull_request".to_owned()),
                confidence: "exact".to_owned(),
                created_by_plugin_id: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(cross, ExternalObjectError::ReferenceNotFound));

        let for_issue = repo.list_mentions_for_issue("c1", "i1").await.unwrap();
        assert_eq!(for_issue.len(), 1);
        let for_object = repo
            .list_mentions_for_object("c1", &object.id)
            .await
            .unwrap();
        assert_eq!(for_object.len(), 1);
    }
}
