pub mod batch;
pub mod document;
pub mod export;
pub mod extraction;
pub mod health;

use axum::Router;

use crate::state::AppState;

pub fn api_routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .merge(document::routes())
        .merge(batch::routes())
        .merge(extraction::routes())
        .merge(export::routes())
}
