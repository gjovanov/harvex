use axum::extract::State;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/model", get(get_model_info))
        .route("/model/switch", post(switch_model))
        .route("/model/health", get(check_llm_health))
}

/// Get current LLM model info and settings.
async fn get_model_info(State(state): State<AppState>) -> Json<Value> {
    let settings = state.llm.settings();
    Json(json!({
        "model_name": settings.model_name,
        "api_url": settings.api_url,
        "context_size": settings.context_size,
        "temperature": settings.temperature,
        "max_tokens": settings.max_tokens,
    }))
}

#[derive(Deserialize)]
struct SwitchModelRequest {
    model_name: String,
}

/// Switch to a different LLM model.
async fn switch_model(
    State(state): State<AppState>,
    Json(body): Json<SwitchModelRequest>,
) -> Result<Json<Value>, ApiError> {
    let old_name = state.llm.model_name();
    state.llm.switch_model(&body.model_name);

    Ok(Json(json!({
        "previous_model": old_name,
        "current_model": body.model_name,
        "message": "Model switched successfully",
    })))
}

/// Check if the LLM API is reachable.
async fn check_llm_health(State(state): State<AppState>) -> Json<Value> {
    let reachable = state.llm.health_check().await.unwrap_or(false);
    let settings = state.llm.settings();

    Json(json!({
        "model_name": settings.model_name,
        "api_url": settings.api_url,
        "reachable": reachable,
    }))
}
