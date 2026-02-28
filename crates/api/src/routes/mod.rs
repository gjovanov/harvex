pub mod batch;
pub mod document;
pub mod export;
pub mod extraction;
pub mod health;
pub mod model;

use axum::Router;

use crate::state::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(document::routes())
        .merge(batch::routes())
        .merge(extraction::routes())
        .merge(export::routes())
        .merge(model::routes())
}
