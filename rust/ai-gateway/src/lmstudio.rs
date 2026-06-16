use std::sync::Arc;

use parking_lot::RwLock;
use std::time::{Duration, Instant};

use foundation::{CallConfig, Chunk, FoundationError, Prompt, Provider, Result, UsageStats};
use reqwest::Client;
use tokio::sync::mpsc;

use crate::Gateway;

const MODEL_CACHE_TTL: Duration = Duration::from_secs(60);

struct CachedModel {
    model: Option<String>,
    fetched_at: Instant,
}

pub struct LmStudioNativeClient {
    pub model: String,
    client: Client,
    base_url: String,
    dynamic_base_url: Option<Arc<RwLock<String>>>,
    dynamic_api_key: Option<Arc<RwLock<Option<String>>>>,
    dynamic_model: Option<Arc<RwLock<String>>>,
    cached_model: Arc<RwLock<Option<CachedModel>>>,
}

impl LmStudioNativeClient {
    pub fn new(
        dynamic_base_url: Option<Arc<RwLock<String>>>,
        dynamic_api_key: Option<Arc<RwLock<Option<String>>>>,
        dynamic_model: Option<Arc<RwLock<String>>>,
    ) -> Self {
        let base_url = std::env::var("LMSTUDIO_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:1234".into());
        let model = std::env::var("LMSTUDIO_MODEL")
            .unwrap_or_else(|_| "local-model".into());
        LmStudioNativeClient {
            model,
            base_url,
            dynamic_base_url,
            dynamic_api_key,
            dynamic_model,
            client: Client::builder()
                .connect_timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build reqwest Client"),
            cached_model: Arc::new(RwLock::new(None)),
        }
    }

    fn effective_base_url(&self) -> String {
        if let Some(ref lock) = self.dynamic_base_url {
            let url = lock.read().clone();
            if !url.is_empty() {
                return url.trim_end_matches("/v1").trim_end_matches('/').to_string();
            }
        }
        self.base_url.trim_end_matches("/v1").trim_end_matches('/').to_string()
    }

    fn effective_api_key(&self) -> Option<String> {
        if let Some(ref lock) = self.dynamic_api_key {
            let key_guard = lock.read();
            if let Some(ref key) = *key_guard {
                if !key.is_empty() {
                    return Some(key.clone());
                }
            }
        }
        std::env::var("LMSTUDIO_API_KEY").ok().filter(|k| !k.is_empty())
    }

    fn effective_model(&self) -> String {
        if let Some(ref lock) = self.dynamic_model {
            let model = lock.read().clone();
            if !model.is_empty() && model != "local-model" {
                return model;
            }
        }
        if self.model != "local-model" {
            return self.model.clone();
        }
        std::env::var("LMSTUDIO_MODEL").unwrap_or_else(|_| "local-model".into())
    }

    fn get_cached_model(&self) -> Option<String> {
        let guard = self.cached_model.read();
        if let Some(ref cached) = *guard {
            if cached.fetched_at.elapsed() < MODEL_CACHE_TTL {
                return cached.model.clone();
            }
        }
        None
    }

    pub fn invalidate_model_cache(&self) {
        let mut guard = self.cached_model.write();
        *guard = None;
    }

    /// Fetch the actual loaded model identifier from LM Studio's /api/v1/models endpoint
    pub(crate) async fn fetch_loaded_model(
        client: &Client,
        base_url: &str,
        api_key: &Option<String>,
    ) -> Option<String> {
        let url = format!("{}/api/v1/models", base_url);
        tracing::debug!("LM Studio fetching models from: {}", url);
        let mut req = client.get(&url).timeout(Duration::from_secs(5));
        if let Some(ref key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                let body_text = resp.text().await.unwrap_or_default();
                tracing::debug!("LM Studio /api/v1/models response: {}", body_text.chars().take(500).collect::<String>());
                match serde_json::from_str::<serde_json::Value>(&body_text) {
                    Ok(json) => {
                        if let Some(models) = json["models"].as_array() {
                            // Find first model with loaded_instances > 0
                            for m in models {
                                if let Some(instances) = m["loaded_instances"].as_array() {
                                    if !instances.is_empty() {
                                        // Use the loaded instance ID, not selected_variant
                                        if let Some(id) = instances[0]["id"].as_str() {
                                            tracing::info!("LM Studio loaded model: {}", id);
                                            return Some(id.to_string());
                                        }
                                    }
                                }
                            }
                            // Fallback: use first model's key
                            if let Some(first) = models.first() {
                                if let Some(key) = first["key"].as_str() {
                                    tracing::warn!("LM Studio model '{}' is not currently loaded. Please load it in LM Studio client.", key);
                                    return Some(key.to_string());
                                }
                            }
                        }
                        tracing::warn!("LM Studio /api/v1/models response has no usable model. json keys: {:?}", json.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                    }
                    Err(e) => tracing::warn!("LM Studio models parse error: {}", e),
                }
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::warn!("LM Studio models endpoint error {}: {}", status, body.chars().take(200).collect::<String>());
            }
            Err(e) => tracing::warn!("LM Studio models fetch error: {}", e),
        }
        None
    }

    /// Convert OpenAI-style messages to LM Studio native format
    fn convert_messages(&self, prompt: &Prompt) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_prompt = None;
        let mut input_parts = Vec::new();

        for msg in &prompt.messages {
            match msg.role.as_str() {
                "system" => {
                    if system_prompt.is_none() {
                        system_prompt = Some(msg.content.clone());
                    } else {
                        system_prompt = Some(format!("{}\n{}", system_prompt.unwrap(), msg.content));
                    }
                }
                "user" => {
                    input_parts.push(serde_json::json!({
                        "type": "text",
                        "content": msg.content
                    }));
                }
                "assistant" => {
                    // LM Studio native API doesn't have assistant role in input.
                    // Add as a labeled text part so the model sees prior turns as context
                    // without conflating them with the user's last message.
                    input_parts.push(serde_json::json!({
                        "type": "text",
                        "content": format!("[Assistant]: {}", msg.content)
                    }));
                }
                _ => {}
            }
        }

        (system_prompt, input_parts)
    }
}

impl Gateway for LmStudioNativeClient {
    fn provider(&self) -> Provider {
        Provider::LMStudio
    }

    fn is_available(&self) -> bool {
        // LM Studio 仅在用户显式配置了模型名后才视为可用
        // 不硬编码 true——默认的 "local-model" 不代表实际配置
        let model = self.effective_model();
        !model.is_empty() && model != "local-model"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>> {
        let (tx, rx) = mpsc::channel(256);
        let explicit_model = config.model.clone();
        let client = self.client.clone();
        let base_url = self.effective_base_url();
        let api_key = self.effective_api_key();
        let default_model = self.effective_model();
        let cached_model = self.get_cached_model();
        let cached_model_arc = self.cached_model.clone();
        let (system_prompt, input) = self.convert_messages(prompt);
        let temperature = config.temperature;
        let max_tokens = config.max_tokens.max(128);
        let stream = config.stream;

        tokio::spawn(async move {
            let model = match explicit_model {
                Some(m) if !m.is_empty() && m != "local-model" => m,
                _ => {
                    if let Some(cached) = cached_model {
                        tracing::debug!("LM Studio using cached model: {}", cached);
                        cached
                    } else {
                        tracing::info!("LM Studio auto-detecting loaded model from /api/v1/models");
                        match Self::fetch_loaded_model(&client, &base_url, &api_key).await {
                            Some(detected) => {
                                tracing::info!("LM Studio resolved model: {}", detected);
                                let mut guard = cached_model_arc.write();
                                *guard = Some(CachedModel {
                                    model: Some(detected.clone()),
                                    fetched_at: Instant::now(),
                                });
                                detected
                            }
                            None => {
                                if default_model != "local-model" {
                                    tracing::warn!("LM Studio auto-detect failed, using configured model: {}", default_model);
                                    default_model.clone()
                                } else {
                                    tracing::error!("LM Studio auto-detect failed and no model configured. Please configure LMSTUDIO_MODEL env var or set model name in the frontend.");
                                    let _ = tx.send(Err(FoundationError::Validation(
                                        "LM Studio model not configured. Set LMSTUDIO_MODEL env var (e.g., qwen/qwen3-4b) or configure via frontend.".into()
                                    ))).await;
                                    return;
                                }
                            }
                        }
                    }
                }
            };
            let mut body = serde_json::json!({
                "model": model,
                "input": input,
                "stream": stream,
                "temperature": temperature,
                "max_output_tokens": max_tokens,
            });

            if let Some(sp) = system_prompt {
                body["system_prompt"] = serde_json::json!(sp);
            }

            let mut req = client
                .post(format!("{}/api/v1/chat", base_url))
                .header("Content-Type", "application/json");

            if let Some(ref key) = api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let result = req.json(&body).send().await;

            let response = match result {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(FoundationError::Io(std::io::Error::other(e.to_string())))).await;
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body_text = response.text().await.unwrap_or_default();
                let _ = tx.send(Err(FoundationError::Validation(format!(
                    "LM Studio API error {}: {}", status, body_text
                )))).await;
                return;
            }

            if !stream {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let content = extract_content(&json);
                        let input_tokens = json["stats"]["input_tokens"].as_u64().unwrap_or(0) as u32;
                        let output_tokens = json["stats"]["total_output_tokens"].as_u64().unwrap_or(0) as u32;
                        if content.is_empty() {
                            let preview = serde_json::to_string(&json).unwrap_or_default();
                            let safe_len = preview.char_indices()
                                .take_while(|(i, _)| *i < 500)
                                .last().map(|(i, c)| i + c.len_utf8()).unwrap_or(500);
                            tracing::warn!(
                                "LM Studio non-stream empty content. output_tokens={} raw_response={}",
                                output_tokens, &preview[..safe_len.min(preview.len())]
                            );
                        } else {
                            tracing::debug!("LM Studio non-stream content: {} chars, {} output_tokens", content.len(), output_tokens);
                        }
                        let usage = UsageStats {
                            prompt_tokens: input_tokens,
                            completion_tokens: output_tokens,
                            total_tokens: input_tokens + output_tokens,
                        };
                        let _ = tx.send(Ok(Chunk {
                            content,
                            reasoning_content: None,
                            finish_reason: Some("stop".into()),
                            index: 0,
                            usage: Some(usage),
                            tool_calls: Vec::new(),
                        })).await;
                    }
                    Err(e) => {
                        let _ = tx.send(Err(FoundationError::Io(std::io::Error::other(e.to_string())))).await;
                    }
                }
                return;
            }

            // Stream handling — LM Studio native format:
            // event: <event_type>\ndata: <json>\n\n
            use futures::StreamExt;
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut chunk_index: u32 = 0;
            let mut current_event_type = String::new();
            let mut final_usage: Option<UsageStats> = None;
            tracing::info!("LM Studio SSE stream started");

            while let Some(bytes_result) = bytes_stream.next().await {
                match bytes_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        while let Some(end) = buffer.find('\n') {
                            let line = buffer[..end].to_string();
                            buffer = buffer[end + 1..].to_string();
                            let line = line.trim_end_matches('\r').to_string();

                            if line.starts_with("event: ") {
                                current_event_type = line[7..].to_string();
                                tracing::debug!("LM Studio SSE event: {}", current_event_type);
                            } else if line.starts_with("data: ") {
                                let data = &line[6..];
                                if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                                    match current_event_type.as_str() {
                                        "message.delta" => {
                                            if let Some(content) = event["content"].as_str() {
                                                tracing::debug!("LM Studio message.delta: {}", content.chars().take(40).collect::<String>());
                                                let _ = tx.send(Ok(Chunk {
                                                    content: content.to_string(),
                                                    reasoning_content: None,
                                                    finish_reason: None,
                                                    index: chunk_index,
                                                    usage: None,
                                                    tool_calls: Vec::new(),
                                                })).await;
                                                chunk_index += 1;
                                            }
                                        }
                                        "chat.end" => {
                                            if let Some(stats) = event.get("result").and_then(|r| r.get("stats")) {
                                                final_usage = Some(UsageStats {
                                                    prompt_tokens: stats["input_tokens"].as_u64().unwrap_or(0) as u32,
                                                    completion_tokens: stats["total_output_tokens"].as_u64().unwrap_or(0) as u32,
                                                    total_tokens: stats["input_tokens"].as_u64().unwrap_or(0) as u32
                                                        + stats["total_output_tokens"].as_u64().unwrap_or(0) as u32,
                                                });
                                            }
                                            let _ = tx.send(Ok(Chunk {
                                                content: String::new(),
                                                reasoning_content: None,
                                                finish_reason: Some("stop".into()),
                                                index: chunk_index,
                                                usage: final_usage.clone(),
                                                tool_calls: Vec::new(),
                                            })).await;
                                            return;
                                        }
                                        "error" => {
                                            let msg = event["error"]["message"].as_str().unwrap_or("unknown error");
                                            let _ = tx.send(Err(FoundationError::Validation(format!(
                                                "LM Studio stream error: {}", msg
                                            )))).await;
                                            return;
                                        }
                                        _ => {} // skip reasoning/tool_call/model_load events
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(FoundationError::Io(std::io::Error::other(e.to_string())))).await;
                        return;
                    }
                }
            }
            // Stream EOF without stop — send terminal chunk to unblock receiver
            let _ = tx.send(Ok(Chunk {
                content: String::new(),
                reasoning_content: None,
                finish_reason: Some("stop".into()),
                index: chunk_index,
                usage: None,
                tool_calls: Vec::new(),
            })).await;
        });

        rx
    }
}

fn extract_content(json: &serde_json::Value) -> String {
    if let Some(output) = json["output"].as_array() {
        let mut text = String::new();
        let mut seen_types: Vec<String> = Vec::new();
        for item in output {
            let typ = item["type"].as_str().unwrap_or("?");
            seen_types.push(typ.to_string());
            let content = item["content"].as_str().unwrap_or("");
            // message / reasoning / thinking — 全部收集
            match typ {
                "message" | "reasoning" | "thinking" => {
                    if !content.is_empty() {
                        text.push_str(content);
                    }
                }
                _ => {}
            }
        }
        tracing::debug!(
            "extract_content: {} output items, types={:?}, total_len={}",
            output.len(), seen_types, text.len()
        );
        if !text.is_empty() {
            return text;
        }
        // Fallback: 如果没有 message/reasoning/thinking，收集所有 content
        for item in output {
            if let Some(c) = item["content"].as_str() {
                if !c.is_empty() { text.push_str(c); }
            }
        }
        return text;
    }
    // Fallback: try OpenAI-compatible format
    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
        return content.to_string();
    }
    tracing::warn!("extract_content: unrecognized response format, keys={:?}", json.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    String::new()
}
