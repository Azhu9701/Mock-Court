# Requirements Document — 万民幡 Web Application

## Intent Analysis Summary
- **User Request**: 将万民幡做成一个 Web 应用
- **Request Type**: New Project (Greenfield)
- **Scope Estimate**: Multiple Components (System-wide)
- **Complexity Estimate**: Complex

## 万民幡是什么

万民幡是一个**实践与理论的反馈循环**系统——将思想人物封装为可调度的 AI 子 agent，同时为实践者在场经验预留开口。核心理念：**知识不在魂里，在实践里**。魂的价值不是提供答案，而是帮助结构化实践者带回的新经验。当分析触及方法论盲区时，系统生成实践议题而非用历史人物填补。

目前已实现为 Claude Code 内的 skill 系统（24 魂 registry + 多模式附体 + Obsidian 存档），本项目将其重构为独立 Web 应用。

---

## Functional Requirements

### FR1: 魂魄 Registry 系统

万民幡核心是 24 个思想人物魂魄，每个魂有一套 ismism 四维坐标体系（场域/本体论/认识论/目的论）。

- **FR1.1** 魂魄列表浏览：展示全部 24 魂，按 ismism 编码分组筛选
- **FR1.2** 魂魄详情页：查看魂档案（名称、ismism 坐标、核心领域、排除场景、品级、召唤统计）
- **FR1.3** 魂魄搜索：按领域关键词、ismism 编码、品级搜索魂
- **FR1.4** Registry 数据源：YAML/JSON 格式的 registry 文件，含魂摘要和召唤统计

### FR2: 附体 — 多模式 AI 对话系统

**核心原则：主程序是纯协作者，不扮演任何魂角色。**

#### FR2.1 单魂附体
- 用户选择单一魂 + 输入问题
- 系统注入魂的 summon_prompt（人格、立场、语言风格）
- 魂独立输出分析（流式展示）
- 适用场景：单一领域，目标明确

#### FR2.2 多魂合议（核心模式）
- 用户输入任务，选择或自动匹配 3-4 个魂
- 多个魂并行分析同一问题
- 所有魂输出完成后 → **辩证综合**阶段：综合官 agent 读取所有魂输出，产出五步综合报告：
  - 共识点 / 分歧点 / 盲区识别 / 主要矛盾 / 行动纲领
- 用户可观看每个魂的实时输出

#### FR2.3 魂间辩论
- 两魂对立论证同一议题（正反方）
- 裁决 agent 独立评判双方论点
- 适用场景：两难决策

#### FR2.4 魂链接力
- 多魂串联：前魂输出作为后魂输入
- 衔接点审查
- 适用场景：多阶段串联任务

#### FR2.5 使用者学习模式
- 魂向使用者系统讲解主题
- 讲解 → 知识卡片生成 → 反馈迭代
- 与附体的区别：魂输出的是教材而非分析判断

#### FR2.6 实践开口模式（关键差异化功能）
- **入口分流**：系统检测使用者是否为议题在场者（含具体案例 / 信息不可搜索 / 第一人称叙述）
- 在场者 → 走独立四步流程：
  - P1 现场收集：系统追问细节，标记盲区
  - P2 魂消化：魂输出「框架 vs 数据映射表」（确认/修正/推翻）
  - P3 魂修正：框架修正写回魂档案
  - P4 行动输出：1-2 条 24 小时内可执行的下一步
- 这是万民幡区别于普通 AI 聊天应用的核心特性

### FR3: 收魂 — 新魂魄创建与炼化

- **FR3.1** 收魂向导：输入人物名 → 多引擎搜索（8 维度：6 基础维 + 主义主义 4 维定位）
- **FR3.2** 炼化流程：raw 素材 → 结构化 Soul Profile（ismism 坐标 + 品级评估 + summon_prompt 生成）
- **FR3.3** 审查流程：幡主独立审查新魂（10 个强制板块：审查类型判定 → 前提假设 → 历史条件 → 肯定/批判 → 条件化判定 → 审查裁定 → 适用边界 → 一致性论证 → 框架无效假设检查 → 审查框架特定性标注）
- **FR3.4** 用户可自定义角色（设定人格、知识背景、语言风格），简化版炼化流程
- **FR3.5** 魂魄管理：升级、散魂（删除）、同步

### FR4: 存档与记录系统

- **FR4.1** 每次附体自动存档：魂原始输出独立文件（单魂 1 文件 / 合议 N+1 文件 / 辩论 3 文件 / 接力 M+1 文件）
- **FR4.2** 召唤记录：call-records（日期/魂/模式/任务/有效性评分）
- **FR4.3** 对话历史浏览与回顾

### FR5: 幡中战绩 — 统计与分析

- **FR5.1** 召唤统计面板：按魂/模式/有效性汇总
- **FR5.2** 未召唤魂检测 → 推荐调整
- **FR5.3** 持续低效魂检测 → 适用边界复审提醒
- **FR5.4** 速率限制与使用配额（免费工具审计约束）

### FR6: 使用者参与环节

- **FR6.1** 附体后追问：「最没想到的是什么？」
- **FR6.2** 自我否定环节：「哪个预设被动摇了？」
- **FR6.3** 空椅子环节（常规附体模式）

---

## Non-Functional Requirements

### NFR1: 技术栈
- **后端**：Rust（actix-web 或 axum）
- **前端**：React/Vue + TypeScript
- **AI 集成**：LLM API 调用（Claude API / OpenAI API），支持 prompt caching
- **存储**：本地优先
  - 结构化数据：SQLite（registry、call-records、users）
  - 魂档案：Markdown/YAML 文件
  - 对话存档：Markdown 文件 + SQLite 索引

### NFR2: 部署
- 本地单机运行（Rust 二进制 + 前端静态文件）
- 可选：Docker 一键启动

### NFR3: UI/UX
- 现代简约风格，类 ChatGPT 对话界面
- 深色/浅色主题
- 中文界面
- 多魂合议可视化：并行 pane 展示各魂输出 + 辩证综合面板

### NFR4: 性能
- 流式响应（SSE/WebSocket）
- 多魂并行调用 LLM API
- 分层数据访问：列表视图只加载摘要，详情才加载完整档案（参考现有 token 优化策略）

### NFR5: AI 协议
- 主程序不扮演魂角色 — 所有魂必须通过独立 API 调用（独立 system prompt）
- 落盘优于呈现：魂输出先保存再展示
- 多魂并行执行，辩证综合独立 agent

### NFR6: 数据可移植性
- 魂档案格式兼容现有 Soul Profile YAML 格式
- 支持导出/导入 Obsidian vault 格式
- registry 和 call-records 标准格式

## Extension Configuration
| Extension | Enabled | Decided At |
|-----------|---------|------------|
| Security Baseline | No | Requirements Analysis |
| Property-Based Testing | No | Requirements Analysis |
