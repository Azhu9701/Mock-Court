# 万民幡 Rust 版 - 深度优化产品需求文档

## Overview
- **Summary**: 对现有万民幡 Rust 版本进行深度优化，实现文档中描述的高级功能，包括流式合议、实时交叉检测、魂的自我审计、DeepSeek V4 缓存优化、模型智能路由等。
- **Purpose**: 从基础可用版本升级到完整的、具有自我进化能力的独立软件系统，提供更好的用户体验和更低的计算成本。
- **Target Users**: 深度思考者、决策者、学习者，需要多视角思维碰撞的用户。

## Goals
1. 实现流式合议（Streaming Conference） - 魂在输出过程中实时交叉检测，动态注入追问
2. 实现魂的长驻进程与自我审计 - 魂有记忆连续性，自动发现问题并提出修正
3. DeepSeek V4 深度优化 - 充分利用上下文缓存、1M 窗口、跨轮思考保留特性
4. 完善知识库检索系统 - 全文检索 + 向量语义检索 + 盲区热力图
5. 增强魂状态可见性 - 修正历史、盲区记录、活跃度统计
6. 模型智能路由 - 根据任务类型自动选择合适的模型和推理强度
7. 成本透明化 - 实时显示 LLM 调用次数、token 消耗、预估费用

## Non-Goals (Out of Scope)
1. 完全重写现有架构 - 在现有基础上渐进式优化
2. 添加新的附体模式 - 专注于优化现有模式
3. 移动端原生应用 - 保持响应式 Web 界面
4. 多用户协作功能 - MVP 阶段暂不实现

## Background & Context
当前项目已有基础架构：
- Rust 后端：Axum WebSocket API + 多个子 crate
- Next.js 前端：完整的 UI 组件和路由
- 支持单魂、合议、辩论、接力、学习、实践开口模式
- 支持 Claude、OpenAI、DeepSeek 多个提供商

但缺少文档中描述的高级功能：
- 流式碰撞检测
- 魂的长驻进程
- 自我审计与修正
- DeepSeek V4 特性优化
- 完善的知识检索

## Functional Requirements

### FR-1: 流式合议与实时交叉检测
- 魂并行输出 token 流
- 实时检测器监控所有魂的输出
- 检测到矛盾、互补、盲区时自动生成追问
- 动态将追问注入相关魂的推理上下文
- 碰撞事件实时推送到前端 UI

### FR-2: 魂长驻进程与记忆连续性
- 每个魂作为独立 tokio task 运行
- 内存中维护魂的状态（最近输出、活跃提案等）
- 利用 DeepSeek V4 跨轮思考保留特性
- 多次召唤间保持思维连续性

### FR-3: 魂自我审计与修正提案
- 每次输出后自动执行审计
- 检测自我矛盾、触及盲区、前提动摇
- 自动生成修正提案
- 提案提交幡主审查流程
- 审查通过后自动更新魂的 summon prompt

### FR-4: DeepSeek V4 深度优化
- 上下文缓存优化（静态前缀优先）
- 1M 窗口支持完整历史注入
- 跨轮思考保留（无需重复发送 system prompt）
- 结构化输出（综合报告直接 JSON）

### FR-5: 模型智能路由
- 根据任务类型选择模型（Flash/Pro/Pro Think High）
- 根据魂重要性分配推理强度
- 成本感知路由（权衡质量与费用）
- 自动降级策略（主模型不可用时切换备用）

### FR-6: 完善的知识库系统
- 全文检索（tantivy）
- 向量语义检索
- 盲区热力图（高频盲区可视化）
- 知识卡片自动提取与管理

### FR-7: 增强的魂状态可见性
- 修正历史时间线
- 盲区记录与标记
- 召唤统计（次数、有效性）
- 主义主义坐标雷达图

### FR-8: 成本透明化
- 实时 token 消耗统计
- LLM 调用次数计数
- 预估费用显示
- 历史成本分析

## Non-Functional Requirements

### NFR-1: 性能
- 合议启动时间 < 3 秒
- Token 流延迟 < 100ms
- 支持同时 10+ 魂并行输出
- 数据库查询响应 < 50ms

### NFR-2: 可靠性
- 会话失败自动恢复
- 魂输出超时保护（5 分钟）
- 数据库事务一致性
- WebSocket 断线重连

### NFR-3: 可维护性
- 模块化设计，清晰的职责分离
- 完善的日志系统
- 类型安全的 API
- 代码注释完整

### NFR-4: 成本优化
- 相比当前版本降低 50%+ 的 LLM 费用
- 充分利用 DeepSeek 缓存折扣
- 智能路由选择性价比最高的模型

## Constraints
- **Technical**: 必须使用 Rust + Next.js 技术栈
- **Business**: 保持开源友好，不引入商业限制库
- **Dependencies**: DeepSeek API 可用性（主要优化目标）

## Assumptions
1. DeepSeek V4 API 正常可用且稳定
2. 用户有有效的 DeepSeek API Key
3. 现有基础架构足够支撑新增功能
4. 用户接受渐进式功能发布

## Acceptance Criteria

### AC-1: 流式合议功能正常
- **Given**: 用户发起合议模式会话
- **When**: 多个魂同时输出
- **Then**: 
  - 每个魂独立流式输出 token
  - 碰撞事件实时显示在 UI 上
  - 追问能动态注入并影响魂的后续输出
- **Verification**: programmatic + human-judgment
- **Notes**: 需要模拟交叉检测场景验证

### AC-2: DeepSeek 缓存优化生效
- **Given**: 使用 DeepSeek 提供商
- **When**: 连续召唤同一魂或使用相同任务上下文
- **Then**: 
  - API 响应时间明显降低（首次 vs 后续）
  - 成本统计显示缓存折扣应用
- **Verification**: programmatic

### AC-3: 自我审计能够发现问题
- **Given**: 魂有历史输出记录
- **When**: 新输出与历史明显矛盾
- **Then**: 
  - 自动生成修正提案
  - 提案显示在魂详情页
- **Verification**: programmatic + human-judgment

### AC-4: 模型路由按预期工作
- **Given**: 不同类型的任务
- **When**: 发起附体/合议
- **Then**: 
  - 简单任务使用 Flash 模型
  - 复杂分析使用 Pro Think High
  - 辩证综合使用最大推理能力
- **Verification**: programmatic

### AC-5: 知识库检索返回相关结果
- **Given**: 有历史合议记录
- **When**: 用户搜索关键词
- **Then**: 
  - 返回相关的历史记录
  - 结果按相关性排序
  - 显示参与魂、核心分歧等元数据
- **Verification**: human-judgment

### AC-6: 魂状态页面信息完整
- **Given**: 魂有多次召唤记录
- **When**: 访问魂详情页
- **Then**: 
  - 显示修正历史时间线
  - 显示盲区记录
  - 显示召唤统计
  - 显示主义主义坐标
- **Verification**: human-judgment

### AC-7: 成本信息实时显示
- **Given**: 正在进行的会话
- **When**: 魂输出和综合完成
- **Then**: 
  - 实时显示 token 消耗
  - 显示 LLM 调用次数
  - 显示预估费用
- **Verification**: programmatic

## Open Questions
1. 修正提案的审查流程是完全自动化还是需要人工干预？
2. 向量检索使用什么实现（pgvector vs qdrant vs 其他）？
3. 魂的长驻进程是否需要持久化到磁盘（重启后恢复）？
4. 盲区热力图的具体可视化方案是什么？
