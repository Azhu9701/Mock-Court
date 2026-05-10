# 万民幡 Rust 版 - 深度优化实现计划

## [x] Task 1: DeepSeek V4 缓存与结构化输出优化
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 优化 PromptBuilder，实现缓存友好的消息顺序
  - 添加结构化输出支持（JSON Schema）
  - 完善 DeepSeek 客户端，支持不同推理强度（Think/Think High/Think Max）
- **Acceptance Criteria Addressed**: AC-2
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
- **Acceptance Criteria Addressed**: AC-4
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
- **Acceptance Criteria Addressed**: AC-1
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
- **Acceptance Criteria Addressed**: AC-2
- **Test Requirements**:
  - `programmatic`: 单元测试进程状态管理
  - `programmatic`: 集成测试验证记忆连续性
  - `human-judgment`: 架构设计审查

## [ ] Task 5: 增强的合议模式（流式+交叉检测）
- **Priority**: P1
- **Depends On**: Task 3, Task 4
- **Description**: 
  - 重构 conference::run 支持流式交叉
  - 集成交叉检测器到合议流程
  - 实现追问动态注入机制
  - 前端更新支持碰撞事件显示
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic`: 集成测试验证碰撞事件生成
  - `programmatic`: 集成测试验证追问注入
  - `human-judgment`: UI 交互测试

## [ ] Task 6: 魂自我审计系统
- **Priority**: P1
- **Depends On**: Task 4
- **Description**: 
  - 实现自我审计逻辑（矛盾检测/盲区检测/前提动摇）
  - 实现修正提案数据结构
  - 添加审计触发机制（每次输出后）
  - 实现提案存储
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `programmatic`: 单元测试审计逻辑
  - `programmatic`: 集成测试提案生成
  - `human-judgment`: 审计策略合理性审查

## [ ] Task 7: 数据库扩展（修正历史/盲区/知识卡片）
- **Priority**: P1
- **Depends On**: None
- **Description**: 
  - 扩展 SQLite schema 添加 soul_revisions 表
  - 添加 blind_spots 表
  - 添加 knowledge_cards 表
  - 扩展 Storage trait 相关方法
- **Acceptance Criteria Addressed**: AC-6, AC-5
- **Test Requirements**:
  - `programmatic`: 数据库迁移测试
  - `programmatic`: CRUD 操作单元测试

## [ ] Task 8: 全文检索集成（tantivy）
- **Priority**: P2
- **Depends On**: Task 7
- **Description**: 
  - 集成 tantivy 库
  - 实现索引构建和更新
  - 实现搜索接口
  - 添加到知识检索 API
- **Acceptance Criteria Addressed**: AC-5
- **Test Requirements**:
  - `programmatic`: 索引构建测试
  - `programmatic`: 搜索结果相关性测试
  - `human-judgment`: 搜索结果质量评估

## [ ] Task 9: 向量语义检索（基础版本）
- **Priority**: P2
- **Depends On**: Task 8
- **Description**: 
  - 选择向量存储方案（SQLite 扩展或轻量级）
  - 实现嵌入生成
  - 实现相似度搜索
  - 混合检索（全文+向量）
- **Acceptance Criteria Addressed**: AC-5
- **Test Requirements**:
  - `programmatic`: 向量生成和搜索测试
  - `human-judgment`: 检索质量评估

## [ ] Task 10: 前端魂详情页增强
- **Priority**: P2
- **Depends On**: Task 7
- **Description**: 
  - 添加修正历史时间线组件
  - 添加盲区记录显示
  - 增强召唤统计可视化
  - 添加修正提案审查界面
- **Acceptance Criteria Addressed**: AC-6
- **Test Requirements**:
  - `human-judgment`: UI 美观性和可用性测试
  - `programmatic`: 组件渲染测试

## [ ] Task 11: 成本透明化增强
- **Priority**: P2
- **Depends On**: Task 1
- **Description**: 
  - 完善成本估算逻辑（考虑缓存折扣）
  - 实时成本统计在会话中显示
  - 添加历史成本分析页面
  - 前端组件更新
- **Acceptance Criteria Addressed**: AC-7
- **Test Requirements**:
  - `programmatic`: 成本计算准确性测试
  - `human-judgment`: UI 显示合理性检查

## [ ] Task 12: 盲区热力图
- **Priority**: P2
- **Depends On**: Task 7, Task 8
- **Description**: 
  - 实现盲区统计逻辑
  - 设计可视化方案
  - 前端组件实现
  - 集成到知识库页面
- **Acceptance Criteria Addressed**: AC-5
- **Test Requirements**:
  - `programmatic`: 盲区统计逻辑测试
  - `human-judgment`: 可视化效果评估

## [ ] Task 13: 集成测试与文档
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
