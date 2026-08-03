use std::{error::Error, sync::Arc};

use staple_app::{config::AppConfig, router, state::AppState};
use staple_data::{
    TursoCompanyRepository, TursoGoalRepository, TursoIssueRepository, TursoProjectRepository,
    migrate, open,
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
    migrate(&companies_db).await?;
    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(companies_db)),
        goals: Arc::new(TursoGoalRepository::new(goals_db)),
        projects: Arc::new(TursoProjectRepository::new(projects_db)),
        issues: Arc::new(TursoIssueRepository::new(issues_db)),
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
