# Component Dependencies — 万民幡 Web Application

## Dependency Matrix

| | Registry | Possession | AI Gateway | Archive | Analytics | Storage | WebSocket |
|---|---|---|---|---|---|---|---|
| **Registry** | — | uses | | | uses | uses | |
| **Possession** | uses | — | uses | uses | uses | | uses |
| **AI Gateway** | | | — | | | | |
| **Archive** | | uses | | — | uses | uses | |
| **Analytics** | uses | | | uses | — | uses | |
| **Storage** | | | | | | — | |
| **WebSocket** | | uses | | | | | — |

## Communication Patterns

### Frontend ↔ Backend

| Pattern | Protocol | Use Case |
|---------|----------|----------|
| Request-Response | REST (axum) | CRUD: souls, sessions, analytics queries |
| Persistent Stream | WebSocket | Possession: soul output streaming, synthesis progress |
| Server Push | WebSocket | System notifications, progress updates |

### Backend Internal

| Pattern | Mechanism | Use Case |
|---------|-----------|----------|
| Direct Call | Function/method | Orchestration → Components |
| Event-driven | tokio channels | AI Gateway → WebSocket (output chunk → broadcast) |
| Async Parallel | tokio::spawn / join_all | Multi-soul conference execution |
| File-based | FS read/write | Archive ↔ Storage, Import/Export |

## Data Flow Diagrams

### Multi-Soul Conference (合议) Flow

```
User Input (task + soul selection)
        │
        ▼
PossessionEngine.start_conference()
        │
        ├──► SoulRegistry.get_soul(魂A) ──► Build summon_prompt
        ├──► SoulRegistry.get_soul(魂B) ──► Build summon_prompt
        ├──► SoulRegistry.get_soul(魂C) ──► Build summon_prompt
        │
        ▼
    tokio::join!(
        AIGateway.call_llm(魂A_prompt) ──► WebSocket.broadcast(soul/魂A)
        AIGateway.call_llm(魂B_prompt) ──► WebSocket.broadcast(soul/魂B)
        AIGateway.call_llm(魂C_prompt) ──► WebSocket.broadcast(soul/魂C)
    )
        │
        ▼ (all complete)
    AIGateway.call_llm(synthesis_prompt) ──► WebSocket.broadcast(synthesis)
        │
        ▼
    Archive.archive_soul_output(魂A) ──► Storage.fs_write()
    Archive.archive_soul_output(魂B) ──► Storage.fs_write()
    Archive.archive_soul_output(魂C) ──► Storage.fs_write()
    Archive.archive_synthesis(report)  ──► Storage.fs_write()
    Archive.record_call()              ──► Storage.db_execute()
```

### Practice Opening (实践开口) Flow

```
User Input (在场者叙述)
        │
        ▼
PossessionEngine.classify_entry() → EntryType::Practitioner
        │
        ▼
P1: Collect ──► Archive.field_data ──► Storage.fs_write()
        │
        ▼
P2: tokio::join!( 1-3 魂 analyze field_data )
    ├── 魂A: "framework vs data" mapping ──► WebSocket
    ├── 魂B: "framework vs data" mapping ──► WebSocket
        │
        ▼
P3: write revisions → SoulRegistry.update_soul() ──► Storage.fs_write()
        │
        ▼
P4: ActionMemo ──► Archive ──► WebSocket → User
```

### Soul Creation (收魂→炼化→审查) Flow

```
User Input (人物名)
        │
        ▼
SoulManager.start_collection(name)
        │
        ▼
AIGateway + WebSearch → raw/{name}/搜索素材.md
        │
        ▼
SoulManager.refine(raw) → SoulProfile (ismism + summon_prompt)
        │
        ▼
SoulManager.review(profile) → AIGateway.call_llm(review_prompt) → ReviewReport
        │
        ▼
SoulRegistry.add_soul(profile) → Storage.fs_write(souls/{name}.yaml)
Registry.update(registry.yaml)   → Storage.fs_write()
```

## Dependency Rules

1. **Storage Layer 是唯一的数据访问点** — 所有组件通过 Storage Layer 读写数据
2. **AI Gateway 是唯一的外部 API 调用点** — 所有组件不直接调用 LLM API
3. **WebSocket Manager 是唯一的实时通道** — 前端只通过 WebSocket 接收流式数据
4. **Possession Engine 不直接写文件** — 通过 Archive System 完成落盘
5. **落盘先于呈现** — Archive 写入完成后才通过 WebSocket 通知前端可展示
