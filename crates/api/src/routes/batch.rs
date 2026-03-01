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
use harvex_services::{BatchDao, DocumentDao, ExtractionDao};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/batch", get(list_batches).post(create_batch))
        .route("/batch/{id}", get(get_batch).delete(delete_batch))
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
        // Reset stuck batch (e.g. pod restarted mid-processing)
        tracing::warn!("Resetting stuck batch {} from 'processing' to 'pending'", id);
        BatchDao::update_status(&state.db, &id, "pending")
            .map_err(|e| ApiError::Internal(format!("Failed to reset batch: {e}")))?;
        // Also reset any stuck documents
        let docs = DocumentDao::list_by_batch(&state.db, &id)
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        for doc in &docs {
            if doc.status == "processing" {
                let _ = DocumentDao::update_status(&state.db, &doc.id, "pending", None);
            }
        }
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

/// Delete a batch and all its documents, extractions, and files.
async fn delete_batch(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Verify batch exists
    let batch = BatchDao::get_by_id(&state.db, &id)
        .map_err(|_| ApiError::NotFound(format!("Batch {id} not found")))?;

    // Allow deleting stuck "processing" batches (e.g. orphaned after pod restart)

    // Cascade: delete extractions → delete documents (get file paths) → delete files → delete batch
    ExtractionDao::delete_by_batch(&state.db, &id)
        .map_err(|e| ApiError::Internal(format!("Failed to delete extractions: {e}")))?;

    let file_paths = DocumentDao::delete_by_batch(&state.db, &id)
        .map_err(|e| ApiError::Internal(format!("Failed to delete documents: {e}")))?;

    // Clean up files from disk
    for path in &file_paths {
        let _ = std::fs::remove_file(path);
    }

    // Remove batch upload directory
    let batch_dir = format!("{}/{}", state.config.storage.upload_dir, id);
    let _ = std::fs::remove_dir_all(&batch_dir);

    // Delete batch record
    BatchDao::delete(&state.db, &id)
        .map_err(|e| ApiError::Internal(format!("Failed to delete batch: {e}")))?;

    Ok(Json(json!({
        "message": "Batch deleted successfully",
        "batch_id": id,
        "files_removed": file_paths.len(),
    })))
}
