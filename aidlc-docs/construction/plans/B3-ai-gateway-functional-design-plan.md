# Functional Design Plan — B3: AI Gateway

## Plan Checklist

- [x] Generate `domain-entities.md` — Provider, Prompt, CallConfig, Chunk, LLMRequest/Response
- [x] Generate `business-logic-model.md` — Gateway trait, Provider impls, streaming, prompt builder
- [x] Generate `business-rules.md` — 错误处理、重试策略、超时、provider 路由规则
- [x] B3 does NOT include frontend → skip frontend-components.md

## Design Questions

### Q1: Provider 抽象
[Answer]: A — Gateway trait，每个 provider 实现该 trait

### Q2: 流式响应策略
[Answer]: A — mpsc channel 逐 chunk 推送

### Q3: Prompt 模板管理
[Answer]: B — 模板引擎（Tera）

### Q4: Provider 速率限制
[Answer]: C — 无限制，依赖 provider API 自身 rate limit 响应处理

### Q5: Provider API Key 配置
[Answer]: A — 环境变量读取 API Key
