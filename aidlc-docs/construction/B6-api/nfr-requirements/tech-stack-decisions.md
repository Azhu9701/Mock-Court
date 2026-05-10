# Tech Stack Decisions — B6: API Layer

## New Dependencies

| Crate | 用途 | 决策理由 |
|-------|------|----------|
| `axum` | HTTP 框架 + WebSocket | Rust 生态主流异步 Web 框架，与 tokio 深度集成 |
| `tower` | 中间件层 | axum 依赖，用于 CORS、tracing、error recovery 中间件 |
| `tower-http` | 标准中间件集 | cors, trace, timeout, limit 等开箱即用 |
| `tracing-subscriber` | 日志输出 | fmt layer 单行格式输出到 stderr |

## Workspace 已有依赖（复用）

| Crate | 用途 |
|-------|------|
| `tokio` | 异步运行时 |
| `serde` / `serde_json` | JSON 序列化 |
| `uuid` | Session ID 等 |
| `chrono` | 时间戳 |
| `tracing` | 结构化日志 |

## 不引入的依赖

| 候选 | 理由 |
|------|------|
| `axum-extra` | 不需要 typed-header、cookie 等扩展（无认证） |
| `tower-http::limit` | Q2: D — 不限制 body size，不需要 |
| `tower-http::rate-limit` | Q3: A — 不需要速率限制 |
| `rustls` / `tokio-rustls` | Q4: A — 不需要 TLS |
| `hyper` (直接) | axum 内置 hyper，不直接使用 |
| `jsonwebtoken` | 无认证需求 |

## 架构决策

| 决策 | 选择 | 理由 |
|------|------|------|
| Web 框架 | axum 0.7 | Rust 生态标准异步 Web 框架 |
| 状态注入 | `axum::Extension<Arc<AppState>>` | 类型安全，零开销 clone |
| 错误处理 | `Result<T, (StatusCode, Json<ApiError>)>` | 直接返回 HTTP 状态码 + 统一错误体 |
| JSON 提取 | `axum::Json<T>` | 自动反序列化 + 错误映射为 400 |
| 路径参数 | `axum::extract::Path<T>` | 自动提取 + 类型检查 |
| 查询参数 | `axum::extract::Query<T>` | 自动反序列化 query string |
| WS 实现 | `axum::extract::ws::WebSocketUpgrade` | axum 内置，零额外依赖 |
| CORS 中间件 | `tower_http::cors::CorsLayer::permissive()` | 一行配置全开放 |
| Tracing 中间件 | `tower_http::trace::TraceLayer` | 自动记录 method + path + status + latency |
| 超时中间件 | `tower_http::timeout::TimeoutLayer` | 30s 请求超时（Q1: A） |
