mod claude;
mod deepseek;
pub mod cache;
pub mod model_router;
mod openai;
pub mod prompt;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use foundation::{
    CallConfig, Chunk, FoundationError, LLMRequest, ModelTier, Prompt, Provider, ProviderInfo, Result,
};
use tokio::sync::mpsc;

use crate::cache::LlMCache;
use crate::claude::ClaudeClient;
use crate::deepseek::DeepSeekClient;
use crate::openai::OpenAIClient;

/// Read API key from env or data/apikeys.json fallback
pub fn load_api_key(env_var: &str, file_key: &str) -> Option<String> {
    if let Ok(key) = std::env::var(env_var) {
        if !key.is_empty() && !key.starts_with("PROXY_") { return Some(key); }
    }
    // Try multiple possible paths for apikeys.json
    let paths = [
        "data/apikeys.json",        // From project root
        "../data/apikeys.json",     // From rust/ directory
        "../../data/apikeys.json",  // From deeper directory
    ];
    
    for path in paths.iter() {
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
    None
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
        if claude_available {
            providers.insert(Provider::Claude, Arc::new(claude));
        }

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
        if openai_available {
            providers.insert(Provider::OpenAI, Arc::new(openai));
        }

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

        GatewayRegistry { providers, all_info, cache: Arc::new(RwLock::new(None)) }
    }

    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.all_info.clone()
    }

    pub fn set_cache(&self, cache: Arc<LlMCache>) {
        let mut guard = self.cache.write().expect("cache lock poisoned");
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

        if let Some(ref cache) = *self.cache.read().expect("cache lock poisoned") {
            let model = req.config.model.as_deref().unwrap_or("unknown");
            let (system_prompt, user_prompt) = extract_prompts(&req.prompt);
            if let Some((cached_content, cached_usage)) = cache.get(
                &format!("{:?}", req.provider).to_lowercase(),
                model,
                &system_prompt,
                &user_prompt,
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
        self.cache.read().expect("cache lock poisoned").clone()
    }

    pub fn try_store_cache(&self, req: &LLMRequest, content: &str, usage: &foundation::UsageStats) {
        if let Some(ref cache) = *self.cache.read().expect("cache lock poisoned") {
            let model = req.config.model.as_deref().unwrap_or("unknown");
            let (system_prompt, user_prompt) = extract_prompts(&req.prompt);
            let _ = cache.set(
                &format!("{:?}", req.provider).to_lowercase(),
                model,
                &system_prompt,
                &user_prompt,
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
