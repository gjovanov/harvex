pub mod error;
pub mod routes;
pub mod state;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(routes::health::routes())
        .nest("/api", routes::api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
