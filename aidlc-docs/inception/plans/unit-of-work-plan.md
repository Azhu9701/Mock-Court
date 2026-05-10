# Unit of Work Plan — 万民幡 Web Application

## Plan Checklist

- [ ] Generate `unit-of-work.md` — 单元定义与职责
- [ ] Generate `unit-of-work-dependency.md` — 依赖矩阵与执行顺序
- [ ] Generate `unit-of-work-story-map.md` — 需求到单元的映射
- [ ] Document code organization strategy
- [ ] Validate unit boundaries and dependencies

## Proposed Unit Decomposition

本项目为本地单体应用（Next.js 前端 + Rust 后端），单元按逻辑模块拆分：

### Backend Units (Rust)

| Unit | Name | Contents | Responsibility |
|------|------|----------|----------------|
| **B1** | Foundation | Storage Layer, data models, config, migrations | SQLite + FS 统一接口，数据模型定义，项目骨架 |
| **B2** | Soul Registry | Soul Registry Service | 魂 CRUD、ismism 索引、registry.yaml 管理 |
| **B3** | AI Gateway | AI Gateway | Claude/OpenAI/DeepSeek 三 provider 抽象，prompt 管理，并发控制 |
| **B4** | Possession Core | Possession Engine + WebSocket Manager | 六模式编排，入口分流，合议并行，WS 实时推送 |
| **B5** | Archive & Analytics | Archive System + Analytics Engine | 存档落盘、call-records、统计聚合、有效性追踪 |
| **B6** | API Layer | axum routes + middleware | REST endpoints + WebSocket handler，跨组件编排 |

### Frontend Units (Next.js)

| Unit | Name | Contents | Responsibility |
|------|------|----------|----------------|
| **F1** | App Shell | Layout, navigation, theme, routing | 应用骨架：顶部导航、侧边栏、深色/浅色主题切换 |
| **F2** | Soul Browser | Soul Registry UI + Soul Manager UI | 魂列表/详情/搜索 + 收魂→炼化→审查流程 |
| **F3** | Possession UI | Conversation UI + Practice Opening UI + WS client | 六模式对话界面、终端式合议布局、实践开口流程 |
| **F4** | Dashboard | Analytics Dashboard + History Browser | 战绩统计面板 + 对话历史浏览 |

## Decomposition Questions

### Question 1: 单元执行顺序
推荐哪种执行顺序？

A) 自底向上 — Foundation → Registry → AI Gateway → Possession → Archive → API → 然后前端（F1 → F2 → F3 → F4）
B) API 优先 — 先定义 API 契约和数据类型（B6 + B1 并行）→ 前后端可并行开发
C) 功能切片 — 按功能垂直切分（魂管理切片 B1+B2+F2 先做 → 对话切片 B3+B4+F3 → 分析切片 B5+F4）
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 2: 代码目录结构
推荐哪种 Rust 后端代码组织方式？

A) Cargo workspace（多个 crate：foundation / registry / ai-gateway / possession / archive / api）
B) 单一 crate + 模块分层（src/foundation/, src/registry/, src/ai/, src/possession/, src/archive/, src/api/）
C) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 3: 前后端是否在同一仓库
推荐哪种仓库策略？

A) Monorepo — 前端（nextjs/）和后端（rust/）在同一仓库，根目录统一管理
B) 先 Monorepo，后续可拆分
C) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 4: 初始包含多少预设魂
第一批内置魂的数量？

A) 6 个常用魂（列宁、毛泽东、费曼、鲁迅、未明子、邓小平）— 对应已入驻 agent
B) 24 个全量魂 — 对应完整 registry
C) 先 6 个，后续逐步添加
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 5: 第一个 MVP 包含哪些模式
推荐 MVP 优先实现的核心模式？

A) 单魂附体 + 合议（核心两种模式即可验证系统可用性）
B) 全部六种模式（单魂/合议/辩论/接力/学习/实践开口）
C) 单魂 → 合议 → 实践开口（三个最具差异化价值的模式）
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 6: 魂数据来源
魂魄数据（Soul Profile）的来源方式？

A) 从现有万民幡 `souls/` 目录和 `registry.yaml` 直接导入
B) 手动创建 Soul Profile 模板，再逐步录入
C) 内置种子数据 + 支持收魂创建新魂
D) Other (please describe after [Answer]: tag below)

[Answer]: A 
