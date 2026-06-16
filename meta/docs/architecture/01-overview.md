# 系统架构

## 全景

```
用户 (浏览器) ──▶ Next.js 前端 ──▶ Axum API ──▶ Possession Engine ──▶ AI Gateway ──▶ Claude/OpenAI/DeepSeek
       ▲               │                │                │                    │
       │               │                │                │                    │
       └─── WebSocket ◀─┘                │                │                    │
            (实时流式)                    │                │                    │
                                         ▼                ▼                    ▼
                                    SQLite DB        Memory Graph         LLM Cache
                                    (会话/消息/       (跨轮记忆/           (SHA256 哈希
                                     归档/知识)        矛盾检测)            命中缓存)
```

## 请求生命周期

```
1. POST /api/v1/possess/analyze
   用户输入任务描述
        │
2. Triage 分流
   分析任务 → 匹配模式 (Single/Conference/Debate/...)
   匹配最佳 Agent 组合
        │
3. SSE 流式返回分析结果
   前端展示推荐 Agent + 模式
   用户确认/调整
        │
4. POST /api/v1/possess
   确认启动会话
        │
5. Possession Engine 并行调度
   ├─ Agent A → Gateway.call() → Stream Tokens ──┐
   ├─ Agent B → Gateway.call() → Stream Tokens ──┤
   └─ Agent C → Gateway.call() → Stream Tokens ──┤
                                                  ▼
6. Semantic Collision Detection (实时)
   检测矛盾/互补/盲区 → 注入碰撞提示
        │
7. Synthesis 辩证综合
   汇总所有 Agent 输出 + 碰撞结果
   生成综合意见
        │
8. WebSocket 广播全程事件
   ┌─ soul_token       (每个 Agent 的流式片段)
   ├─ soul_done        (Agent 输出完成)
   ├─ collision        (检测到碰撞)
   ├─ synthesis_chunk  (综合流式片段)
   ├─ synthesis_done   (综合完成)
   ├─ cost             (Token 消耗)
   └─ session_complete (会话结束)
        │
9. 前端实时渲染
   useWebSocket hook → 50ms 节流 → React State → 多列视图
        │
10. 会话归档
    Archive System → SQLite (会话/消息/调用记录)
```

## Crate 分层

```
┌──────────────────────────────────────────┐
│ api                                       │  HTTP/WS 服务
│ Axum 路由 + 中间件 + WebSocket 处理        │
├──────────────────────────────────────────┤
│ possession                                │  编排核心
│ 6 种模式 + 碰撞检测 + 流式管线 + 记忆图谱   │
├──────────────┬──────────────┬─────────────┤
│ registry     │ ai-gateway   │ archive     │  领域服务
│ Agent 索引    │ AI 提供商网关 │ 归档/成本    │
│ 搜索/匹配     │ 缓存/路由     │ 分析统计     │
├──────────────┴──────────────┴─────────────┤
│ foundation                                 │  基础设施
│ 共享类型 + Storage trait + 错误 + 配置      │
└──────────────────────────────────────────┘
```

依赖方向：自上而下，无循环依赖。

## 6 种内置推理模式

| 模式 | Agent 数 | 流程 | 适用场景 |
|------|---------|------|---------|
| **Single** | 1 | 单轮问答 | 快速咨询 |
| **Conference** | 2-5 | 并行输出 → 碰撞检测 → 综合 | 复杂问题多视角分析 |
| **Debate** | 2 | 对立两栏 → 多轮攻防 → 裁决 | 对立观点辨析 |
| **Relay** | 2+ | 串行传递 → 每阶段接力 | 多步骤任务推进 |
| **Learn** | 2 | 一 Agent 教另一 Agent（费曼法） | 知识传授 |
| **Practice Opening** | 1+ | 四阶段循环（调研→消化→修正→行动） | 方法论实践 |

## WebSocket 事件体系

| 事件类型 | 方向 | 说明 |
|---------|------|------|
| `session_started` | S→C | 会话已创建 |
| `soul_started` | S→C | 某个 Agent 开始输出 |
| `soul_token` | S→C | Agent 流式文本片段 |
| `soul_done` | S→C | Agent 输出完成 |
| `soul_error` | S→C | Agent 输出异常 |
| `collision` | S→C | 检测到 Agent 间矛盾/互补/盲区 |
| `synthesis_started` | S→C | 综合阶段开始 |
| `synthesis_chunk` | S→C | 综合流式片段 |
| `synthesis_done` | S→C | 综合完成 |
| `cost` | S→C | Token 消耗统计 |
| `tool_call_started` | S→C | 工具调用开始 |
| `tool_result` | S→C | 工具返回结果 |
| `session_complete` | S→C | 会话完成 |
| `error` | S→C | 系统错误 |
| `intervene` | C→S | 用户实时干预（追问/纠正） |
