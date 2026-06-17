mod claude;
mod deepseek;
pub mod cache;
pub mod model_router;
mod openai;
pub mod prompt;
mod lmstudio;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

use foundation::{
    CallConfig, Chunk, FoundationError, LLMRequest, ModelTier, Prompt, Provider, ProviderInfo, Result,
};
use tokio::sync::mpsc;

use crate::cache::LlMCache;
use crate::claude::ClaudeClient;
use crate::deepseek::DeepSeekClient;
use crate::lmstudio::LmStudioNativeClient;
use crate::openai::OpenAIClient;

const HEALTH_CHECK_TTL: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct HealthState {
    healthy: bool,
    checked_at: Instant,
    last_error: Option<String>,
}

impl Default for HealthState {
    fn default() -> Self {
        Self {
            healthy: true,
            checked_at: Instant::now(),
            last_error: None,
        }
    }
}

impl HealthState {
    fn is_fresh(&self) -> bool {
        self.checked_at.elapsed() < HEALTH_CHECK_TTL
    }

    fn mark_healthy(&mut self) {
        self.healthy = true;
        self.checked_at = Instant::now();
        self.last_error = None;
    }

    fn mark_unhealthy(&mut self, error: String) {
        self.healthy = false;
        self.checked_at = Instant::now();
        self.last_error = Some(error);
    }
}

/// Read API key from env, then data/apikeys.json, then provider config files (Web UI).
///
/// Priority:
/// 1. Environment variable (e.g. OPENAI_API_KEY)
/// 2. data/apikeys.json[file_key]
/// 3. Provider config file saved by Web UI (e.g. data/openai.json → api_key field)
///
/// The Web UI saves keys to per-provider files,
/// so on restart we must read from those files as a fallback.
pub fn load_api_key(env_var: &str, file_key: &str) -> Option<String> {
    if let Ok(key) = std::env::var(env_var) {
        if !key.is_empty() && !key.starts_with("PROXY_") { return Some(key); }
    }
    let data_dir = std::env::var("WANMINFAN_DATA_DIR").unwrap_or_else(|_| "data".into());

    // Paths for data/apikeys.json (classic fallback)
    let apikey_paths = [
        format!("{}/apikeys.json", data_dir),
        "data/apikeys.json".to_string(),
        "../data/apikeys.json".to_string(),
    ];
    for path in apikey_paths.iter() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                if let Some(key) = map.get(file_key).cloned() {
                    if !key.is_empty() {
                        return Some(key);
                    }
                }
            }
        }
    }

    // Web UI 保存的 per-provider config 文件（服务重启后恢复 key）
    let config_file_paths: &[&str] = match file_key {
        "openai" => &["data/openai.json", "data/openai_endpoint.json"],
        "anthropic" => &["data/claude.json"],
        _ => &[],
    };
    for rel_path in config_file_paths {
        // 优先用绝对路径（基于 data_dir），再回退到相对路径
        let candidates = [
            format!("{}/{}", data_dir, rel_path.trim_start_matches("data/")),
            rel_path.to_string(),
            format!("../{}", rel_path),
        ];
        for path in &candidates {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(key) = json.get("api_key").and_then(|v| v.as_str()) {
                        if !key.is_empty() {
                            return Some(key.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// 如果设置了 AI_RELAY_URL，所有 provider 统一走中转站
/// e.g. AI_RELAY_URL=https://your-relay-server/v1
pub fn relay_base_url(default: &str) -> String {
    std::env::var("AI_RELAY_URL").unwrap_or_else(|_| default.to_string())
}

pub trait Gateway: Send + Sync {
    fn provider(&self) -> Provider;
    fn is_available(&self) -> bool;
    fn call(
        &self,
        prompt: &Prompt,
        config: &CallConfig,
    ) -> mpsc::Receiver<Result<Chunk>>;
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Clone)]
pub struct GatewayRegistry {
    providers: HashMap<Provider, Arc<dyn Gateway>>,
    all_info: Vec<ProviderInfo>,
    cache: Arc<RwLock<Option<Arc<LlMCache>>>>,
    preferred_provider: Arc<RwLock<Option<Provider>>>,
    lmstudio_base_url: Arc<RwLock<String>>,
    lmstudio_api_key: Arc<RwLock<Option<String>>>,
    lmstudio_model: Arc<RwLock<String>>,
    openai_base_url: Arc<RwLock<String>>,
    openai_api_key: Arc<RwLock<Option<String>>>,
    openai_model: Arc<RwLock<String>>,
    claude_base_url: Arc<RwLock<String>>,
    claude_api_key: Arc<RwLock<Option<String>>>,
    claude_model: Arc<RwLock<String>>,
    health_states: Arc<RwLock<HashMap<Provider, HealthState>>>,
}

impl GatewayRegistry {
    pub fn new() -> Self {
        let mut providers: HashMap<Provider, Arc<dyn Gateway>> = HashMap::new();
        let mut all_info = Vec::new();

        // Claude
        let claude = ClaudeClient::new();
        let claude_available = claude.is_available();
        let claude_tier = ModelTier::for_provider(&Provider::Claude, &claude.model);
        all_info.push(ProviderInfo {
            provider: Provider::Claude,
            model: claude.model.clone(),
            available: claude_available,
            tier: claude_tier,
        });
        // 始终注册——可用性由 is_available() 动态检查
        // 这样启动后从 data/claude.json 恢复 key 时 downcast 能成功
        providers.insert(Provider::Claude, Arc::new(claude));

        // OpenAI
        let openai = OpenAIClient::new();
        let openai_available = openai.is_available();
        let openai_tier = ModelTier::for_provider(&Provider::OpenAI, &openai.model);
        all_info.push(ProviderInfo {
            provider: Provider::OpenAI,
            model: openai.model.clone(),
            available: openai_available,
            tier: openai_tier,
        });
        providers.insert(Provider::OpenAI, Arc::new(openai));

        // DeepSeek
        let deepseek = DeepSeekClient::new();
        let deepseek_available = deepseek.is_available();
        let deepseek_tier = ModelTier::for_provider(&Provider::DeepSeek, &deepseek.model);
        all_info.push(ProviderInfo {
            provider: Provider::DeepSeek,
            model: deepseek.model.clone(),
            available: deepseek_available,
            tier: deepseek_tier,
        });
        if deepseek_available {
            providers.insert(Provider::DeepSeek, Arc::new(deepseek));
        }

        // LM Studio (本地原生 API)
        let lmstudio_base_url: Arc<RwLock<String>> = Arc::new(RwLock::new(
            std::env::var("LMSTUDIO_BASE_URL").unwrap_or_else(|_| "http://localhost:1234".into())
        ));
        let lmstudio_api_key: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(
            std::env::var("LMSTUDIO_API_KEY").ok().filter(|k| !k.is_empty())
        ));
        let lmstudio_model: Arc<RwLock<String>> = Arc::new(RwLock::new(
            std::env::var("LMSTUDIO_MODEL").unwrap_or_else(|_| "local-model".into())
        ));
        let lmstudio = LmStudioNativeClient::new(
            Some(lmstudio_base_url.clone()),
            Some(lmstudio_api_key.clone()),
            Some(lmstudio_model.clone()),
        );
        let lmstudio_available = lmstudio.is_available();
        let lmstudio_tier = ModelTier::for_provider(&Provider::LMStudio, &lmstudio.model);
        all_info.push(ProviderInfo {
            provider: Provider::LMStudio,
            model: lmstudio.model.clone(),
            available: lmstudio_available,
            tier: lmstudio_tier,
        });
        // LM Studio 始终注册——可用性由 pick_provider 动态检查 gateway.is_available()
        providers.insert(Provider::LMStudio, Arc::new(lmstudio));

        let openai_base_url: Arc<RwLock<String>> = Arc::new(RwLock::new(
            std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into())
        ));
        let openai_api_key: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(
            crate::load_api_key("OPENAI_API_KEY", "openai")
        ));
        let openai_model: Arc<RwLock<String>> = Arc::new(RwLock::new(
            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".into())
        ));
        let claude_base_url: Arc<RwLock<String>> = Arc::new(RwLock::new(
            std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com/v1".into())
        ));
        let claude_api_key: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(
            crate::load_api_key("ANTHROPIC_API_KEY", "anthropic")
        ));
        let claude_model: Arc<RwLock<String>> = Arc::new(RwLock::new(
            std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".into())
        ));

        let mut health_states: HashMap<Provider, HealthState> = HashMap::new();
        for p in [Provider::Claude, Provider::DeepSeek, Provider::LMStudio, Provider::OpenAI] {
            health_states.insert(p, HealthState::default());
        }

        GatewayRegistry {
            providers,
            all_info,
            cache: Arc::new(RwLock::new(None)),
            preferred_provider: Arc::new(RwLock::new(None)),
            lmstudio_base_url,
            lmstudio_api_key,
            lmstudio_model,
            openai_base_url,
            openai_api_key,
            openai_model,
            claude_base_url,
            claude_api_key,
            claude_model,
            health_states: Arc::new(RwLock::new(health_states)),
        }
    }

    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        let mut info = self.all_info.clone();
        for item in info.iter_mut() {
            match item.provider {
                Provider::LMStudio => {
                    let model = self.lmstudio_model.read().clone();
                    if !model.is_empty() {
                        item.model = model;
                    }
                }
                Provider::OpenAI => {
                    let model = self.openai_model.read().clone();
                    if !model.is_empty() {
                        item.model = model;
                    }
                }
                Provider::Claude => {
                    let model = self.claude_model.read().clone();
                    if !model.is_empty() {
                        item.model = model;
                    }
                }
                _ => {}
            }
            if let Some(gw) = self.providers.get(&item.provider) {
                let gw_available = gw.is_available();
                let registry_keyed = match item.provider {
                    Provider::Claude => self.claude_api_key.read().is_some(),
                    Provider::OpenAI => self.openai_api_key.read().is_some(),
                    _ => false,
                };
                if gw_available || registry_keyed {
                    // 有 key 即视为可用——不信任旧的 unhealthy 缓存
                    // （缓存的 false 可能是端点配错时遗留的，修正后应恢复可用）
                    item.available = true;
                } else {
                    item.available = false;
                }
            }
        }
        info
    }

    pub fn set_preferred_provider(&self, provider: Option<Provider>) {
        let mut p = self.preferred_provider.write();
        *p = provider;
    }

    pub fn lmstudio_base_url(&self) -> String {
        self.lmstudio_base_url.read().clone()
    }

    pub fn set_lmstudio_base_url(&self, url: String) {
        let mut u = self.lmstudio_base_url.write();
        *u = url;
        if let Some(gw) = self.providers.get(&Provider::LMStudio) {
            if let Some(lmstudio) = (**gw).as_any().downcast_ref::<crate::lmstudio::LmStudioNativeClient>() {
                lmstudio.invalidate_model_cache();
            }
        }
    }

    pub fn lmstudio_api_key(&self) -> Option<String> {
        self.lmstudio_api_key.read().clone()
    }

    pub fn set_lmstudio_api_key(&self, key: Option<String>) {
        let mut k = self.lmstudio_api_key.write();
        *k = key;
        if let Some(gw) = self.providers.get(&Provider::LMStudio) {
            if let Some(lmstudio) = (**gw).as_any().downcast_ref::<crate::lmstudio::LmStudioNativeClient>() {
                lmstudio.invalidate_model_cache();
            }
        }
    }

    pub fn lmstudio_model(&self) -> String {
        self.lmstudio_model.read().clone()
    }

    pub fn set_lmstudio_model(&self, model: String) {
        let mut m = self.lmstudio_model.write();
        *m = model;
        if let Some(gw) = self.providers.get(&Provider::LMStudio) {
            if let Some(lmstudio) = (**gw).as_any().downcast_ref::<crate::lmstudio::LmStudioNativeClient>() {
                lmstudio.invalidate_model_cache();
            }
        }
    }

    pub fn openai_base_url(&self) -> String {
        self.openai_base_url.read().clone()
    }

    pub fn set_openai_base_url(&self, url: String) {
        *self.openai_base_url.write() = url;
        if let Some(gw) = self.providers.get(&Provider::OpenAI) {
            if let Some(client) = (**gw).as_any().downcast_ref::<crate::openai::OpenAIClient>() {
                client.set_dynamic_base_url(self.openai_base_url.read().clone());
            }
        }
    }

    pub fn openai_api_key(&self) -> Option<String> {
        self.openai_api_key.read().clone()
    }

    pub fn set_openai_api_key(&self, key: Option<String>) {
        *self.openai_api_key.write() = key;
        if let Some(gw) = self.providers.get(&Provider::OpenAI) {
            if let Some(client) = (**gw).as_any().downcast_ref::<crate::openai::OpenAIClient>() {
                client.set_dynamic_api_key(self.openai_api_key.read().clone());
            }
        }
    }

    pub fn openai_model(&self) -> String {
        self.openai_model.read().clone()
    }

    pub fn set_openai_model(&self, model: String) {
        *self.openai_model.write() = model;
        if let Some(gw) = self.providers.get(&Provider::OpenAI) {
            if let Some(client) = (**gw).as_any().downcast_ref::<crate::openai::OpenAIClient>() {
                client.set_dynamic_model(self.openai_model.read().clone());
            }
        }
    }

    pub fn claude_base_url(&self) -> String {
        self.claude_base_url.read().clone()
    }

    pub fn set_claude_base_url(&self, url: String) {
        *self.claude_base_url.write() = url;
        if let Some(gw) = self.providers.get(&Provider::Claude) {
            if let Some(client) = (**gw).as_any().downcast_ref::<crate::claude::ClaudeClient>() {
                client.set_dynamic_base_url(self.claude_base_url.read().clone());
            }
        }
    }

    pub fn claude_api_key(&self) -> Option<String> {
        self.claude_api_key.read().clone()
    }

    pub fn set_claude_api_key(&self, key: Option<String>) {
        *self.claude_api_key.write() = key;
        if let Some(gw) = self.providers.get(&Provider::Claude) {
            if let Some(client) = (**gw).as_any().downcast_ref::<crate::claude::ClaudeClient>() {
                client.set_dynamic_api_key(self.claude_api_key.read().clone());
            }
        }
    }

    pub fn claude_model(&self) -> String {
        self.claude_model.read().clone()
    }

    pub fn set_claude_model(&self, model: String) {
        *self.claude_model.write() = model;
        if let Some(gw) = self.providers.get(&Provider::Claude) {
            if let Some(client) = (**gw).as_any().downcast_ref::<crate::claude::ClaudeClient>() {
                client.set_dynamic_model(self.claude_model.read().clone());
            }
        }
    }

    /// 主动查询 LM Studio 当前加载的模型名，返回模型 ID
    pub async fn fetch_lmstudio_loaded_model(&self) -> Option<String> {
        let base_url = self.lmstudio_base_url.read().clone();
        let api_key = self.lmstudio_api_key.read().clone();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;
        LmStudioNativeClient::fetch_loaded_model(&client, &base_url, &api_key).await
    }

    fn get_health_state(&self, provider: &Provider) -> HealthState {
        self.health_states
            .read()
            .get(provider)
            .cloned()
            .unwrap_or_default()
    }

    pub fn mark_provider_healthy(&self, provider: &Provider) {
        let mut states = self.health_states.write();
        if let Some(state) = states.get_mut(provider) {
            state.mark_healthy();
        }
    }

    pub fn mark_provider_unhealthy(&self, provider: &Provider, error: String) {
        let mut states = self.health_states.write();
        if let Some(state) = states.get_mut(provider) {
            state.mark_unhealthy(error.clone());
        }
        tracing::warn!("Provider {:?} marked unhealthy: {}", provider, error);
    }

    fn is_healthy_with_check(&self, provider: &Provider, gw: &Arc<dyn Gateway>) -> bool {
        if !gw.is_available() {
            return false;
        }
        let health = self.get_health_state(provider);
        if health.is_fresh() {
            return health.healthy;
        }
        true
    }

    pub fn pick_provider(&self) -> Option<Provider> {
        let pref = self.preferred_provider.read().clone();
        if let Some(ref p) = pref {
            if let Some(gw) = self.providers.get(p) {
                if self.is_healthy_with_check(p, gw) {
                    return Some(*p);
                }
            }
        }
        for p in [Provider::Claude, Provider::DeepSeek, Provider::LMStudio, Provider::OpenAI] {
            if let Some(gw) = self.providers.get(&p) {
                if self.is_healthy_with_check(&p, gw) {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn list_available_providers(&self) -> Vec<Provider> {
        let mut available = Vec::new();
        let pref = self.preferred_provider.read().clone();
        if let Some(ref p) = pref {
            if let Some(gw) = self.providers.get(p) {
                if self.is_healthy_with_check(p, gw) {
                    available.push(*p);
                }
            }
        }
        for p in [Provider::Claude, Provider::DeepSeek, Provider::LMStudio, Provider::OpenAI] {
            if available.contains(&p) {
                continue;
            }
            if let Some(gw) = self.providers.get(&p) {
                if self.is_healthy_with_check(&p, gw) {
                    available.push(p);
                }
            }
        }
        available
    }

    pub fn try_next_provider(&self, current: &Provider) -> Option<Provider> {
        let all = [Provider::LMStudio, Provider::DeepSeek, Provider::Claude, Provider::OpenAI];
        let current_idx = all.iter().position(|p| p == current);
        if let Some(idx) = current_idx {
            for i in (idx + 1)..all.len() {
                let p = all[i];
                if let Some(gw) = self.providers.get(&p) {
                    if self.is_healthy_with_check(&p, gw) {
                        return Some(p);
                    }
                }
            }
            for i in 0..idx {
                let p = all[i];
                if let Some(gw) = self.providers.get(&p) {
                    if self.is_healthy_with_check(&p, gw) {
                        return Some(p);
                    }
                }
            }
        }
        self.pick_provider()
    }

    /// Pick provider info respecting preferred_provider, falling back to first available.
    pub fn pick_provider_info(&self) -> ProviderInfo {
        let pref = self.preferred_provider.read().clone();
        // 使用 list_providers 获取动态更新的可用性
        let info = self.list_providers();
        if let Some(ref p) = pref {
            if let Some(item) = info.iter().find(|i| &i.provider == p && i.available) {
                return item.clone();
            }
        }
        info.iter()
            .find(|i| i.available)
            .cloned()
            .unwrap_or_else(|| ProviderInfo {
                provider: Provider::DeepSeek,
                model: "deepseek-chat".into(),
                available: true,
                tier: ModelTier::Pro,
            })
    }

    pub fn set_cache(&self, cache: Arc<LlMCache>) {
        let mut guard = self.cache.write();
        *guard = Some(cache);
    }

    pub fn get(&self, provider: &Provider) -> Option<&Arc<dyn Gateway>> {
        self.providers.get(provider)
    }

    pub fn call(&self, req: &LLMRequest) -> Result<mpsc::Receiver<Result<Chunk>>> {
        let gateway = self
            .providers
            .get(&req.provider)
            .ok_or_else(|| FoundationError::Validation(format!(
                "Provider {:?} not available",
                req.provider
            )))?;

        // Skip cache when tools are configured — tool call responses can't be cached
        // because the cached result has no tool_calls, which would bypass tool execution
        let has_tools = req.config.tools.as_ref().map_or(false, |t| !t.is_empty());

        if !has_tools {
            if let Some(ref cache) = *self.cache.read() {
                let model = req.config.model.as_deref().unwrap_or("unknown");
                let (system_prompt, user_prompt) = extract_prompts(&req.prompt);
                let tool_names = extract_tool_names(&req.config);
                if let Some((cached_content, cached_usage)) = cache.get(
                    &format!("{:?}", req.provider).to_lowercase(),
                    model,
                    &system_prompt,
                    &user_prompt,
                    &tool_names,
                ) {
                tracing::info!(
                    "Cache hit for provider={:?} model={} ({} chars)",
                    req.provider,
                    model,
                    cached_content.len()
                );
                let (tx, rx) = mpsc::channel::<Result<Chunk>>(1);
                let _ = tx.try_send(Ok(Chunk {
                    content: cached_content,
                    reasoning_content: None,
                    finish_reason: Some("stop".to_string()),
                    index: 0,
                    usage: Some(cached_usage),
                    tool_calls: Vec::new(),
                }));
                return Ok(rx);
            }
        }
        }

        Ok(gateway.call(&req.prompt, &req.config))
    }

    pub fn call_parallel(
        &self,
        requests: &[LLMRequest],
    ) -> Vec<(Provider, mpsc::Receiver<Result<Chunk>>)> {
        requests
            .iter()
            .filter_map(|req| {
                self.call(req)
                    .ok()
                    .map(|rx| (req.provider.clone(), rx))
            })
            .collect()
    }

    pub fn get_cache(&self) -> Option<Arc<LlMCache>> {
        self.cache.read().clone()
    }

    pub fn try_store_cache(&self, req: &LLMRequest, content: &str, usage: &foundation::UsageStats) {
        if let Some(ref cache) = *self.cache.read() {
            let model = req.config.model.as_deref().unwrap_or("unknown");
            let (system_prompt, user_prompt) = extract_prompts(&req.prompt);
            let tool_names = extract_tool_names(&req.config);
            let _ = cache.set(
                &format!("{:?}", req.provider).to_lowercase(),
                model,
                &system_prompt,
                &user_prompt,
                &tool_names,
                content,
                usage,
            );
        }
    }

    /// 查询 DeepSeek 账户余额（运维接口）
    pub async fn check_deepseek_balance(&self) -> foundation::Result<serde_json::Value> {
        let gateway = self.providers.get(&foundation::Provider::DeepSeek)
            .ok_or_else(|| foundation::FoundationError::Validation("DeepSeek provider not available".into()))?;
        // 通过 Gateway trait 的 as_any 进行 downcast
        if let Some(deepseek) = (**gateway).as_any().downcast_ref::<crate::deepseek::DeepSeekClient>() {
            deepseek.check_balance().await
        } else {
            Err(foundation::FoundationError::Validation("DeepSeek client type mismatch".into()))
        }
    }
}

fn extract_prompts(prompt: &Prompt) -> (String, String) {
    let mut system = String::new();
    let mut user = String::new();
    for msg in &prompt.messages {
        match msg.role.as_str() {
            "system" => {
                if !system.is_empty() {
                    system.push('\n');
                }
                system.push_str(&msg.content);
            }
            "user" => {
                if !user.is_empty() {
                    user.push('\n');
                }
                user.push_str(&msg.content);
            }
            _ => {}
        }
    }
    (system, user)
}

fn extract_tool_names(config: &CallConfig) -> String {
    match &config.tools {
        Some(tools) => {
            let mut names: Vec<&str> = tools.iter().map(|t| t.function.name.as_str()).collect();
            names.sort();
            names.join(",")
        }
        None => String::new(),
    }
}
