# 万民幡 Rust 版 - 完整 UI 升级实现计划

## [x] Task 1: DeepSeek V4 缓存与结构化输出优化
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 优化 PromptBuilder，实现缓存友好的消息顺序
  - 添加结构化输出支持（JSON Schema）
  - 完善 DeepSeek 客户端，支持不同推理强度（Think/Think High/Think Max）
- **Acceptance Criteria Addressed**: AC-10
- **Test Requirements**:
  - `programmatic`: 验证缓存优化前后的响应时间对比
  - `programmatic`: 验证结构化输出能正确解析
  - `human-judgment`: 检查代码结构清晰，易于维护
- **Notes**: 优先实现，因为能立即带来成本降低

## [x] Task 2: 模型智能路由系统
- **Priority**: P0
- **Depends On**: Task 1
- **Description**: 
  - 设计并实现 ModelRouter trait
  - 实现任务类型分类（简单魂/核心分析/综合/审查）
  - 根据任务类型自动选择模型和推理强度
  - 添加降级策略
- **Acceptance Criteria Addressed**: AC-12
- **Test Requirements**:
  - `programmatic`: 单元测试验证路由逻辑正确
  - `programmatic`: 集成测试验证不同任务使用不同模型
  - `human-judgment`: 代码审查路由策略合理性

## [x] Task 3: 流式交叉检测器（基础框架）
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 设计交叉检测数据结构
  - 实现 token 流缓冲区
  - 实现基础的冲突检测逻辑（关键词匹配）
  - 添加碰撞事件类型到 WsEventType
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `programmatic`: 单元测试验证缓冲区管理
  - `programmatic`: 单元测试验证基础冲突检测
  - `human-judgment`: 设计文档审查

## [/] Task 4: 魂长驻进程管理器
- **Priority**: P1
- **Depends On**: Task 2
- **Description**: 
  - 实现 SoulProcess struct（tokio task + channel）
  - 实现 SoulRegistry 扩展支持长驻进程
  - 添加进程生命周期管理（启动/休眠/唤醒）
  - 利用 DeepSeek 跨轮思考保留
- **Acceptance Criteria Addressed**: AC-10
- **Test Requirements**:
  - `programmatic`: 单元测试进程状态管理
  - `programmatic`: 集成测试验证记忆连续性
  - `human-judgment`: 架构设计审查

## [ ] Task 5: 三区制布局框架（前端）
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 实现响应式布局容器组件
  - 桌面端三列网格布局
  - 移动端标签页布局
  - 碰撞通知栏容器
  - 辩证综合面板容器
- **Acceptance Criteria Addressed**: AC-1, AC-5
- **Test Requirements**:
  - `human-judgment`: 验证布局在不同屏幕尺寸正常显示
  - `programmatic`: 组件渲染测试
- **Notes**: 这是 UI 升级的基础框架

## [ ] Task 6: 魂面板组件（多列并行）
- **Priority**: P0
- **Depends On**: Task 5
- **Description**: 
  - 实现单个魂面板组件
  - 集成 token 流显示（打字机效果）
  - 实现进度条显示
  - 添加碰撞 badge 提示
  - 鼠标悬停显示领域标签
- **Acceptance Criteria Addressed**: AC-2
- **Test Requirements**:
  - `human-judgment`: UI 交互测试
  - `programmatic`: 组件状态更新测试

## [ ] Task 7: 碰撞通知栏组件
- **Priority**: P0
- **Depends On**: Task 5
- **Description**: 
  - 实现碰撞事件显示组件
  - 实现碰撞摘要显示
  - 点击展开详细内容
  - 历史碰撞展开/收起
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `human-judgment`: 交互测试
  - `programmatic`: 事件处理测试

## [ ] Task 8: 辩证综合面板组件（持续更新）
- **Priority**: P0
- **Depends On**: Task 5
- **Description**: 
  - 实现共识点显示（带进度条）
  - 实现分歧点显示（可展开）
  - 实现盲区点显示（可展开）
  - 实现矛盾点和建议行动显示
  - 持续更新无需等待完成
- **Acceptance Criteria Addressed**: AC-4
- **Test Requirements**:
  - `human-judgment`: UI 流畅性测试
  - `programmatic`: 实时更新测试

## [ ] Task 9: 增强的合议模式（流式+交叉检测）
- **Priority**: P0
- **Depends On**: Task 3, Task 4, Task 6, Task 7, Task 8
- **Description**: 
  - 重构 conference::run 支持流式交叉
  - 集成交叉检测器到合议流程
  - 实现追问动态注入机制
  - 前端 WebSocket 事件处理更新
  - 集成新 UI 组件
- **Acceptance Criteria Addressed**: AC-1, AC-2, AC-3, AC-4
- **Test Requirements**:
  - `programmatic`: 集成测试验证碰撞事件生成
  - `programmatic`: 集成测试验证追问注入
  - `human-judgment`: 端到端 UI 交互测试

## [ ] Task 10: 辩论模式 UI 升级
- **Priority**: P1
- **Depends On**: Task 5
- **Description**: 
  - 实现两列对立布局组件
  - 实现中间裁决栏组件
  - 实现阶段性结论显示
  - 实现双方论点对比
  - 集成辩论模式后端逻辑
- **Acceptance Criteria Addressed**: AC-6
- **Test Requirements**:
  - `human-judgment`: UI 交互测试
  - `programmatic`: 组件渲染测试

## [ ] Task 11: 接力模式 UI 升级
- **Priority**: P1
- **Depends On**: Task 5
- **Description**: 
  - 实现横向时间轴组件
  - 实现阶段卡片组件
  - 实现衔接风险点高亮
  - 点击卡片展开完整输出
  - 集成接力模式后端逻辑
- **Acceptance Criteria Addressed**: AC-7
- **Test Requirements**:
  - `human-judgment`: UI 交互测试
  - `programmatic`: 组件渲染测试

## [ ] Task 12: 魂自我审计系统
- **Priority**: P1
- **Depends On**: Task 4
- **Description**: 
  - 实现自我审计逻辑（矛盾检测/盲区检测/前提动摇）
  - 实现修正提案数据结构
  - 添加审计触发机制（每次输出后）
  - 实现提案存储
- **Acceptance Criteria Addressed**: AC-11
- **Test Requirements**:
  - `programmatic`: 单元测试审计逻辑
  - `programmatic`: 集成测试提案生成
  - `human-judgment`: 审计策略合理性审查

## [ ] Task 13: 数据库扩展（修正历史/盲区/知识卡片）
- **Priority**: P1
- **Depends On**: None
- **Description**: 
  - 扩展 SQLite schema 添加 soul_revisions 表
  - 添加 blind_spots 表
  - 添加 knowledge_cards 表
  - 扩展 Storage trait 相关方法
- **Acceptance Criteria Addressed**: AC-8, AC-9
- **Test Requirements**:
  - `programmatic`: 数据库迁移测试
  - `programmatic`: CRUD 操作单元测试

## [ ] Task 14: 魂状态详情页增强
- **Priority**: P1
- **Depends On**: Task 13
- **Description**: 
  - 添加修正历史时间线组件
  - 添加盲区记录显示组件
  - 增强召唤统计可视化
  - 添加修正提案审查界面
  - 完善主义主义坐标雷达图
- **Acceptance Criteria Addressed**: AC-8
- **Test Requirements**:
  - `human-judgment`: UI 美观性和可用性测试
  - `programmatic`: 组件渲染测试

## [ ] Task 15: 全文检索集成（tantivy）
- **Priority**: P2
- **Depends On**: Task 13
- **Description**: 
  - 集成 tantivy 库
  - 实现索引构建和更新
  - 实现搜索接口
  - 添加到知识检索 API
- **Acceptance Criteria Addressed**: AC-9
- **Test Requirements**:
  - `programmatic`: 索引构建测试
  - `programmatic`: 搜索结果相关性测试
  - `human-judgment`: 搜索结果质量评估

## [ ] Task 16: 向量语义检索（基础版本）
- **Priority**: P2
- **Depends On**: Task 15
- **Description**: 
  - 选择向量存储方案（SQLite 扩展或轻量级）
  - 实现嵌入生成
  - 实现相似度搜索
  - 混合检索（全文+向量）
- **Acceptance Criteria Addressed**: AC-9
- **Test Requirements**:
  - `programmatic`: 向量生成和搜索测试
  - `human-judgment`: 检索质量评估

## [ ] Task 17: 知识库检索界面
- **Priority**: P1
- **Depends On**: Task 15, Task 16
- **Description**: 
  - 实现搜索框组件
  - 实现多维筛选组件
  - 实现结果列表组件（参与魂/核心分歧/知识卡片）
  - 实现盲区热力图组件
  - 集成检索 API
- **Acceptance Criteria Addressed**: AC-9
- **Test Requirements**:
  - `human-judgment`: UI 交互测试
  - `programmatic`: 搜索功能测试

## [ ] Task 18: 成本透明化增强
- **Priority**: P2
- **Depends On**: Task 1
- **Description**: 
  - 完善成本估算逻辑（考虑缓存折扣）
  - 实时成本统计在会话中显示
  - 添加历史成本分析页面
  - 前端组件更新
- **Acceptance Criteria Addressed**: AC-13
- **Test Requirements**:
  - `programmatic`: 成本计算准确性测试
  - `human-judgment`: UI 显示合理性检查

## [ ] Task 19: 盲区热力图
- **Priority**: P2
- **Depends On**: Task 13, Task 17
- **Description**: 
  - 实现盲区统计逻辑
  - 设计可视化方案
  - 前端组件实现
  - 集成到知识库页面
- **Acceptance Criteria Addressed**: AC-9
- **Test Requirements**:
  - `programmatic`: 盲区统计逻辑测试
  - `human-judgment`: 可视化效果评估

## [ ] Task 20: TUI 界面（ratatui）
- **Priority**: P2
- **Depends On**: Task 9
- **Description**: 
  - 集成 ratatui 库
  - 实现魂 token 流显示
  - 实现碰撞通知显示
  - 实现辩证综合显示
  - 实现键盘快捷键
- **Acceptance Criteria Addressed**: AC-14
- **Test Requirements**:
  - `human-judgment`: TUI 交互测试
  - `programmatic`: 终端渲染测试

## [x] Task 13: 集成测试与文档
- **Priority**: P1
- **Depends On**: All above
- **Description**: 
  - 端到端集成测试
  - 性能基准测试
  - 更新 API 文档
  - 用户指南更新
- **Acceptance Criteria Addressed**: All ACs
- **Test Requirements**:
  - `programmatic`: 完整测试套件通过
  - `human-judgment`: 文档完整性和准确性审查
