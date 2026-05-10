mod claude;
mod deepseek;
pub mod model_router;
mod openai;
pub mod prompt;

use std::collections::HashMap;
use std::sync::Arc;

use foundation::{
    CallConfig, Chunk, FoundationError, LLMRequest, ModelTier, Prompt, Provider, ProviderInfo, Result,
};
use tokio::sync::mpsc;

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
    ) -> mpsc::UnboundedReceiver<Result<Chunk>>;
}

#[derive(Clone)]
pub struct GatewayRegistry {
    providers: HashMap<Provider, Arc<dyn Gateway>>,
    all_info: Vec<ProviderInfo>,
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

        GatewayRegistry { providers, all_info }
    }

    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.all_info.clone()
    }

    pub fn get(&self, provider: &Provider) -> Option<&Arc<dyn Gateway>> {
        self.providers.get(provider)
    }

    pub fn call(&self, req: &LLMRequest) -> Result<mpsc::UnboundedReceiver<Result<Chunk>>> {
        let gateway = self
            .providers
            .get(&req.provider)
            .ok_or_else(|| FoundationError::Validation(format!(
                "Provider {:?} not available",
                req.provider
            )))?;
        Ok(gateway.call(&req.prompt, &req.config))
    }

    pub fn call_parallel(
        &self,
        requests: &[LLMRequest],
    ) -> Vec<(Provider, mpsc::UnboundedReceiver<Result<Chunk>>)> {
        requests
            .iter()
            .filter_map(|req| {
                self.call(req)
                    .ok()
                    .map(|rx| (req.provider.clone(), rx))
            })
            .collect()
    }
}
