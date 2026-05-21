use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use foundation::{CallConfig, Chunk, FoundationError, Prompt, Provider, Result, UsageStats};
use reqwest::Client;
use tokio::sync::mpsc;

use crate::Gateway;

pub struct OpenAIClient {
    api_key: Option<String>,
    pub model: String,
    client: Client,
    base_url: String,
    /// 运行时可动态切换的 base URL（仅 LM Studio 使用）
    dynamic_base_url: Option<Arc<RwLock<String>>>,
    /// 运行时可动态切换的 API key（仅 LM Studio 使用）
    dynamic_api_key: Option<Arc<RwLock<Option<String>>>>,
    /// 运行时可动态切换的模型名（仅 LM Studio 使用）
    dynamic_model: Option<Arc<RwLock<String>>>,
    provider_type: Provider,
}

impl OpenAIClient {
    pub fn new() -> Self {
        let api_key = crate::load_api_key("OPENAI_API_KEY", "openai");
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".into());
        OpenAIClient {
            api_key,
            model,
            base_url: "https://api.openai.com/v1".into(),
            provider_type: Provider::OpenAI,
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to build reqwest Client"),
            dynamic_base_url: None,
            dynamic_api_key: None,
            dynamic_model: None,
        }
    }

    pub fn new_lmstudio(
        dynamic_base_url: Option<Arc<RwLock<String>>>,
        dynamic_api_key: Option<Arc<RwLock<Option<String>>>>,
        dynamic_model: Option<Arc<RwLock<String>>>,
    ) -> Self {
        let api_key = std::env::var("LMSTUDIO_API_KEY").ok()
            .filter(|k| !k.is_empty());
        let model = std::env::var("LMSTUDIO_MODEL").unwrap_or_else(|_| "local-model".into());
        let base_url = std::env::var("LMSTUDIO_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:1234/v1".into());
        OpenAIClient {
            api_key,
            model,
            base_url,
            provider_type: Provider::LMStudio,
            client: Client::builder()
                .timeout(Duration::from_secs(300)) // 本地模型可能较慢
                .build()
                .expect("Failed to build reqwest Client"),
            dynamic_base_url,
            dynamic_api_key,
            dynamic_model,
        }
    }

    /// 获取当前实际使用的 base URL
    fn effective_base_url(&self) -> String {
        if let Some(ref lock) = self.dynamic_base_url {
            let url = lock.read().expect("lock poisoned").clone();
            if !url.is_empty() {
                return url;
            }
        }
        self.base_url.clone()
    }

    /// 获取当前实际使用的 API key（优先 dynamic，fallback 到静态）
    fn effective_api_key(&self) -> Option<String> {
        if let Some(ref lock) = self.dynamic_api_key {
            if let Ok(key_guard) = lock.read() {
                if let Some(ref key) = *key_guard {
                    if !key.is_empty() {
                        return Some(key.clone());
                    }
                }
            }
        }
        self.api_key.clone()
    }

    /// 获取当前实际使用的模型名（优先 dynamic，fallback 到静态）
    fn effective_model(&self) -> String {
        if let Some(ref lock) = self.dynamic_model {
            let model = lock.read().expect("lock poisoned").clone();
            if !model.is_empty() {
                return model;
            }
        }
        self.model.clone()
    }
}

impl Gateway for OpenAIClient {
    fn provider(&self) -> Provider {
        self.provider_type
    }

    fn is_available(&self) -> bool {
        self.provider_type == Provider::LMStudio || self.effective_api_key().is_some()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call(
        &self,
        prompt: &Prompt,
        config: &CallConfig,
    ) -> mpsc::Receiver<Result<Chunk>> {
        let (tx, rx) = mpsc::channel(256);
        let api_key = self.effective_api_key();
        let is_lmstudio = self.provider_type == Provider::LMStudio;
        if api_key.is_none() && !is_lmstudio {
            let _ = tx.try_send(Err(FoundationError::Validation(
                "OpenAI API key not configured".into(),
            )));
            return rx;
        }
        let model = config.model.clone().unwrap_or_else(|| self.effective_model());
        let client = self.client.clone();
        let base_url = self.effective_base_url();
        let config = config.clone();

        let messages: Vec<serde_json::Value> = {
            let msgs: Vec<serde_json::Value> = prompt
                .messages
                .iter()
                .map(|m| {
                    let mut msg = serde_json::json!({"role": m.role, "content": m.content});
                    if let Some(tc) = &m.tool_calls {
                        msg["tool_calls"] = serde_json::to_value(tc).unwrap();
                    }
                    if let Some(tcid) = &m.tool_call_id {
                        msg["tool_call_id"] = serde_json::json!(tcid);
                    }
                    msg
                })
                .collect();

            msgs
        };

        let use_stream = config.stream || !is_lmstudio; // 尊重 config.stream 设置
        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens.max(128), // ensure at least some tokens
            "stream": use_stream,
        });
        if use_stream {
            body["stream_options"] = serde_json::json!({ "include_usage": true });
        }

        tokio::spawn(async move {
            let mut req = client
                .post(format!("{}/chat/completions", base_url))
                .header("Content-Type", "application/json");
            if let Some(ref key) = api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            let result = req.json(&body)
                .send()
                .await;

            let response = match result {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(FoundationError::Io(std::io::Error::other(e.to_string())))).await;
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                let _ = tx.send(Err(FoundationError::Validation(format!(
                    "OpenAI API error {}: {}",
                    status, body
                )))).await;
                return;
            }

            if !use_stream {
                // 非流式：解析完整 JSON 响应
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let msg = &json["choices"][0]["message"];
                        let mut content = msg["content"].as_str().unwrap_or("").to_string();
                        // LM Studio: 思考内容在 reasoning_content 字段
                        if content.is_empty() {
                            content = msg["reasoning_content"].as_str().unwrap_or("").to_string();
                        }
                        let usage = UsageStats {
                            prompt_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                            completion_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
                            total_tokens: json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
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

            use futures::StreamExt;

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut chunk_index: u32 = 0;

            while let Some(bytes_result) = stream.next().await {
                match bytes_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        while let Some(end) = buffer.find('\n') {
                            let line = buffer[..end].to_string();
                            buffer = buffer[end + 1..].to_string();
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    let _ = tx.send(Ok(Chunk {
                                        content: String::new(),
                                        reasoning_content: None,
                                        finish_reason: Some("stop".into()),
                                        index: chunk_index,
                                        usage: None,
                                        tool_calls: Vec::new(),
                                    })).await;
                                    return;
                                }
                                if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                                    if let Some(choices) = event["choices"].as_array() {
                                        for choice in choices {
                                            let content_text = choice["delta"]["content"].as_str();
                                            let reasoning_text = choice["delta"]["reasoning_content"].as_str();
                                            if content_text.is_some() || reasoning_text.is_some() {
                                                let _ = tx.send(Ok(Chunk {
                                                    content: content_text.unwrap_or("").to_string(),
                                                    reasoning_content: reasoning_text.map(|s| s.to_string()),
                                                    finish_reason: choice["finish_reason"].as_str().map(|s| s.to_string()),
                                                    index: chunk_index,
                                                    usage: None,
                                                    tool_calls: Vec::new(),
                                                })).await;
                                                chunk_index += 1;
                                            }
                                        }
                                    }
                                    // OpenAI sends usage in a separate chunk when stream_options.include_usage is set
                                    if let Some(u) = event.get("usage") {
                                        let usage = UsageStats {
                                            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                                            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                                            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
                                        };
                                        let _ = tx.send(Ok(Chunk {
                                            content: String::new(),
                                            reasoning_content: None,
                                            finish_reason: None,
                                            index: chunk_index,
                                            usage: Some(usage),
                                            tool_calls: Vec::new(),
                                        })).await;
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
        });

        rx
    }
}
