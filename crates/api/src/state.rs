use std::sync::Arc;

use harvex_config::Settings;
use harvex_db::DbPool;
use harvex_services::{Pipeline, ProgressEvent};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Settings,
    pub pipeline: Arc<Pipeline>,
    pub progress_tx: broadcast::Sender<ProgressEvent>,
}

impl AppState {
    pub fn new(config: Settings, db: DbPool) -> Self {
        let pipeline = Pipeline::new(db.clone(), config.processing.max_concurrent);
        let progress_tx = pipeline.progress_sender();

        Self {
            db,
            config,
            pipeline: Arc::new(pipeline),
            progress_tx,
        }
    }
}
