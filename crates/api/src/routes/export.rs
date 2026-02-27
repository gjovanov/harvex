use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use crate::error::ApiError;
use crate::state::AppState;
use harvex_services::export::ExportService;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/export/json/{batch_id}", get(export_json))
        .route("/export/excel/{batch_id}", get(export_excel))
}

async fn export_json(
    State(state): State<AppState>,
    Path(batch_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let data = ExportService::to_json(&state.db, &batch_id)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let filename = format!("attachment; filename=\"batch_{batch_id}.json\"");

    Ok((
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (header::CONTENT_DISPOSITION, filename),
        ],
        data,
    ))
}

async fn export_excel(
    State(state): State<AppState>,
    Path(batch_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let data = ExportService::to_excel(&state.db, &batch_id)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let filename = format!("attachment; filename=\"batch_{batch_id}.xlsx\"");

    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            ),
            (header::CONTENT_DISPOSITION, filename),
        ],
        data,
    ))
}
