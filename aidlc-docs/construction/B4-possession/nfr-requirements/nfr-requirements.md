# NFR Requirements — B4: Possession Core

## Performance

| 指标 | 目标 | 依据 |
|------|------|------|
| `classify_entry()` 延迟 | < 1ms | 纯规则匹配，无 I/O |
| 单魂 LLM 首字节时间 | < 5s | 取决于 LLM provider |
| 合议 N 魂并行总耗时 | < 300s | 最长单魂耗时即总耗时（并行） |
| WebSocket 消息延迟 | < 50ms | 本地 loopback |
| Session 恢复时间 | < 2s | SQLite 读取 + 内存重建 |

## 并发 (Q1: A, Q2: B)

| 指标 | 目标 |
|------|------|
| 最大并发 WS 连接 | 10（单用户本地使用） |
| Conference 最大并行魂数 | 10 |
| 最大同时活跃 session | 5（每 session 一个 WS 连接） |
| LLM 调用超时 | 300s（Q3: C） |

## Session 恢复 (Q4: C)

```
重启恢复流程:
  1. 启动时查询 SQLite sessions WHERE status = 'active'
  2. 对每个 active session:
     - 从 messages 表重建已完成的魂输出
     - 重新建立 WS 连接（客户端重连）
     - 未完成的魂调用重新发起 LLM 请求
     - 续传流式输出到重连的 WS
  3. 恢复完成后继续正常流程
```

## 可靠性

| 要求 | 描述 |
|------|------|
| 流式中断处理 | WS 断连不中断 LLM 调用，客户端重连后从已落盘内容恢复 |
| 落盘先于呈现 | 继承 B5 规则，魂输出完整内容先落盘再发 SoulDone |
| 并发安全 | WS session map 使用 RwLock，每个魂的 sender 独立 |
| Graceful shutdown | 收到 SIGTERM 后等待活跃 LLM 调用完成或超时 |

## WebSocket (Q5: A)

| 决策 | 选择 |
|------|------|
| 库 | axum 内置 WebSocket（升级 HTTP → WS） |
| 频道模型 | soul/{name} 独立 + system 统一（见 functional-design Q5） |
| 消息格式 | JSON `WsEvent { event_type, payload, soul_name, seq }` |
| 心跳 | 每 30s ping/pong |

## 存储

| 指标 | 目标 |
|------|------|
| Session 元数据 | SQLite（继承 B1） |
| Messages 持久化 | SQLite（继承 B1） |
| 魂输出落盘 | FS archive（继承 B5） |
| WS session 状态 | 内存 HashMap（重启时从 SQLite 重建） |
