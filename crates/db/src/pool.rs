use duckdb::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::migrations;

#[derive(Clone)]
pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(db_path: &str) -> Result<Self, duckdb::Error> {
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;
        info!("Connected to DuckDB at {}", db_path);

        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        migrations::run(&pool)?;

        Ok(pool)
    }

    pub fn new_in_memory() -> Result<Self, duckdb::Error> {
        let conn = Connection::open_in_memory()?;
        info!("Connected to in-memory DuckDB");

        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        migrations::run(&pool)?;

        Ok(pool)
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("DB mutex poisoned")
    }
}
