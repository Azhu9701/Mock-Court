# NFR Design Plan — B4: Possession Core

## Plan Steps

- [x] Step 1: 创建 `nfr-design-patterns.md` — 设计模式文档
- [x] Step 2: 创建 `logical-components.md` — 逻辑组件与依赖关系

## NFR Design Questions

### Q1: WS 重连后消息续传策略
- [Answer]: B — 只续传当前进行中的流式输出，不重放历史（轻量续传）

### Q2: Graceful Shutdown 策略
- [Answer]: B — 等待所有活跃 LLM 调用完成（最长等 300s 超时），不接受新请求
