# 性能优化 Spec

## Why
当前万民幡系统在运行时存在多个可量化的性能瓶颈：SQLite 并发写入能力受限、无界 Channel 存在内存泄漏风险、RwLock 争用影响吞吐、前端缺少虚拟列表导致长列表渲染卡顿、recharts 同步导入增大首屏包体积。本次优化旨在消除这些瓶颈，全面提升运行时性能和用户体验。

## What Changes
- Rust 后端：SQLite WAL 模式 + 连接池、有界 Channel、DashMap 替换 RwLock、LLM 语义缓存、归档分页导出、请求限流
- **BREAKING**: Channel 从 `unbounded` 改为 `bounded`，需在创建 channel 时指定容量并处理发送失败的情况
- Next.js 前端：虚拟列表、recharts 懒加载、打包优化配置、轮询改 WebSocket 推送、Markdown 清洗结果缓存

## Impact
- Affected specs: 万民幡完整 UI 升级、万民幡深度优化（NFR-1 性能指标）
- Affected code: `rust/foundation/src/sqlite.rs`, `rust/possession/src/ws.rs`, `rust/possession/src/stream.rs`, `rust/ai-gateway/src/model_router.rs`, `rust/archive/src/lib.rs`, `rust/api/src/main.rs`, `nextjs/app/sessions/[id]/page.tsx`, `nextjs/app/analytics/page.tsx`, `nextjs/next.config.ts`, `nextjs/hooks/use-websocket.ts`, `nextjs/components/soul-chat-bubble.tsx`

## ADDED Requirements

### Requirement: SQLite 并发性能优化
系统 SHALL 启用 SQLite WAL 模式以允许并发读写，并引入连接池以支持高并发数据库操作。

#### Scenario: 合议期间多个魂同时写入数据库
- **WHEN** 合议会话中多个魂完成输出并同时写入数据库
- **THEN** 写入操作不会因锁争用而串行等待，数据库查询响应时间 < 50ms
- **AND** 不会出现 "database is locked" 错误

### Requirement: 有界 Channel 防止内存泄漏
系统 SHALL 将 WebSocket 广播和流式传输中使用的 `unbounded_channel` 替换为 `bounded_channel`，防止消费者断开或变慢时消息无限积压。

#### Scenario: WebSocket 客户端断开连接
- **WHEN** 前端 WebSocket 连接断开但 LLM 仍在输出
- **THEN** Channel 缓冲区满后 LLM 输出侧收到 back-pressure，可被及时取消
- **AND** 不会因消息无限积压导致内存持续增长

### Requirement: RwLock 争用优化
系统 SHALL 将 `WsSessionManager` 和 `Registry` 中的 `RwLock<HashMap>` 替换为 `DashMap`，消除读写锁争用。

#### Scenario: 高并发广播
- **WHEN** 多个魂同时广播 token 到 WebSocket
- **THEN** 广播操作不会因单个写锁而串行化
- **AND** 吞吐量相比 RwLock 版本有可测量的提升

### Requirement: LLM 语义缓存
系统 SHALL 对相同 input + system prompt 的 LLM 调用实现语义缓存（基于 prompt hash），避免重复调用消耗费用和时间。

#### Scenario: 连续召唤同一魂处理相同任务
- **WHEN** 用户在短时间内用相同 input 连续召唤同一魂
- **THEN** 第二次调用命中缓存，直接返回缓存结果
- **AND** 不产生额外的 LLM API 调用费用

### Requirement: 归档导出分页
系统 SHALL 对归档导出功能实现分页加载，避免一次性加载所有数据导致 OOM。

#### Scenario: 导出大量历史会话
- **WHEN** 用户导出包含大量会话的归档
- **THEN** 数据按批次加载和输出，内存使用量保持稳定
- **AND** 导出功能不会因数据量大而崩溃

### Requirement: 前端虚拟列表
系统 SHALL 在历史会话详情页使用虚拟滚动渲染消息列表，仅渲染可视区域内的消息卡片。

#### Scenario: 查看包含大量消息的历史会话
- **WHEN** 用户打开包含 100+ 条消息的历史会话详情页
- **THEN** DOM 中仅存在可视区域内的约 10-20 条消息卡片
- **AND** 页面滚动流畅，无卡顿
- **AND** 初始渲染时间 < 200ms

### Requirement: recharts 懒加载
系统 SHALL 使用 `next/dynamic` 对 analytics 页面的 recharts 图表组件进行懒加载，避免同步导入增大主 bundle。

#### Scenario: 访问非 analytics 页面
- **WHEN** 用户访问首页或附体页面
- **THEN** recharts 代码不会被打包到当前页面的 JS bundle 中
- **AND** 首屏 JS 体积减少约 200KB+

### Requirement: Next.js 打包优化配置
系统 SHALL 在 `next.config.ts` 中配置 `optimizePackageImports` 优化常用库的导入。

#### Scenario: 生产构建
- **WHEN** 执行 `next build`
- **THEN** lucide-react 图标按需导入，不会全量打包
- **AND** 各页面 bundle 大小在合理范围

### Requirement: SidebarSessions 轮询替换
系统 SHALL 将 SidebarSessions 的 5 秒轮询替换为 WebSocket 或 BroadcastChannel 推送，减少不必要的 HTTP 请求。

#### Scenario: 侧边栏会话列表更新
- **WHEN** 新的附体会话被创建或完成
- **THEN** 侧边栏自动收到通知并更新
- **AND** 不再每 5 秒发起 HTTP 请求

### Requirement: Markdown 渲染优化
系统 SHALL 对 Markdown 渲染前的 HTML 标签清洗做缓存，避免每次渲染都重复执行字符串替换。

#### Scenario: 魂流式输出过程中
- **WHEN** 魂以 50ms 间隔持续输出 token
- **THEN** 已渲染内容的 HTML 清洗结果被缓存
- **AND** 只对新增加的 content 部分进行清洗

### Requirement: 请求限流
系统 SHALL 在后端 API 层添加请求速率限制，防止滥用导致系统过载。

#### Scenario: 短时间内大量 API 请求
- **WHEN** 同一 IP 在短时间内发起大量 API 调用
- **THEN** 超出限制的请求返回 429 Too Many Requests
- **AND** 正常用户不受影响

## MODIFIED Requirements

### Requirement: NFR-1 性能指标（来自深度优化/UI 升级 Spec）
系统 SHALL 满足以下增强的性能指标：
- 合议启动时间 < 3 秒（不变）
- Token 流延迟 < 100ms（不变）
- 支持同时 10+ 魂并行输出（不变）
- 数据库查询响应 < 30ms（从 50ms 优化）
- 页面切换响应 < 150ms（从 200ms 优化）
- 前端首屏 JS bundle < 300KB gzipped（新增）
- 长历史会话详情页渲染 < 200ms（新增）
