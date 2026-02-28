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
        .route("/model/settings", post(update_settings))
        .route("/model/health", get(check_llm_health))
        .route("/model/list", get(list_models))
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
        "vision_model_name": settings.vision_model_name,
        "vision_dpi": settings.vision_dpi,
        "vision_max_pages": settings.vision_max_pages,
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

#[derive(Deserialize)]
struct UpdateSettingsRequest {
    api_url: Option<String>,
    api_key: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    context_size: Option<u32>,
    vision_model_name: Option<String>,
}

/// Update LLM settings at runtime.
async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<Value>, ApiError> {
    state.llm.update_settings(
        body.api_url.as_deref(),
        body.api_key.as_deref(),
        body.temperature,
        body.max_tokens,
        body.context_size,
        body.vision_model_name.as_deref(),
    );

    let settings = state.llm.settings();
    Ok(Json(json!({
        "message": "Settings updated",
        "current": {
            "model_name": settings.model_name,
            "api_url": settings.api_url,
            "context_size": settings.context_size,
            "temperature": settings.temperature,
            "max_tokens": settings.max_tokens,
            "vision_model_name": settings.vision_model_name,
            "vision_dpi": settings.vision_dpi,
            "vision_max_pages": settings.vision_max_pages,
        }
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

/// List available models from the LLM API (e.g., Ollama).
async fn list_models(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let models = state
        .llm
        .list_models()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let current = state.llm.model_name();

    Ok(Json(json!({
        "current_model": current,
        "available": models,
    })))
}
