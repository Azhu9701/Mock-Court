# Component Definitions — 万民幡 Web Application

## Frontend Components (Next.js App Router)

### 1. Soul Registry UI
- **Purpose**: 魂魄注册表浏览与搜索界面
- **Responsibilities**:
  - 24 魂列表展示（按 ismism 编码分组筛选）
  - 魂详情面板（档案、ismism 坐标、品级、召唤统计）
  - 关键词搜索与 ismism 编码过滤
  - 魂对比视图
- **Interface**: `/souls`, `/souls/[name]`

### 2. Conversation UI
- **Purpose**: 多模式对话界面，含合议可视化
- **Responsibilities**:
  - 单魂对话面板
  - 多魂合议终端式布局（N 魂 pane + 辩证综合面板）
  - 辩论双栏视图（正方 vs 反方 + 裁决）
  - 接力步骤串联视图
  - 流式输出实时渲染
  - WebSocket 连接管理
- **Interface**: `/possess/[mode]`

### 3. Practice Opening UI
- **Purpose**: 实践开口四步流程界面
- **Responsibilities**:
  - P1 对话式现场收集（追问交互）
  - P2-P4 分步展示（魂消化报告 → 修正记录 → 行动输出）
  - 盲区快照可视化
- **Interface**: `/practice-opening`

### 4. Soul Manager UI
- **Purpose**: 魂魄 CRUD 管理界面
- **Responsibilities**:
  - 收魂向导（人物名输入 → 搜索进度 → 素材预览）
  - 炼化流程展示（raw 素材 → Soul Profile 预览）
  - 审查面板（10 板块审查报告展示）
  - 升级/散魂操作界面
- **Interface**: `/souls/manage`

### 5. Analytics Dashboard
- **Purpose**: 战绩统计与有效性追踪
- **Responsibilities**:
  - 召唤统计图表（按魂/模式/有效性汇总）
  - 未召唤魂检测与推荐
  - 持续低效魂边界复审提醒
  - 速率限制与配额展示
- **Interface**: `/analytics`

### 6. History Browser
- **Purpose**: 对话历史浏览与回顾
- **Responsibilities**:
  - 会话列表（按日期/模式筛选）
  - 对话回放（魂输出原文浏览）
  - 存档导出
- **Interface**: `/history`

## Backend Components (Rust — axum)

### 7. Soul Registry Service
- **Purpose**: 魂魄数据管理与查询
- **Responsibilities**:
  - Registry YAML 解析与缓存
  - 魂档案 CRUD
  - ismism 坐标索引与搜索
  - 魂品级管理
- **Data**: `registry.yaml`, `souls/{name}.yaml`

### 8. Possession Engine
- **Purpose**: 附体模式编排核心
- **Responsibilities**:
  - 模式路由（单魂/合议/辩论/接力/学习/实践开口）
  - 魂选择与匹配
  - 多魂并行执行编排
  - 辩证综合触发与协调
  - 入口分流检测（在场者判断）
- **Data**: sessions, prompt templates

### 9. AI Gateway
- **Purpose**: LLM API 抽象层
- **Responsibilities**:
  - 多 provider 支持（Claude API / OpenAI API）
  - Prompt 构建与 summon_prompt 注入
  - 并发调用控制（多魂并行时的速率限制）
  - 流式响应代理（SSE → WebSocket 桥接）
  - Prompt caching 管理
- **Data**: provider configs, prompt templates

### 10. Archive System
- **Purpose**: 存档与记录管理
- **Responsibilities**:
  - 魂输出独立文件写入（落盘先于呈现）
  - call-records 写入与查询
  - 对话历史索引（SQLite）
  - 存档完整性校验（文件数 + 完整性检查）
- **Data**: `archive/`, `call-records.yaml`, SQLite

### 11. Analytics Engine
- **Purpose**: 统计数据计算
- **Responsibilities**:
  - 召唤统计聚合（按魂/模式/有效性）
  - 有效性评分趋势分析
  - 速率限制跟踪
- **Data**: SQLite (analytics tables)

### 12. Storage Layer
- **Purpose**: 统一存储接口
- **Responsibilities**:
  - SQLite 连接池与迁移
  - 文件系统读写（魂档案 YAML/MD）
  - 数据可移植性（导出/导入）
- **Data**: SQLite DB, filesystem
