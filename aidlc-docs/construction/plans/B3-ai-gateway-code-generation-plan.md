# Code Generation Plan — B3: AI Gateway

## Unit Context
- **Unit**: B3 AI Gateway
- **Crate**: `rust/ai-gateway/`
- **Dependencies**: `foundation`
- **Stories covered**: FR2.1-2.6, FR3.1-3.3, FR5.4

## Plan Steps

### Foundation 类型扩展
- [x] Step 1: 更新 `rust/foundation/src/models.rs` — 添加 `Provider`, `Prompt`, `PromptMessage`, `CallConfig`, `Chunk`, `LLMRequest`, `LLMResponse`, `UsageStats`

### Crate 初始化
- [x] Step 2: 创建 `rust/ai-gateway/Cargo.toml` — reqwest + reqwest-eventsource + tera
- [x] Step 3: 更新 workspace `Cargo.toml` — 添加 ai-gateway 成员

### 核心代码
- [x] Step 4: 创建 `rust/ai-gateway/src/lib.rs` — Gateway trait + GatewayRegistry
- [x] Step 5: 创建 `rust/ai-gateway/src/claude.rs` — ClaudeClient
- [x] Step 6: 创建 `rust/ai-gateway/src/openai.rs` — OpenAIClient
- [x] Step 7: 创建 `rust/ai-gateway/src/deepseek.rs` — DeepSeekClient
- [x] Step 8: 创建 `rust/ai-gateway/src/prompt.rs` — PromptBuilder + Tera（6 个模板内嵌为常量）
- [x] Step 9: 模板已内嵌在 prompt.rs 中，无需独立文件

### 验证
- [x] Step 10: `cargo check` — 0 errors, 0 warnings

## File List
| File | Purpose |
|------|---------|
| `rust/foundation/src/models.rs` | 新增 LLM 类型 |
| `rust/ai-gateway/Cargo.toml` | Crate deps |
| `Cargo.toml` | Workspace 成员 |
| `rust/ai-gateway/src/lib.rs` | Trait + Registry |
| `rust/ai-gateway/src/claude.rs` | Claude client |
| `rust/ai-gateway/src/openai.rs` | OpenAI client |
| `rust/ai-gateway/src/deepseek.rs` | DeepSeek client |
| `rust/ai-gateway/src/prompt.rs` | Prompt builder |
| `rust/ai-gateway/src/prompts/*.tera` | 6 模板文件 |
