# Application Design Plan — 万民幡 Web Application

## Plan Checklist

- [ ] Generate components.md — 组件定义与职责
- [ ] Generate component-methods.md — 方法签名与接口契约
- [ ] Generate services.md — 服务定义与编排模式
- [ ] Generate component-dependency.md — 依赖关系与通信模式
- [ ] Validate design completeness and consistency

## Component Identification

### Proposed Component Architecture

| 组件 | 职责 | 类型 |
|------|------|------|
| **Soul Registry** | 魂魄注册表，管理 24 魂 ismism 坐标、搜索、筛选 | 前端+后端 |
| **Possession Engine** | 附体引擎，六种模式编排（单魂/合议/辩论/接力/学习/实践开口） | 后端核心 |
| **Soul Manager** | 魂魄 CRUD，收魂→炼化→审查入幡流程 | 后端 |
| **Archive System** | 对话存档、魂输出独立文件管理、call-records | 后端 |
| **Analytics Dashboard** | 战绩面板，召唤统计、有效性追踪 | 前端+后端 |
| **Conversation UI** | 对话界面，流式展示、多魂合议可视化 | 前端 |
| **AI Gateway** | LLM API 调用、prompt 管理、并发控制 | 后端 |
| **Storage Layer** | SQLite + 文件系统统一接口 | 后端 |

## Design Questions

Please answer each question by filling in the letter choice after the [Answer]: tag.

### Question 1: 前端架构模式
推荐使用哪种前端架构组织 Conversation UI + Analytics Dashboard？

A) React + TypeScript + Vite（单页应用 SPA）
B) Next.js App Router（支持 SSR/SSG + API Routes）
C) Vue 3 + TypeScript + Vite（单页应用 SPA）
D) 由技术负责人根据 Rust 后端最佳配合选择
E) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 2: 后端 API 风格
Rust 后端 API 层推荐哪种风格？

A) RESTful API（标准资源导向，actix-web/axum + serde）
B) GraphQL（灵活查询，async-graphql）
C) REST + SSE（流式对话）混合
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 3: Rust Web 框架
推荐使用哪个 Rust Web 框架？

A) axum（tokio 生态，类型安全，推荐）
B) actix-web（成熟稳定，性能高）
C) Rocket（易用性好，macro 丰富）
D) 由技术负责人根据项目特点选择
E) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 4: 前端与后端的通信方式
多魂合议时，前端如何接收多个魂的实时输出？

A) SSE（Server-Sent Events）— 每个魂一个 event stream
B) WebSocket — 单连接多 channel
C) Polling — 轮询状态接口
D) Other (please describe after [Answer]: tag below)

[Answer]: B

### Question 5: 前端多魂合议可视化布局
多魂合议界面（3-4 魂并行 + 辩证综合）的推荐布局？

A) 垂直分栏 — 上方 N 魂并排 pane，下方辩证综合面板
B) 网格布局 — N+1 等大 pane，用户自由调整
C) 标签切换 — 各魂输出独立 tab，综合 tab 独立
D) 类 cmux 终端式 — 多个独立面板，实时滚动
E) Other (please describe after [Answer]: tag below)

[Answer]: D

### Question 6: 存储策略
魂档案和对话存档的存储方式？

A) SQLite + 本地文件系统（魂档案 YAML/MD 文件 + SQLite 索引）
B) 纯 SQLite（全部数据存 SQLite，含魂档案 BLOB）
C) 纯文件系统（YAML/MD/JSON 文件，无数据库）
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 7: AI Provider 集成
LLM API 调用层的设计？

A) 统一 AI Gateway 抽象（支持 Claude API + OpenAI API，多 provider 切换）
B) 只支持 Claude API（与现有万民幡 skill 一致）
C) 先支持单一 provider，后续扩展
D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 8: 实践开口模式的前端交互
实践开口 P1-P4 四步流程在前端的呈现方式？

A) 向导式（Step-by-step wizard，每步独立页面）
B) 对话式（类 chat 界面，系统追问驱动流程）
C) 混合式（P1 对话式收集 + P2-P4 分步展示）
D) Other (please describe after [Answer]: tag below)

[Answer]: C 
