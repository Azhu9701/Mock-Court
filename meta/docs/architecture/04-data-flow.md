# 数据流：从用户输入到前端渲染

## 完整请求链路

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. POST /api/v1/possess/analyze                                 │
│    Body: { task: "分析AI对就业的影响", judgment: "...", ... }     │
└──────────────────────────┬──────────────────────────────────────┘
                           │ axum handler (possess.rs)
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Triage 分流 (possession::triage)                              │
│    - 解析任务文本，提取关键词                                      │
│    - 匹配推理模式 (Single/Conference/Debate/...)                   │
│    - 从 registry 搜索匹配的 Agent 组合                            │
│    - 返回 EntryType + 推荐 Agent 列表                             │
└──────────────────────────┬──────────────────────────────────────┘
                           │ SSE stream
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. 前端展示分析结果                                               │
│    PossessionEntry 组件渲染推荐 Agent + 模式                       │
│    用户可调整 Agent 选择/模式                                      │
└──────────────────────────┬──────────────────────────────────────┘
                           │ 用户确认
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. POST /api/v1/possess                                          │
│    Body: { mode: "conference", agents: ["A","B","C"], ... }      │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Engine::start_possession()                                    │
│    - 创建 Session                                                │
│    - 初始化 WsSessionManager 广播通道                              │
│    - 按模式派发: modes::conference::run()                         │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. Conference 模式执行                                            │
│                                                                   │
│    for each agent:                                               │
│      ├─ 构建 Prompt (PromptBuilder + DomainProfile)               │
│      ├─ Gateway.call(LLMRequest) → mpsc::Receiver<Chunk>          │
│      │     ├─ Provider 选择 (OpenAI/Claude/DeepSeek/LMStudio)     │
│      │     ├─ HTTP POST to provider API (stream=true)             │
│      │     └─ 逐行解析 SSE → emit Chunk                          │
│      └─ Stream 管线:                                              │
│            ├─ 读取 Chunk → WsEvent::SoulToken                     │
│            ├─ WsSessionManager::broadcast()                       │
│            ├─ 50ms 节流 (缓冲批量推送)                             │
│            └─ SoulDone on finish                                  │
│                                                                   │
│    cross_detector::detect_all()  ← 所有 Agent 输出完毕             │
│      ├─ 识别矛盾 → WsEvent::Collision                             │
│      ├─ 识别互补 → 合并为综合提示                                   │
│      └─ 识别盲区 → 标注未覆盖角度                                   │
│                                                                   │
│    synthesis:                                                     │
│      ├─ 构建综合 Prompt (所有输出 + 碰撞结果)                       │
│      ├─ Gateway.call() → Stream                                   │
│      └─ WsEvent::SynthesisChunk → SynthesisDone                   │
│                                                                   │
│    memory_graph.add_memory()  ← 各 Agent 输出 + 综合               │
│    archive.archive_session()  ← 持久化                             │
│    WsEvent::SessionComplete                                       │
└─────────────────────────────────────────────────────────────────┘
```

## 前端 WebSocket 数据流

```
┌──────────────────────────────────────────────────────────────────┐
│ useWebSocket(sessionId)                                          │
│                                                                   │
│ 1. new WebSocket("ws://host/ws/possess/:sessionId/main")         │
│                                                                   │
│ 2. onmessage → JSON.parse → WsEvent                              │
│       │                                                           │
│       ├─ soul_token     → buffer[agent] += content               │
│       │                    throttle (50ms) → setState              │
│       ├─ soul_done      → mark agent complete                     │
│       ├─ collision      → setCollisions([...])                    │
│       ├─ synthesis_chunk → buffer["synthesis"] += content        │
│       ├─ cost           → setCost(total_tokens, estimated_$)      │
│       ├─ tool_call      → setToolCalls([...])                     │
│       └─ session_complete → setStatus("done")                     │
│                                                                   │
│ 3. Reconnection: exponential backoff (1s/2s/4s, max 3 retries)   │
│    Fallback: API fetch for completed session content              │
│                                                                   │
│ State → Component Tree:                                           │
│   SessionRunner                                                   │
│     ├─ ConferenceView (多列布局)                                   │
│     │   ├─ AgentChatBubble[] (流式文本 + 思考过程)                  │
│     │   └─ ToolCallIndicator                                      │
│     ├─ CollisionNotification (碰撞提示)                            │
│     └─ SynthesisPanel (综合面板)                                    │
└──────────────────────────────────────────────────────────────────┘
```

## 关键缓冲策略

| 位置 | 策略 | 原因 |
|------|------|------|
| Gateway → Engine | `mpsc::channel(64)` | 提供商标配，平衡内存与延迟 |
| Engine → WS | `broadcast` 64 容量 | 多订阅者共享，慢消费者不阻塞 |
| WS → 前端 | 50ms 节流批量 | 减少 React setState 次数，避免浏览器卡顿 |
| Memory Graph | 每 Agent 完成时写入 | 不阻塞推理，异步持久化 |
| LLM Cache | SHA256 哈希键 | 精确匹配缓存，TTL 3600s |
