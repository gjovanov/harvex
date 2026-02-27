use axum::extract::{Path, State};
use axum::routing::get;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::AppState;
use harvex_services::BatchDao;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/batch", get(list_batches).post(create_batch))
        .route("/batch/{id}", get(get_batch))
}

#[derive(Deserialize)]
struct CreateBatchRequest {
    name: String,
    model_name: Option<String>,
}

async fn create_batch(
    State(state): State<AppState>,
    Json(body): Json<CreateBatchRequest>,
) -> Result<Json<Value>, ApiError> {
    let batch = BatchDao::create(&state.db, &body.name, body.model_name.as_deref())?;
    Ok(Json(serde_json::to_value(batch).unwrap()))
}

async fn list_batches(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let batches = BatchDao::list(&state.db)?;
    Ok(Json(json!(batches)))
}

async fn get_batch(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let batch = BatchDao::get_by_id(&state.db, &id).map_err(|_| {
        ApiError::NotFound(format!("Batch {id} not found"))
    })?;
    Ok(Json(serde_json::to_value(batch).unwrap()))
}
