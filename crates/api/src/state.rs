use std::sync::Arc;

use harvex_config::Settings;
use harvex_db::DbPool;
use harvex_services::{LlmEngine, Pipeline, ProgressEvent};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Settings,
    pub pipeline: Arc<Pipeline>,
    pub llm: Arc<LlmEngine>,
    pub progress_tx: broadcast::Sender<ProgressEvent>,
}

impl AppState {
    pub fn new(config: Settings, db: DbPool) -> Self {
        let pipeline = Pipeline::new(
            db.clone(),
            config.processing.max_concurrent,
            config.llm.clone(),
        );
        let progress_tx = pipeline.progress_sender();
        let llm = pipeline.llm_engine();

        Self {
            db,
            config,
            pipeline: Arc::new(pipeline),
            llm,
            progress_tx,
        }
    }
}
