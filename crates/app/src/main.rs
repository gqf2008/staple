use std::{error::Error, sync::Arc};

use staple_app::{config::AppConfig, router, state::AppState};
use staple_data::{TursoCompanyRepository, migrate, open};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::from_env()?;
    init_logging(&config);

    let db = open(&staple_data::DbConfig::from_env()).await?;
    migrate(&db).await?;
    let state = AppState {
        companies: Arc::new(TursoCompanyRepository::new(db)),
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
