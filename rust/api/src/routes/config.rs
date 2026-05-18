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
        .route("/provider", post(set_provider).get(get_provider))
        .route("/balance", get(check_balance))
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

#[derive(Debug, Deserialize)]
struct SetProviderRequest {
    provider: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProviderResponse {
    provider: Option<String>,
}

async fn get_provider(
    State(state): State<Arc<AppState>>,
) -> Json<ProviderResponse> {
    let provider = state.preferred_provider.read().unwrap().map(|p| format!("{:?}", p).to_lowercase());
    Json(ProviderResponse { provider })
}

async fn set_provider(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetProviderRequest>,
) -> Result<Json<ProviderResponse>, (axum::http::StatusCode, Json<ApiError>)> {
    let p = match body.provider.as_deref() {
        Some("claude") | Some("anthropic") => Some(foundation::Provider::Claude),
        Some("openai") => Some(foundation::Provider::OpenAI),
        Some("deepseek") => Some(foundation::Provider::DeepSeek),
        Some("lmstudio") => Some(foundation::Provider::LMStudio),
        Some("") | None => None,
        _ => return Err((axum::http::StatusCode::BAD_REQUEST, Json(ApiError { error: "未知 provider".into() }))),
    };
    {
        let mut pref = state.preferred_provider.write().unwrap();
        *pref = p;
    }
    state.engine.gateway().set_preferred_provider(p);
    let provider = p.map(|p| format!("{:?}", p).to_lowercase());
    Ok(Json(ProviderResponse { provider }))
}

async fn check_balance(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<ApiError>)> {
    let balance = state.engine.gateway().check_deepseek_balance().await.map_err(map_api_error)?;
    Ok(Json(balance))
}
