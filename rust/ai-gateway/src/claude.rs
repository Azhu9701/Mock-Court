use std::time::Duration;

use foundation::{CallConfig, Chunk, FoundationError, Prompt, Provider, Result, UsageStats};
use reqwest::Client;
use tokio::sync::mpsc;

use crate::Gateway;

pub struct ClaudeClient {
    api_key: Option<String>,
    pub model: String,
    client: Client,
}

impl ClaudeClient {
    pub fn new() -> Self {
        let api_key = crate::load_api_key("ANTHROPIC_API_KEY", "anthropic");
        let model =
            std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".into());
        ClaudeClient {
            api_key,
            model,
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to build reqwest Client"),
        }
    }
}

impl Gateway for ClaudeClient {
    fn provider(&self) -> Provider {
        Provider::Claude
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
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
                    "Claude API key not configured".into(),
                )));
                return rx;
            }
        };
        let model = self.model.clone();
        let client = self.client.clone();
        let config = config.clone();

        let system = prompt
            .messages
            .iter()
            .filter(|m| m.role == "system")
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        let messages: Vec<serde_json::Value> = prompt
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();

        let body = if system.is_empty() {
            serde_json::json!({
                "model": model,
                "max_tokens": config.max_tokens,
                "temperature": config.temperature,
                "messages": messages,
                "stream": true,
            })
        } else {
            serde_json::json!({
                "model": model,
                "max_tokens": config.max_tokens,
                "temperature": config.temperature,
                "system": system,
                "messages": messages,
                "stream": true,
            })
        };

        tokio::spawn(async move {
            let result = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
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
                    "Claude API error {}: {}",
                    status, body
                )))).await;
                return;
            }

            use futures::StreamExt;

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut chunk_index: u32 = 0;
            let mut input_tokens: u32 = 0;
            let mut output_tokens: u32 = 0;

            while let Some(bytes_result) = stream.next().await {
                match bytes_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        while let Some(end) = buffer.find('\n') {
                            let line = buffer[..end].to_string();
                            buffer = buffer[end + 1..].to_string();
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                                    match event["type"].as_str() {
                                        Some("message_start") => {
                                            if let Some(u) = event["message"]["usage"]["input_tokens"].as_u64() {
                                                input_tokens = u as u32;
                                            }
                                        }
                                        Some("content_block_delta") => {
                                            if let Some(text) = event["delta"]["text"].as_str() {
                                                let _ = tx.send(Ok(Chunk {
                                                    content: text.to_string(),
                                                    reasoning_content: None,
                                                    finish_reason: None,
                                                    index: chunk_index,
                                                    usage: None,
                                                    tool_calls: Vec::new(),
                                                })).await;
                                                chunk_index += 1;
                                            }
                                        }
                                        Some("message_delta") => {
                                            if let Some(u) = event["usage"]["output_tokens"].as_u64() {
                                                output_tokens = u as u32;
                                            }
                                        }
                                        Some("message_stop") => {
                                            let usage = UsageStats {
                                                prompt_tokens: input_tokens,
                                                completion_tokens: output_tokens,
                                                total_tokens: input_tokens + output_tokens,
                                            };
                                            let _ = tx.send(Ok(Chunk {
                                            content: String::new(),
                                            reasoning_content: None,
                                            finish_reason: Some("stop".into()),
                                            index: chunk_index,
                                            usage: Some(usage),
                                            tool_calls: Vec::new(),
                                        })).await;
                                        }
                                        _ => {}
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
