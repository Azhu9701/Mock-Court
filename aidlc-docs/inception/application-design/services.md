# Service Layer — 万民幡 Web Application

## Service Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Next.js Frontend                      │
│  SoulRegistry UI │ Conversation UI │ PracticeOpening UI │
│  SoulManager UI  │ AnalyticsBoard  │  HistoryBrowser    │
└────────────┬──────────────────────────────┬──────────────┘
             │        WebSocket (streaming) │ REST (CRUD)
             ▼                              ▼
┌─────────────────────────────────────────────────────────┐
│                  axum API Layer                          │
│  REST Routes (souls, sessions, archive, analytics)      │
│  WebSocket Handler (possession streaming)               │
└────────────┬──────────────────────────────┬──────────────┘
             │                              │
    ┌────────▼────────┐          ┌─────────▼─────────┐
    │ Orchestration   │          │  Storage Service   │
    │ Service         │          │  (SQLite + FS)     │
    │                 │          └───────────────────┘
    │ PossessionEngine│
    │   + AIGateway   │
    │   + ArchiveSys  │
    └────────┬────────┘
             │
    ┌────────▼────────┐
    │  AI Gateway      │
    │  (Claude/OpenAI) │
    └─────────────────┘
```

## Service Definitions

### 1. Orchestration Service

**Purpose**: 编排多步骤工作流，协调组件间交互

**Responsibilities**:
- 接收前端请求，路由到对应模式
- 协调 Possession Engine + AI Gateway + Archive System
- 管理 WebSocket session 生命周期
- 入口分流判断（在场者检测）

**Key Interactions**:
- `POST /api/possess/single` → PossessionEngine.possess_single() → AI Gateway → WebSocket stream
- `POST /api/possess/conference` → PossessionEngine.start_conference() → parallel AI calls → Synthesis → Archive
- `POST /api/possess/debate` → PossessionEngine.start_debate() → sequential AI calls → Verdict → Archive
- `POST /api/practice-opening` → PossessionEngine.run_practice_opening_P1-P4() → Archive

**Orchestration Flows**:

```
单魂附体:
  Request → Match Soul → Build Prompt → AI Gateway.stream() → WebSocket → Archive

合议 (核心):
  Request → Match 3-4 Souls → [Spawn N parallel AI calls] → 
  Wait All Complete → Run Dialectical Synthesis → WebSocket → Archive(N+1 files)

辩论:
  Request → Soul A output → Soul B output (with A's argument) → 
  Verdict → WebSocket → Archive(3 files)

实践开口:
  Entry Detection → P1(Collect) → P2(Digest by souls) → 
  P3(Revise soul profiles) → P4(Action memo) → Archive
```

### 2. Storage Service

**Purpose**: 统一数据持久化接口

**Responsibilities**:
- SQLite 数据库管理（sessions、call-records、analytics）
- 文件系统管理（souls/、archive/、registry.yaml）
- 数据导入导出
- 迁移与备份

**Data Layout**:
```
data/
├── registry.yaml              # 魂注册表
├── souls/                     # 魂档案
│   ├── 列宁.yaml
│   ├── 毛泽东.yaml
│   └── ...
├── archive/                   # 对话存档
│   ├── 单魂/{魂名}/YYYY-MM-DD-{任务}.md
│   ├── 合议/{任务}/           # N+1 文件
│   ├── 辩论/{议题}/           # 3 文件
│   └── 接力/{任务}/           # M+1 文件
├── call-records.yaml          # 召唤记录
└── wanminfan.db               # SQLite (sessions, analytics, index)
```

### 3. Streaming Service

**Purpose**: WebSocket 实时通信管理

**Responsibilities**:
- WebSocket 连接建立与鉴权
- 多 channel 管理（每魂一个输出 channel + synthesis channel）
- 断线重连与状态恢复
- 心跳检测

**Channel Design**:
```
Session/{id}/
├── control          # 控制消息（start, stop, status）
├── soul/{name}      # 每魂独立输出流
├── synthesis        # 辩证综合输出
└── system           # 系统通知（errors, progress）
```
