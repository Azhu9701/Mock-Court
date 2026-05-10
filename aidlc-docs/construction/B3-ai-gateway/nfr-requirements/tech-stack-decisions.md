# Tech Stack Decisions — B3: AI Gateway

## Decisions

### 1. HTTP Client: reqwest (Q1: A)

**决策**: 使用 `reqwest` crate。

**理由**:
- Rust 生态事实标准，最成熟
- 内置 HTTP/2、连接池、TLS（rustls）、streaming
- 支持 `Response::bytes_stream()` 用于 SSE 流式解析
- 与 tokio 深度集成

### 2. SSE 解析: reqwest-eventsource (Q2: C)

**决策**: 使用 `reqwest-eventsource` crate。

**理由**:
- reqwest 生态专用 SSE 扩展，解析 `data:`/`event:`/`id:` 字段
- 内置自动重连支持（本系统不需要但可用）
- 比手动解析更可靠（处理 `data:` 行分割、多行事件等边缘情况）
- 比 `eventsource-stream` 更轻量，且与 reqwest 无缝集成

### 3. 模板引擎: Tera

**决策**: 使用 `tera` crate（Q3 Functional Design 选择 B）。

**理由**:
- Rust 最成熟模板引擎，类 Jinja2/Django 语法
- 支持条件、循环、过滤器，满足 prompt 模板需求
- 模板字符串预编译，渲染性能好
- 编译时检查模板语法

### 4. 连接管理: reqwest 默认 (Q3: A)

**决策**: 使用 `reqwest::Client::new()` 默认配置。

**理由**:
- 24魂 × 最多 6 种模式，并发量不高
- 默认连接池配置（pool_max_idle_per_host=1）足够
- 简单优于过早优化

### 5. Async: tokio::spawn (Q4: B)

**决策**: 每个 LLM 调用 spawn 到独立 tokio task。

**理由**:
- 便于并行（channels 不阻塞）
- 支持取消（`JoinHandle::abort()`）
- 隔离错误：一个 task panic 不影响其他 task
- 调用方通过 `mpsc::Receiver` 异步消费 chunk

## Dependencies

```toml
[dependencies]
foundation = { path = "../foundation" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
reqwest = { version = "0.12", features = ["json", "stream"] }
reqwest-eventsource = "0.6"
tera = "1"
```

## Environment Variables

| Variable | Provider | Required |
|----------|----------|----------|
| `ANTHROPIC_API_KEY` | Claude | Yes |
| `ANTHROPIC_MODEL` | Claude | No (default: claude-sonnet-4-6) |
| `OPENAI_API_KEY` | OpenAI | Yes |
| `OPENAI_MODEL` | OpenAI | No (default: gpt-4o) |
| `DEEPSEEK_API_KEY` | DeepSeek | Yes |
| `DEEPSEEK_MODEL` | DeepSeek | No (default: deepseek-chat) |
