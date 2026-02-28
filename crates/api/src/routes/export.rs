use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;
use harvex_services::export::{ExportFilter, ExportService};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/export/json/{batch_id}", get(export_json))
        .route("/export/excel/{batch_id}", get(export_excel))
        .route("/export/csv/{batch_id}", get(export_csv))
}

#[derive(Deserialize, Default)]
struct ExportQuery {
    document_type: Option<String>,
    min_confidence: Option<f64>,
}

impl From<ExportQuery> for ExportFilter {
    fn from(q: ExportQuery) -> Self {
        ExportFilter {
            document_type: q.document_type,
            min_confidence: q.min_confidence,
        }
    }
}

async fn export_json(
    State(state): State<AppState>,
    Path(batch_id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let filter = ExportFilter::from(query);
    let data = ExportService::to_json(&state.db, &batch_id, &filter)
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
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let filter = ExportFilter::from(query);
    let data = ExportService::to_excel(&state.db, &batch_id, &filter)
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

async fn export_csv(
    State(state): State<AppState>,
    Path(batch_id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let filter = ExportFilter::from(query);
    let data = ExportService::to_csv(&state.db, &batch_id, &filter)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let filename = format!("attachment; filename=\"batch_{batch_id}.csv\"");

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, filename),
        ],
        data,
    ))
}
