use std::time::Duration;

use foundation::{CallConfig, Chunk, FoundationError, Prompt, Provider, ReasoningEffort as FoundationReasoningEffort, Result, ToolCall, ToolCallFunction, UsageStats};
use reqwest::Client;
use tokio::sync::mpsc;

use crate::Gateway;

pub struct DeepSeekClient {
    api_key: Option<String>,
    pub model: String,
    client: Client,
    default_reasoning_effort: FoundationReasoningEffort,
    default_thinking_enabled: bool,
}

impl DeepSeekClient {
    pub fn new() -> Self {
        let api_key = crate::load_api_key("DEEPSEEK_API_KEY", "deepseek");
        let model = std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-v4-pro".into());
        let default_reasoning = std::env::var("DEEPSEEK_REASONING_EFFORT")
            .unwrap_or_else(|_| "think".into());
        let default_thinking = std::env::var("DEEPSEEK_THINKING")
            .unwrap_or_else(|_| "true".into());

        DeepSeekClient {
            api_key,
            model,
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .pool_max_idle_per_host(5)
                .build()
                .expect("Failed to build reqwest Client"),
            default_reasoning_effort: FoundationReasoningEffort::from_str(&default_reasoning),
            default_thinking_enabled: default_thinking.to_lowercase() == "true" || default_thinking == "1",
        }
    }

    pub async fn check_balance(&self) -> Result<serde_json::Value> {
        let api_key = self.api_key.as_ref().ok_or_else(|| FoundationError::Validation("DeepSeek API key not configured".into()))?;
        let resp = self.client.get("https://api.deepseek.com/user/balance")
            .header("Authorization", format!("Bearer {}", api_key))
            .send().await.map_err(|e| FoundationError::Io(std::io::Error::other(e.to_string())))?;
        let body = resp.text().await.map_err(|e| FoundationError::Io(std::io::Error::other(e.to_string())))?;
        serde_json::from_str(&body).map_err(|e| FoundationError::Validation(format!("Balance: {}", e)))
    }
}

impl Gateway for DeepSeekClient {
    fn provider(&self) -> Provider {
        Provider::DeepSeek
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
                    "DeepSeek API key not configured".into(),
                )));
                return rx;
            }
        };
        let model = config.model.clone().unwrap_or_else(|| self.model.clone());
        let client = self.client.clone();
        let config = config.clone();

        let messages: Vec<serde_json::Value> = prompt
            .messages
            .iter()
            .map(|m| {
                let mut msg = serde_json::json!({"role": m.role, "content": m.content});
                if let Some(rc) = &m.reasoning_content {
                    msg["reasoning_content"] = serde_json::json!(rc);
                } else if m.role == "assistant" && m.tool_calls.is_some() {
                    // DeepSeek v4 启用 thinking 时，assistant tool call 消息必须有 reasoning_content
                    msg["reasoning_content"] = serde_json::json!("");
                }
                if let Some(tc) = &m.tool_calls {
                    msg["tool_calls"] = serde_json::to_value(tc).unwrap();
                }
                if let Some(tcid) = &m.tool_call_id {
                    msg["tool_call_id"] = serde_json::json!(tcid);
                }
                msg
            })
            .collect();

        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
        });

        if config.temperature > 0.0 {
            body["temperature"] = serde_json::json!(config.temperature);
        }
        if config.max_tokens > 0 {
            body["max_tokens"] = serde_json::json!(config.max_tokens);
        }

        let reasoning_effort = config.reasoning_effort.unwrap_or(self.default_reasoning_effort);
        body["reasoning_effort"] = serde_json::json!(reasoning_effort.as_str());

        let thinking_enabled = config.thinking_enabled.unwrap_or(self.default_thinking_enabled);
        body["thinking"] = serde_json::json!({
            "type": if thinking_enabled { "enabled" } else { "disabled" }
        });

        if let Some(tools) = &config.tools {
            body["tools"] = serde_json::to_value(tools).unwrap();
            if let Some(tool_choice) = &config.tool_choice {
                body["tool_choice"] = serde_json::json!(tool_choice);
            }
        }

        if let Some(structured) = &config.structured_output {
            if structured.enabled {
                if let Some(schema) = &structured.json_schema {
                    body["response_format"] = serde_json::json!({
                        "type": "json_schema",
                        "json_schema": schema
                    });
                } else {
                    body["response_format"] = serde_json::json!({
                        "type": "json_object"
                    });
                }
            }
        }

        tokio::spawn(async move {
            let result = client
                .post("https://api.deepseek.com/chat/completions")
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
                    "DeepSeek API error {}: {}",
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
                        if let Ok(s) = std::str::from_utf8(&bytes) {
                            buffer.push_str(s);
                        }
                        let mut pos = 0usize;
                        while let Some(end) = buffer[pos..].find('\n') {
                            let abs_end = pos + end;
                            let line = &buffer[pos..abs_end];
                            pos = abs_end + 1;
                            if let Some(data) = line.strip_prefix("data: ") {
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
                                    // 诊断日志：记录原始 SSE 事件
                                    if let Some(choices) = event["choices"].as_array() {
                                        for choice in choices {
                                            let delta_content = choice["delta"]["content"].as_str().unwrap_or("");
                                            let delta_reasoning = choice["delta"]["reasoning_content"].as_str().unwrap_or("");
                                            let finish = choice["finish_reason"].as_str().unwrap_or("");
                                            if !delta_content.is_empty() || !delta_reasoning.is_empty() || !finish.is_empty() {
                                                tracing::debug!(
                                                    "DeepSeek SSE: content_len={} reasoning_len={} finish_reason={}",
                                                    delta_content.len(), delta_reasoning.len(), finish
                                                );
                                            }
                                            // DeepSeek 思考模型：reasoning_content 是思维链，content 是最终回答
                                            let reasoning = choice["delta"]["reasoning_content"].as_str();
                                            let content = choice["delta"]["content"].as_str();

                                            // 处理 tool_calls delta
                                            if let Some(tool_calls) = choice["delta"]["tool_calls"].as_array() {
                                                let mut parsed_calls: Vec<ToolCall> = Vec::new();
                                                for tc in tool_calls {
                                                    let function = &tc["function"];
                                                    let call = ToolCall {
                                                        id: tc["id"].as_str().unwrap_or("").to_string(),
                                                        r#type: "function".to_string(),
                                                        function: ToolCallFunction {
                                                            name: function["name"].as_str().unwrap_or("").to_string(),
                                                            arguments: function["arguments"].as_str().unwrap_or("").to_string(),
                                                        },
                                                    };
                                                    parsed_calls.push(call);
                                                }
                                                if !parsed_calls.is_empty() {
                                                    let _ = tx.send(Ok(Chunk {
                                                        content: String::new(),
                                                        reasoning_content: None,
                                                        finish_reason: None,
                                                        index: chunk_index,
                                                        usage: None,
                                                        tool_calls: parsed_calls,
                                                    })).await;
                                                    chunk_index += 1;
                                                }
                                            }

                                            // 发送思维链（如果有）
                                            if let Some(reasoning_text) = reasoning.filter(|s| !s.is_empty()) {
                                                let _ = tx.send(Ok(Chunk {
                                                    content: String::new(),
                                                    reasoning_content: Some(reasoning_text.to_string()),
                                                    finish_reason: None,
                                                    index: chunk_index,
                                                    usage: None,
                                                    tool_calls: Vec::new(),
                                                })).await;
                                                chunk_index += 1;
                                            }

                                            // 发送最终回答（如果有）
                                            if let Some(text) = content.filter(|s| !s.is_empty()) {
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
                        buffer.drain(..pos);
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
