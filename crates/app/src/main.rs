use std::{error::Error, sync::Arc};

use staple_app::storage::LocalStorage;
use staple_app::{config::AppConfig, router, state::AppState};
use staple_data::{
    SecretCipher, TursoActivityRepository, TursoApiKeyRepository, TursoApprovalRepository,
    TursoAssetRepository, TursoCompanyRepository, TursoCostRepository, TursoDecisionRepository,
    TursoDocumentRepository, TursoExternalObjectRepository, TursoGoalRepository,
    TursoHeartbeatRepository, TursoIssueCommentRepository, TursoIssueRelationRepository,
    TursoIssueRepository, TursoProjectRepository, TursoSecretRepository, TursoSkillRepository,
    TursoWorkProductRepository, default_key_path, migrate, open,
};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::from_env()?;
    init_logging(&config);

    let db_config = staple_data::DbConfig::from_env();
    let companies_db = open(&db_config).await?;
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
    migrate(&companies_db).await?;
    let secret_cipher = SecretCipher::load_or_create(default_key_path())
        .map_err(|error| Box::<dyn Error>::from(error.to_string()))?;
    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(companies_db)),
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
    };

    let listener = TcpListener::bind((config.host.as_str(), config.port)).await?;
    tracing::info!(host = %config.host, port = config.port, "staple listening");

    topcoat::serve(listener, router(state)).await?;
    Ok(())
}

/// Initializes `tracing` with the configured `EnvFilter` directive.
fn init_logging(config: &AppConfig) {
    let filter = EnvFilter::try_new(&config.log_filter).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
