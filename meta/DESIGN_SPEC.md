# 框架提炼设计规格

> 目标：从 Snake Skin 项目中提炼出可复用、跨领域的前后端框架，产物保存在 `meta/` 目录。

## 决策记录

| 维度 | 决策 |
|------|------|
| 产物形态 | 文档 + 模板代码 + CLI 脚手架工具（`snake init`） |
| 抽象层级 | 三层：通用编排内核 → 领域模板层 → Agent 人格层 |
| 项目结构 | 前后端 Monorepo（Rust workspace + Next.js 同仓） |
| 实施方案 | 渐进式：先文档 + 模板生成器，未来 crate 化独立发布 |
| 文档语调 | 实用主义工程手册，不写哲学白皮书 |

## 框架核心 vs 领域示例

### 提炼为框架核心的模块

| 模块 | 当前位置 | 提炼后角色 |
|------|---------|-----------|
| **AI Gateway** 多提供商网关 | `rust/ai-gateway/` | 统一 LLM 调用接口，支持 Claude/OpenAI/DeepSeek/LM Studio |
| **Possession Engine** 调度内核 | `rust/possession/` | 多 Agent 并行调度、模式派发、碰撞检测、流式输出 |
| **WebSocket + SSE 通信层** | `rust/possession/src/ws.rs` `stream.rs` | 实时事件广播、会话管理、前端流式渲染管线 |
| **Tool 注册调用系统** | `rust/possession/src/tools.rs` | 工具定义、解析 tool_call、执行、结果格式化 |
| **Storage 抽象 + SQLite** | `rust/foundation/src/storage.rs` `sqlite.rs` | 可替换的持久化层，默认 SQLite WAL + FTS5 |
| **Domain 配置系统** | `rust/foundation/src/domain.rs` `config/domain.yaml` | 术语模板、坐标维度、综合 Prompt，切换领域不换代码 |
| **Collect/Refine 管线** | `rust/api/src/collector.rs` 等 | 从自然语言生成 Agent 配置，基于反馈迭代优化 |
| **Memory Graph 记忆图谱** | `rust/possession/src/soul/memory_graph.rs` | petgraph 图结构，Agent 跨轮记忆、矛盾检测、引用链 |
| **前端流式渲染组件** | `nextjs/components/` `nextjs/hooks/use-websocket.ts` | Agent 多列视图、流式气泡、碰撞通知、WS 连接管理 |

### 留在原项目作为示例的模块

| 模块 | 说明 |
|------|------|
| 具体 Agent 数据文件 | `data/souls/*.md`，26+ 个预设魂，作为 Agent 定义参考示例 |
| ISMISM 坐标算法 | `rust/registry/src/ismism.rs`，保留为坐标匹配的一种实现示例 |
| 具体 domain.yaml 内容 | `config/domain.labor.yaml` 等，保留为领域配置模板示例 |
| 审查官审讯流 | 保留为自定义交互模式的参考实现 |
| 实践开口模式 | 保留为自定义推理模式的参考实现 |
| 信誉机制 | 保留为可选的 Agent 质量评估示例 |
| 杂交引擎 | 保留为实验性 Agent 生命周期管理示例 |

## 产出物清单

### 1. `meta/docs/` 文档体系

```
meta/docs/
├── architecture/
│   ├── 01-overview.md              # 架构全景 + 请求生命周期
│   ├── 02-crate-design.md          # 7 crate 分层与依赖关系
│   ├── 03-crate-api-reference.md   # 模块间接口契约（Trait、类型签名）
│   ├── 04-data-flow.md             # HTTP → Engine → Gateway → Stream → WS → Frontend
│   └── 05-how-domain-switching-works.md  # domain.yaml 驱动机制
├── guides/
│   ├── 01-quickstart.md            # 5分钟跑起来
│   ├── 02-domain-config.md         # 定义自己的领域
│   ├── 03-create-agent.md          # Collect：自动生成 Agent
│   ├── 04-refine-agent.md          # Refine：迭代优化 Agent
│   ├── 05-custom-mode.md           # 自定义推理模式
│   ├── 06-add-tool.md              # 注册新工具
│   ├── 07-add-provider.md          # 接入新 AI 提供商
│   └── 08-frontend-integration.md  # 复用前端流式组件
└── reference/
    ├── api-routes.md               # REST API 路由表
    ├── websocket-event-reference.md # WS 事件类型与负载
    ├── config-schema.md            # 配置项与默认值
    ├── type-index.md               # 核心类型索引
    └── error-codes.md              # 错误类型与处理
```

### 2. `meta/templates/` 模板体系

```
meta/templates/
├── base/
│   ├── domain.yaml.tmpl            # 领域配置骨架
│   └── agent.md.tmpl               # Agent 定义文件骨架
└── domains/
    ├── philosophy/domain.yaml      # 哲学/思想辩论领域
    ├── legal/domain.yaml           # 法律顾问领域
    └── labor/domain.yaml           # 劳动者权益领域
```

### 3. `snake init` CLI 脚手架

现有 `rust/cli` 扩展 `init` 子命令：

```bash
snake init <project-name> [flags]
```

**Flags**:

| Flag | 默认值 | 说明 |
|------|--------|------|
| `--domain` | `custom` | 领域模板：custom / philosophy / legal / labor |
| `--agents` | 无 | 预置 Agent 数量 |
| `--port` | `3001` | API 端口 |
| `--frontend-port` | `3000` | 前端端口 |
| `--skip-frontend` | `false` | 仅生成 Rust 后端 |

**生成的目录结构**:

```
my-project/
├── config/
│   ├── default.yaml
│   └── domain.yaml
├── rust/
│   ├── Cargo.toml               # workspace
│   ├── my-agent-app/            # 业务 crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── agents/
│   │       ├── modes/
│   │       └── tools/
│   └── foundation/              # 框架 crate
├── nextjs/
│   ├── package.json
│   ├── app/
│   ├── components/
│   ├── hooks/
│   └── lib/api.ts
├── data/
│   ├── agents/
│   └── knowledge/
├── .env.example
└── README.md
```

## 三层抽象模型

### L1 · 通用编排内核

框架提供，二次开发者不修改：

- **Gateway**: 多 AI 提供商统一调用（实现 `Gateway` trait 即可接入新提供商）
- **Engine**: Agent 并行调度、6 种内置模式、流式输出、碰撞检测
- **Stream/WS**: 实时事件广播、会话管理、前端 WS 连接
- **Tools**: 工具注册与调用管线
- **Storage**: 可替换持久化抽象（默认 SQLite）
- **Memory**: Agent 跨轮记忆图谱（petgraph）

### L2 · 领域模板层

通过 `domain.yaml` 切换，不改代码：

```yaml
domain:
  name: "legal"                    # 领域标识
  system_name: "法律智囊团"        # 系统名称
  agent_noun: "顾问"               # Agent 称谓
  user_title: "委托人"             # 用户称谓

dimensions:
  - id: "law_area"
    label: "法域"
    values: ["民事", "刑事", "行政", "国际"]
  - id: "position"
    label: "立场"
    values: ["原告方", "被告方", "中立", "公益"]
  - id: "method"
    label: "方法"
    values: ["法条", "判例", "目的论", "社会学"]
  - id: "value"
    label: "取向"
    values: ["秩序", "权利", "衡平", "变革"]

synthesis:
  template: |
    请从以下{{agent_count}}位{{agent_noun}}的分析中，
    识别共识、矛盾与盲区，给出综合意见……
```

### L3 · Agent 人格层

二次开发者的主要工作区：

```yaml
# data/agents/contract-expert.md
---
name: "合同审查员"
model: "claude-3-5-sonnet"
tools: ["search_law", "analyze_contract"]
trigger_keywords: ["合同", "违约", "赔偿"]
system_prompt: |
  你是专注于合同法的法律顾问。分析合同时应关注……
```

## 去领域化改造清单

| 文件 | 改造内容 |
|------|---------|
| `ai-gateway/src/prompt.rs` | 硬编码术语 → `DomainProfile` 模板变量注入 |
| `possession/src/modes/*.rs` | 模式中的中文 Prompt → 提取到配置模板 |
| `registry/src/search.rs` | ISMISM 四维坐标 → 泛型坐标匹配 trait |
| `possession/src/triage.rs` | 中文关键词匹配 → 从 domain 配置读取触发词 |
| `api/src/routes/*.rs` | 路由中的领域术语 → 从 `DomainProfile` 读取 |
| 前端组件 `soul-*` | 保留组件实现，新增 `agent-*` 别名导出 |
| `SoulProfile` 类型 | 新增 `AgentProfile` 为标准名，旧名作为兼容别名 |

## Memory Graph 提炼

### 核心接口

```rust
trait MemoryStore: Send + Sync {
    async fn save_node(&self, node: MemoryNode) -> Result<String>;
    async fn save_edge(&self, edge: MemoryEdge) -> Result<()>;
    async fn query_by_agent(&self, agent_id: &str, window: TimeWindow) -> Result<Vec<MemoryNode>>;
    async fn query_semantic(&self, query: &str, top_k: usize) -> Result<Vec<(MemoryNode, f64)>>;
    async fn subgraph(&self, session_id: &str) -> Result<MemoryGraph>;
}

struct MemoryGraph {
    nodes: StableGraph<MemoryNode, MemoryEdge>,
    agent_index: HashMap<String, Vec<NodeIndex>>,
}
```

### 跨 Agent 记忆共享策略

通过 domain.yaml 配置：

```yaml
memory:
  default_share: "session"   # none | session | all | agent_a → agent_b
  max_turns: 20
  persist: true
```

### 与 Engine 集成点

1. Agent 输出完成 → `memory_graph.add_memory(agent, content, turn)`
2. 下轮推理前 → `memory_graph.query_by_agent(agent, window)` 注入上下文
3. Conference 模式 → `memory_graph.detect_contradiction()` 碰撞检测
4. Session 结束 → `memory_store.persist()` 落盘

## 演进路线图

| 阶段 | 内容 |
|------|------|
| **Phase 1** (当前) | 完成 `meta/` 文档体系 + 模板文件 + CLI `init` 骨架 |
| **Phase 2** | 框架核心 crate 独立发布到 crates.io（agent-gateway、agent-engine 等） |
| **Phase 3** | `snake init` 支持从 crates.io 拉取依赖（而非本地 path） |
| **Phase 4** | 插件市场：社区可发布 Agent 定义、领域配置、自定义模式 |
