# Domain Entities — B3: AI Gateway

## New Types

### Provider — LLM Provider 枚举

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Provider {
    Claude,
    OpenAI,
    DeepSeek,
}
```

### ProviderInfo — Provider 元信息

```rust
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub provider: Provider,
    pub model: String,
    pub available: bool,
}
```

### Prompt + PromptMessage — 提示词

```rust
#[derive(Debug, Clone)]
pub struct Prompt {
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone)]
pub struct PromptMessage {
    pub role: String,  // "system" | "user" | "assistant"
    pub content: String,
}
```

### CallConfig — 调用配置

```rust
#[derive(Debug, Clone)]
pub struct CallConfig {
    pub temperature: f64,
    pub max_tokens: u32,
    pub stream: bool,
}

impl Default for CallConfig {
    fn default() -> Self {
        CallConfig {
            temperature: 0.7,
            max_tokens: 4096,
            stream: true,
        }
    }
}
```

### Chunk — 流式响应块

```rust
#[derive(Debug, Clone)]
pub struct Chunk {
    pub content: String,
    pub finish_reason: Option<String>,
    pub index: u32,
}
```

### LLMRequest / LLMResponse

```rust
#[derive(Debug, Clone)]
pub struct LLMRequest {
    pub provider: Provider,
    pub prompt: Prompt,
    pub config: CallConfig,
}

#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub provider: Provider,
    pub content: String,
    pub usage: UsageStats,
}

#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

## Relations

```
Gateway (trait)
├── ClaudeClient → reqwest → api.anthropic.com
├── OpenAIClient → reqwest → api.openai.com
└── DeepSeekClient → reqwest → api.deepseek.com

PromptBuilder
├── summon_prompt — 魂召唤 prompt 模板
├── synthesis_prompt — 辩证综合 prompt 模板
├── review_prompt — 审查 prompt 模板
├── debate_prompt — 辩论 prompt 模板
├── relay_prompt — 接力 prompt 模板
└── practice_opening_prompt — 实践开口 prompt 模板
```
