# WebSocket 事件参考

## 连接

```
ws://<host>:<port>/ws/possess/:session_id/:channel
```

| 参数 | 说明 |
|------|------|
| `session_id` | 会话 ID |
| `channel` | 频道名（`main` 为综合频道，也可指定 Agent 名订阅单 Agent 流） |

## 事件格式

```json
{
  "event_type": "soul_token",
  "session_id": "abc-123",
  "agent_name": "合同审查员",
  "data": { "content": "根据《民法典》第" },
  "timestamp": "2026-06-16T10:30:00Z"
}
```

## 事件类型

### Agent 流式事件

| event_type | data | 说明 |
|-----------|------|------|
| `soul_started` | `{ "agent_name": "..." }` | Agent 开始输出 |
| `soul_token` | `{ "content": "..." }` | 流式文本片段 |
| `soul_done` | `{ "agent_name": "...", "tokens": 1024 }` | Agent 输出完成 |
| `soul_error` | `{ "agent_name": "...", "error": "..." }` | Agent 异常 |
| `soul_calling` | `{ "agent_name": "..." }` | Agent 正在调用 LLM |

### 综合事件

| event_type | data | 说明 |
|-----------|------|------|
| `synthesis_started` | `{}` | 综合阶段开始 |
| `synthesis_chunk` | `{ "content": "..." }` | 综合流式片段 |
| `synthesis_done` | `{ "tokens": 512 }` | 综合完成 |

### 碰撞检测

| event_type | data | 说明 |
|-----------|------|------|
| `collision` | `{ "type": "contradiction\|complement\|blindspot", "agents": ["A","B"], "description": "..." }` | 检测到碰撞 |

### 工具调用

| event_type | data | 说明 |
|-----------|------|------|
| `tool_call_started` | `{ "agent_name": "...", "tool_name": "...", "params": {...} }` | 工具调用开始 |
| `tool_result` | `{ "agent_name": "...", "tool_name": "...", "result": "..." }` | 工具返回结果 |

### 成本

| event_type | data | 说明 |
|-----------|------|------|
| `cost` | `{ "prompt_tokens": 500, "completion_tokens": 200, "total_tokens": 700, "estimated_cost": 0.002 }` | Token 消耗 |

### 系统事件

| event_type | data | 说明 |
|-----------|------|------|
| `session_started` | `{ "session_id": "...", "mode": "conference", "agents": ["A","B","C"] }` | 会话创建 |
| `session_complete` | `{ "session_id": "..." }` | 会话完成 |
| `soul_recommendations` | `{ "recommendations": [...] }` | Agent 推荐列表 |
| `process_step` | `{ "step": "classification\|matching\|analysis", "message": "..." }` | 处理步骤 |
| `system_message` | `{ "message": "..." }` | 系统消息 |
| `error` | `{ "error": "..." }` | 系统错误 |

## 客户端发送事件（干预）

```json
{
  "event_type": "intervene",
  "session_id": "abc-123",
  "data": { "message": "请从法律角度补充分析" }
}
```

## 前端订阅示例

```typescript
const ws = new WebSocket(`ws://localhost:3001/ws/possess/${sessionId}/main`);

ws.onmessage = (event) => {
  const { event_type, agent_name, data } = JSON.parse(event.data);
  
  switch (event_type) {
    case "soul_token":
      appendContent(agent_name, data.content);
      break;
    case "soul_done":
      markComplete(agent_name);
      break;
    case "collision":
      showCollision(data);
      break;
    case "synthesis_chunk":
      appendSynthesis(data.content);
      break;
    case "session_complete":
      finish();
      break;
  }
};
```
