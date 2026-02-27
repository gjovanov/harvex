use harvex_api::{build_router, state::AppState};
use harvex_config::Settings;
use harvex_db::DbPool;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,harvex_api=debug,harvex_services=debug")
        }))
        .init();

    let config = Settings::load()?;
    info!("Starting Harvex on {}:{}", config.server.host, config.server.port);

    // Ensure upload directory exists
    std::fs::create_dir_all(&config.storage.upload_dir)?;

    let db = DbPool::new(&config.database.path)?;
    let state = AppState::new(config.clone(), db);
    let app = build_router(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
