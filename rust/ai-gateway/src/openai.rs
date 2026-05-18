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
        }
    }

    pub fn new_lmstudio() -> Self {
        let api_key = std::env::var("LMSTUDIO_API_KEY").ok().or_else(|| Some("lm-studio".into()));
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
        }
    }
}

impl Gateway for OpenAIClient {
    fn provider(&self) -> Provider {
        self.provider_type
    }

    fn is_available(&self) -> bool {
        self.provider_type == Provider::LMStudio || self.api_key.is_some()
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
        let api_key = match &self.api_key {
            Some(k) => k.clone(),
            None => {
                let _ = tx.try_send(Err(FoundationError::Validation(
                    "OpenAI API key not configured".into(),
                )));
                return rx;
            }
        };
        let model = self.model.clone();
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let config = config.clone();

        let messages: Vec<serde_json::Value> = prompt
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

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
            "stream": true,
            "stream_options": { "include_usage": true },
        });

        tokio::spawn(async move {
            let result = client
                .post(format!("{}/chat/completions", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
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
                                            if let Some(text) = choice["delta"]["content"].as_str() {
                                                let _ = tx.send(Ok(Chunk {
                                                    content: text.to_string(),
                                                    reasoning_content: None,
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
