# Logical Components — B3: AI Gateway

## Component Architecture

```
GatewayRegistry (src/lib.rs)
├── ClaudeClient (src/claude.rs)
│   └── POST api.anthropic.com/v1/messages → SSE stream
├── OpenAIClient (src/openai.rs)
│   └── POST api.openai.com/v1/chat/completions → SSE stream
├── DeepSeekClient (src/deepseek.rs)
│   └── POST api.deepseek.com/v1/chat/completions → SSE stream
└── PromptBuilder (src/prompt.rs)
    └── Tera templates (src/prompts/*.tera)
```

## Component: GatewayRegistry (`src/lib.rs`)

**职责**: Provider 注册、路由、生命周期管理。

```
GatewayRegistry {
    providers: HashMap<Provider, Arc<dyn Gateway>>
}

impl GatewayRegistry {
    new() -> Self                           // 自动检测并注册已配置的 provider
    list_providers() -> Vec<ProviderInfo>   // 所有 provider 可用性
    get(provider) -> Option<&Arc<dyn Gateway>>
    call(req) -> Result<Receiver<Chunk>>    // 单次调用
    call_parallel(reqs) -> Vec<JoinHandle<Receiver>>  // 并行调用
}
```

## Component: ClaudeClient (`src/claude.rs`)

**职责**: Anthropic Claude Messages API 调用 + SSE 解析。

```
ClaudeClient {
    api_key: Option<String>,
    model: String,
    client: reqwest::Client,
}

impl ClaudeClient {
    new() -> Self                          // 读环境变量
}

impl Gateway for ClaudeClient {
    call(prompt, config):
      1. 将 Prompt.messages 转换为 Anthropic Messages API 格式
         - messages: Vec<{role, content}>
         - system: 提取 role="system" 的消息作为顶层 system 参数
      2. POST https://api.anthropic.com/v1/messages
         Headers: x-api-key, anthropic-version, anthropic-beta
      3. 如果 config.stream → 启用 SSE 流式响应
      4. 使用 reqwest-eventsource 解析事件流:
         content_block_delta → tx.send(chunk)
         message_stop → tx关闭
```

## Component: OpenAIClient (`src/openai.rs`)

**职责**: OpenAI Chat Completion API 调用 + SSE 解析。

```
OpenAIClient {
    api_key: Option<String>,
    model: String,
    client: reqwest::Client,
}

impl Gateway for OpenAIClient {
    call(prompt, config):
      1. 将 Prompt.messages 转换为 OpenAI Chat Completion 格式
         - messages: Vec<{role, content}>
         - system role 作为 messages[0]
      2. POST https://api.openai.com/v1/chat/completions
         Headers: Authorization: Bearer {api_key}
      3. Body: { model, messages, stream: true, temperature, max_tokens }
      4. 解析 SSE: data: {"choices":[{"delta":{"content":"..."}}]}
         - 遇到 data: [DONE] → tx关闭
```

## Component: DeepSeekClient (`src/deepseek.rs`)

**职责**: DeepSeek API 调用（兼容 OpenAI 格式）。

```
DeepSeekClient {
    api_key: Option<String>,
    model: String,
    client: reqwest::Client,
}

// 与 OpenAIClient 共享几乎相同的实现
// 仅 base_url 不同: https://api.deepseek.com/v1/chat/completions
```

## Component: PromptBuilder (`src/prompt.rs`)

**职责**: 加载模板、渲染 prompt、构建 Prompt 结构。

```
PromptBuilder {
    tera: Tera,
}

impl PromptBuilder {
    new() -> Result<Self>                  // 加载 src/prompts/*.tera
    build_summon_prompt(soul, task) -> Result<Prompt>
    build_synthesis_prompt(outputs, task) -> Result<Prompt>
    build_review_prompt(soul) -> Result<Prompt>
    build_debate_prompt(soul_a, soul_b, topic) -> Result<Prompt>
    build_relay_prompt(soul, prev_output, task) -> Result<Prompt>
    build_practice_prompt(data, soul) -> Result<Prompt>
}
```

## File Structure

```
rust/ai-gateway/
├── Cargo.toml
└── src/
    ├── lib.rs          # Gateway trait + GatewayRegistry
    ├── claude.rs       # ClaudeClient + impl Gateway
    ├── openai.rs       # OpenAIClient + impl Gateway
    ├── deepseek.rs     # DeepSeekClient + impl Gateway
    ├── prompt.rs       # PromptBuilder + Tera context building
    └── prompts/
        ├── summon_prompt.tera
        ├── synthesis_prompt.tera
        ├── review_prompt.tera
        ├── debate_prompt.tera
        ├── relay_prompt.tera
        └── practice_opening.tera
```
