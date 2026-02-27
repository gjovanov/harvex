use harvex_config::Settings;
use harvex_db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Settings,
}

impl AppState {
    pub fn new(config: Settings, db: DbPool) -> Self {
        Self { db, config }
    }
}
