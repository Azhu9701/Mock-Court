use std::sync::Arc;
use std::sync::RwLock;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/model", post(set_default_model).get(get_default_model))
}

#[derive(Debug, Clone, Default)]
pub struct ModelConfig {
    pub model: String,
    pub reasoning: String,
}

lazy_static::lazy_static! {
    static ref DEFAULT_MODEL_CONFIG: RwLock<ModelConfig> = RwLock::new(ModelConfig {
        model: std::env::var("AIONUI_DEFAULT_MODEL").unwrap_or_else(|_| "deepseek-v4-pro".to_string()),
        reasoning: std::env::var("AIONUI_DEFAULT_REASONING").unwrap_or_else(|_| "think".to_string()),
    });
}

#[derive(Debug, Deserialize)]
struct SetModelRequest {
    model: String,
    #[serde(default = "default_reasoning")]
    reasoning: String,
}

fn default_reasoning() -> String {
    std::env::var("AIONUI_DEFAULT_REASONING").unwrap_or_else(|_| "think".to_string())
}

#[derive(Debug, Serialize)]
struct ModelResponse {
    model: String,
    reasoning: String,
}

async fn get_default_model() -> Json<ModelResponse> {
    let config = DEFAULT_MODEL_CONFIG.read().unwrap();
    Json(ModelResponse {
        model: config.model.clone(),
        reasoning: config.reasoning.clone(),
    })
}

async fn set_default_model(
    Json(body): Json<SetModelRequest>,
) -> Result<Json<ModelResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let mut config = DEFAULT_MODEL_CONFIG.write().unwrap();
    config.model = body.model;
    config.reasoning = body.reasoning;
    Ok(Json(ModelResponse {
        model: config.model.clone(),
        reasoning: config.reasoning.clone(),
    }))
}
