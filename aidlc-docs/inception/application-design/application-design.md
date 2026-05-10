# Application Design — 万民幡 Web Application

## Architecture Overview

- **Frontend**: Next.js App Router + TypeScript
- **Backend**: Rust (axum) + tokio
- **Communication**: REST (CRUD) + WebSocket (streaming)
- **Storage**: SQLite + 本地文件系统
- **AI**: 多 provider Gateway (Claude API + OpenAI API + DeepSeek API)

## Component Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Next.js Frontend                        │
│  SoulRegistry │ Conversation │ PracticeOpening          │
│  SoulManager  │ Analytics    │ History                  │
└────────────┬──────────────────────────────┬──────────────┘
             │        WebSocket              │ REST
             ▼                               ▼
┌─────────────────────────────────────────────────────────┐
│                   axum API Layer                         │
│  REST Routes  │  WebSocket Handler  │  Middleware        │
└────────────┬──────────────────────────────┬──────────────┘
             │                              │
    ┌────────▼────────┐          ┌─────────▼─────────┐
    │ Orchestration   │          │  Storage Service   │
    │ Service         │          │  SQLite + FS       │
    │                 │          └───────────────────┘
    │ PossessionEngine│
    │ AIGateway       │
    │ ArchiveSystem   │
    │ AnalyticsEngine │
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │  AI Providers              │
    │  Claude │ OpenAI │ DeepSeek │
    └──────────────────────────┘
```

## Component Summary

| # | Component | Type | Key Responsibility |
|---|-----------|------|--------------------|
| 1 | Soul Registry UI | Frontend | 魂浏览、搜索、详情 |
| 2 | Conversation UI | Frontend | 多模式对话、终端式合议布局 |
| 3 | Practice Opening UI | Frontend | 四步流程向导+对话混合 |
| 4 | Soul Manager UI | Frontend | 收魂→炼化→审查界面 |
| 5 | Analytics Dashboard | Frontend | 召唤统计、有效性追踪 |
| 6 | History Browser | Frontend | 对话历史浏览回放 |
| 7 | Soul Registry Service | Backend | 魂数据管理与 ismism 索引 |
| 8 | Possession Engine | Backend | 六模式编排与入口分流 |
| 9 | AI Gateway | Backend | 多 provider LLM 调用 |
| 10 | Archive System | Backend | 存档落盘与完整性校验 |
| 11 | Analytics Engine | Backend | 统计聚合与检测告警 |
| 12 | Storage Layer | Backend | SQLite + FS 统一接口 |
| 13 | WebSocket Manager | Backend | 实时流式通信 |

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Frontend | Next.js App Router | SSR/SSG + API Routes，现代化 React 生态 |
| Backend | Rust axum | Tokio 异步生态，多魂并行天然匹配 |
| API Style | REST + WebSocket | REST 管理数据，WS 推送实时输出 |
| Storage | SQLite + FS | SQLite 索引会话/统计，FS 存魂档案 |
| AI Gateway | 多 Provider 抽象 | 支持 Claude/OpenAI/DeepSeek 切换，prompt caching |
| Stream | WebSocket | 单连接多 channel，适合多魂并行输出 |
| Layout | 类终端多 pane | 匹配现有 cmux 可视化模式 |

## Communication Patterns

| Pattern | Protocol | Use |
|---------|----------|-----|
| CRUD | REST | souls, sessions, analytics |
| Streaming | WebSocket | soul output, synthesis progress |
| Internal | tokio channels | AI → WS broadcast |
| Parallel | tokio::join! | multi-soul conference |
| Archive | FS write | 落盘优先于呈现 |

## Data Layout

```
data/
├── registry.yaml
├── souls/{name}.yaml
├── archive/{mode}/{task}/
├── call-records.yaml
└── wanminfan.db
```

## Dependency Rules

1. Storage Layer 是唯一数据访问点
2. AI Gateway 是唯一外部 API 调用点
3. WebSocket Manager 是唯一实时通道
4. Possession Engine 不直接写文件
5. 落盘先于呈现 — Archive 完成才通知前端
