use axum::extract::{Path, State};
use axum::routing::get;
use axum::Json;
use axum::Router;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::AppState;
use harvex_services::ExtractionDao;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/batch/{batch_id}/extraction", get(list_extractions))
        .route(
            "/batch/{batch_id}/extraction/{extraction_id}",
            get(get_extraction),
        )
}

async fn list_extractions(
    State(state): State<AppState>,
    Path(batch_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let extractions = ExtractionDao::list_by_batch(&state.db, &batch_id)?;
    Ok(Json(json!(extractions)))
}

async fn get_extraction(
    State(state): State<AppState>,
    Path((_batch_id, extraction_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let extraction = ExtractionDao::get_by_id(&state.db, &extraction_id)
        .map_err(|_| ApiError::NotFound(format!("Extraction {extraction_id} not found")))?;
    Ok(Json(serde_json::to_value(extraction).unwrap()))
}
