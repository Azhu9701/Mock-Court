# NFR Requirements — F3: Possession UI

## Performance

| 指标 | 目标 | 策略 |
|------|------|------|
| 10 魂并行渲染 | 无卡顿 | useTransition 批量更新 (Q2: B) |
| 单 chunk 处理 | < 1ms | 字符串拼接，无 re-render |
| 首屏加载 | < 2s | Wizard 静态组件 |
| WS 连接建立 | < 500ms | 本地 loopback |

## WebSocket (Q1: A — 原生 API)

| 要求 | 描述 |
|------|------|
| 客户端 | 原生 `new WebSocket(url)`，零依赖 |
| 重连 | 手写 exponential backoff (1s→2s→4s, max 3) |
| 心跳 | 依赖 axum 内置 ping/pong (30s) |
| 消息格式 | JSON `WsEvent` 解析 |
| 断线检测 | onclose 事件触发重连 |

## 渲染优化 (Q2: B — useTransition)

```typescript
const [isPending, startTransition] = useTransition();

function onChunk(event: WsEvent) {
  // 直接更新数据（不触发重渲染）
  bufferRef.current[event.soul_name] += event.payload;
  // 批量提交渲染
  startTransition(() => {
    setMessages({ ...bufferRef.current });
  });
}
```

## 自动滚动 (Q3: C — 仅 streaming)

| 规则 | 描述 |
|------|------|
| Streaming 中 | 新 chunk → 自动 scrollToBottom |
| done 后 | 停止自动滚动，用户自由浏览 |
| 手动上滚 | 不中断 streaming 渲染 |

## 并发

| 指标 | 目标 |
|------|------|
| 同时活跃 WS 连接 | 1 (单次附体) |
| scrollToBottom 频率 | 随 chunk rate (max ~10/s) |

## 可靠性

| 要求 | 描述 |
|------|------|
| 断线重连 | exponential backoff 1s-2s-4s, max 3次 |
| 刷新恢复 | 读取 URL sessionId → 重连 WS |
| 错误降级 | WS 断开显示 overlay，不丢失已接收内容 |
