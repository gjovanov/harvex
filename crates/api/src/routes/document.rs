use axum::extract::{Multipart, Path, Query, State};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::error::ApiError;
use crate::state::AppState;
use harvex_services::{BatchDao, DocumentDao};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/document/upload", post(upload_documents))
        .route("/document", get(list_documents))
        .route("/document/{id}", get(get_document).delete(delete_document))
}

#[derive(Deserialize)]
struct ListDocumentsQuery {
    batch_id: String,
}

async fn upload_documents(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    let upload_dir = &state.config.storage.upload_dir;
    let max_size = state.config.storage.max_file_size_mb * 1024 * 1024;

    let mut batch_name: Option<String> = None;
    let mut model_name: Option<String> = None;
    let mut files: Vec<(String, String, Vec<u8>)> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "batch_name" => {
                batch_name = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| ApiError::BadRequest(e.to_string()))?,
                );
            }
            "model_name" => {
                model_name = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| ApiError::BadRequest(e.to_string()))?,
                );
            }
            "files" | "files[]" => {
                let file_name = field
                    .file_name()
                    .unwrap_or("unknown")
                    .to_string();
                let content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

                if data.len() as u64 > max_size {
                    return Err(ApiError::BadRequest(format!(
                        "File {} exceeds max size of {} MB",
                        file_name, state.config.storage.max_file_size_mb
                    )));
                }

                files.push((file_name, content_type, data.to_vec()));
            }
            _ => {}
        }
    }

    if files.is_empty() {
        return Err(ApiError::BadRequest("No files provided".to_string()));
    }

    let batch_name = batch_name.unwrap_or_else(|| {
        format!(
            "Batch {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M")
        )
    });

    // Create batch
    let batch = BatchDao::create(&state.db, &batch_name, model_name.as_deref())?;
    let batch_dir = PathBuf::from(upload_dir).join(&batch.id);
    std::fs::create_dir_all(&batch_dir)?;

    let mut documents = Vec::new();

    for (original_name, content_type, data) in &files {
        let stored_name = format!("{}_{}", nanoid::nanoid!(10), original_name);
        let file_path = batch_dir.join(&stored_name);
        std::fs::write(&file_path, data)?;

        let doc = DocumentDao::create(
            &state.db,
            &batch.id,
            &stored_name,
            original_name,
            content_type,
            data.len() as i64,
            file_path.to_str().unwrap_or(""),
        )?;
        documents.push(doc);
    }

    BatchDao::set_total_files(&state.db, &batch.id, documents.len() as i32)?;

    Ok(Json(json!({
        "batch": batch,
        "documents": documents,
    })))
}

async fn list_documents(
    State(state): State<AppState>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Value>, ApiError> {
    let docs = DocumentDao::list_by_batch(&state.db, &query.batch_id)?;
    Ok(Json(json!(docs)))
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let doc = DocumentDao::get_by_id(&state.db, &id)
        .map_err(|_| ApiError::NotFound(format!("Document {id} not found")))?;
    Ok(Json(serde_json::to_value(doc).unwrap()))
}

async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Get document to find file path
    let doc = DocumentDao::get_by_id(&state.db, &id)
        .map_err(|_| ApiError::NotFound(format!("Document {id} not found")))?;

    // Delete file from disk
    let _ = std::fs::remove_file(&doc.file_path);

    // Delete from DB
    DocumentDao::delete(&state.db, &id)?;

    Ok(Json(json!({"deleted": true, "id": id})))
}
