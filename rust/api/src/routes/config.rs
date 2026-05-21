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
        .route("/provider/test", post(test_provider))
        .route("/providers", get(list_providers))
        .route("/lmstudio-url", post(set_lmstudio_url).get(get_lmstudio_url))
        .route("/lmstudio-key", post(set_lmstudio_key).get(get_lmstudio_key))
        .route("/lmstudio-model", post(set_lmstudio_model).get(get_lmstudio_model))
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

#[derive(Debug, Serialize)]
struct LmStudioUrlResponse {
    url: String,
}

#[derive(Debug, Deserialize)]
struct SetLmStudioUrlRequest {
    url: String,
}

async fn get_lmstudio_url(
    State(state): State<Arc<AppState>>,
) -> Json<LmStudioUrlResponse> {
    let url = state.engine.gateway().lmstudio_base_url();
    Json(LmStudioUrlResponse { url })
}

async fn set_lmstudio_url(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetLmStudioUrlRequest>,
) -> Json<LmStudioUrlResponse> {
    state.engine.gateway().set_lmstudio_base_url(body.url.clone());
    Json(LmStudioUrlResponse { url: body.url })
}

#[derive(Debug, Serialize)]
struct LmStudioKeyResponse {
    key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SetLmStudioKeyRequest {
    key: Option<String>,
}

async fn get_lmstudio_key(
    State(state): State<Arc<AppState>>,
) -> Json<LmStudioKeyResponse> {
    let key = state.engine.gateway().lmstudio_api_key();
    Json(LmStudioKeyResponse { key })
}

async fn set_lmstudio_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetLmStudioKeyRequest>,
) -> Json<LmStudioKeyResponse> {
    let key = body.key.filter(|k| !k.is_empty());
    state.engine.gateway().set_lmstudio_api_key(key.clone());
    Json(LmStudioKeyResponse { key })
}

#[derive(Debug, Serialize)]
struct LmStudioModelResponse {
    model: String,
}

#[derive(Debug, Deserialize)]
struct SetLmStudioModelRequest {
    model: String,
}

async fn get_lmstudio_model(
    State(state): State<Arc<AppState>>,
) -> Json<LmStudioModelResponse> {
    let model = state.engine.gateway().lmstudio_model();
    Json(LmStudioModelResponse { model })
}

async fn set_lmstudio_model(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetLmStudioModelRequest>,
) -> Json<LmStudioModelResponse> {
    state.engine.gateway().set_lmstudio_model(body.model.clone());
    Json(LmStudioModelResponse { model: body.model })
}

#[derive(Debug, Serialize)]
struct ProviderStatus {
    id: String,
    name: String,
    model: String,
    available: bool,
    has_key: bool,
    tier: String,
    active: bool,
}

async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<ProviderStatus>> {
    let preferred = state.preferred_provider.read().unwrap().clone();
    let gateway = state.engine.gateway();

    let statuses: Vec<ProviderStatus> = gateway.list_providers().into_iter().map(|info| {
        let id = format!("{:?}", info.provider).to_lowercase();
        let tier = format!("{:?}", info.tier);
        let has_key = match info.provider {
            foundation::Provider::LMStudio => true,
            _ => info.available,
        };
        ProviderStatus {
            active: preferred == Some(info.provider),
            id,
            name: match info.provider {
                foundation::Provider::Claude => "Claude".into(),
                foundation::Provider::OpenAI => "OpenAI".into(),
                foundation::Provider::DeepSeek => "DeepSeek".into(),
                foundation::Provider::LMStudio => "LM Studio".into(),
            },
            model: info.model,
            available: info.available,
            has_key,
            tier,
        }
    }).collect();

    Json(statuses)
}

#[derive(Debug, Deserialize)]
struct TestProviderRequest {
    provider: String,
    #[serde(default)]
    api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct TestProviderResponse {
    ok: bool,
    message: String,
    latency_ms: Option<u64>,
}

async fn test_provider(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TestProviderRequest>,
) -> Json<TestProviderResponse> {
    let provider = match body.provider.as_str() {
        "claude" | "anthropic" => foundation::Provider::Claude,
        "openai" => foundation::Provider::OpenAI,
        "deepseek" => foundation::Provider::DeepSeek,
        "lmstudio" => foundation::Provider::LMStudio,
        _ => return Json(TestProviderResponse { ok: false, message: "未知 provider".into(), latency_ms: None }),
    };

    // 测试 LM Studio 时，临时设置 API key（如果有提供）
    let _temp_key_guard = if provider == foundation::Provider::LMStudio {
        if let Some(ref key) = body.api_key {
            if !key.is_empty() {
                state.engine.gateway().set_lmstudio_api_key(Some(key.clone()));
            }
        }
        Some(())
    } else {
        None
    };

    let gateway = state.engine.gateway();
    let prompt = foundation::Prompt {
        messages: vec![foundation::PromptMessage {
            role: "user".into(),
            content: "回复「连通测试成功」四个字".into(),
            ..Default::default()
        }],
    };
    let config = foundation::CallConfig {
        temperature: 0.0,
        max_tokens: 64,
        stream: false,
        model: None,
        tools: None,
        tool_choice: None,
        reasoning_effort: None,
        structured_output: None,
        thinking_enabled: None,
    };
    let req = foundation::LLMRequest { provider: provider.clone(), prompt, config };

    let start = std::time::Instant::now();
    match gateway.call(&req) {
        Ok(mut rx) => {
            let mut content = String::new();
            let deadline = tokio::time::timeout(std::time::Duration::from_secs(20), async {
                while let Some(chunk_result) = rx.recv().await {
                    match chunk_result {
                        Ok(chunk) => {
                            if !chunk.content.is_empty() {
                                content.push_str(&chunk.content);
                            } else if let Some(ref rc) = chunk.reasoning_content {
                                // DeepSeek 思考模型：内容在 reasoning_content 里
                                content.push_str(rc);
                            }
                            if chunk.finish_reason.is_some() {
                                break;
                            }
                        }
                        Err(e) => {
                            content = format!("错误: {}", e);
                            break;
                        }
                    }
                }
            }).await;

            let latency = start.elapsed().as_millis() as u64;
            match deadline {
                Ok(()) => Json(TestProviderResponse {
                    ok: !content.is_empty(),
                    message: if content.is_empty() { "连接成功但未接收到内容".into() } else { content },
                    latency_ms: Some(latency),
                }),
                Err(_) => Json(TestProviderResponse {
                    ok: false,
                    message: "连接超时（20秒）".into(),
                    latency_ms: Some(latency),
                }),
            }
        }
        Err(e) => Json(TestProviderResponse {
            ok: false,
            message: format!("调用失败: {}", e),
            latency_ms: None,
        }),
    }
}
