use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use foundation::{CallConfig, Chunk, FoundationError, Prompt, Provider, Result, UsageStats};
use reqwest::Client;
use tokio::sync::mpsc;

use crate::Gateway;

fn serialize_claude_message(m: &foundation::PromptMessage) -> serde_json::Value {
    match m.role.as_str() {
        "assistant" if m.tool_calls.is_some() => {
            let blocks: Vec<serde_json::Value> = m
                .tool_calls
                .as_ref()
                .unwrap()
                .iter()
                .map(|tc| {
                    let input: serde_json::Value =
                        serde_json::from_str(&tc.function.arguments).unwrap_or_else(|_| {
                            tracing::warn!(
                                "Tool call arguments not valid JSON for '{}', sending as-is: {}",
                                tc.function.name,
                                &tc.function.arguments[..tc.function.arguments.len().min(200)]
                            );
                            serde_json::json!({ "_raw": tc.function.arguments })
                        });
                    serde_json::json!({
                        "type": "tool_use",
                        "id": tc.id,
                        "name": tc.function.name,
                        "input": input
                    })
                })
                .collect();
            serde_json::json!({"role": "assistant", "content": blocks})
        }
        "tool" => {
            serde_json::json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": m.tool_call_id,
                    "content": m.content
                }]
            })
        }
        _ => {
            serde_json::json!({"role": m.role, "content": m.content})
        }
    }
}

pub struct ClaudeClient {
    api_key: Option<String>,
    pub model: String,
    client: Client,
    dynamic_base_url: Option<Arc<RwLock<String>>>,
    dynamic_api_key: Option<Arc<RwLock<Option<String>>>>,
    dynamic_model: Option<Arc<RwLock<String>>>,
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
            dynamic_base_url: None,
            dynamic_api_key: None,
            dynamic_model: None,
        }
    }
}

impl ClaudeClient {
    fn effective_base_url(&self) -> String {
        if let Some(ref lock) = self.dynamic_base_url {
            let url = lock.read().clone();
            if !url.is_empty() {
                return url;
            }
        }
        crate::relay_base_url("https://api.anthropic.com/v1")
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
        self.api_key.clone()
    }

    fn effective_model(&self) -> String {
        if let Some(ref lock) = self.dynamic_model {
            let model = lock.read().clone();
            if !model.is_empty() {
                return model;
            }
        }
        self.model.clone()
    }

    pub fn set_dynamic_base_url(&self, url: String) {
        if let Some(ref lock) = self.dynamic_base_url {
            *lock.write() = url;
        }
    }

    pub fn set_dynamic_api_key(&self, key: Option<String>) {
        if let Some(ref lock) = self.dynamic_api_key {
            *lock.write() = key;
        }
    }

    pub fn set_dynamic_model(&self, model: String) {
        if let Some(ref lock) = self.dynamic_model {
            *lock.write() = model;
        }
    }
}

impl Gateway for ClaudeClient {
    fn provider(&self) -> Provider {
        Provider::Claude
    }

    fn is_available(&self) -> bool {
        self.effective_api_key().is_some()
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
        let api_key = match self.effective_api_key() {
            Some(k) => k,
            None => {
                let _ = tx.try_send(Err(FoundationError::Validation(
                    "Claude API key not configured".into(),
                )));
                return rx;
            }
        };
        let model = self.effective_model();
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
            .map(serialize_claude_message)
            .collect();

        let mut body = if system.is_empty() {
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

        if let Some(tools) = &config.tools {
            let claude_tools: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.function.name,
                        "description": t.function.description,
                        "input_schema": t.function.parameters
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(claude_tools);
        }

        let effective_url = self.effective_base_url();
        tokio::spawn(async move {

            let result = client
                .post(format!("{}/messages", effective_url))
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
            let mut stop_reason: Option<String> = None;

            // Tool use state tracking
            let mut in_tool_use = false;
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_args = String::new();

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
                                        Some("content_block_start") => {
                                            if event["content_block"]["type"].as_str() == Some("tool_use") {
                                                in_tool_use = true;
                                                current_tool_id = event["content_block"]["id"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string();
                                                current_tool_name = event["content_block"]["name"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string();
                                                current_tool_args.clear();
                                            }
                                        }
                                        Some("content_block_delta") => {
                                            if in_tool_use {
                                                if let Some(partial) = event["delta"]["partial_json"].as_str() {
                                                    current_tool_args.push_str(partial);
                                                }
                                            } else if let Some(text) = event["delta"]["text"].as_str() {
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
                                        Some("content_block_stop") => {
                                            if in_tool_use {
                                                let tool_call = foundation::ToolCall {
                                                    id: current_tool_id.clone(),
                                                    r#type: "function".to_string(),
                                                    function: foundation::ToolCallFunction {
                                                        name: current_tool_name.clone(),
                                                        arguments: current_tool_args.clone(),
                                                    },
                                                };
                                                let _ = tx.send(Ok(Chunk {
                                                    content: String::new(),
                                                    reasoning_content: None,
                                                    finish_reason: None,
                                                    index: chunk_index,
                                                    usage: None,
                                                    tool_calls: vec![tool_call],
                                                })).await;
                                                chunk_index += 1;
                                                in_tool_use = false;
                                                current_tool_id.clear();
                                                current_tool_name.clear();
                                                current_tool_args.clear();
                                            }
                                        }
                                        Some("message_delta") => {
                                            if let Some(u) = event["usage"]["output_tokens"].as_u64() {
                                                output_tokens = u as u32;
                                            }
                                            if let Some(sr) = event["delta"]["stop_reason"].as_str() {
                                                stop_reason = Some(sr.to_string());
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
                                                finish_reason: Some(
                                                    stop_reason.clone().unwrap_or_else(|| "stop".into()),
                                                ),
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
            // Stream EOF without message_stop — send terminal chunk to unblock receiver
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
