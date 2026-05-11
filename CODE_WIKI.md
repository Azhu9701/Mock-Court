# 万民幡 (Soul Banner Lite) Code Wiki

> 这不是聊天界面——这是多视角同时碰撞的观测窗口。

**版本**: 1.0.0  
**最后更新**: 2026-05-11  
**技术栈**: Rust (Axum) + Next.js

---

## 目录

1. [项目概述](#1-项目概述)
2. [项目架构](#2-项目架构)
3. [模块依赖关系](#3-模块依赖关系)
4. [核心模块详解](#4-核心模块详解)
   - [4.1 Foundation 模块](#41-foundation-模块)
   - [4.2 Registry 模块](#42-registry-模块)
   - [4.3 AI Gateway 模块](#43-ai-gateway-模块)
   - [4.4 Archive 模块](#44-archive-模块)
   - [4.5 Possession 模块](#45-possession-模块)
   - [4.6 API 模块](#46-api-模块)
5. [关键数据结构](#5-关键数据结构)
6. [API 路由说明](#6-api-路由说明)
7. [数据库设计](#7-数据库设计)
8. [项目运行方式](#8-项目运行方式)
9. [配置说明](#9-配置说明)
10. [内置魂列表](#10-内置魂列表)

---

## 1. 项目概述

万民幡是一个**多 AI 人格并行推理系统**。它同时召唤多个具有独立世界观的思想家（称为"魂"），让他们围绕同一个问题展开思考、碰撞、辩论与综合。

### 核心特性

| 特性 | 说明 |
|------|------|
| 流式交叉检测 | 多魂并行输出时实时检测碰撞（矛盾/互补/盲区） |
| 魂长驻进程 | 每个魂作为独立 tokio task，支持状态管理和跨轮记忆 |
| 魂自我审计 | 输出后自动检测自我矛盾、边界违反、前提动摇 |
| 模型智能路由 | 根据任务类型自动选择模型和推理强度 |
| 成本透明化 | 实时 token 消耗统计、预估费用 |
| 全文+向量检索 | SQLite FTS5 全文搜索 + 余弦相似度向量检索 |

### 技术栈

- **后端**: Rust
  - Web 框架: Axum 0.7 + WebSocket
  - 数据库: SQLite (WAL 模式) + FTS5 全文搜索
  - AI 网关: Claude / OpenAI / DeepSeek 多提供商
  - 异步运行时: Tokio
- **前端**: Next.js (shadcn/ui + Tailwind CSS)

---

## 2. 项目架构

```
┌─────────────┐    ┌──────────────┐    ┌───────────────┐
│  Next.js 前端│───▶│  Axum API    │───▶│  AI Gateway   │
│  (shadcn/ui) │    │  (端口 3096)  │    │  Claude/GPT/DS│
└─────────────┘    └──────┬───────┘    └───────────────┘
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │ Registry │   │Possession│   │ Archive  │
    │  魂注册表 │   │  附体引擎 │   │  归档分析 │
    └──────────┘   └──────────┘   └──────────┘
```

### 项目目录结构

```
├── rust/
│   ├── foundation/    # 基础设施（SQLite/存储/错误处理/配置）
│   ├── registry/      # 魂注册表 + 全文检索
│   ├── ai-gateway/    # AI 模型网关（Claude/OpenAI/DeepSeek）
│   ├── archive/       # 归档 + 成本追踪 + 分析
│   ├── possession/    # 附体引擎（合议/辩论/接力等模式）
│   └── api/           # Axum HTTP API + WebSocket
├── nextjs/            # Next.js 前端
├── config/            # 配置文件
├── data/              # 数据目录（souls/archive/db）
├── scripts/           # 辅助脚本
└── start.sh           # 一键启动脚本
```

---

## 3. 模块依赖关系

```
Cargo.toml (workspace)
├── foundation (核心基础)
├── registry (魂注册表) → foundation
├── ai-gateway (AI网关) → foundation
├── archive (归档系统) → foundation
├── possession (附体引擎) → foundation, registry, ai-gateway
└── api (HTTP API) → foundation, registry, ai-gateway, archive, possession
```

### Workspace 依赖

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
r2d2 = "0.8"
r2d2_sqlite = "0.24"
dashmap = "5"
sha2 = "0.10"
hex = "0.4"
```

---

## 4. 核心模块详解

### 4.1 Foundation 模块

**路径**: `rust/foundation/src/`

Foundation 是整个项目的基础设施层，提供配置管理、错误处理、数据存储、SQLite 数据库封装和向量搜索等功能。

#### 4.1.1 模块导出 (lib.rs)

```rust
pub mod config;        // 配置管理
pub mod error;         // 错误类型定义
pub mod fs_store;      // 文件系统存储
pub mod health;        // 健康检查
pub mod models;        // 数据模型
pub mod sqlite;        // SQLite 数据库封装
pub mod storage;       // 存储抽象
pub mod vector_search; // 向量搜索
```

#### 4.1.2 核心类型

| 类型 | 说明 |
|------|------|
| `Config` | 应用配置结构 |
| `FoundationError` | 错误类型枚举 |
| `Result<T>` | 结果类型别名 |
| `Storage` | 存储抽象 trait |
| `SqliteDb` | SQLite 数据库实现 |

#### 4.1.3 关键 Trait

**Storage Trait** - 定义存储层接口

```rust
pub trait Storage: Send + Sync {
    // Soul 操作
    async fn list_soul_names(&self) -> Result<Vec<String>>;
    async fn read_soul(&self, name: &str) -> Result<SoulProfile>;
    async fn write_soul(&self, profile: &SoulProfile) -> Result<()>;
    async fn delete_soul(&self, name: &str) -> Result<()>;
    
    // Session 操作
    async fn create_session(&self, session: &Session) -> Result<()>;
    async fn get_session(&self, id: &str) -> Result<Session>;
    async fn update_session(&self, session: &Session) -> Result<()>;
    async fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>>;
    async fn delete_session(&self, id: &str) -> Result<()>;
    
    // Message 操作
    async fn append_message(&self, msg: &Message) -> Result<()>;
    async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>>;
    
    // Call Records
    async fn record_call(&self, record: &CallRecord) -> Result<()>;
    async fn query_call_records(&self, filter: &CallFilter) -> Result<Vec<CallRecord>>;
    
    // Archive
    async fn archive_soul_output(&self, session_id: &str, soul: &str, content: &str) -> Result<String>;
    async fn archive_synthesis(&self, session_id: &str, content: &str) -> Result<String>;
    
    // Knowledge
    async fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeResult>>;
    async fn rebuild_fts(&self) -> Result<usize>;
}
```

#### 4.1.4 配置管理 (config.rs)

```rust
pub struct Config {
    pub data_dir: String,           // 数据目录
    pub souls_dir: String,           // 魂文件目录
    pub archive_dir: String,         // 归档目录
    pub db_path: String,             // 数据库路径
    pub registry_path: String,       // 注册表路径
    pub call_records_path: String,   // 调用记录路径
    pub server_host: String,         // 服务器主机
    pub server_port: u16,            // 服务器端口
    pub nextjs_port: u16,            // 前端端口
    pub searxng_url: String,         // SearXNG 搜索服务 URL
}
```

---

### 4.2 Registry 模块

**路径**: `rust/registry/src/`

魂注册表模块负责管理和搜索"魂"（思想家人格）。

#### 4.2.1 模块结构

```rust
mod ismism;              // 主义主义四维坐标计算
mod search;              // 搜索算法
pub mod fulltext_search; // 全文搜索
```

#### 4.2.2 SoulRegistry 结构

```rust
pub struct SoulRegistry {
    store: Arc<dyn Storage>,              // 存储后端
    souls: DashMap<String, SoulProfile>,  // 魂内存缓存
    inverted_index: DashMap<String, Vec<String>>, // 倒排索引
}
```

#### 4.2.3 核心方法

| 方法 | 说明 |
|------|------|
| `new(store)` | 异步创建注册表实例 |
| `reload()` | 重新加载所有魂 |
| `list_souls(filter)` | 列出所有魂，支持过滤 |
| `get_soul(name)` | 获取指定魂 |
| `search_souls(query)` | 搜索魂 |
| `create_soul(profile)` | 创建新魂 |
| `update_soul(profile)` | 更新魂 |
| `delete_soul(name)` | 删除魂 |
| `get_ismism_distribution()` | 获取主义主义分布统计 |

#### 4.2.4 搜索算法

- **全文搜索**: 基于倒排索引的关键词匹配
- **最近邻搜索**: 基于主义主义四维坐标的余弦相似度搜索

---

### 4.3 AI Gateway 模块

**路径**: `rust/ai-gateway/src/`

AI 网关模块提供统一的 AI 模型调用接口，支持多个提供商。

#### 4.3.1 支持的提供商

| 提供商 | 模型 | 配置键 |
|--------|------|--------|
| Claude | claude-3-5-sonnet | `claude` |
| OpenAI | gpt-4o | `openai` |
| DeepSeek | deepseek-chat | `deepseek` |

#### 4.3.2 Gateway Trait

```rust
pub trait Gateway: Send + Sync {
    fn provider(&self) -> Provider;
    fn is_available(&self) -> bool;
    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>>;
}
```

#### 4.3.3 GatewayRegistry 结构

```rust
pub struct GatewayRegistry {
    providers: HashMap<Provider, Arc<dyn Gateway>>, // 提供商实例
    all_info: Vec<ProviderInfo>,                      // 提供商信息
    cache: Arc<RwLock<Option<Arc<LlMCache>>>>,        // LLM 缓存
}
```

#### 4.3.4 核心方法

| 方法 | 说明 |
|------|------|
| `new()` | 创建网关注册表，自动初始化所有提供商 |
| `list_providers()` | 列出所有可用提供商 |
| `get(provider)` | 获取指定提供商 |
| `call(req)` | 调用指定提供商的 LLM |
| `call_parallel(reqs)` | 并行调用多个 LLM |
| `set_cache(cache)` | 设置 LLM 缓存 |

#### 4.3.5 模型路由 (model_router.rs)

根据任务类型和模型能力自动选择最佳模型和推理强度：

```rust
pub enum ReasoningEffort {
    NonThink,  // 不使用深度推理（简单任务）
    Think,     // 标准推理（Pro模型默认）
    ThinkHigh, // 深度推理（复杂分析）
    ThinkMax,  // 最大深度推理（综合/自我审计）
}
```

#### 4.3.6 缓存机制 (cache.rs)

- 使用 SHA256 哈希缓存 LLM 响应
- 默认 TTL: 3600 秒
- 缓存键: `provider:model:system_prompt:user_prompt`

---

### 4.4 Archive 模块

**路径**: `rust/archive/src/`

归档模块负责会话归档、成本追踪和分析。

#### 4.4.1 子模块

```rust
mod analytics;      // 数据分析
mod archive;         // 归档核心
pub mod audit;      // 审计引擎
mod call_records;   // 调用记录
pub mod cost_tracking; // 成本追踪
```

#### 4.4.2 ArchiveSystem 结构

```rust
pub struct ArchiveSystem {
    store: Arc<dyn Storage>,                              // 存储后端
    export_statuses: RwLock<HashMap<String, ExportStatus>>, // 导出状态
    summon_stats_cache: RwLock<Option<(SummonStats, Instant)>>, // 召唤统计缓存
    stats_ttl: Duration,                                  // 缓存 TTL
}
```

#### 4.4.3 核心方法

| 方法 | 说明 |
|------|------|
| `archive_soul_output()` | 归档魂输出 |
| `archive_synthesis()` | 归档综合结果 |
| `archive_debate()` | 归档辩论结果 |
| `record_call()` | 记录调用 |
| `query_call_records()` | 查询调用记录 |
| `get_summon_stats()` | 获取召唤统计 |
| `get_soul_effectiveness()` | 获取魂效能趋势 |
| `detect_unsummoned_souls()` | 检测未被召唤的魂 |
| `export_archive()` | 导出归档数据 |
| `verify_archive()` | 验证归档完整性 |

#### 4.4.4 分析功能 (analytics.rs)

| 函数 | 说明 |
|------|------|
| `compute_summon_stats()` | 计算召唤统计 |
| `compute_soul_effectiveness()` | 计算魂效能 |
| `compute_mode_distribution()` | 计算模式分布 |
| `detect_unsummoned_souls_impl()` | 检测长期未召唤的魂 |
| `detect_low_effectiveness_impl()` | 检测低效能魂 |

---

### 4.5 Possession 模块

**路径**: `rust/possession/src/`

附体引擎是系统的核心模块，负责协调多个魂的并行推理和交互。

#### 4.5.1 模块结构

```
possession/
├── modes/           # 附体模式实现
│   ├── conference.rs   # 合议模式
│   ├── debate.rs       # 辩论模式
│   ├── learn.rs        # 学习模式
│   ├── practice_opening.rs  # 实践开口模式
│   ├── relay.rs        # 接力模式
│   └── single.rs       # 单魂模式
├── soul/            # 魂处理
│   ├── mod.rs
│   ├── process.rs      # 魂进程
│   └── self_audit.rs   # 魂自我审计
├── cross_detector.rs  # 交叉检测
├── recovery.rs        # 恢复管理
├── stream.rs          # 流式处理
├── tools.rs           # 工具注册
├── triage.rs          # 入口分流
├── ws.rs              # WebSocket 管理
└── lib.rs             # 模块导出
```

#### 4.5.2 附体模式

| 模式 | 说明 |
|------|------|
| `Single` | 单一魂独立输出，适合快速咨询 |
| `Conference` | 多魂同时输出，流式交叉检测，实时碰撞，辩证综合 |
| `Debate` | 两列对立 + 中间裁决栏 |
| `Relay` | 横向时间轴，阶段卡片传递 |
| `Learn` | 魂教学模式 |
| `PracticeOpening` | 方法论实践模式（四阶段：场域/消化/修正/行动） |

#### 4.5.3 PossessionEngine 结构

```rust
pub struct PossessionEngine {
    store: Arc<dyn Storage>,           // 存储后端
    registry: Arc<SoulRegistry>,       // 魂注册表
    gateway: Arc<GatewayRegistry>,     // AI 网关
    ws_manager: WsSessionManager,      // WebSocket 管理
    shutdown_flag: AtomicBool,          // 关闭标志
    tool_registry: tools::ToolRegistry, // 工具注册表
}
```

#### 4.5.4 核心方法

| 方法 | 说明 |
|------|------|
| `new(store, registry, gateway)` | 创建附体引擎 |
| `start_possession(input, system_tx)` | 启动附体会话 |
| `tool_registry()` | 获取工具注册表 |
| `is_shutdown()` / `set_shutdown()` | 关闭状态管理 |

#### 4.5.5 WebSocket 事件类型

```rust
pub enum WsEventType {
    // Soul 流式事件
    SoulChunk,      // soul_token - 魂输出片段
    SoulDone,       // soul_done - 魂输出完成
    SoulError,      // soul_error - 魂输出错误
    
    // 综合事件
    SynthesisChunk, // synthesis_chunk - 综合输出片段
    SynthesisDone,  // synthesis_done - 综合完成
    
    // 系统事件
    SystemMessage,  // system - 系统消息
    Error,          // error - 错误
    SessionComplete, // done - 会话完成
    
    // 流程事件
    SessionStarted,    // session_started
    EntryClassified,   // entry_classified - 入口分类
    SoulStarted,       // soul_started - 魂开始
    SoulCalling,       // soul_calling - 魂调用中
    SynthesisStarted,  // synthesis_started - 综合开始
    ProcessStep,       // process_step - 处理步骤
    
    // 交叉检测
    Collision,         // collision - 碰撞检测
    
    // 成本追踪
    Cost,             // cost - 成本信息
    
    // 工具调用
    ToolCallStarted,   // tool_call_started - 工具调用开始
    ToolResult,        // tool_result - 工具结果
}
```

#### 4.5.6 入口分流 (triage.rs)

根据用户输入自动分类到合适的附体模式：

```rust
pub fn triage(input: &PossessionInput) -> EntryType
```

分流逻辑：
1. 分析任务内容
2. 识别关键词（合议/辩论/学习等）
3. 匹配最佳魂组合
4. 确定附体模式

---

### 4.6 API 模块

**路径**: `rust/api/src/`

API 模块是 HTTP 服务器，提供 REST API 和 WebSocket 接口。

#### 4.6.1 路由结构

```rust
pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))           // 健康检查
        .nest("/souls", souls::router())                // 魂管理
        .nest("/possess", possess::router())             // 附体操作
        .nest("/sessions", sessions::router())           // 会话管理
        .nest("/analytics", analytics::router())          // 分析统计
        .nest("/archive", archive::router())             // 归档管理
        .nest("/apikey", apikey::router())               // API Key 管理
        .nest("/knowledge", knowledge::router())          // 知识库
        .nest("/config", config::router())               // 配置管理
        .nest("/searxng", searxng::router())             // 搜索集成
}
```

#### 4.6.2 子路由

| 路由 | 文件 | 说明 |
|------|------|------|
| `/souls` | souls.rs | 魂的 CRUD 操作 |
| `/possess` | possess.rs | 附体会话创建和管理 |
| `/sessions` | sessions.rs | 会话列表和详情 |
| `/analytics` | analytics.rs | 统计数据查询 |
| `/archive` | archive.rs | 归档操作 |
| `/apikey` | apikey.rs | API Key 管理 |
| `/knowledge` | knowledge.rs | 知识库搜索 |
| `/config` | config.rs | 配置读写 |
| `/searxng` | searxng.rs | 搜索服务集成 |

#### 4.6.3 WebSocket 接口

```
/ws/possess/:session_id/:channel
```

参数：
- `session_id`: 会话 ID
- `channel`: 频道（soul/synthesis/system）

#### 4.6.4 中间件

| 中间件 | 说明 |
|--------|------|
| `rate_limiter` | 请求限流（默认: 30 req/s） |
| `cors` | 跨域资源共享 |
| `trace` | 请求追踪 |

---

## 5. 关键数据结构

### 5.1 SoulProfile

魂的配置和元数据：

```rust
pub struct SoulProfile {
    pub name: String,                  // 魂名称
    pub ismism_code: String,           // 主义主义四维坐标 (如 "1-2-3-4")
    pub field: String,                 // 场域
    pub ontology: String,             // 存在论
    pub epistemology: String,          // 认识论
    pub teleology: String,            // 目的论
    pub domains: Vec<String>,          // 擅长领域
    pub exclude_scenarios: Vec<String>, // 排除场景
    pub summon_count: u32,             // 召唤次数
    pub effectiveness: EffectivenessStats, // 效能统计
    pub summon_prompt: String,          // 召唤咒语
    pub practice_observations: Vec<PracticeObservation>, // 实践观察
    
    // Agent 特定字段
    pub title: String,                 // 标题
    pub description: String,           // 描述
    pub voice: String,                 // 声音特征
    pub mind: String,                  // 思维模式
    pub self_declare: String,          // 自我声明
    pub skills_expertise: Vec<String>, // 技能专长
    pub model: String,                 // 推荐模型
    pub tools: String,                 // 工具配置
    pub trigger_keywords: Vec<String>, // 触发关键词
    pub compat: Vec<String>,           // 兼容魂
    pub incompat: Vec<String>,         // 不兼容魂
}
```

### 5.2 IsmismCode

主义主义四维坐标：

```rust
pub struct IsmismCode {
    pub field: u8,        // 场域论 (0-9)
    pub ontology: u8,     // 存在论 (0-9)
    pub epistemology: u8, // 认识论 (0-9)
    pub teleology: u8,    // 目的论 (0-9)
}

impl IsmismCode {
    // 计算两个坐标之间的欧几里得距离
    pub fn distance(&self, other: &IsmismCode, weights: Option<(f64, f64, f64, f64)>) -> f64;
}
```

### 5.3 Session

会话信息：

```rust
pub struct Session {
    pub id: String,              // 会话 ID
    pub title: String,            // 标题（任务描述）
    pub mode: PossessionMode,     // 附体模式
    pub status: SessionStatus,    // 状态
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PossessionMode {
    Single,
    Conference,
    Debate,
    Relay,
    Learn,
    PracticeOpening,
}

pub enum SessionStatus {
    Active,
    Completed,
    Archived,
    Inconsistent,
}
```

### 5.4 Message

消息结构：

```rust
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub soul_name: Option<String>,
    pub content: String,
    pub seq: u32,
    pub created_at: DateTime<Utc>,
}

pub enum MessageRole {
    User,
    Soul,
    Synthesis,
    System,
}
```

### 5.5 LLM 请求和响应

```rust
pub struct LLMRequest {
    pub provider: Provider,
    pub prompt: Prompt,
    pub config: CallConfig,
}

pub struct CallConfig {
    pub temperature: f64,
    pub max_tokens: u32,
    pub stream: bool,
    pub model: Option<String>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub structured_output: Option<StructuredOutputConfig>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<String>,
}

pub struct Chunk {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub finish_reason: Option<String>,
    pub index: u32,
    pub usage: Option<UsageStats>,
    pub tool_calls: Vec<ToolCall>,
}

pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

---

## 6. API 路由说明

### 6.1 健康检查

```
GET /api/v1/health
```

响应：
```json
{ "status": "ok" }
```

### 6.2 魂管理 (/api/v1/souls)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 获取魂列表 |
| GET | `/:name` | 获取指定魂详情 |
| POST | `/` | 创建新魂 |
| PUT | `/:name` | 更新魂 |
| DELETE | `/:name` | 删除魂 |
| GET | `/search?q=` | 搜索魂 |
| GET | `/stats` | 获取召唤统计 |
| GET | `/ismism/distribution` | 获取主义主义分布 |

### 6.3 附体操作 (/api/v1/possess)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/start` | 启动附体会话 |
| GET | `/:session_id` | 获取会话状态 |
| POST | `/:session_id/stop` | 停止会话 |

启动请求示例：
```json
{
  "mode": "conference",
  "task": "分析人工智能对就业的影响",
  "souls": ["马克思", "尼采", "费曼"],
  "topic": "AI与就业",
  "judgment": "我认为技术进步最终会增加就业",
  "worry": "担心贫富差距扩大",
  "unknown": "具体的时间节点"
}
```

### 6.4 会话管理 (/api/v1/sessions)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 获取会话列表 |
| GET | `/:id` | 获取会话详情（含消息） |
| DELETE | `/:id` | 删除会话 |

### 6.5 分析统计 (/api/v1/analytics)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/summon-stats` | 获取召唤统计 |
| GET | `/effectiveness/:soul` | 获取魂效能趋势 |
| GET | `/mode-distribution` | 获取模式分布 |
| GET | `/alerts/unsummoned` | 检测未召唤的魂 |
| GET | `/alerts/low-effectiveness` | 检测低效能魂 |

### 6.6 归档管理 (/api/v1/archive)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/verify/:session_id` | 验证归档完整性 |
| POST | `/export` | 导出归档 |
| GET | `/export/:task_id` | 获取导出状态 |

### 6.7 知识库 (/api/v1/knowledge)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/search?q=` | 搜索知识 |
| POST | `/rebuild` | 重建全文索引 |
| GET | `/topics` | 获取知识主题列表 |
| GET | `/cards` | 获取知识卡片 |

---

## 7. 数据库设计

### 7.1 表结构

```sql
-- 会话表
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    mode        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- 消息表
CREATE TABLE messages (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    role        TEXT NOT NULL,
    soul_name   TEXT,
    content     TEXT NOT NULL,
    seq         INTEGER NOT NULL,
    created_at  TEXT NOT NULL
);

-- 调用记录表
CREATE TABLE call_records (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES sessions(id),
    soul_name       TEXT NOT NULL,
    mode            TEXT NOT NULL,
    task_summary    TEXT NOT NULL,
    effectiveness   TEXT NOT NULL,
    notes           TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    prompt_tokens   INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens    INTEGER NOT NULL DEFAULT 0
);

-- 魂修订表
CREATE TABLE soul_revisions (
    id              TEXT PRIMARY KEY,
    soul_name       TEXT NOT NULL,
    revision_type   TEXT NOT NULL,
    description     TEXT NOT NULL,
    old_value       TEXT,
    new_value       TEXT,
    reviewer        TEXT,
    reviewed_at     TEXT,
    created_at      TEXT NOT NULL
);

-- 盲区表
CREATE TABLE blind_spots (
    id              TEXT PRIMARY KEY,
    soul_name       TEXT NOT NULL,
    dimension       TEXT NOT NULL,
    description     TEXT NOT NULL,
    detected_at     TEXT NOT NULL,
    resolved_at     TEXT,
    resolved_by     TEXT,
    resolution      TEXT
);

-- 知识卡片表
CREATE TABLE knowledge_cards (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    content         TEXT NOT NULL,
    source_soul     TEXT,
    source_session  TEXT,
    tags            TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- 修订提案表
CREATE TABLE revision_proposals (
    id              TEXT PRIMARY KEY,
    soul_name       TEXT NOT NULL,
    proposal_type   TEXT NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL,
    proposed_changes TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    created_by      TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    reviewed_at     TEXT,
    reviewer        TEXT,
    review_notes    TEXT
);

-- LLM 缓存表
CREATE TABLE llm_cache (
    hash            TEXT PRIMARY KEY,
    provider        TEXT NOT NULL,
    model           TEXT NOT NULL,
    response_content TEXT NOT NULL,
    usage_json      TEXT,
    created_at      TEXT NOT NULL
);

-- FTS5 全文索引
CREATE VIRTUAL TABLE knowledge_fts USING fts5(
    soul_name, content, mode, task_summary, created_at, session_id,
    tokenize='trigram'
);
```

### 7.2 索引

```sql
CREATE INDEX idx_messages_session ON messages(session_id, seq);
CREATE INDEX idx_call_records_soul ON call_records(soul_name);
CREATE INDEX idx_call_records_session ON call_records(session_id);
CREATE INDEX idx_sessions_mode ON sessions(mode);
CREATE INDEX idx_sessions_created ON sessions(created_at);
CREATE INDEX idx_soul_revisions_soul ON soul_revisions(soul_name);
CREATE INDEX idx_blind_spots_soul ON blind_spots(soul_name);
CREATE INDEX idx_knowledge_cards_soul ON knowledge_cards(source_soul);
CREATE INDEX idx_revision_proposals_soul ON revision_proposals(soul_name);
CREATE INDEX idx_revision_proposals_status ON revision_proposals(status);
CREATE INDEX idx_llm_cache_created ON llm_cache(created_at);
```

### 7.3 SQLite 配置

```rust
conn.execute_batch("PRAGMA journal_mode = WAL;")?;
conn.execute_batch("PRAGMA synchronous = NORMAL;")?;
conn.execute_batch("PRAGMA foreign_keys = ON;")?;
conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
```

---

## 8. 项目运行方式

### 8.1 前置条件

- Rust 工具链 (1.75+)
- Node.js 18+ 和 pnpm
- 至少一个 AI 提供商的 API Key (DeepSeek / Claude / OpenAI)

### 8.2 配置 API Key

创建 `data/apikeys.json` 文件：

```json
{
  "deepseek": "your-deepseek-api-key",
  "openai": "your-openai-api-key",
  "claude": "your-claude-api-key"
}
```

### 8.3 启动方式

#### 一键启动（推荐）

```bash
bash start.sh
```

#### 分别启动

```bash
# 启动 API 服务（端口 3096）
cargo run -p api --release

# 启动前端（端口 3000）
cd nextjs && pnpm dev
```

### 8.4 访问地址

- 前端: http://localhost:3000
- API: http://127.0.0.1:3096
- 健康检查: http://127.0.0.1:3096/api/v1/health

---

## 9. 配置说明

配置文件位于 `config/default.yaml`：

```yaml
# 数据目录配置
data_dir: "./data"
souls_dir: "./data/souls"
archive_dir: "./data/archive"
db_path: "./data/wanminfan.db"
registry_path: "./data/registry.yaml"
call_records_path: "./data/call-records.yaml"

# 服务器配置
server_host: "127.0.0.1"
server_port: 3001
nextjs_port: 3000

# 搜索服务
searxng_url: "http://127.0.0.1:8080"

# 限流配置
rate_limit:
  enabled: true
  requests_per_second: 30
  burst_size: 60
```

---

## 10. 内置魂列表

| 魂名 | 主义主义坐标 | 领域 |
|------|-------------|------|
| 马克思 | 1-2-1-3 | 政治经济学、哲学 |
| 列宁 | 1-2-1-3 | 革命理论 |
| 毛泽东 | 1-2-1-3 | 革命实践 |
| 邓小平 | 1-2-1-3 | 改革开放 |
| 鲁迅 | 2-3-2-2 | 文学、批判 |
| 尼采 | 2-1-2-1 | 哲学、价值观 |
| 黑格尔 | 2-1-1-3 | 辩证法、哲学 |
| 费曼 | 3-3-3-4 | 物理学、教育 |
| 马斯克 | 3-2-3-4 | 科技、企业 |
| 黄仁勋 | 3-2-3-4 | 科技、企业 |
| 庄子 | 2-1-1-2 | 道家哲学 |
| 孔子 | 1-2-2-3 | 儒家思想 |
| 胡塞尔 | 2-1-2-4 | 现象学 |
| 波伏娃 | 1-3-2-3 | 女权主义、哲学 |
| 法农 | 1-2-1-3 | 后殖民理论 |
| 葛兰西 | 1-2-1-3 | 文化霸权 |
| 伊本赫勒敦 | 1-2-2-3 | 历史哲学 |
| 稻盛和夫 | 3-2-3-3 | 经营哲学 |
| 未明子 | 2-2-2-3 | 当代哲学 |
| 乔布斯 | 3-2-3-4 | 产品设计 |
| Aaron Swartz | 2-3-2-4 | 互联网 activism |
| Karpathy | 3-3-3-4 | 深度学习 |
| 海绵宝宝 | 4-4-4-4 | 乐观主义 |
| 斯大林 | 1-2-1-3 | 革命理论 |
| 祝鹤槐 | 2-2-2-3 | 待定 |
| 罗永浩 | 3-2-3-4 | 产品、创业 |

---

## 附录 A: 错误类型

```rust
pub enum FoundationError {
    SoulNotFound(String),           // 魂不存在
    SessionNotFound(String),         // 会话不存在
    InvalidState(String),           // 无效状态
    Validation(String),             // 验证失败
    Sqlite(rusqlite::Error),        // SQLite 错误
    Io(std::io::Error),             // IO 错误
    Storage(String),                // 存储错误
}
```

---

## 附录 B: 环境变量

| 变量 | 说明 | 优先级 |
|------|------|--------|
| `DEEPSEEK_API_KEY` | DeepSeek API Key | 高 |
| `OPENAI_API_KEY` | OpenAI API Key | 高 |
| `CLAUDE_API_KEY` | Claude API Key | 高 |

---

**文档版本**: 1.0.0  
**生成日期**: 2026-05-11
