# Tech Stack Decisions — F3: Possession UI

## New Dependencies

无新增外部依赖。全部使用浏览器原生 API 和 F1/F2 已有库。

## WS 实现 (Q1: A)

```typescript
// hooks/use-websocket.ts
function useWebSocket(url: string) {
  const wsRef = useRef<WebSocket | null>(null);
  const retryRef = useRef(0);

  function connect() {
    const ws = new WebSocket(url);
    ws.onmessage = (e) => handleMessage(JSON.parse(e.data));
    ws.onclose = () => {
      if (retryRef.current < 3) {
        setTimeout(connect, Math.pow(2, retryRef.current) * 1000);
        retryRef.current++;
      }
    };
    wsRef.current = ws;
  }
}
```

## Inherited Stack

| 库 | 来源 | 用途 |
|----|------|------|
| next 15 | F1 | App Router |
| tailwind 4 | F1 | 样式 |
| shadcn/ui | F1 | Button, Card, Textarea, Badge, Progress |
| lucide-react | F1 | 图标 |
| recharts | F2 | (留用，非必需) |
| react-hook-form + zod | F2 | Wizard 表单验证 |

## 不引入

| 候选 | 理由 |
|------|------|
| socket.io | 不需要 — 后端是 axum 原生 WS |
| reconnecting-websocket | Q1: A — 手写重连 |
| react-virtuoso | Q2: B — useTransition 足够 |
| framer-motion | 无复杂动画需求 |
