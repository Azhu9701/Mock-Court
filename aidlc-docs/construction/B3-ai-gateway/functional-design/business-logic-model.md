# Business Logic Model — B3: AI Gateway

## Gateway Trait

```rust
use tokio::sync::mpsc;

pub trait Gateway: Send + Sync {
    fn provider(&self) -> Provider;
    fn is_available(&self) -> bool;
    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>>;
}
```

**设计要点**:
- `call()` 返回 `mpsc::Receiver`（Q2 选择 channel 推送），调用方通过 receiver 逐 chunk 消费
- `is_available()` 检查 API key 是否配置
- trait object 通过 `Arc<dyn Gateway>` 在 provider registry 中注册

## GatewayRegistry — Provider 注册中心

```rust
pub struct GatewayRegistry {
    providers: HashMap<Provider, Arc<dyn Gateway>>,
}

impl GatewayRegistry {
    pub fn new(providers: Vec<Arc<dyn Gateway>>) -> Self
    pub fn list_providers(&self) -> Vec<ProviderInfo>
    pub fn get(&self, provider: &Provider) -> Option<&Arc<dyn Gateway>>
    pub fn call(&self, req: &LLMRequest) -> Result<mpsc::Receiver<Result<Chunk>>>
    pub async fn call_parallel(&self, requests: &[LLMRequest])
        -> Vec<(Provider, mpsc::Receiver<Result<Chunk>>)>
}
```

**并行调用**: `call_parallel()` 对每个 request 调用对应 provider 的 `call()`，所有调用并发发起。

## Provider 实现

### ClaudeClient

```
ClaudeClient::new() 读取 ANTHROPIC_API_KEY 环境变量
  ├── base_url: "https://api.anthropic.com/v1/messages"
  ├── api_version: "2023-06-01"
  └── model: 从环境变量 ANTHROPIC_MODEL 或默认 "claude-sonnet-4-6"

call(prompt, config):
  1. 将 Prompt.messages 转换为 Anthropic Messages API 格式
  2. POST /v1/messages with SSE headers
  3. 解析 SSE 事件流
  4. 对每个 content_block_delta 事件 → chunk.send(Chunk { content, ... })
  5. 流结束后关闭 sender
```

### OpenAIClient

```
OpenAIClient::new() 读取 OPENAI_API_KEY 环境变量
  ├── base_url: "https://api.openai.com/v1/chat/completions"
  └── model: 从环境变量 OPENAI_MODEL 或默认 "gpt-4o"

call(prompt, config):
  1. 将 Prompt.messages 转换为 OpenAI Chat Completion 格式
  2. POST {base_url} with "stream: true"
  3. 解析 SSE 事件流 (data: [DONE])
  4. 对每个 delta.content → chunk.send(Chunk { content, ... })
```

### DeepSeekClient

```
DeepSeekClient::new() 读取 DEEPSEEK_API_KEY 环境变量
  ├── base_url: "https://api.deepseek.com/v1/chat/completions"
  └── model: "deepseek-chat"

call(prompt, config):
  与 OpenAIClient 相同（DeepSeek 兼容 OpenAI API 格式）
```

## PromptBuilder

```rust
pub struct PromptBuilder {
    templates: HashMap<String, Tera>,
}
```

**模板文件**（内嵌在 `src/prompts/` 下）:

| 模板 | 用途 | 变量 |
|------|------|------|
| `summon_prompt.tera` | 魂召唤 | `{soul.name}`, `{soul.field}`, `{soul.ismism_code}`, `{soul.summon_prompt}`, `{task}` |
| `synthesis_prompt.tera` | 辩证综合 | `{outputs}` (魂输出数组), `{task}` |
| `review_prompt.tera` | 审查 | `{soul.name}`, `{soul.output}`, `{soul.summon_prompt}` |
| `debate_prompt.tera` | 辩论 | `{soul_a.name}`, `{soul_b.name}`, `{topic}` |
| `relay_prompt.tera` | 接力 | `{soul.name}`, `{prev_output}`, `{task}` |
| `practice_opening.tera` | 实践开口 | `{practitioner_data}`, `{soul.name}` |

## 数据流

```
Possession Engine (B4)
  → PromptBuilder::build_summon_prompt(soul, task)
    → Prompt { messages }
  → GatewayRegistry::call(LLMRequest)
    → ClaudeClient::call(prompt, config)
      → POST api.anthropic.com (SSE)
        → 解析 SSE → mpsc::Sender → Chunk
  → Possession Engine (B4) 接收 mpsc::Receiver
    → 逐 chunk 推送到 WebSocket
```
