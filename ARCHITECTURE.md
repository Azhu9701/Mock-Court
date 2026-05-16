# 万民幡 (Soul Banner) 架构文档

## 概述

这是一个**多 AI 人格并行推理系统**——同时召唤多个具有独立世界观的「魂」（思想家 / AI Agent），围绕同一问题展开多视角思考、碰撞、辩论与综合。项目的核心设计理念是：**将思想人物封装为可调度的子 agent，同时为实践者在场经验保留开口**。

---

## 整体拓扑

```
审查官（幡主，即用户）
  一审 → 补视角 → 终裁仲裁
        │
  前端 Next.js (shadcn/ui) ──▶ Axum API (Rust, 端口 3096) ──▶ AI Gateway (Claude/GPT/DS)
                                   │
           ┌───────────────────────┼──────────────────────┐
           ▼                       ▼                      ▼
       Registry (魂注册表)    Possession (附体引擎)    Archive (归档分析)
       Foundation (基础设施层)
```

---

## 后端 — Rust 四 crate 架构

```
rust/
├── foundation/    # 基础设施层
├── registry/      # 魂注册表
├── possession/    # 附体引擎（核心）
├── api/           # HTTP/WS 服务层
├── ai-gateway/    # AI 模型网关
└── archive/       # 归档与审计
```

### 1. `foundation` — 基础设施层

**职责**：共享数据模型、存储抽象、错误类型

- 数据模型定义（`SoulProfile`、`Session`、`EffectivenessStats` 等）
- SQLite（WAL 模式）+ FTS5 全文搜索
- 向量搜索（`vector_search.rs`）
- 文件系统存储 + 健康检查
- 统一错误类型

**关键文件**：
| 文件 | 功能 |
|------|------|
| `models.rs` | SoulProfile、Session 等核心数据结构 |
| `storage.rs` | 存储 trait 定义 |
| `sqlite.rs` | SQLite 实现 |
| `fs_store.rs` | 文件系统存储实现 |
| `vector_search.rs` | 向量/语义搜索 |
| `config.rs` | 配置加载 |
| `error.rs` | 统一错误类型 |
| `health.rs` | 健康检查 |

### 2. `registry` — 魂注册表

**职责**：魂的注册、搜索、匹配

- 魂的 CRUD 管理
- 主义主义四维搜索（场域论/存在论/认识论/目的论）
- ISMISM 雷达匹配、全文搜索
- 魂数据存储在 `data/souls/*.md`，有 26 个预设魂

**关键文件**：
| 文件 | 功能 |
|------|------|
| `lib.rs` | Registry 主逻辑 |
| `search.rs` | 魂搜索 |
| `fulltext_search.rs` | FTS5 全文搜索 |
| `ismism.rs` | 主义主义坐标匹配 |

### 3. `possession` — 附体引擎（核心）

**职责**：多魂并行调度的核心引擎

- **6 种附体模式**：
  - `single` — 单魂独立输出
  - `conference` — 合议模式（多魂并行 + 碰撞检测 + 辩证综合）
  - `debate` — 辩论模式（正反两栏 + 裁决）
  - `relay` — 接力模式（阶段卡片传递）
  - `learn` — 教学/学习模式（费曼学习法）
  - `practice_opening` — 实践开口模式

- **魂进程干预架构**：
  - 语义碰撞引擎（`semantic_collision.rs`）：实时检测不同魂输出的矛盾
  - 记忆图谱（`memory_graph.rs`）：每个魂的独立记忆拓扑
  - 自适应拓扑（`topology.rs`）：动态调整魂间关系
  - 信誉机制（`reputation.rs`）：魂的历史表现评估
  - 杂交引擎（`hybridization.rs`）：魂的融合与派生
  - 教条检测（`cross_detector.rs`）：交叉验证防教条

- **审查官权威机制**：一审 → 补视角 → 终裁

- WebSocket 实时流控制 + SSE 流式输出
- 实时干预管线（`intervention.rs`）

**关键文件**：
| 文件 | 功能 |
|------|------|
| `modes/single.rs` | 单魂模式 |
| `modes/conference.rs` | 合议模式 |
| `modes/debate.rs` | 辩论模式 |
| `modes/relay.rs` | 接力模式 |
| `modes/learn.rs` | 学习模式 |
| `modes/practice_opening.rs` | 实践开口 |
| `modes/teaching.rs` | 教学系统 |
| `modes/topology.rs` | 自适应拓扑 |
| `soul/process.rs` | 魂进程管理 |
| `soul/memory_graph.rs` | 记忆图谱 |
| `soul/intervention.rs` | 实时干预 |
| `soul/reputation.rs` | 信誉系统 |
| `soul/hybridization.rs` | 杂交引擎 |
| `soul/self_audit.rs` | 魂自审 |
| `semantic_collision.rs` | 语义碰撞检测 |
| `cross_detector.rs` | 教条交叉检测 |
| `triage.rs` | 分诊/优先级 |
| `recovery.rs` | 故障恢复 |
| `stream.rs` | SSE 流式输出 |
| `ws.rs` | WebSocket 处理 |
| `tools.rs` | 工具调用 |

### 4. `api` — HTTP/WS 服务层

**职责**：对外暴露 REST API + WebSocket

- Axum 0.7 Web 框架
- RESTful 路由：souls、sessions、possess、archive、knowledge、analytics、searxng、apikey、config
- WebSocket 处理器
- 中间件（rate limiter 等）
- OCR、Web 搜索工具集成

**关键文件**：
| 文件 | 功能 |
|------|------|
| `main.rs` | 服务入口，路由挂载 |
| `state.rs` | 共享应用状态 |
| `store.rs` | 数据存储接口 |
| `middleware.rs` | 中间件（rate limit 等） |
| `error.rs` | API 错误处理 |
| `ws.rs` | WebSocket 管理 |
| `routes/souls.rs` | 魂 CRUD API |
| `routes/sessions.rs` | 会话 API |
| `routes/possess.rs` | 附体执行 API |
| `routes/archive.rs` | 归档查询 API |
| `routes/knowledge.rs` | 知识库 API |
| `routes/analytics.rs` | 统计分析 API |
| `routes/searxng.rs` | SearXNG 搜索代理 |
| `routes/config.rs` | 配置管理 API |
| `routes/apikey.rs` | API Key 管理 |
| `collector.rs` | 魂数据采集 |
| `ocr.rs` | OCR 识别 |
| `web_search_tool.rs` | Web 搜索工具 |

### 5. `ai-gateway` — AI 模型网关

**职责**：多 AI 提供商的统一接入与智能路由

- 多提供商支持：Claude / OpenAI / DeepSeek
- 模型智能路由（`model_router.rs`）
- 提示词工程（`prompt.rs`）
- 缓存层（`cache.rs`）

**关键文件**：
| 文件 | 功能 |
|------|------|
| `lib.rs` | 网关统一接口 |
| `claude.rs` | Anthropic Claude 适配器 |
| `openai.rs` | OpenAI 适配器 |
| `deepseek.rs` | DeepSeek 适配器 |
| `model_router.rs` | 模型智能路由 |
| `prompt.rs` | 提示词构造与管理 |
| `cache.rs` | 响应缓存层 |

### 6. `archive` — 归档与审计

**职责**：会话归档、成本追踪、统计分析

- 会话归档与回溯
- 成本追踪（`cost_tracking.rs`）
- 分析统计（`analytics.rs`）
- 调用记录（`call_records.rs`）

**关键文件**：
| 文件 | 功能 |
|------|------|
| `lib.rs` | 归档模块入口 |
| `archive.rs` | 归档核心逻辑 |
| `audit.rs` | 审计日志 |
| `call_records.rs` | 调用记录 |
| `cost_tracking.rs` | Token 成本追踪 |
| `analytics.rs` | 使用统计分析 |

---

## 前端 — Next.js 16 + shadcn/ui

```
nextjs/
├── app/           # 页面路由
├── components/    # 业务组件 + UI 组件
├── config/        # 前端配置
├── contexts/      # React Context
├── hooks/         # 自定义 Hooks
├── lib/           # 工具库 + API 客户端
└── public/        # 静态资源
```

### 页面路由

| 路由 | 功能 |
|------|------|
| `/` | 首页/仪表盘 |
| `/souls` | 魂总览、收藏、筛选 |
| `/souls/[name]` | 单魂详情页 |
| `/souls/collect` | 收魂入口 |
| `/souls/refine` | 炼化入口 |
| `/possess` | 附体模式选择 |
| `/possess/[sessionId]` | 会话进行中（多列视图） |
| `/sessions` | 历史会话列表 |
| `/sessions/[id]` | 会话详情与回顾 |
| `/knowledge` | 知识库浏览器 |
| `/analytics` | 分析面板 |
| `/searxng` | SearXNG 搜索代理 |

### 核心组件

**视图组件**（对应附体模式）：
| 组件 | 对应模式 |
|------|---------|
| `single-view.tsx` | 单魂输出 |
| `conference-view.tsx` | 合议模式 |
| `debate-view.tsx` | 辩论模式 |
| `relay-view.tsx` | 接力模式 |
| `learn-view.tsx` | 学习模式 |
| `synthesis-panel.tsx` | 辩证综合面板 |

**魂管理组件**：
| 组件 | 功能 |
|------|------|
| `soul-card-grid.tsx` | 魂卡片网格 |
| `soul-card.tsx` | 单个魂卡片 |
| `soul-panel.tsx` | 魂信息面板 |
| `soul-filter-bar.tsx` | 魂筛选栏 |
| `soul-focus-panel.tsx` | 魂聚焦面板 |
| `soul-carousel.tsx` | 魂轮播 |
| `soul-effectiveness-table.tsx` | 魂效力表 |
| `soul-overview-panel.tsx` | 魂总览 |
| `soul-prompt.tsx` | 魂召唤咒语展示 |
| `soul-model-config.tsx` | 魂模型配置 |
| `soul-chat-bubble.tsx` | 魂对话气泡 |
| `soul-response-card.tsx` | 魂响应卡片 |
| `soul-responses-grid.tsx` | 魂响应网格 |
| `ismism-radar.tsx` | 主义主义雷达图 |
| `edit-soul-dialog.tsx` | 编辑魂对话框 |
| `delete-soul-button.tsx` | 删除魂按钮 |
| `delete-soul-confirm-dialog.tsx` | 删除确认弹窗 |
| `summon-button.tsx` | 召唤按钮 |

**会话管理组件**：
| 组件 | 功能 |
|------|------|
| `session-runner.tsx` | 会话执行器 |
| `session-status-bar.tsx` | 会话状态栏 |
| `session-timeline.tsx` | 会话时间线 |
| `session-detail-view.tsx` | 会话详情 |
| `session-actions.tsx` | 会话操作 |
| `session-context-header.tsx` | 会话上下文头 |
| `post-session-review.tsx` | 会后回顾 |
| `possession-entry.tsx` | 附体入口 |

**审查官交互组件**：
| 组件 | 功能 |
|------|------|
| `follow-up-input.tsx` | 追问输入 |
| `collision-notification.tsx` | 碰撞通知 |
| `effectiveness-panel.tsx` | 效力面板 |
| `tool-call-indicator.tsx` | 工具调用指示器 |
| `verification-dialog.tsx` | 验证对话框 |
| `quick-actions.tsx` | 快捷操作 |

**实践开口组件**：
| 组件 | 功能 |
|------|------|
| `practice-opening-dialog.tsx` | 实践开口弹窗 |
| `practice-opening-view.tsx` | 实践开口视图 |
| `practice-observations.tsx` | 实践观察记录 |

**其他功能组件**：
| 组件 | 功能 |
|------|------|
| `knowledge-browser.tsx` | 知识库浏览器 |
| `searxng-search.tsx` | SearXNG 搜索 |
| `dashboard-charts.tsx` | 仪表盘图表 |
| `settings-dialog.tsx` | 设置弹窗 |
| `article-modal.tsx` | 文章模态框 |
| `attachment-upload.tsx` | 附件上传 |
| `mode-bar-chart.tsx` | 模式柱状图 |
| `cost-display.tsx` | 成本展示 |
| `stat-card.tsx` | 统计卡片 |

**布局组件**：
| 组件 | 功能 |
|------|------|
| `shell-layout.tsx` | 壳布局 |
| `sidebar.tsx` | 侧边栏 |
| `sidebar-nav.tsx` | 侧栏导航 |
| `sidebar-sessions.tsx` | 侧栏会话列表 |
| `sidebar-footer.tsx` | 侧栏底部 |
| `sidebar-logo.tsx` | 侧栏 Logo |
| `header.tsx` | 顶部栏 |
| `breadcrumb.tsx` | 面包屑 |

### 状态管理

| 模块 | 类型 | 用途 |
|------|------|------|
| `contexts/sidebar-context.tsx` | React Context | 侧栏折叠/展开状态 |
| `contexts/breadcrumb-context.tsx` | React Context | 面包屑导航状态 |
| `hooks/use-websocket.ts` | Custom Hook | WebSocket 连接与消息处理 |
| `hooks/use-clean-content.ts` | Custom Hook | 内容清洗（去标签等） |

### 工具库

| 文件 | 功能 |
|------|------|
| `lib/api.ts` | 前端 API 调用封装 |
| `lib/soul-utils.ts` | 魂相关工具函数 |
| `lib/utils.ts` | 通用工具函数 |
| `lib/pending-session.ts` | 待处理会话管理 |

### 前端配置

| 文件 | 功能 |
|------|------|
| `config/presets.ts` | 预设配置 |
| `config/nav.ts` | 导航配置 |
| `config/models.ts` | 模型配置 |
| `config/possession-modes.ts` | 附体模式配置 |
| `config/soul-filter.ts` | 魂筛选配置 |

---

## 数据层

- **数据库**：SQLite（WAL 模式）+ FTS5 全文搜索
- **魂定义**：`data/souls/*.md`（Markdown 格式，26 个预设魂）
- **归档存储**：`data/archive/`（按日期组织）
- **配置**：`config/default.yaml`（YAML 格式）
- **API Key**：`data/apikeys.json`

### 魂数据结构 (SoulProfile)

每个魂包含以下核心字段：
- `name` — 名称
- `ismism_code` — 主义主义四维坐标（场域论/存在论/认识论/目的论，如 `4-1-4-3`）
- `field` / `ontology` / `epistemology` / `teleology` — 分解后的坐标
- `domains` — 擅长领域
- `exclude_scenarios` — 排除场景
- `summon_prompt` — 召唤咒语/系统提示词
- `self_declare` — 自我声明
- `summon_count` — 被召唤次数
- `effectiveness` — 效力统计
- `practice_observations` — 实践观察记录
- `tags` — 标签

---

## 技术栈总览

| 层 | 技术 | 说明 |
|----|------|------|
| **后端框架** | Axum 0.7 | Rust 异步 Web 框架 |
| **异步运行时** | Tokio | Rust 异步生态标准 |
| **数据库** | SQLite + FTS5 | WAL 模式，全文搜索 |
| **图计算** | petgraph | 魂记忆图谱 |
| **序列化** | Serde + serde_json | JSON 序列化 |
| **前端框架** | Next.js 16 | React 19 + TypeScript |
| **UI 组件** | shadcn/ui + Tailwind CSS | 原子化 CSS + 无头组件 |
| **实时通信** | WebSocket + SSE | 流式输出 + 双向通信 |
| **AI 提供商** | Claude / OpenAI / DeepSeek | 多模型智能路由 |
| **包管理** | pnpm | 前端依赖管理 |
| **测试** | Vitest | 前端单元测试 |

---

## 预设魂列表（26个）

马克思 · 列宁 · 毛泽东 · 邓小平 · 鲁迅 · 尼采 · 黑格尔 · 费曼 · 马斯克 · 黄仁勋 · 庄子 · 孔子 · 胡塞尔 · 波伏娃 · 法农 · 葛兰西 · 伊本赫勒敦 · 稻盛和夫 · 未明子 · 乔布斯 · 海绵宝宝 · 斯大林 · Karpathy · Aaron Swartz · 祝鹤槐 · 轮臂

---

## 附体模式对比

| 模式 | 魂数 | 交互方式 | 适用场景 |
|------|------|---------|---------|
| **单魂** | 1 | 一对一问答 | 快速咨询 |
| **合议** | 3-5 | 多列并行 + 碰撞检测 + 辩证综合 | 复杂问题多视角分析 |
| **辩论** | 2 | 两列对立 + 中间裁决 | 对立观点辨析 |
| **接力** | N | 横向时间轴，阶段卡片传递 | 多阶段任务推进 |
| **学习** | 2+ | 魂间教学（费曼学习法） | 知识学习与传授 |
| **实践开口** | 1+ | 方法论实践 | 理论结合实际操作 |
