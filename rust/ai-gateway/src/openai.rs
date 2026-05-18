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
    system_prompt_override: Option<String>,
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
            system_prompt_override: None,
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to build reqwest Client"),
        }
    }

    pub fn new_lmstudio() -> Self {
        let api_key = std::env::var("LMSTUDIO_API_KEY").ok()
            .filter(|k| !k.is_empty());
        let model = std::env::var("LMSTUDIO_MODEL").unwrap_or_else(|_| "local-model".into());
        let base_url = std::env::var("LMSTUDIO_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:1234/v1".into());
        let system_prompt_override = std::env::var("LMSTUDIO_SYSTEM_PROMPT").ok()
            .filter(|s| !s.is_empty());
        OpenAIClient {
            api_key,
            model,
            base_url,
            provider_type: Provider::LMStudio,
            system_prompt_override,
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
        let api_key = self.api_key.clone();
        let is_lmstudio = self.provider_type == Provider::LMStudio;
        if api_key.is_none() && !is_lmstudio {
            let _ = tx.try_send(Err(FoundationError::Validation(
                "OpenAI API key not configured".into(),
            )));
            return rx;
        }
        let model = config.model.clone().unwrap_or_else(|| self.model.clone());
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let config = config.clone();

        let messages: Vec<serde_json::Value> = {
            let mut msgs: Vec<serde_json::Value> = prompt
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

            // LM Studio: 注入系统提示词，追加在已有 system message 后面
            if let Some(ref sp) = self.system_prompt_override {
                if let Some(first) = msgs.first() {
                    if first["role"].as_str() == Some("system") {
                        // 找到最后一条连续的 system message，拼接在它后面
                        let last_sys_idx = msgs.iter()
                            .position(|m| m["role"].as_str() != Some("system"))
                            .map(|i| i.saturating_sub(1))
                            .unwrap_or(msgs.len() - 1);
                        let existing = msgs[last_sys_idx]["content"].as_str().unwrap_or("");
                        msgs[last_sys_idx]["content"] = serde_json::json!(format!("{}\n\n{}", existing, sp));
                    } else {
                        msgs.insert(0, serde_json::json!({"role": "system", "content": sp}));
                    }
                } else {
                    msgs.insert(0, serde_json::json!({"role": "system", "content": sp}));
                }
            }

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
                        // LM Studio 0.3.23+: 思考内容在 reasoning 字段
                        if content.is_empty() {
                            content = msg["reasoning"].as_str().unwrap_or("").to_string();
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
