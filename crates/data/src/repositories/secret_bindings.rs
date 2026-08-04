//! Secret binding repository: provider configs, bindings, user secret
//! definitions/declarations, and access events (upstream
//! company_secret_bindings.ts + company_secret_provider_configs.ts +
//! user_secret_definitions.ts + user_secret_declarations.ts +
//! secret_access_events.ts).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `company_secret_provider_configs` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretProviderConfigRecord {
    pub id: String,
    pub company_id: String,
    pub provider: String,
    pub display_name: String,
    pub status: String,
    pub is_default: bool,
    pub config: serde_json::Value,
    pub health_status: Option<String>,
    pub health_checked_at: Option<String>,
    pub health_message: Option<String>,
    pub health_details: Option<serde_json::Value>,
    pub disabled_at: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a secret provider config.
#[derive(Debug, Clone)]
pub struct NewSecretProviderConfig {
    pub company_id: String,
    pub provider: String,
    pub display_name: String,
    pub status: String,
    pub is_default: bool,
    pub config: serde_json::Value,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A row of the `company_secret_bindings` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretBindingRecord {
    pub id: String,
    pub company_id: String,
    pub secret_id: String,
    pub target_type: String,
    pub target_id: String,
    pub config_path: String,
    pub version_selector: String,
    pub required: bool,
    pub label: Option<String>,
    pub projection_class: String,
    pub projection_allowlist_key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for setting a secret binding (upsert on the target path).
#[derive(Debug, Clone)]
pub struct NewSecretBinding {
    pub company_id: String,
    pub secret_id: String,
    pub target_type: String,
    pub target_id: String,
    pub config_path: String,
    pub version_selector: String,
    pub required: bool,
    pub label: Option<String>,
    pub projection_class: String,
    pub projection_allowlist_key: Option<String>,
}

/// A row of the `user_secret_definitions` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSecretDefinitionRecord {
    pub id: String,
    pub company_id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub provider: String,
    pub managed_mode: String,
    pub provider_config_id: Option<String>,
    pub provider_metadata: Option<serde_json::Value>,
    pub usage_guidance: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
    pub updated_by_agent_id: Option<String>,
    pub updated_by_user_id: Option<String>,
    pub deleted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a user secret definition.
#[derive(Debug, Clone)]
pub struct NewUserSecretDefinition {
    pub company_id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub provider: String,
    pub managed_mode: String,
    pub provider_config_id: Option<String>,
    pub provider_metadata: Option<serde_json::Value>,
    pub usage_guidance: Option<String>,
    pub created_by_agent_id: Option<String>,
    pub created_by_user_id: Option<String>,
}

/// A row of the `user_secret_declarations` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSecretDeclarationRecord {
    pub id: String,
    pub company_id: String,
    pub user_secret_definition_id: String,
    pub target_type: String,
    pub target_id: String,
    pub config_path: String,
    pub env_key: String,
    pub version_selector: String,
    pub required: bool,
    pub allow_missing_override: bool,
    pub label: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a user secret declaration.
#[derive(Debug, Clone)]
pub struct NewUserSecretDeclaration {
    pub company_id: String,
    pub user_secret_definition_id: String,
    pub target_type: String,
    pub target_id: String,
    pub config_path: String,
    pub env_key: String,
    pub version_selector: String,
    pub required: bool,
    pub allow_missing_override: bool,
    pub label: Option<String>,
}

/// A row of the `secret_access_events` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretAccessEventRecord {
    pub id: String,
    pub company_id: String,
    pub secret_id: Option<String>,
    pub user_secret_definition_id: Option<String>,
    pub secret_scope: String,
    pub version: Option<i64>,
    pub provider: String,
    pub responsible_user_id: Option<String>,
    pub credential_owner_user_id: Option<String>,
    pub credential_subject_type: Option<String>,
    pub credential_subject_id: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub consumer_type: String,
    pub consumer_id: String,
    pub config_path: Option<String>,
    pub issue_id: Option<String>,
    pub heartbeat_run_id: Option<String>,
    pub plugin_id: Option<String>,
    pub outcome: String,
    pub error_code: Option<String>,
    pub created_at: String,
}

/// Input for creating a secret access event.
#[derive(Debug, Clone)]
pub struct NewSecretAccessEvent {
    pub company_id: String,
    pub secret_id: Option<String>,
    pub user_secret_definition_id: Option<String>,
    pub secret_scope: String,
    pub version: Option<i64>,
    pub provider: String,
    pub responsible_user_id: Option<String>,
    pub credential_owner_user_id: Option<String>,
    pub credential_subject_type: Option<String>,
    pub credential_subject_id: Option<String>,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub consumer_type: String,
    pub consumer_id: String,
    pub config_path: Option<String>,
    pub issue_id: Option<String>,
    pub heartbeat_run_id: Option<String>,
    pub plugin_id: Option<String>,
    pub outcome: String,
    pub error_code: Option<String>,
}

/// Secret binding repository errors.
#[derive(Debug, Error)]
pub enum SecretBindingError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The secret does not exist in this company.
    #[error("secret not found")]
    SecretNotFound,
    /// The user secret definition does not exist in this company.
    #[error("user secret definition not found")]
    DefinitionNotFound,
    /// The provider config does not exist in this company.
    #[error("provider config not found")]
    ProviderConfigNotFound,
    /// A referenced record is missing or belongs to another company.
    #[error("reference not found")]
    ReferenceNotFound,
    /// A unique constraint rejected the insert.
    #[error("record already exists")]
    AlreadyExists,
}

/// Secret binding persistence contract.
#[async_trait]
pub trait SecretBindingRepository: Send + Sync {
    /// Creates a secret provider config.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on invalid references or duplicates.
    async fn create_provider_config(
        &self,
        input: NewSecretProviderConfig,
    ) -> Result<SecretProviderConfigRecord, SecretBindingError>;

    /// Lists provider configs for a company.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on database failure.
    async fn list_provider_configs(
        &self,
        company_id: &str,
    ) -> Result<Vec<SecretProviderConfigRecord>, SecretBindingError>;

    /// Sets (upserts) a secret binding for a target path.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on invalid references.
    async fn set_binding(
        &self,
        input: NewSecretBinding,
    ) -> Result<SecretBindingRecord, SecretBindingError>;

    /// Lists secret bindings for a company.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on database failure.
    async fn list_bindings(
        &self,
        company_id: &str,
    ) -> Result<Vec<SecretBindingRecord>, SecretBindingError>;

    /// Creates a user secret definition.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on invalid references or duplicates.
    async fn create_user_secret_definition(
        &self,
        input: NewUserSecretDefinition,
    ) -> Result<UserSecretDefinitionRecord, SecretBindingError>;

    /// Lists user secret definitions for a company.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on database failure.
    async fn list_user_secret_definitions(
        &self,
        company_id: &str,
    ) -> Result<Vec<UserSecretDefinitionRecord>, SecretBindingError>;

    /// Creates a user secret declaration.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on invalid references or duplicates.
    async fn create_user_secret_declaration(
        &self,
        input: NewUserSecretDeclaration,
    ) -> Result<UserSecretDeclarationRecord, SecretBindingError>;

    /// Lists user secret declarations for a company.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on database failure.
    async fn list_user_secret_declarations(
        &self,
        company_id: &str,
    ) -> Result<Vec<UserSecretDeclarationRecord>, SecretBindingError>;

    /// Creates a secret access event.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on invalid references.
    async fn create_access_event(
        &self,
        input: NewSecretAccessEvent,
    ) -> Result<SecretAccessEventRecord, SecretBindingError>;

    /// Lists secret access events for a company.
    ///
    /// # Errors
    ///
    /// Returns [`SecretBindingError`] on database failure.
    async fn list_access_events(
        &self,
        company_id: &str,
    ) -> Result<Vec<SecretAccessEventRecord>, SecretBindingError>;
}

/// Turso/libSQL implementation of [`SecretBindingRepository`].
#[derive(Debug)]
pub struct TursoSecretBindingRepository {
    db: Database,
}

impl TursoSecretBindingRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

const PROVIDER_COLUMNS: &str = "id, company_id, provider, display_name, status, is_default,
                                config, health_status, health_checked_at, health_message,
                                health_details, disabled_at, created_by_agent_id,
                                created_by_user_id, created_at, updated_at";
const BINDING_COLUMNS: &str = "id, company_id, secret_id, target_type, target_id, config_path,
                               version_selector, required, label, projection_class,
                               projection_allowlist_key, created_at, updated_at";
const DEFINITION_COLUMNS: &str = "id, company_id, key, name, description, status, provider,
                                  managed_mode, provider_config_id, provider_metadata,
                                  usage_guidance, created_by_agent_id, created_by_user_id,
                                  updated_by_agent_id, updated_by_user_id, deleted_at,
                                  created_at, updated_at";
const DECLARATION_COLUMNS: &str = "id, company_id, user_secret_definition_id, target_type,
                                   target_id, config_path, env_key, version_selector, required,
                                   allow_missing_override, label, created_at, updated_at";
const EVENT_COLUMNS: &str = "id, company_id, secret_id, user_secret_definition_id, secret_scope,
                             version, provider, responsible_user_id, credential_owner_user_id,
                             credential_subject_type, credential_subject_id, actor_type,
                             actor_id, consumer_type, consumer_id, config_path, issue_id,
                             heartbeat_run_id, plugin_id, outcome, error_code, created_at";

fn parse_json(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw).unwrap_or_default()
}

fn bool_from_int(value: i64) -> bool {
    value != 0
}

fn row_to_provider(row: &libsql::Row) -> Result<SecretProviderConfigRecord, libsql::Error> {
    Ok(SecretProviderConfigRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        provider: helpers::row_text(row, 2)?.expect("provider"),
        display_name: helpers::row_text(row, 3)?.expect("display_name"),
        status: helpers::row_text(row, 4)?.expect("status"),
        is_default: bool_from_int(helpers::row_i64(row, 5)?),
        config: parse_json(&helpers::row_text(row, 6)?.expect("config")),
        health_status: helpers::row_text(row, 7)?,
        health_checked_at: helpers::row_text(row, 8)?,
        health_message: helpers::row_text(row, 9)?,
        health_details: helpers::row_text(row, 10)?.map(|raw| parse_json(&raw)),
        disabled_at: helpers::row_text(row, 11)?,
        created_by_agent_id: helpers::row_text(row, 12)?,
        created_by_user_id: helpers::row_text(row, 13)?,
        created_at: helpers::row_text(row, 14)?.expect("created_at"),
        updated_at: helpers::row_text(row, 15)?.expect("updated_at"),
    })
}

fn row_to_binding(row: &libsql::Row) -> Result<SecretBindingRecord, libsql::Error> {
    Ok(SecretBindingRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        secret_id: helpers::row_text(row, 2)?.expect("secret_id"),
        target_type: helpers::row_text(row, 3)?.expect("target_type"),
        target_id: helpers::row_text(row, 4)?.expect("target_id"),
        config_path: helpers::row_text(row, 5)?.expect("config_path"),
        version_selector: helpers::row_text(row, 6)?.expect("version_selector"),
        required: bool_from_int(helpers::row_i64(row, 7)?),
        label: helpers::row_text(row, 8)?,
        projection_class: helpers::row_text(row, 9)?.expect("projection_class"),
        projection_allowlist_key: helpers::row_text(row, 10)?,
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
        updated_at: helpers::row_text(row, 12)?.expect("updated_at"),
    })
}

fn row_to_definition(row: &libsql::Row) -> Result<UserSecretDefinitionRecord, libsql::Error> {
    Ok(UserSecretDefinitionRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        key: helpers::row_text(row, 2)?.expect("key"),
        name: helpers::row_text(row, 3)?.expect("name"),
        description: helpers::row_text(row, 4)?,
        status: helpers::row_text(row, 5)?.expect("status"),
        provider: helpers::row_text(row, 6)?.expect("provider"),
        managed_mode: helpers::row_text(row, 7)?.expect("managed_mode"),
        provider_config_id: helpers::row_text(row, 8)?,
        provider_metadata: helpers::row_text(row, 9)?.map(|raw| parse_json(&raw)),
        usage_guidance: helpers::row_text(row, 10)?,
        created_by_agent_id: helpers::row_text(row, 11)?,
        created_by_user_id: helpers::row_text(row, 12)?,
        updated_by_agent_id: helpers::row_text(row, 13)?,
        updated_by_user_id: helpers::row_text(row, 14)?,
        deleted_at: helpers::row_text(row, 15)?,
        created_at: helpers::row_text(row, 16)?.expect("created_at"),
        updated_at: helpers::row_text(row, 17)?.expect("updated_at"),
    })
}

fn row_to_declaration(row: &libsql::Row) -> Result<UserSecretDeclarationRecord, libsql::Error> {
    Ok(UserSecretDeclarationRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        user_secret_definition_id: helpers::row_text(row, 2)?.expect("user_secret_definition_id"),
        target_type: helpers::row_text(row, 3)?.expect("target_type"),
        target_id: helpers::row_text(row, 4)?.expect("target_id"),
        config_path: helpers::row_text(row, 5)?.expect("config_path"),
        env_key: helpers::row_text(row, 6)?.expect("env_key"),
        version_selector: helpers::row_text(row, 7)?.expect("version_selector"),
        required: bool_from_int(helpers::row_i64(row, 8)?),
        allow_missing_override: bool_from_int(helpers::row_i64(row, 9)?),
        label: helpers::row_text(row, 10)?,
        created_at: helpers::row_text(row, 11)?.expect("created_at"),
        updated_at: helpers::row_text(row, 12)?.expect("updated_at"),
    })
}

fn row_to_event(row: &libsql::Row) -> Result<SecretAccessEventRecord, libsql::Error> {
    Ok(SecretAccessEventRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        secret_id: helpers::row_text(row, 2)?,
        user_secret_definition_id: helpers::row_text(row, 3)?,
        secret_scope: helpers::row_text(row, 4)?.expect("secret_scope"),
        version: helpers::row_i64_opt(row, 5)?,
        provider: helpers::row_text(row, 6)?.expect("provider"),
        responsible_user_id: helpers::row_text(row, 7)?,
        credential_owner_user_id: helpers::row_text(row, 8)?,
        credential_subject_type: helpers::row_text(row, 9)?,
        credential_subject_id: helpers::row_text(row, 10)?,
        actor_type: helpers::row_text(row, 11)?.expect("actor_type"),
        actor_id: helpers::row_text(row, 12)?,
        consumer_type: helpers::row_text(row, 13)?.expect("consumer_type"),
        consumer_id: helpers::row_text(row, 14)?.expect("consumer_id"),
        config_path: helpers::row_text(row, 15)?,
        issue_id: helpers::row_text(row, 16)?,
        heartbeat_run_id: helpers::row_text(row, 17)?,
        plugin_id: helpers::row_text(row, 18)?,
        outcome: helpers::row_text(row, 19)?.expect("outcome"),
        error_code: helpers::row_text(row, 20)?,
        created_at: helpers::row_text(row, 21)?.expect("created_at"),
    })
}

fn map_insert_error(error: libsql::Error) -> SecretBindingError {
    let message = error.to_string();
    if message.contains("UNIQUE constraint failed") {
        SecretBindingError::AlreadyExists
    } else if message.contains("FOREIGN KEY constraint failed") {
        SecretBindingError::ReferenceNotFound
    } else {
        SecretBindingError::Db(error)
    }
}

#[async_trait]
impl SecretBindingRepository for TursoSecretBindingRepository {
    async fn create_provider_config(
        &self,
        input: NewSecretProviderConfig,
    ) -> Result<SecretProviderConfigRecord, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SecretBindingError::CompanyNotFound);
        }
        if let Some(agent_id) = &input.created_by_agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", agent_id, &input.company_id)
                .await?
        {
            return Err(SecretBindingError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO company_secret_provider_configs
                   (id, company_id, provider, display_name, status, is_default, config,
                    created_by_agent_id, created_by_user_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.provider,
                    input.display_name,
                    input.status,
                    i64::from(input.is_default),
                    input.config.to_string(),
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {PROVIDER_COLUMNS} FROM company_secret_provider_configs WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows
                    .next()
                    .await?
                    .expect("provider config was just inserted");
                Ok(row_to_provider(&row)?)
            }
            Err(error) => Err(map_insert_error(error)),
        }
    }

    async fn list_provider_configs(
        &self,
        company_id: &str,
    ) -> Result<Vec<SecretProviderConfigRecord>, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {PROVIDER_COLUMNS} FROM company_secret_provider_configs
                     WHERE company_id = ?1 ORDER BY provider, display_name"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut configs = Vec::new();
        while let Some(row) = rows.next().await? {
            configs.push(row_to_provider(&row)?);
        }
        Ok(configs)
    }

    async fn set_binding(
        &self,
        input: NewSecretBinding,
    ) -> Result<SecretBindingRecord, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SecretBindingError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "company_secrets",
            &input.secret_id,
            &input.company_id,
        )
        .await?
        {
            return Err(SecretBindingError::SecretNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO company_secret_bindings
               (id, company_id, secret_id, target_type, target_id, config_path,
                version_selector, required, label, projection_class,
                projection_allowlist_key, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, target_type, target_id, config_path) DO UPDATE SET
               secret_id = excluded.secret_id,
               version_selector = excluded.version_selector,
               required = excluded.required,
               label = excluded.label,
               projection_class = excluded.projection_class,
               projection_allowlist_key = excluded.projection_allowlist_key,
               updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.secret_id,
                input.target_type.clone(),
                input.target_id.clone(),
                input.config_path.clone(),
                input.version_selector,
                i64::from(input.required),
                input.label,
                input.projection_class,
                input.projection_allowlist_key
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                "SELECT id FROM company_secret_bindings
                 WHERE company_id = ?1 AND target_type = ?2 AND target_id = ?3 AND config_path = ?4",
                libsql::params![
                    input.company_id,
                    input.target_type,
                    input.target_id,
                    input.config_path
                ],
            )
            .await?;
        let row = rows.next().await?.expect("binding was just upserted");
        let id = helpers::row_text(&row, 0)?.expect("id");
        let mut rows = conn
            .query(
                &format!("SELECT {BINDING_COLUMNS} FROM company_secret_bindings WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("binding was just upserted");
        Ok(row_to_binding(&row)?)
    }

    async fn list_bindings(
        &self,
        company_id: &str,
    ) -> Result<Vec<SecretBindingRecord>, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {BINDING_COLUMNS} FROM company_secret_bindings
                     WHERE company_id = ?1 ORDER BY target_type, target_id, config_path"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut bindings = Vec::new();
        while let Some(row) = rows.next().await? {
            bindings.push(row_to_binding(&row)?);
        }
        Ok(bindings)
    }

    async fn create_user_secret_definition(
        &self,
        input: NewUserSecretDefinition,
    ) -> Result<UserSecretDefinitionRecord, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SecretBindingError::CompanyNotFound);
        }
        if let Some(provider_config_id) = &input.provider_config_id
            && !helpers::row_belongs_to_company(
                &conn,
                "company_secret_provider_configs",
                provider_config_id,
                &input.company_id,
            )
            .await?
        {
            return Err(SecretBindingError::ProviderConfigNotFound);
        }
        if let Some(agent_id) = &input.created_by_agent_id
            && !helpers::row_belongs_to_company(&conn, "agents", agent_id, &input.company_id)
                .await?
        {
            return Err(SecretBindingError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO user_secret_definitions
                   (id, company_id, key, name, description, status, provider, managed_mode,
                    provider_config_id, provider_metadata, usage_guidance,
                    created_by_agent_id, created_by_user_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.key,
                    input.name,
                    input.description,
                    input.status,
                    input.provider,
                    input.managed_mode,
                    input.provider_config_id,
                    input.provider_metadata.map(|value| value.to_string()),
                    input.usage_guidance,
                    input.created_by_agent_id,
                    input.created_by_user_id
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {DEFINITION_COLUMNS} FROM user_secret_definitions WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("definition was just inserted");
                Ok(row_to_definition(&row)?)
            }
            Err(error) => Err(map_insert_error(error)),
        }
    }

    async fn list_user_secret_definitions(
        &self,
        company_id: &str,
    ) -> Result<Vec<UserSecretDefinitionRecord>, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {DEFINITION_COLUMNS} FROM user_secret_definitions
                     WHERE company_id = ?1 AND deleted_at IS NULL ORDER BY key"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut definitions = Vec::new();
        while let Some(row) = rows.next().await? {
            definitions.push(row_to_definition(&row)?);
        }
        Ok(definitions)
    }

    async fn create_user_secret_declaration(
        &self,
        input: NewUserSecretDeclaration,
    ) -> Result<UserSecretDeclarationRecord, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SecretBindingError::CompanyNotFound);
        }
        if !helpers::row_belongs_to_company(
            &conn,
            "user_secret_definitions",
            &input.user_secret_definition_id,
            &input.company_id,
        )
        .await?
        {
            return Err(SecretBindingError::DefinitionNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let result = conn
            .execute(
                "INSERT INTO user_secret_declarations
                   (id, company_id, user_secret_definition_id, target_type, target_id,
                    config_path, env_key, version_selector, required,
                    allow_missing_override, label, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                libsql::params![
                    id.clone(),
                    input.company_id,
                    input.user_secret_definition_id,
                    input.target_type,
                    input.target_id,
                    input.config_path,
                    input.env_key,
                    input.version_selector,
                    i64::from(input.required),
                    i64::from(input.allow_missing_override),
                    input.label
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!(
                            "SELECT {DECLARATION_COLUMNS} FROM user_secret_declarations WHERE id = ?1"
                        ),
                        libsql::params![id],
                    )
                    .await?;
                let row = rows.next().await?.expect("declaration was just inserted");
                Ok(row_to_declaration(&row)?)
            }
            Err(error) => Err(map_insert_error(error)),
        }
    }

    async fn list_user_secret_declarations(
        &self,
        company_id: &str,
    ) -> Result<Vec<UserSecretDeclarationRecord>, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {DECLARATION_COLUMNS} FROM user_secret_declarations
                     WHERE company_id = ?1 ORDER BY target_type, target_id, config_path"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut declarations = Vec::new();
        while let Some(row) = rows.next().await? {
            declarations.push(row_to_declaration(&row)?);
        }
        Ok(declarations)
    }

    async fn create_access_event(
        &self,
        input: NewSecretAccessEvent,
    ) -> Result<SecretAccessEventRecord, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(SecretBindingError::CompanyNotFound);
        }
        if let Some(secret_id) = &input.secret_id
            && !helpers::row_belongs_to_company(
                &conn,
                "company_secrets",
                secret_id,
                &input.company_id,
            )
            .await?
        {
            return Err(SecretBindingError::SecretNotFound);
        }
        if let Some(definition_id) = &input.user_secret_definition_id
            && !helpers::row_belongs_to_company(
                &conn,
                "user_secret_definitions",
                definition_id,
                &input.company_id,
            )
            .await?
        {
            return Err(SecretBindingError::DefinitionNotFound);
        }
        if let Some(issue_id) = &input.issue_id
            && !helpers::row_belongs_to_company(&conn, "issues", issue_id, &input.company_id)
                .await?
        {
            return Err(SecretBindingError::ReferenceNotFound);
        }
        if let Some(run_id) = &input.heartbeat_run_id
            && !helpers::row_belongs_to_company(&conn, "heartbeat_runs", run_id, &input.company_id)
                .await?
        {
            return Err(SecretBindingError::ReferenceNotFound);
        }
        if let Some(plugin_id) = &input.plugin_id
            && !helpers::find_row(&conn, "plugins", plugin_id).await?
        {
            return Err(SecretBindingError::ReferenceNotFound);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO secret_access_events
               (id, company_id, secret_id, user_secret_definition_id, secret_scope, version,
                provider, responsible_user_id, credential_owner_user_id,
                credential_subject_type, credential_subject_id, actor_type, actor_id,
                consumer_type, consumer_id, config_path, issue_id, heartbeat_run_id, plugin_id,
                outcome, error_code, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, ?19, ?20, ?21,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            libsql::params![
                id.clone(),
                input.company_id,
                input.secret_id,
                input.user_secret_definition_id,
                input.secret_scope,
                input.version,
                input.provider,
                input.responsible_user_id,
                input.credential_owner_user_id,
                input.credential_subject_type,
                input.credential_subject_id,
                input.actor_type,
                input.actor_id,
                input.consumer_type,
                input.consumer_id,
                input.config_path,
                input.issue_id,
                input.heartbeat_run_id,
                input.plugin_id,
                input.outcome,
                input.error_code
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!("SELECT {EVENT_COLUMNS} FROM secret_access_events WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("access event was just inserted");
        Ok(row_to_event(&row)?)
    }

    async fn list_access_events(
        &self,
        company_id: &str,
    ) -> Result<Vec<SecretAccessEventRecord>, SecretBindingError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {EVENT_COLUMNS} FROM secret_access_events
                     WHERE company_id = ?1 ORDER BY created_at DESC"
                ),
                libsql::params![company_id],
            )
            .await?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().await? {
            events.push(row_to_event(&row)?);
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoSecretBindingRepository) {
        let dir = TempDir::new().unwrap();
        let db = open(&crate::DbConfig::local(dir.path().join("test.db")))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        let repo = TursoSecretBindingRepository::new(db);
        (dir, repo)
    }

    async fn seed(conn: &crate::Connection) -> String {
        conn.execute(
            "INSERT INTO companies (id, name, issue_prefix, attachment_max_bytes)
             VALUES ('c1', 'Alpha', 'ALPHA', 1024), ('c2', 'Beta', 'BETA', 1024)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO company_secrets (id, company_id, name) VALUES ('sec1', 'c1', 'github_token')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO agents (id, company_id, name, role, adapter_type)
             VALUES ('a1', 'c1', 'Builder', 'senior', 'cli')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO issues (id, company_id, title, issue_number, identifier)
             VALUES ('i1', 'c1', 'Build all', 1, 'ALPHA-1')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO heartbeat_runs (id, company_id, agent_id, invocation_source)
             VALUES ('h1', 'c1', 'a1', 'manual')",
            (),
        )
        .await
        .unwrap();
        "sec1".to_owned()
    }

    #[tokio::test]
    async fn secret_binding_lifecycle_and_dedupe() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();
        seed(&conn).await;

        // Provider configs; default per provider is unique.
        let provider = repo
            .create_provider_config(NewSecretProviderConfig {
                company_id: "c1".to_owned(),
                provider: "aws".to_owned(),
                display_name: "AWS Secrets Manager".to_owned(),
                status: "ready".to_owned(),
                is_default: true,
                config: serde_json::json!({ "region": "us-east-1" }),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(provider.provider, "aws");
        assert!(provider.is_default);
        assert!(matches!(
            repo.create_provider_config(NewSecretProviderConfig {
                company_id: "c1".to_owned(),
                provider: "aws".to_owned(),
                display_name: "Other AWS".to_owned(),
                status: "ready".to_owned(),
                is_default: true,
                config: serde_json::json!({}),
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap_err(),
            SecretBindingError::AlreadyExists
        ));
        assert_eq!(repo.list_provider_configs("c1").await.unwrap().len(), 1);
        assert!(repo.list_provider_configs("c2").await.unwrap().is_empty());

        // Binding upsert: same target path updates instead of duplicating.
        let binding = repo
            .set_binding(NewSecretBinding {
                company_id: "c1".to_owned(),
                secret_id: "sec1".to_owned(),
                target_type: "agent".to_owned(),
                target_id: "a1".to_owned(),
                config_path: "env.GITHUB_TOKEN".to_owned(),
                version_selector: "latest".to_owned(),
                required: true,
                label: Some("GitHub".to_owned()),
                projection_class: "unclassified".to_owned(),
                projection_allowlist_key: None,
            })
            .await
            .unwrap();
        assert_eq!(binding.config_path, "env.GITHUB_TOKEN");
        let updated = repo
            .set_binding(NewSecretBinding {
                company_id: "c1".to_owned(),
                secret_id: "sec1".to_owned(),
                target_type: "agent".to_owned(),
                target_id: "a1".to_owned(),
                config_path: "env.GITHUB_TOKEN".to_owned(),
                version_selector: "v2".to_owned(),
                required: false,
                label: None,
                projection_class: "classified".to_owned(),
                projection_allowlist_key: Some("allow".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(updated.version_selector, "v2");
        assert!(!updated.required);
        assert_eq!(repo.list_bindings("c1").await.unwrap().len(), 1);
        assert!(repo.list_bindings("c2").await.unwrap().is_empty());

        // User secret definition + declaration.
        let definition = repo
            .create_user_secret_definition(NewUserSecretDefinition {
                company_id: "c1".to_owned(),
                key: "gh_pat".to_owned(),
                name: "GitHub PAT".to_owned(),
                description: Some("Personal access token".to_owned()),
                status: "active".to_owned(),
                provider: "local_encrypted".to_owned(),
                managed_mode: "paperclip_managed".to_owned(),
                provider_config_id: Some(provider.id.clone()),
                provider_metadata: None,
                usage_guidance: Some("Use for pushes".to_owned()),
                created_by_agent_id: Some("a1".to_owned()),
                created_by_user_id: None,
            })
            .await
            .unwrap();
        assert_eq!(
            definition.provider_config_id.as_deref(),
            Some(provider.id.as_str())
        );
        assert!(matches!(
            repo.create_user_secret_definition(NewUserSecretDefinition {
                company_id: "c1".to_owned(),
                key: "gh_pat".to_owned(),
                name: "Duplicate".to_owned(),
                description: None,
                status: "active".to_owned(),
                provider: "local_encrypted".to_owned(),
                managed_mode: "paperclip_managed".to_owned(),
                provider_config_id: None,
                provider_metadata: None,
                usage_guidance: None,
                created_by_agent_id: None,
                created_by_user_id: None,
            })
            .await
            .unwrap_err(),
            SecretBindingError::AlreadyExists
        ));

        let declaration = repo
            .create_user_secret_declaration(NewUserSecretDeclaration {
                company_id: "c1".to_owned(),
                user_secret_definition_id: definition.id.clone(),
                target_type: "agent".to_owned(),
                target_id: "a1".to_owned(),
                config_path: "env.GH_PAT".to_owned(),
                env_key: "GH_PAT".to_owned(),
                version_selector: "latest".to_owned(),
                required: true,
                allow_missing_override: false,
                label: Some("PAT".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(declaration.env_key, "GH_PAT");
        assert!(matches!(
            repo.create_user_secret_declaration(NewUserSecretDeclaration {
                company_id: "c1".to_owned(),
                user_secret_definition_id: definition.id.clone(),
                target_type: "agent".to_owned(),
                target_id: "a1".to_owned(),
                config_path: "env.GH_PAT".to_owned(),
                env_key: "OTHER".to_owned(),
                version_selector: "latest".to_owned(),
                required: false,
                allow_missing_override: false,
                label: None,
            })
            .await
            .unwrap_err(),
            SecretBindingError::AlreadyExists
        ));
        assert_eq!(
            repo.list_user_secret_declarations("c1")
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            repo.list_user_secret_definitions("c2")
                .await
                .unwrap()
                .is_empty()
        );

        // Access events.
        let event = repo
            .create_access_event(NewSecretAccessEvent {
                company_id: "c1".to_owned(),
                secret_id: Some("sec1".to_owned()),
                user_secret_definition_id: Some(definition.id.clone()),
                secret_scope: "company".to_owned(),
                version: Some(1),
                provider: "local_encrypted".to_owned(),
                responsible_user_id: None,
                credential_owner_user_id: Some("u1".to_owned()),
                credential_subject_type: Some("agent".to_owned()),
                credential_subject_id: Some("a1".to_owned()),
                actor_type: "agent".to_owned(),
                actor_id: Some("a1".to_owned()),
                consumer_type: "heartbeat".to_owned(),
                consumer_id: "h1".to_owned(),
                config_path: Some("env.GITHUB_TOKEN".to_owned()),
                issue_id: Some("i1".to_owned()),
                heartbeat_run_id: Some("h1".to_owned()),
                plugin_id: None,
                outcome: "granted".to_owned(),
                error_code: None,
            })
            .await
            .unwrap();
        assert_eq!(event.outcome, "granted");
        assert_eq!(event.consumer_type, "heartbeat");
        let events = repo.list_access_events("c1").await.unwrap();
        assert_eq!(events.len(), 1);
        assert!(repo.list_access_events("c2").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn cross_company_rejection() {
        let (_dir, repo) = repo().await;
        let conn = crate::connect(&repo.db).await.unwrap();
        seed(&conn).await;

        // Binding a c1 secret from c2 is rejected.
        assert!(matches!(
            repo.set_binding(NewSecretBinding {
                company_id: "c2".to_owned(),
                secret_id: "sec1".to_owned(),
                target_type: "agent".to_owned(),
                target_id: "a1".to_owned(),
                config_path: "env.TOKEN".to_owned(),
                version_selector: "latest".to_owned(),
                required: true,
                label: None,
                projection_class: "unclassified".to_owned(),
                projection_allowlist_key: None,
            })
            .await
            .unwrap_err(),
            SecretBindingError::SecretNotFound
        ));

        // An access event from c2 referencing c1's secret is rejected.
        assert!(matches!(
            repo.create_access_event(NewSecretAccessEvent {
                company_id: "c2".to_owned(),
                secret_id: Some("sec1".to_owned()),
                user_secret_definition_id: None,
                secret_scope: "company".to_owned(),
                version: None,
                provider: "local_encrypted".to_owned(),
                responsible_user_id: None,
                credential_owner_user_id: None,
                credential_subject_type: None,
                credential_subject_id: None,
                actor_type: "agent".to_owned(),
                actor_id: None,
                consumer_type: "heartbeat".to_owned(),
                consumer_id: "h1".to_owned(),
                config_path: None,
                issue_id: None,
                heartbeat_run_id: None,
                plugin_id: None,
                outcome: "denied".to_owned(),
                error_code: None,
            })
            .await
            .unwrap_err(),
            SecretBindingError::SecretNotFound
        ));
    }
}
