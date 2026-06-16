# Crate 设计与依赖关系

## Workspace 结构

```
rust/
├── Cargo.toml            # workspace root
├── foundation/           # 基础设施层（无内部依赖）
├── ai-gateway/           # AI 网关（→ foundation）
├── registry/             # Agent 注册表（→ foundation）
├── archive/              # 归档系统（→ foundation）
├── possession/           # 编排引擎（→ foundation + ai-gateway + registry）
├── api/                  # HTTP/WS 服务（→ 全部）
└── cli/                  # 命令行工具（→ foundation + ai-gateway）
```

## 各 Crate 职责

### foundation — 基础设施

**无内部依赖**。所有 crate 的基础。

| 模块 | 职责 |
|------|------|
| `models.rs` | 共享数据类型：AgentProfile、Session、Message、LLMRequest、CallConfig |
| `storage.rs` | `Storage` trait：定义全部持久化接口 |
| `sqlite.rs` | SQLite WAL + FTS5 实现 |
| `config.rs` | 配置加载（YAML 文件 + 环境变量） |
| `domain.rs` | DomainProfile：领域术语模板、坐标维度、综合 Prompt |
| `error.rs` | 统一错误类型 `FoundationError` |
| `health.rs` | 健康检查 |
| `vector_search.rs` | 向量/语义搜索 |

### ai-gateway — AI 网关

**依赖**: `foundation`

| 模块 | 职责 |
|------|------|
| `lib.rs` | `GatewayRegistry`：统一提供商管理器 |
| `openai.rs` | OpenAI Chat Completions 适配器 |
| `claude.rs` | Anthropic Messages API 适配器 |
| `deepseek.rs` | DeepSeek Chat 适配器（含 Reasoner 推理） |
| `lmstudio.rs` | LM Studio 本地模型适配器 |
| `model_router.rs` | 按任务类型自动选择模型和推理强度 |
| `prompt.rs` | PromptBuilder：从 DomainProfile 渲染领域化 Prompt |
| `cache.rs` | SHA256 响应缓存（LRU，默认 3600s TTL） |

**核心 Trait**:

```rust
pub trait Gateway: Send + Sync {
    fn provider(&self) -> Provider;
    fn is_available(&self) -> bool;
    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>>;
}
```

### registry — Agent 注册表

**依赖**: `foundation`

| 模块 | 职责 |
|------|------|
| `lib.rs` | `SoulRegistry`（框架名 `AgentRegistry`）：Agent CRUD、内存缓存 |
| `search.rs` | 坐标匹配：泛型 `CoordinateMatcher` trait |
| `fulltext_search.rs` | 倒排索引全文搜索 |
| `ismism.rs` | ISMISM 四维坐标（领域示例实现） |

**核心 Trait**:

```rust
pub trait CoordinateMatcher: Send + Sync {
    fn distance(&self, a: &[f64], b: &[f64]) -> f64;
    fn nearest(&self, target: &[f64], candidates: &[Vec<f64>], top_k: usize) -> Vec<usize>;
}
```

### archive — 归档系统

**依赖**: `foundation`

| 模块 | 职责 |
|------|------|
| `lib.rs` | `ArchiveSystem`：归档入口 |
| `archive.rs` | 会话归档、消息序列化 |
| `audit.rs` | 审计日志 |
| `call_records.rs` | LLM 调用记录 |
| `cost_tracking.rs` | Token 消耗统计 |
| `analytics.rs` | Agent 召唤统计、效能趋势、模式分布 |

### possession — 编排引擎

**依赖**: `foundation` + `ai-gateway` + `registry`

| 模块 | 职责 |
|------|------|
| `lib.rs` | `PossessionEngine`：引擎入口 |
| `modes/single.rs` | 单 Agent 模式 |
| `modes/conference.rs` | 合议模式（并行 + 碰撞 + 综合） |
| `modes/debate.rs` | 辩论模式（对立 + 多轮 + 裁决） |
| `modes/relay.rs` | 接力模式（串行阶段卡） |
| `modes/learn.rs` | 学习模式 |
| `modes/practice_opening.rs` | 实践开口模式 |
| `modes/topology.rs` | 拓扑规划器（Conference 模式 Agent 分组策略） |
| `triage.rs` | 任务分析 + 模式分类 + Agent 匹配 |
| `stream.rs` | SSE/WS 流式输出管线 |
| `ws.rs` | `WsSessionManager`：会话广播管理 |
| `tools.rs` | `ToolRegistry`：工具注册与调用 |
| `cross_detector.rs` | 跨 Agent 矛盾检测 |
| `semantic_collision.rs` | 语义碰撞检测引擎 |
| `recovery.rs` | 故障恢复 |
| `soul/process.rs` | Agent 进程管理（tokio task 生命周期） |
| `soul/memory_graph.rs` | 记忆图谱（→ 提炼为 foundation/src/memory/） |
| `soul/self_audit.rs` | Agent 自我审计 |
| `soul/intervention.rs` | 实时干预管线 |

### api — HTTP/WS 服务

**依赖**: 全部

| 模块 | 职责 |
|------|------|
| `main.rs` | Axum 服务入口、路由挂载 |
| `state.rs` | `AppState`：Arc 共享应用状态 |
| `store.rs` | `AppStore`：数据存储接口 |
| `middleware.rs` | 中间件（rate limit、CORS、tracing） |
| `ws.rs` | WebSocket handler |
| `routes/souls.rs` | Agent CRUD API |
| `routes/possess.rs` | 会话执行 API |
| `routes/sessions.rs` | 会话管理 API |
| `routes/analytics.rs` | 统计分析 API |
| `routes/archive.rs` | 归档管理 API |
| `routes/knowledge.rs` | 知识库 API |
| `routes/config.rs` | 配置管理 API（含 Domain 切换） |
| `routes/searxng.rs` | SearXNG 搜索代理 |
| `routes/apikey.rs` | API Key 管理 |
| `routes/auth_route.rs` | 认证路由 |
| `collector.rs` | Agent 自动创建（Collect） |
| `ocr.rs` | OCR 识别 |
| `web_search_tool.rs` | Web 搜索工具 |
| `coding_tools.rs` | 编程工具集成 |
| `worker_tools.rs` | 工友工具集成 |
| `bing.rs` / `duckduckgo.rs` | 搜索引擎适配器 |

### cli — 命令行工具

**依赖**: `foundation` + `ai-gateway`

```
snake
├── run        # 提交任务到 API 服务
├── souls      # 列出/搜索 Agent
├── sessions   # 查看历史会话
└── init       # [NEW] 生成新项目骨架
```

## Workspace 公共依赖

```toml
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
