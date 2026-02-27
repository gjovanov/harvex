use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use futures_util::stream::Stream;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::error::ApiError;
use crate::state::AppState;
use harvex_services::BatchDao;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/batch", get(list_batches).post(create_batch))
        .route("/batch/{id}", get(get_batch))
        .route("/batch/{id}/process", post(process_batch))
        .route("/batch/{id}/progress", get(batch_progress))
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
    let batch = BatchDao::get_by_id(&state.db, &id)
        .map_err(|_| ApiError::NotFound(format!("Batch {id} not found")))?;
    Ok(Json(serde_json::to_value(batch).unwrap()))
}

/// Start processing a batch. Returns immediately; processing runs in background.
async fn process_batch(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Verify batch exists
    let batch = BatchDao::get_by_id(&state.db, &id)
        .map_err(|_| ApiError::NotFound(format!("Batch {id} not found")))?;

    if batch.status == "processing" {
        return Err(ApiError::BadRequest(
            "Batch is already being processed".into(),
        ));
    }

    // Spawn processing in background
    let pipeline = state.pipeline.clone();
    let batch_id = id.clone();
    tokio::spawn(async move {
        if let Err(e) = pipeline.process_batch(&batch_id).await {
            tracing::error!("Batch processing failed: {e}");
        }
    });

    Ok(Json(json!({
        "status": "processing",
        "batch_id": id,
        "message": "Batch processing started",
    })))
}

/// SSE endpoint for batch processing progress.
async fn batch_progress(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let batch_id = id;
    let rx = state.progress_tx.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        match result {
            Ok(event) if event.batch_id == batch_id => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().data(data)))
            }
            _ => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
