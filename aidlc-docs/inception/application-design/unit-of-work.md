# Unit of Work — 万民幡 Web Application

## Decomposition Strategy

- **Architecture**: 本地单体应用，Monorepo
- **Backend**: Cargo workspace（6 crates）
- **Frontend**: Next.js App Router（4 组分）
- **Execution order**: 自底向上（B1 → B6 → F1 → F4）
- **Total units**: 10

## Code Organization

```
wanminfan/
├── Cargo.toml                 # workspace root
├── rust/
│   ├── foundation/            # B1: Storage Layer + data models
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── registry/              # B2: Soul Registry Service
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── ai-gateway/            # B3: AI Gateway
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── possession/            # B4: Possession Engine + WS
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── archive/               # B5: Archive + Analytics
│   │   ├── Cargo.toml
│   │   └── src/
│   └── api/                   # B6: axum routes + middleware
│       ├── Cargo.toml
│       └── src/
├── nextjs/                    # Next.js frontend
│   ├── app/
│   │   ├── layout.tsx         # F1: App Shell
│   │   ├── souls/             # F2: Soul Browser
│   │   ├── possess/           # F3: Possession UI
│   │   └── analytics/         # F4: Dashboard
│   └── ...
├── data/                      # runtime data (gitignored)
│   ├── registry.yaml
│   ├── souls/
│   ├── archive/
│   └── call-records.yaml
└── scripts/                   # import, migration, seed
    ├── import-souls.sh
    └── seed.sql
```

## Backend Units

### B1: Foundation
- **Crate**: `foundation`
- **Dependencies**: tokio, serde, rusqlite, serde_yaml
- **Key Artifacts**:
  - `src/lib.rs` — Storage trait + SQLite + FS impls
  - `src/models.rs` — Soul, Session, CallRecord data types
  - `src/config.rs` — app config
  - `migrations/` — SQL migration files

### B2: Soul Registry
- **Crate**: `registry` (depends on `foundation`)
- **Key Artifacts**:
  - `src/lib.rs` — SoulRegistry struct
  - `src/ismism.rs` — ismism 四维坐标解析与搜索
  - `src/search.rs` — 全文搜索 + 坐标过滤

### B3: AI Gateway
- **Crate**: `ai-gateway` (depends on `foundation`)
- **Key Artifacts**:
  - `src/lib.rs` — Gateway trait + provider registry
  - `src/claude.rs` — Claude API client
  - `src/openai.rs` — OpenAI API client
  - `src/deepseek.rs` — DeepSeek API client
  - `src/prompt.rs` — summon_prompt / synthesis_prompt 构建

### B4: Possession Core
- **Crate**: `possession` (depends on `foundation`, `registry`, `ai-gateway`)
- **Key Artifacts**:
  - `src/lib.rs` — PossessionEngine
  - `src/modes/single.rs` — 单魂附体
  - `src/modes/conference.rs` — 合议编排 + 辩证综合触发
  - `src/modes/debate.rs` — 辩论编排
  - `src/modes/relay.rs` — 接力编排
  - `src/modes/learning.rs` — 学习模式
  - `src/modes/practice_opening.rs` — 实践开口 P1-P4
  - `src/entry_classifier.rs` — 入口分流检测
  - `src/ws.rs` — WebSocket session 管理

### B5: Archive & Analytics
- **Crate**: `archive` (depends on `foundation`)
- **Key Artifacts**:
  - `src/lib.rs` — ArchiveSystem
  - `src/archive.rs` — 魂输出独立文件写入 + 完整性校验
  - `src/call_records.rs` — call-records CRUD
  - `src/analytics.rs` — 统计聚合 + 低效检测

### B6: API Layer
- **Crate**: `api` (depends on all B1-B5)
- **Key Artifacts**:
  - `src/main.rs` — axum server entry
  - `src/routes/souls.rs` — /api/souls REST routes
  - `src/routes/possess.rs` — /api/possess REST + WS routes
  - `src/routes/archive.rs` — /api/archive routes
  - `src/routes/analytics.rs` — /api/analytics routes
  - `src/middleware.rs` — logging, error handling, CORS

## Frontend Units

### F1: App Shell
- **Path**: `app/layout.tsx`, `app/page.tsx`
- **Components**: Navbar, Sidebar, ThemeProvider, ShellLayout

### F2: Soul Browser
- **Path**: `app/souls/`
- **Pages**: `/souls` (list), `/souls/[name]` (detail), `/souls/manage` (CRUD)
- **Components**: SoulList, SoulDetail, SoulSearch, SoulComparison, CollectWizard, RefineView, ReviewPanel

### F3: Possession UI
- **Path**: `app/possess/`
- **Pages**: `/possess/single`, `/possess/conference`, `/possess/debate`, `/possess/relay`, `/possess/learn`
- **Components**: TermPanel (终端式 pane), ConferenceGrid (多 pane 终端布局), PossessionChat, PracticeOpeningWizard, WebSocketProvider

### F4: Dashboard
- **Path**: `app/analytics/`, `app/history/`
- **Components**: StatsChart, SoulEffectivenessTable, UnsummonedAlert, HistoryList, SessionReplay

## Execution Order (自底向上)

```
Phase 1: B1 Foundation ──────► 数据模型 + Storage 接口
Phase 2: B2 Registry ────────► depends on B1
         B3 AI Gateway ──────► depends on B1
         B5 Archive ─────────► depends on B1
Phase 3: B4 Possession ──────► depends on B2, B3
Phase 4: B6 API ─────────────► depends on B4, B5
Phase 5: F1 App Shell ───────► depends on B6 (API contract)
Phase 6: F2 Soul Browser ────► depends on F1
         F3 Possession UI ───► depends on F1, B4 (WS)
         F4 Dashboard ───────► depends on F1, B5
```

## Scope: 24 全量魂 + 全六种模式

所有功能和魂都在初始构建中实现，不做 MVP 缩减。
