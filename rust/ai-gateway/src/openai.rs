use std::sync::Arc;

use parking_lot::RwLock;
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
            base_url: crate::relay_base_url("https://api.openai.com/v1"),
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

    /// 获取当前实际使用的 base URL
    fn effective_base_url(&self) -> String {
        if let Some(ref lock) = self.dynamic_base_url {
            let url = lock.read().clone();
            if !url.is_empty() {
                return url;
            }
        }
        self.base_url.clone()
    }

    /// 获取当前实际使用的 API key（优先 dynamic，fallback 到静态）
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

    /// 获取当前实际使用的模型名（优先 dynamic，fallback 到静态）
    fn effective_model(&self) -> String {
        if let Some(ref lock) = self.dynamic_model {
            let model = lock.read().clone();
            if !model.is_empty() {
                return model;
            }
        }
        self.model.clone()
    }
}

impl OpenAIClient {
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

        // LM Studio 开启全局结构化输出时，流式 + 自由文本 = 空内容。
        // Workaround：无 schema 时强制非流式，并提供一个自由文本 JSON schema，
        // 让模型输出 {"response": "..."}，后端解析提取。
        let use_stream = if is_lmstudio && config.structured_output.is_none() {
            false
        } else {
            config.stream || !is_lmstudio
        };
        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens.max(128),
            "stream": use_stream,
        });
        if use_stream {
            body["stream_options"] = serde_json::json!({ "include_usage": true });
        }
        // response_format 控制
        if let Some(ref so) = config.structured_output {
            if so.enabled {
                if let Some(ref schema) = so.json_schema {
                    body["response_format"] = serde_json::json!({
                        "type": "json_schema",
                        "json_schema": {
                            "name": "response",
                            "schema": schema,
                            "strict": true
                        }
                    });
                } else {
                    body["response_format"] = serde_json::json!({ "type": "json_object" });
                }
            }
        } else if is_lmstudio {
            // LM Studio 只支持 json_schema / text，不支持 json_object。
            // 用一个极简 schema 让模型输出 {"response": "..."}
            body["response_format"] = serde_json::json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "text_response",
                    "schema": {
                        "type": "object",
                        "properties": {
                            "response": { "type": "string" }
                        }
                    }
                }
            });
        }

        // Tool definitions
        if let Some(tools) = &config.tools {
            body["tools"] = serde_json::to_value(tools).unwrap();
            if let Some(tool_choice) = &config.tool_choice {
                body["tool_choice"] = serde_json::json!(tool_choice);
            }
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
                        tracing::debug!("LM Studio raw response json: {}", serde_json::to_string(&json).unwrap_or_default());
                        let msg = &json["choices"][0]["message"];
                        let mut content = msg["content"].as_str().unwrap_or("").to_string();
                        tracing::debug!("LM Studio message content: '{}'", content);
                        // LM Studio: 思考内容在 reasoning_content 字段
                        if content.is_empty() {
                            content = msg["reasoning_content"].as_str().unwrap_or("").to_string();
                            tracing::debug!("LM Studio fallback to reasoning_content: '{}'", content);
                        }
                        // LM Studio 全局结构化输出 workaround：提取 {"response": "..."} 中的文本
                        if is_lmstudio && content.trim_start().starts_with('{') {
                            tracing::debug!("LM Studio content looks like JSON, attempting parse");
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(text) = parsed["response"].as_str() {
                                    tracing::debug!("LM Studio extracted response: '{}'", text);
                                    content = text.to_string();
                                } else {
                                    tracing::debug!("LM Studio JSON parse ok but no 'response' field, keys: {:?}", parsed.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                                }
                            } else {
                                tracing::debug!("LM Studio JSON parse failed");
                            }
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
            // Accumulate tool calls across streaming deltas
            let mut accumulated_tool_calls: Vec<foundation::ToolCall> = Vec::new();

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
                                    // Flush any accumulated tool calls before finishing
                                    if !accumulated_tool_calls.is_empty() {
                                        let _ = tx.send(Ok(Chunk {
                                            content: String::new(),
                                            reasoning_content: None,
                                            finish_reason: Some("tool_calls".into()),
                                            index: chunk_index,
                                            usage: None,
                                            tool_calls: accumulated_tool_calls.clone(),
                                        })).await;
                                        accumulated_tool_calls.clear();
                                        chunk_index += 1;
                                    }
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
                                            // Parse tool_calls deltas (incremental)
                                            if let Some(tool_calls) = choice["delta"]["tool_calls"].as_array() {
                                                for tc in tool_calls {
                                                    let index = tc["index"].as_u64().unwrap_or(0) as usize;
                                                    while accumulated_tool_calls.len() <= index {
                                                        accumulated_tool_calls.push(foundation::ToolCall {
                                                            id: String::new(),
                                                            r#type: "function".to_string(),
                                                            function: foundation::ToolCallFunction {
                                                                name: String::new(),
                                                                arguments: String::new(),
                                                            },
                                                        });
                                                    }
                                                    if let Some(id) = tc["id"].as_str() {
                                                        accumulated_tool_calls[index].id = id.to_string();
                                                    }
                                                    if let Some(name) = tc["function"]["name"].as_str() {
                                                        accumulated_tool_calls[index].function.name = name.to_string();
                                                    }
                                                    if let Some(args) = tc["function"]["arguments"].as_str() {
                                                        accumulated_tool_calls[index].function.arguments.push_str(args);
                                                    }
                                                }
                                            }

                                            let content_text = choice["delta"]["content"].as_str();
                                            let reasoning_text = choice["delta"]["reasoning_content"].as_str();
                                            let finish = choice["finish_reason"].as_str();

                                            // On tool_calls finish reason, flush accumulated tool calls
                                            // Don't set finish_reason here — [DONE] will be the terminal chunk
                                            if finish == Some("tool_calls") && !accumulated_tool_calls.is_empty() {
                                                let _ = tx.send(Ok(Chunk {
                                                    content: String::new(),
                                                    reasoning_content: None,
                                                    finish_reason: None,
                                                    index: chunk_index,
                                                    usage: None,
                                                    tool_calls: accumulated_tool_calls.clone(),
                                                })).await;
                                                accumulated_tool_calls.clear();
                                                chunk_index += 1;
                                            }

                                            if content_text.is_some() || reasoning_text.is_some() {
                                                let _ = tx.send(Ok(Chunk {
                                                    content: content_text.unwrap_or("").to_string(),
                                                    reasoning_content: reasoning_text.map(|s| s.to_string()),
                                                    finish_reason: finish.map(|s| s.to_string()),
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
            // Stream EOF without [DONE] — flush any remaining tool calls then send terminal
            if !accumulated_tool_calls.is_empty() {
                let _ = tx.send(Ok(Chunk {
                    content: String::new(),
                    reasoning_content: None,
                    finish_reason: Some("tool_calls".into()),
                    index: chunk_index,
                    usage: None,
                    tool_calls: accumulated_tool_calls,
                })).await;
                chunk_index += 1;
            }
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
