pub mod error;
pub mod routes;
pub mod state;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let max_body = state.config.storage.max_file_size_mb as usize * 1024 * 1024;

    Router::new()
        .merge(routes::health::routes())
        .nest("/api", routes::api_routes())
        .layer(DefaultBodyLimit::max(max_body))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
