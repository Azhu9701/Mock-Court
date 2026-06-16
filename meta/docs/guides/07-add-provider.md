# 接入新的 AI 提供商

## Gateway Trait

```rust
pub trait Gateway: Send + Sync {
    fn provider(&self) -> Provider;
    fn is_available(&self) -> bool;
    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>>;
}
```

实现这个 trait 就能接入任何 LLM 服务。

## 示例：接入 Groq

```rust
// rust/my-agent-app/src/providers/groq.rs

use ai_gateway::{Gateway, Prompt, CallConfig, Chunk, Provider, UsageStats};
use tokio::sync::mpsc;

pub struct GroqGateway {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl GroqGateway {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.groq.com/openai/v1".into(),
            client: reqwest::Client::new(),
        }
    }
}

impl Gateway for GroqGateway {
    fn provider(&self) -> Provider {
        Provider::Custom("groq".into())
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>> {
        let (tx, rx) = mpsc::channel(64);
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();
        let config = config.clone();
        let prompt_messages = prompt.to_openai_messages(); // 转成 OpenAI 格式

        tokio::spawn(async move {
            let body = serde_json::json!({
                "model": config.model.as_deref().unwrap_or("llama-3.1-70b"),
                "messages": prompt_messages,
                "temperature": config.temperature,
                "max_tokens": config.max_tokens,
                "stream": true,
            });

            let resp = match client
                .post(format!("{}/chat/completions", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(e.into())).await;
                    return;
                }
            };

            // 解析 SSE 流
            let mut stream = resp.bytes_stream();
            while let Some(Ok(chunk_bytes)) = stream.next().await {
                let text = String::from_utf8_lossy(&chunk_bytes);
                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" { break; }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            let content = json["choices"][0]["delta"]["content"]
                                .as_str().unwrap_or("");
                            let _ = tx.send(Ok(Chunk {
                                content: content.into(),
                                reasoning_content: None,
                                finish_reason: None,
                                index: 0,
                                usage: None,
                            })).await;
                        }
                    }
                }
            }
        });

        rx
    }
}
```

## 注册到 GatewayRegistry

```rust
// main.rs
let mut registry = GatewayRegistry::new(config);
registry.register(Arc::new(GroqGateway::new(&api_key)));

// 设为默认提供商
registry.set_preferred_provider(Provider::Custom("groq".into()));
```

## OpenAI 兼容提供商

如果你的提供商兼容 OpenAI Chat Completions API（vLLM、Ollama、Groq、Together AI 等），只需配置 URL：

```json
// data/providers.json
{
  "custom_providers": [
    {
      "name": "my-llama-server",
      "base_url": "http://192.168.1.100:8080/v1",
      "api_key": "not-needed",
      "models": ["llama-3.1-70b", "mixtral-8x7b"],
      "type": "openai_compatible"
    }
  ]
}
```

框架自动用 OpenAI 适配器连接。

## Prompt 格式转换

不同提供商使用不同消息格式。框架提供格式转换：

```rust
// OpenAI/兼容格式
let messages = prompt.to_openai_messages();

// Anthropic Claude 格式
let (system, messages) = prompt.to_claude_format();

// DeepSeek 格式（同 OpenAI）
let messages = prompt.to_openai_messages();
```
