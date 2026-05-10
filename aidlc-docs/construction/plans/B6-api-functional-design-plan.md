# Functional Design Plan — B6: API Layer

## Plan Steps

- [x] Step 1: 创建 `domain-entities.md` — HTTP request/response 类型
- [x] Step 2: 创建 `business-logic-model.md` — axum router 结构 + 中间件链
- [x] Step 3: 创建 `business-rules.md` — 路由规则/错误处理/CORS

## Design Questions

### Q1: API 路由前缀
- [Answer]: A — `/api/v1/...`（版本化前缀）

### Q2: WebSocket 路由
- [Answer]: C — `WS /ws/possess/{session_id}/{channel}` 按 session + channel 订阅

### Q3: CORS 策略
- [Answer]: A — 允许所有来源（本地使用）

### Q4: 错误响应格式
- [Answer]: A — `{ "error": "message" }` 纯文本

### Q5: 认证/授权
- [Answer]: A — 无需认证（本地单用户应用）
