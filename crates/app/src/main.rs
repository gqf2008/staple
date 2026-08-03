use std::{error::Error, sync::Arc};

use staple_adapters::{
    AdapterRegistry, CliAdapter, CliAdapterConfig, PluginError, PluginManifest, PluginReport,
};
use staple_app::storage::LocalStorage;
use staple_app::{config::AppConfig, router, state::AppState};
use staple_data::{
    SecretCipher, TursoActivityRepository, TursoAgentRepository, TursoApiKeyRepository,
    TursoApprovalRepository, TursoAssetRepository, TursoCompanyRepository, TursoCostRepository,
    TursoDecisionRepository, TursoDocumentRepository, TursoEnvironmentRepository,
    TursoExternalObjectRepository, TursoGoalRepository, TursoHeartbeatRepository,
    TursoIssueCommentRepository, TursoIssueRelationRepository, TursoIssueRepository,
    TursoIssueStructureRepository, TursoLabelRepository, TursoPermissionGrantRepository,
    TursoProjectRepository, TursoRoutineRepository, TursoSecretRepository, TursoSkillRepository,
    TursoWorkProductRepository, TursoWorkspaceRepository, default_key_path, migrate, open,
};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::from_env()?;
    init_logging(&config);

    let db_config = staple_data::DbConfig::from_env();
    let companies_db = open(&db_config).await?;
    let agents_db = open(&db_config).await?;
    let permission_grants_db = open(&db_config).await?;
    let goals_db = open(&db_config).await?;
    let projects_db = open(&db_config).await?;
    let issues_db = open(&db_config).await?;
    let comments_db = open(&db_config).await?;
    let documents_db = open(&db_config).await?;
    let assets_db = open(&db_config).await?;
    let relations_db = open(&db_config).await?;
    let work_products_db = open(&db_config).await?;
    let heartbeat_db = open(&db_config).await?;
    let costs_db = open(&db_config).await?;
    let approvals_db = open(&db_config).await?;
    let activity_db = open(&db_config).await?;
    let secrets_db = open(&db_config).await?;
    let api_keys_db = open(&db_config).await?;
    let decisions_db = open(&db_config).await?;
    let external_objects_db = open(&db_config).await?;
    let skills_db = open(&db_config).await?;
    let environments_db = open(&db_config).await?;
    let workspaces_db = open(&db_config).await?;
    let labels_db = open(&db_config).await?;
    let issue_structure_db = open(&db_config).await?;
    let routines_db = open(&db_config).await?;
    migrate(&companies_db).await?;
    let secret_cipher = SecretCipher::load_or_create(default_key_path())
        .map_err(|error| Box::<dyn Error>::from(error.to_string()))?;
    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(companies_db)),
        agents: Arc::new(TursoAgentRepository::new(agents_db)),
        permission_grants: Arc::new(TursoPermissionGrantRepository::new(permission_grants_db)),
        goals: Arc::new(TursoGoalRepository::new(goals_db)),
        projects: Arc::new(TursoProjectRepository::new(projects_db)),
        issues: Arc::new(TursoIssueRepository::new(issues_db)),
        comments: Arc::new(TursoIssueCommentRepository::new(comments_db)),
        documents: Arc::new(TursoDocumentRepository::new(documents_db)),
        assets: Arc::new(TursoAssetRepository::new(assets_db)),
        relations: Arc::new(TursoIssueRelationRepository::new(relations_db)),
        storage: LocalStorage::new("data/uploads"),
        work_products: Arc::new(TursoWorkProductRepository::new(work_products_db)),
        heartbeat: Arc::new(TursoHeartbeatRepository::new(heartbeat_db)),
        costs: Arc::new(TursoCostRepository::new(costs_db)),
        approvals: Arc::new(TursoApprovalRepository::new(approvals_db)),
        activity: Arc::new(TursoActivityRepository::new(activity_db)),
        secrets: Arc::new(TursoSecretRepository::new(secrets_db, secret_cipher)),
        api_keys: Arc::new(TursoApiKeyRepository::new(api_keys_db)),
        decisions: Arc::new(TursoDecisionRepository::new(decisions_db)),
        external_objects: Arc::new(TursoExternalObjectRepository::new(external_objects_db)),
        skills: Arc::new(TursoSkillRepository::new(skills_db)),
        environments: Arc::new(TursoEnvironmentRepository::new(environments_db)),
        workspaces: Arc::new(TursoWorkspaceRepository::new(workspaces_db)),
        labels: Arc::new(TursoLabelRepository::new(labels_db)),
        issue_structure: Arc::new(TursoIssueStructureRepository::new(issue_structure_db)),
        routines: Arc::new(TursoRoutineRepository::new(routines_db)),
        adapters: Arc::new({
            let mut registry = AdapterRegistry::new();
            registry.register(Box::new(CliAdapter::new(CliAdapterConfig::default())));
            registry
        }),
        plugin_reports: load_plugin_reports(),
    };

    let listener = TcpListener::bind((config.host.as_str(), config.port)).await?;
    tracing::info!(host = %config.host, port = config.port, "staple listening");

    topcoat::serve(listener, router(state)).await?;
    Ok(())
}

/// Loads external adapter plugins from `STAPLE_ADAPTER_PLUGINS` (or the
/// default `~/.paperclip/adapter-plugins.json` when present) and registers
/// them into the app's adapter registry. Returns explicit diagnostics.
fn load_plugin_reports() -> Vec<PluginReport> {
    let path = std::env::var("STAPLE_ADAPTER_PLUGINS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_owned());
            std::path::PathBuf::from(home)
                .join(".paperclip")
                .join("adapter-plugins.json")
        });
    if !path.exists() {
        return Vec::new();
    }
    match PluginManifest::load(&path) {
        Ok(manifest) => {
            let mut registry = AdapterRegistry::new();
            manifest.register_into(&mut registry)
        }
        Err(error) => vec![PluginReport {
            r#type: "*manifest*".to_owned(),
            loaded: false,
            error: Some(match error {
                PluginError::Read(path, source) => {
                    format!("cannot read {path}: {source}")
                }
                PluginError::InvalidJson(message) => format!("invalid JSON: {message}"),
                PluginError::UnsupportedContract(version) => {
                    format!("unsupported contract version {version}")
                }
            }),
        }],
    }
}

/// Initializes `tracing` with the configured `EnvFilter` directive.
fn init_logging(config: &AppConfig) {
    let filter = EnvFilter::try_new(&config.log_filter).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
