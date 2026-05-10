# NFR Requirements Plan — B4: Possession Core

## Plan Steps

- [x] Step 1: 创建 `nfr-requirements.md` — 性能/并发/可靠性要求
- [x] Step 2: 创建 `tech-stack-decisions.md` — WebSocket 库选择与依赖

## NFR Questions

### Q1: WebSocket 并发连接数
- [Answer]: A — 最多 10 个并发连接（单用户本地使用）

### Q2: 合议模式最大并行魂数
- [Answer]: B — 最多 10 个魂并行

### Q3: LLM 调用超时
- [Answer]: C — 300 秒（长文本/复杂推理允许更长时间）

### Q4: Session 状态恢复
- [Answer]: C — 完全恢复，包括 WS 重连和流式续传

### Q5: WebSocket 库选择
- [Answer]: A — axum 内置 WebSocket（与 B6 API crate 统一，零额外依赖）
