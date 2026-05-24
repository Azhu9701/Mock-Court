use std::sync::Arc;
use std::sync::RwLock;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{map_api_error, ApiError};
use crate::state::AppState;

const DEFAULT_MODEL_FILE: &str = "data/default-model.json";

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
        .route("/relay", post(set_relay).get(get_relay))
        .route("/relay/test", post(test_relay))
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model: String,
    pub reasoning: String,
}

fn load_model_config() -> ModelConfig {
    if let Ok(content) = std::fs::read_to_string(DEFAULT_MODEL_FILE) {
        if let Ok(config) = serde_json::from_str::<ModelConfig>(&content) {
            return config;
        }
    }
    ModelConfig {
        model: std::env::var("AIONUI_DEFAULT_MODEL").unwrap_or_else(|_| "deepseek-v4-pro".to_string()),
        reasoning: std::env::var("AIONUI_DEFAULT_REASONING").unwrap_or_else(|_| "think".to_string()),
    }
}

fn save_model_config(config: &ModelConfig) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        // ensure parent directory exists
        if let Some(parent) = std::path::Path::new(DEFAULT_MODEL_FILE).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(DEFAULT_MODEL_FILE, json);
    }
}

lazy_static::lazy_static! {
    static ref DEFAULT_MODEL_CONFIG: RwLock<ModelConfig> = RwLock::new(load_model_config());
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
    save_model_config(&config);
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
    has_key: bool,
}

#[derive(Debug, Deserialize)]
struct SetLmStudioKeyRequest {
    key: Option<String>,
}

async fn get_lmstudio_key(
    State(state): State<Arc<AppState>>,
) -> Json<LmStudioKeyResponse> {
    let key = state.engine.gateway().lmstudio_api_key();
    Json(LmStudioKeyResponse { has_key: key.is_some() })
}

async fn set_lmstudio_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetLmStudioKeyRequest>,
) -> Json<LmStudioKeyResponse> {
    let key = body.key.filter(|k| !k.is_empty());
    state.engine.gateway().set_lmstudio_api_key(key);
    let has_key = state.engine.gateway().lmstudio_api_key().is_some();
    Json(LmStudioKeyResponse { has_key })
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
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
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
        _ => return Json(TestProviderResponse { ok: false, message: "未知 provider".into(), latency_ms: None, model: None }),
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

    // LM Studio：主动探测当前加载的模型名
    let detected_model: Option<String> = if provider == foundation::Provider::LMStudio {
        state.engine.gateway().fetch_lmstudio_loaded_model().await
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
        max_tokens: 1024,
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
                    model: detected_model.clone(),
                }),
                Err(_) => Json(TestProviderResponse {
                    ok: false,
                    message: "连接超时（20秒）".into(),
                    latency_ms: Some(latency),
                    model: detected_model.clone(),
                }),
            }
        }
        Err(e) => Json(TestProviderResponse {
            ok: false,
            message: format!("调用失败: {}", e),
            latency_ms: None,
            model: detected_model,
        }),
    }
}

// ── 中转站 (Agent Proxy) 配置 ──

const RELAY_CONFIG_FILE: &str = "data/relay.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelayConfig {
    url: String,
    api_key: String,
}

fn load_relay_config() -> RelayConfig {
    if let Ok(content) = std::fs::read_to_string(RELAY_CONFIG_FILE) {
        if let Ok(config) = serde_json::from_str::<RelayConfig>(&content) {
            return config;
        }
    }
    RelayConfig {
        url: std::env::var("AI_RELAY_URL").unwrap_or_default(),
        api_key: std::env::var("AGENT_PROXY_KEY").unwrap_or_default(),
    }
}

fn save_relay_config(config: &RelayConfig) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        if let Some(parent) = std::path::Path::new(RELAY_CONFIG_FILE).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(RELAY_CONFIG_FILE, json);
    }
}

lazy_static::lazy_static! {
    static ref RELAY_CONFIG: RwLock<RelayConfig> = RwLock::new(load_relay_config());
}

#[derive(Debug, Deserialize)]
struct SetRelayRequest {
    url: String,
    api_key: String,
}

#[derive(Debug, Serialize)]
struct RelayStatus {
    url: String,
    has_key: bool,
    configured: bool,
    block_prod_edit: bool,
}

async fn get_relay() -> Json<RelayStatus> {
    let cfg = RELAY_CONFIG.read().unwrap();
    let block = std::env::var("AI_RELAY_URL").ok().filter(|v| !v.is_empty()).is_some();
    Json(RelayStatus {
        url: cfg.url.clone(),
        has_key: !cfg.api_key.is_empty(),
        configured: !cfg.url.is_empty() && !cfg.api_key.is_empty(),
        block_prod_edit: block,
    })
}

async fn set_relay(
    Json(body): Json<SetRelayRequest>,
) -> Result<Json<RelayStatus>, (axum::http::StatusCode, Json<ApiError>)> {
    // 如果生产环境已通过 env var 设置，禁止前端覆盖
    if std::env::var("AI_RELAY_URL").ok().filter(|v| !v.is_empty()).is_some() {
        let cfg = RELAY_CONFIG.read().unwrap();
        return Ok(Json(RelayStatus {
            url: cfg.url.clone(),
            has_key: !cfg.api_key.is_empty(),
            configured: !cfg.url.is_empty() && !cfg.api_key.is_empty(),
            block_prod_edit: true,
        }));
    }

    let config = RelayConfig {
        url: body.url.trim().to_string(),
        api_key: body.api_key.trim().to_string(),
    };
    if !config.url.is_empty() {
        std::env::set_var("AI_RELAY_URL", &config.url);
    }
    if !config.api_key.is_empty() {
        std::env::set_var("AGENT_PROXY_KEY", &config.api_key);
    }

    let url = config.url.clone();
    let has_key = !config.api_key.is_empty();
    let configured = !config.url.is_empty() && !config.api_key.is_empty();

    save_relay_config(&config);
    *RELAY_CONFIG.write().unwrap() = config;

    Ok(Json(RelayStatus {
        url,
        has_key,
        configured,
        block_prod_edit: false,
    }))
}

async fn test_relay(
    Json(body): Json<SetRelayRequest>,
) -> Json<serde_json::Value> {
    let url = body.url.trim().trim_end_matches('/').to_string();
    let api_key = body.api_key.trim().to_string();
    let models_url = format!("{}/models", url);
    let chat_url = format!("{}/chat/completions", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let start = std::time::Instant::now();

    // Test 1: GET /models
    let models_result = client
        .get(&models_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await;

    // Test 2: POST /chat/completions with a minimal prompt
    let chat_result = client
        .post(&chat_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 5
        }))
        .send()
        .await;

    let latency = start.elapsed().as_millis() as u64;

    let mut models = vec![];
    if let Ok(resp) = models_result {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                    models = data.iter()
                        .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                        .collect();
                }
            }
        }
    }

    let chat_ok = chat_result.map(|r| r.status().is_success()).unwrap_or(false);

    Json(serde_json::json!({
        "ok": chat_ok || !models.is_empty(),
        "models_count": models.len(),
        "models": models.iter().take(10).collect::<Vec<_>>(),
        "chat_ok": chat_ok,
        "latency_ms": latency,
    }))
}
