use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::{json, Value};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "harvex",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
