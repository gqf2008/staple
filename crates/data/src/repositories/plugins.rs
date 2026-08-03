//! Plugin registry, per-company config, company settings, and managed
//! resources repository (upstream plugins.ts family).

use async_trait::async_trait;
use libsql::Database;
use thiserror::Error;
use uuid::Uuid;

use super::helpers;

/// A row of the `plugins` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRecord {
    /// Plugin id.
    pub id: String,
    /// Unique plugin key (derived from the manifest id).
    pub plugin_key: String,
    /// Package name.
    pub package_name: String,
    /// Version.
    pub version: String,
    /// Plugin API version.
    pub api_version: i64,
    /// Categories JSON.
    pub categories: Vec<String>,
    /// Full manifest JSON.
    pub manifest_json: serde_json::Value,
    /// Status.
    pub status: String,
    /// Install order.
    pub install_order: Option<i64>,
    /// Resolved package path.
    pub package_path: Option<String>,
    /// Last error.
    pub last_error: Option<String>,
    /// ISO 8601 installation time.
    pub installed_at: String,
    /// ISO 8601 last update.
    pub updated_at: String,
}

/// A row of the `plugin_config` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfigRecord {
    /// Config id.
    pub id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Company id.
    pub company_id: String,
    /// Config JSON.
    pub config_json: serde_json::Value,
    /// Last error.
    pub last_error: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_company_settings` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCompanySettingRecord {
    /// Setting id.
    pub id: String,
    /// Company id.
    pub company_id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Whether the plugin is enabled for the company.
    pub enabled: bool,
    /// Settings JSON.
    pub settings_json: serde_json::Value,
    /// Last error.
    pub last_error: Option<String>,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// A row of the `plugin_managed_resources` table.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManagedResourceRecord {
    /// Resource id.
    pub id: String,
    /// Company id.
    pub company_id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Plugin key.
    pub plugin_key: String,
    /// Resource kind.
    pub resource_kind: String,
    /// Resource key.
    pub resource_key: String,
    /// Resource id.
    pub resource_id: String,
    /// Defaults JSON.
    pub defaults_json: serde_json::Value,
    /// ISO 8601 creation.
    pub created_at: String,
}

/// Input for registering a plugin.
#[derive(Debug, Clone)]
pub struct NewPlugin {
    /// Unique plugin key.
    pub plugin_key: String,
    /// Package name.
    pub package_name: String,
    /// Version.
    pub version: String,
    /// Plugin API version.
    pub api_version: i64,
    /// Categories.
    pub categories: Vec<String>,
    /// Manifest JSON.
    pub manifest_json: serde_json::Value,
    /// Install order.
    pub install_order: Option<i64>,
    /// Package path.
    pub package_path: Option<String>,
}

/// Input for upserting plugin config.
#[derive(Debug, Clone)]
pub struct UpsertPluginConfig {
    /// Plugin id.
    pub plugin_id: String,
    /// Company id.
    pub company_id: String,
    /// Config JSON.
    pub config_json: serde_json::Value,
}

/// Input for upserting company settings.
#[derive(Debug, Clone)]
pub struct UpsertCompanySettings {
    /// Company id.
    pub company_id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Enabled.
    pub enabled: bool,
    /// Settings JSON.
    pub settings_json: serde_json::Value,
}

/// Input for creating a managed resource.
#[derive(Debug, Clone)]
pub struct NewManagedResource {
    /// Company id.
    pub company_id: String,
    /// Plugin id.
    pub plugin_id: String,
    /// Plugin key.
    pub plugin_key: String,
    /// Resource kind.
    pub resource_kind: String,
    /// Resource key.
    pub resource_key: String,
    /// Resource id.
    pub resource_id: String,
    /// Defaults JSON.
    pub defaults_json: serde_json::Value,
}

/// Plugin repository errors.
#[derive(Debug, Error)]
pub enum PluginError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] libsql::Error),
    /// The database connection could not be established.
    #[error("connection error: {0}")]
    Data(#[from] crate::connection::DataError),
    /// The company does not exist.
    #[error("company not found")]
    CompanyNotFound,
    /// The plugin does not exist.
    #[error("plugin not found")]
    PluginNotFound,
    /// A plugin with this key already exists.
    #[error("plugin already exists")]
    AlreadyExists,
    /// The resource does not exist.
    #[error("resource not found")]
    NotFound,
}

/// Plugin persistence contract.
#[async_trait]
pub trait PluginRepository: Send + Sync {
    /// Registers a plugin (upsert on plugin key).
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn register(&self, input: NewPlugin) -> Result<PluginRecord, PluginError>;

    /// Lists plugins.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn list(&self) -> Result<Vec<PluginRecord>, PluginError>;

    /// Gets a plugin by id.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn get(&self, id: &str) -> Result<Option<PluginRecord>, PluginError>;

    /// Gets a plugin by key.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn get_by_key(&self, plugin_key: &str) -> Result<Option<PluginRecord>, PluginError>;

    /// Updates status/last error.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn update_status(
        &self,
        id: &str,
        status: &str,
        last_error: Option<Option<String>>,
    ) -> Result<Option<PluginRecord>, PluginError>;

    /// Deletes a plugin.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn delete(&self, id: &str) -> Result<Option<PluginRecord>, PluginError>;

    /// Upserts per-company plugin config.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] when plugin or company is missing.
    async fn upsert_config(
        &self,
        input: UpsertPluginConfig,
    ) -> Result<PluginConfigRecord, PluginError>;

    /// Lists per-company configs for a plugin.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn list_configs(&self, plugin_id: &str) -> Result<Vec<PluginConfigRecord>, PluginError>;

    /// Gets a config for a plugin + company.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn get_config(
        &self,
        plugin_id: &str,
        company_id: &str,
    ) -> Result<Option<PluginConfigRecord>, PluginError>;

    /// Upserts company settings.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] when plugin or company is missing.
    async fn upsert_company_settings(
        &self,
        input: UpsertCompanySettings,
    ) -> Result<PluginCompanySettingRecord, PluginError>;

    /// Lists company settings for a plugin.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn list_company_settings(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginCompanySettingRecord>, PluginError>;

    /// Creates a managed resource (upsert on company+plugin+kind+key).
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] when plugin or company is missing.
    async fn upsert_managed_resource(
        &self,
        input: NewManagedResource,
    ) -> Result<PluginManagedResourceRecord, PluginError>;

    /// Lists managed resources for a plugin + company.
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn list_managed_resources(
        &self,
        plugin_id: &str,
        company_id: &str,
    ) -> Result<Vec<PluginManagedResourceRecord>, PluginError>;

    /// Deletes a managed resource (company-scoped).
    ///
    /// # Errors
    ///
    /// Returns [`PluginError`] on database failure.
    async fn delete_managed_resource(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PluginManagedResourceRecord>, PluginError>;
}

/// Turso/libSQL implementation of [`PluginRepository`].
#[derive(Debug)]
pub struct TursoPluginRepository {
    db: Database,
}

impl TursoPluginRepository {
    /// Creates a repository over the given database.
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn row_to_plugin(row: &libsql::Row) -> Result<PluginRecord, libsql::Error> {
    Ok(PluginRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_key: helpers::row_text(row, 1)?.expect("plugin_key"),
        package_name: helpers::row_text(row, 2)?.expect("package_name"),
        version: helpers::row_text(row, 3)?.expect("version"),
        api_version: helpers::row_i64(row, 4)?,
        categories: helpers::row_text(row, 5)?
            .map(|raw| serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default())
            .unwrap_or_default(),
        manifest_json: helpers::row_text(row, 6)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        status: helpers::row_text(row, 7)?.expect("status"),
        install_order: helpers::row_i64_opt(row, 8)?,
        package_path: helpers::row_text(row, 9)?,
        last_error: helpers::row_text(row, 10)?,
        installed_at: helpers::row_text(row, 11)?.expect("installed_at"),
        updated_at: helpers::row_text(row, 12)?.expect("updated_at"),
    })
}

fn row_to_config(row: &libsql::Row) -> Result<PluginConfigRecord, libsql::Error> {
    Ok(PluginConfigRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        plugin_id: helpers::row_text(row, 1)?.expect("plugin_id"),
        company_id: helpers::row_text(row, 2)?.expect("company_id"),
        config_json: helpers::row_text(row, 3)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        last_error: helpers::row_text(row, 4)?,
        created_at: helpers::row_text(row, 5)?.expect("created_at"),
    })
}

fn row_to_setting(row: &libsql::Row) -> Result<PluginCompanySettingRecord, libsql::Error> {
    Ok(PluginCompanySettingRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        plugin_id: helpers::row_text(row, 2)?.expect("plugin_id"),
        enabled: helpers::row_i64(row, 3)? != 0,
        settings_json: helpers::row_text(row, 4)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        last_error: helpers::row_text(row, 5)?,
        created_at: helpers::row_text(row, 6)?.expect("created_at"),
    })
}

fn row_to_resource(row: &libsql::Row) -> Result<PluginManagedResourceRecord, libsql::Error> {
    Ok(PluginManagedResourceRecord {
        id: helpers::row_text(row, 0)?.expect("id"),
        company_id: helpers::row_text(row, 1)?.expect("company_id"),
        plugin_id: helpers::row_text(row, 2)?.expect("plugin_id"),
        plugin_key: helpers::row_text(row, 3)?.expect("plugin_key"),
        resource_kind: helpers::row_text(row, 4)?.expect("resource_kind"),
        resource_key: helpers::row_text(row, 5)?.expect("resource_key"),
        resource_id: helpers::row_text(row, 6)?.expect("resource_id"),
        defaults_json: helpers::row_text(row, 7)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(serde_json::Value::Null),
        created_at: helpers::row_text(row, 8)?.expect("created_at"),
    })
}

const PLUGIN_COLUMNS: &str = "id, plugin_key, package_name, version, api_version, categories,
    manifest_json, status, install_order, package_path, last_error, installed_at, updated_at";

const CONFIG_COLUMNS: &str = "id, plugin_id, company_id, config_json, last_error, created_at";
const SETTING_COLUMNS: &str =
    "id, company_id, plugin_id, enabled, settings_json, last_error, created_at";
const RESOURCE_COLUMNS: &str = "id, company_id, plugin_id, plugin_key, resource_kind, resource_key,
    resource_id, defaults_json, created_at";

#[async_trait]
impl PluginRepository for TursoPluginRepository {
    async fn register(&self, input: NewPlugin) -> Result<PluginRecord, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let id = Uuid::new_v4().to_string();
        let categories =
            serde_json::to_string(&input.categories).unwrap_or_else(|_| "[]".to_owned());
        let manifest = input.manifest_json.to_string();
        let result = conn
            .execute(
                "INSERT INTO plugins
                   (id, plugin_key, package_name, version, api_version, categories,
                    manifest_json, status, install_order, package_path, installed_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'installed', ?8, ?9,
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                         strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT (plugin_key)
                 DO UPDATE SET package_name = excluded.package_name,
                               version = excluded.version,
                               api_version = excluded.api_version,
                               categories = excluded.categories,
                               manifest_json = excluded.manifest_json,
                               install_order = excluded.install_order,
                               package_path = excluded.package_path,
                               status = CASE WHEN plugins.status = 'uninstalled'
                                             THEN 'installed' ELSE plugins.status END,
                               last_error = NULL,
                               updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                libsql::params![
                    id.clone(),
                    input.plugin_key.clone(),
                    input.package_name,
                    input.version,
                    input.api_version,
                    categories,
                    manifest,
                    input.install_order,
                    input.package_path
                ],
            )
            .await;
        match result {
            Ok(_) => {
                let mut rows = conn
                    .query(
                        &format!("SELECT {PLUGIN_COLUMNS} FROM plugins WHERE plugin_key = ?1"),
                        libsql::params![input.plugin_key],
                    )
                    .await?;
                let row = rows.next().await?.expect("plugin was just upserted");
                Ok(row_to_plugin(&row)?)
            }
            Err(error) if error.to_string().contains("UNIQUE constraint failed") => {
                Err(PluginError::AlreadyExists)
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn list(&self) -> Result<Vec<PluginRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {PLUGIN_COLUMNS} FROM plugins ORDER BY install_order, plugin_key"),
                libsql::params![],
            )
            .await?;
        let mut plugins = Vec::new();
        while let Some(row) = rows.next().await? {
            plugins.push(row_to_plugin(&row)?);
        }
        Ok(plugins)
    }

    async fn get(&self, id: &str) -> Result<Option<PluginRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {PLUGIN_COLUMNS} FROM plugins WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_plugin(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_by_key(&self, plugin_key: &str) -> Result<Option<PluginRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {PLUGIN_COLUMNS} FROM plugins WHERE plugin_key = ?1"),
                libsql::params![plugin_key],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_plugin(&row)?)),
            None => Ok(None),
        }
    }

    async fn update_status(
        &self,
        id: &str,
        status: &str,
        last_error: Option<Option<String>>,
    ) -> Result<Option<PluginRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut values: Vec<libsql::Value> = Vec::new();
        values.push(libsql::Value::from(status));
        let mut sets = vec!["status = ?1".to_owned()];
        if let Some(error) = last_error {
            sets.push("last_error = ?2".to_owned());
            values.push(
                error
                    .map(libsql::Value::from)
                    .unwrap_or(libsql::Value::Null),
            );
        }
        values.push(libsql::Value::from(id));
        let where_param = values.len();
        let sql = format!(
            "UPDATE plugins SET {}, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id = ?{where_param}",
            sets.join(", ")
        );
        let updated = conn.execute(&sql, values).await?;
        if updated == 0 {
            return Ok(None);
        }
        let mut rows = conn
            .query(
                &format!("SELECT {PLUGIN_COLUMNS} FROM plugins WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let row = rows.next().await?.expect("plugin exists");
        Ok(Some(row_to_plugin(&row)?))
    }

    async fn delete(&self, id: &str) -> Result<Option<PluginRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {PLUGIN_COLUMNS} FROM plugins WHERE id = ?1"),
                libsql::params![id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_plugin(&row)?;
        conn.execute("DELETE FROM plugins WHERE id = ?1", libsql::params![id])
            .await?;
        Ok(Some(record))
    }

    async fn upsert_config(
        &self,
        input: UpsertPluginConfig,
    ) -> Result<PluginConfigRecord, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(PluginError::CompanyNotFound);
        }
        if !helpers::find_row(&conn, "plugins", &input.plugin_id).await? {
            return Err(PluginError::PluginNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let config = input.config_json.to_string();
        conn.execute(
            "INSERT INTO plugin_config (id, plugin_id, company_id, config_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (plugin_id, company_id)
             DO UPDATE SET config_json = excluded.config_json,
                           last_error = NULL,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![id.clone(), input.plugin_id.clone(), input.company_id.clone(), config],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CONFIG_COLUMNS} FROM plugin_config WHERE plugin_id = ?1 AND company_id = ?2"
                ),
                libsql::params![input.plugin_id, input.company_id],
            )
            .await?;
        let row = rows.next().await?.expect("config was just upserted");
        Ok(row_to_config(&row)?)
    }

    async fn list_configs(&self, plugin_id: &str) -> Result<Vec<PluginConfigRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!("SELECT {CONFIG_COLUMNS} FROM plugin_config WHERE plugin_id = ?1 ORDER BY company_id"),
                libsql::params![plugin_id],
            )
            .await?;
        let mut configs = Vec::new();
        while let Some(row) = rows.next().await? {
            configs.push(row_to_config(&row)?);
        }
        Ok(configs)
    }

    async fn get_config(
        &self,
        plugin_id: &str,
        company_id: &str,
    ) -> Result<Option<PluginConfigRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {CONFIG_COLUMNS} FROM plugin_config WHERE plugin_id = ?1 AND company_id = ?2"
                ),
                libsql::params![plugin_id, company_id],
            )
            .await?;
        match rows.next().await? {
            Some(row) => Ok(Some(row_to_config(&row)?)),
            None => Ok(None),
        }
    }

    async fn upsert_company_settings(
        &self,
        input: UpsertCompanySettings,
    ) -> Result<PluginCompanySettingRecord, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(PluginError::CompanyNotFound);
        }
        if !helpers::find_row(&conn, "plugins", &input.plugin_id).await? {
            return Err(PluginError::PluginNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let settings = input.settings_json.to_string();
        conn.execute(
            "INSERT INTO plugin_company_settings
               (id, company_id, plugin_id, enabled, settings_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, plugin_id)
             DO UPDATE SET enabled = excluded.enabled,
                           settings_json = excluded.settings_json,
                           last_error = NULL,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.plugin_id.clone(),
                i64::from(input.enabled),
                settings
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {SETTING_COLUMNS} FROM plugin_company_settings WHERE company_id = ?1 AND plugin_id = ?2"
                ),
                libsql::params![input.company_id, input.plugin_id],
            )
            .await?;
        let row = rows.next().await?.expect("setting was just upserted");
        Ok(row_to_setting(&row)?)
    }

    async fn list_company_settings(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PluginCompanySettingRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {SETTING_COLUMNS} FROM plugin_company_settings WHERE plugin_id = ?1 ORDER BY company_id"
                ),
                libsql::params![plugin_id],
            )
            .await?;
        let mut settings = Vec::new();
        while let Some(row) = rows.next().await? {
            settings.push(row_to_setting(&row)?);
        }
        Ok(settings)
    }

    async fn upsert_managed_resource(
        &self,
        input: NewManagedResource,
    ) -> Result<PluginManagedResourceRecord, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        if !helpers::company_exists(&conn, &input.company_id).await? {
            return Err(PluginError::CompanyNotFound);
        }
        if !helpers::find_row(&conn, "plugins", &input.plugin_id).await? {
            return Err(PluginError::PluginNotFound);
        }
        let id = Uuid::new_v4().to_string();
        let defaults = input.defaults_json.to_string();
        conn.execute(
            "INSERT INTO plugin_managed_resources
               (id, company_id, plugin_id, plugin_key, resource_kind, resource_key, resource_id,
                defaults_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT (company_id, plugin_id, resource_kind, resource_key)
             DO UPDATE SET resource_id = excluded.resource_id,
                           defaults_json = excluded.defaults_json,
                           updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            libsql::params![
                id.clone(),
                input.company_id.clone(),
                input.plugin_id.clone(),
                input.plugin_key.clone(),
                input.resource_kind.clone(),
                input.resource_key.clone(),
                input.resource_id.clone(),
                defaults
            ],
        )
        .await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RESOURCE_COLUMNS} FROM plugin_managed_resources
                     WHERE company_id = ?1 AND plugin_id = ?2 AND resource_kind = ?3 AND resource_key = ?4"
                ),
                libsql::params![
                    input.company_id,
                    input.plugin_id,
                    input.resource_kind,
                    input.resource_key
                ],
            )
            .await?;
        let row = rows.next().await?.expect("resource was just upserted");
        Ok(row_to_resource(&row)?)
    }

    async fn list_managed_resources(
        &self,
        plugin_id: &str,
        company_id: &str,
    ) -> Result<Vec<PluginManagedResourceRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RESOURCE_COLUMNS} FROM plugin_managed_resources
                     WHERE plugin_id = ?1 AND company_id = ?2 ORDER BY resource_kind, resource_key"
                ),
                libsql::params![plugin_id, company_id],
            )
            .await?;
        let mut resources = Vec::new();
        while let Some(row) = rows.next().await? {
            resources.push(row_to_resource(&row)?);
        }
        Ok(resources)
    }

    async fn delete_managed_resource(
        &self,
        company_id: &str,
        id: &str,
    ) -> Result<Option<PluginManagedResourceRecord>, PluginError> {
        let conn = crate::connection::connect(&self.db).await?;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {RESOURCE_COLUMNS} FROM plugin_managed_resources WHERE company_id = ?1 AND id = ?2"
                ),
                libsql::params![company_id, id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let record = row_to_resource(&row)?;
        conn.execute(
            "DELETE FROM plugin_managed_resources WHERE id = ?1",
            libsql::params![id],
        )
        .await?;
        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{migrate, open};

    async fn repo() -> (TempDir, TursoPluginRepository) {
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
        (dir, TursoPluginRepository::new(db))
    }

    #[tokio::test]
    async fn register_upsert_status_config_settings_resources() {
        let (_dir, repo) = repo().await;
        let plugin = repo
            .register(NewPlugin {
                plugin_key: "acme.tool".to_owned(),
                package_name: "@acme/tool".to_owned(),
                version: "1.0.0".to_owned(),
                api_version: 1,
                categories: vec!["tools".to_owned()],
                manifest_json: serde_json::json!({ "id": "acme.tool", "name": "Tool" }),
                install_order: Some(1),
                package_path: Some("/tmp/tool".to_owned()),
            })
            .await
            .unwrap();
        assert_eq!(plugin.status, "installed");
        let re = repo
            .register(NewPlugin {
                plugin_key: "acme.tool".to_owned(),
                package_name: "@acme/tool".to_owned(),
                version: "1.1.0".to_owned(),
                api_version: 1,
                categories: vec!["tools".to_owned()],
                manifest_json: serde_json::json!({ "id": "acme.tool" }),
                install_order: Some(1),
                package_path: None,
            })
            .await
            .unwrap();
        assert_eq!(re.version, "1.1.0");

        let updated = repo
            .update_status(&plugin.id, "error", Some(Some("boom".to_owned())))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, "error");
        assert_eq!(updated.last_error.as_deref(), Some("boom"));

        let config = repo
            .upsert_config(UpsertPluginConfig {
                plugin_id: plugin.id.clone(),
                company_id: "c1".to_owned(),
                config_json: serde_json::json!({ "token": "x" }),
            })
            .await
            .unwrap();
        assert_eq!(config.config_json["token"], "x");
        assert!(repo.list_configs(&plugin.id).await.unwrap().len() == 1);

        let setting = repo
            .upsert_company_settings(UpsertCompanySettings {
                company_id: "c1".to_owned(),
                plugin_id: plugin.id.clone(),
                enabled: false,
                settings_json: serde_json::json!({ "policy": "strict" }),
            })
            .await
            .unwrap();
        assert!(!setting.enabled);
        assert!(repo.list_company_settings(&plugin.id).await.unwrap().len() == 1);

        let resource = repo
            .upsert_managed_resource(NewManagedResource {
                company_id: "c1".to_owned(),
                plugin_id: plugin.id.clone(),
                plugin_key: "acme.tool".to_owned(),
                resource_kind: "agent".to_owned(),
                resource_key: "defaults".to_owned(),
                resource_id: "r1".to_owned(),
                defaults_json: serde_json::json!({ "mode": "x" }),
            })
            .await
            .unwrap();
        assert_eq!(resource.resource_id, "r1");
        assert!(
            repo.list_managed_resources(&plugin.id, "c1")
                .await
                .unwrap()
                .len()
                == 1
        );
        assert!(
            repo.delete_managed_resource("c1", &resource.id)
                .await
                .unwrap()
                .is_some()
        );
        // Cross-company delete is not found.
        let resource = repo
            .upsert_managed_resource(NewManagedResource {
                company_id: "c1".to_owned(),
                plugin_id: plugin.id.clone(),
                plugin_key: "acme.tool".to_owned(),
                resource_kind: "agent".to_owned(),
                resource_key: "defaults2".to_owned(),
                resource_id: "r2".to_owned(),
                defaults_json: serde_json::json!({}),
            })
            .await
            .unwrap();
        assert!(
            repo.delete_managed_resource("c2", &resource.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(repo.delete(&plugin.id).await.unwrap().is_some());
        assert!(repo.get(&plugin.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn unknown_company_or_plugin_rejected() {
        let (_dir, repo) = repo().await;
        let err = repo
            .upsert_config(UpsertPluginConfig {
                plugin_id: "missing".to_owned(),
                company_id: "c1".to_owned(),
                config_json: serde_json::json!({}),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, PluginError::PluginNotFound));
    }
}
