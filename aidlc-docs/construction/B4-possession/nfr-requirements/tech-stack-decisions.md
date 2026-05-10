# Tech Stack Decisions — B4: Possession Core

## Dependencies

| Crate | 用途 | 决策理由 |
|-------|------|----------|
| `foundation` | Storage trait, models | B1 依赖 |
| `registry` | SoulRegistry 魂查询 | B2 依赖 |
| `ai-gateway` | GatewayRegistry LLM 调用 | B3 依赖 |
| `tokio` | 异步运行时, spawn, mpsc | workspace 已有 |
| `serde` / `serde_json` | WsEvent JSON 序列化 | workspace 已有 |
| `uuid` | Session ID 生成 | workspace 已有 |
| `chrono` | 时间戳 | workspace 已有 |
| `tracing` | 日志 | workspace 已有 |
| `axum` | WebSocket 支持 | Q5: A — 与 B6 API crate 统一 |

## 不引入新依赖

WebSocket 使用 axum 内置支持，B4 crate 本身只需添加 axum 依赖用于 WebSocket 类型。实际 WS upgrade 由 B6 API crate 的 axum router 处理，B4 只负责 WS 消息逻辑。

## 架构决策

| 决策 | 选择 | 理由 |
|------|------|------|
| WS 实现 | axum 内置 | 与 B6 统一，零额外依赖链 |
| 频道模型 | tokio::mpsc::unbounded_channel | 与 B3 Gateway 的 SSE→WS 桥接一致 |
| Session 恢复 | SQLite + 内存重建 | Q4: C 完全恢复 |
| 并行调用 | tokio::spawn + JoinSet | 合议模式并行魂调用 + 超时控制 |
