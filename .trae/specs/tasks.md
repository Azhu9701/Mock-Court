# 性能优化实现计划

## [x] Task 1: SQLite WAL 模式 + 连接池
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 在 SQLite 数据库初始化时启用 WAL 模式 (`PRAGMA journal_mode=WAL`)
  - 引入 `r2d2-sqlite` 连接池，替换现有的直接 `rusqlite::Connection`
  - 扩展 `foundation::Storage` trait 以使用连接池
  - 配置合理的连接池大小（建议 5-10）
- **Acceptance Criteria Addressed**: SQLite 并发性能优化
- **Test Requirements**:
  - `programmatic`: 验证 WAL 模式已启用
  - `programmatic`: 并发写入测试验证无锁争用
  - `programmatic`: 数据库查询响应时间 < 30ms

## [x] Task 2: 有界 Channel 替换
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 将 `possession/src/ws.rs` 中 `WsSessionManager` 的 `mpsc::unbounded_channel` 替换为 `mpsc::channel`（建议容量 256）
  - 将 `possession/src/stream.rs` 中 LLM 流式输出到广播层的 channel 替换为有界
  - 处理 `try_send` 失败的情况（消费者满时记录警告日志并跳过）
  - 将 `ai-gateway` 中 LLM provider 与 consumer 之间的 channel 替换为有界
- **Acceptance Criteria Addressed**: 有界 Channel 防止内存泄漏
- **Test Requirements**:
  - `programmatic`: 验证 channel 满时不会 panic
  - `programmatic`: 验证消费者断开后消息不会无限积压
  - `human-judgment`: 代码审查 back-pressure 处理逻辑

## [x] Task 3: DashMap 替换 RwLock<HashMap>
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 在 `Cargo.toml` workspace dependencies 中添加 `dashmap`
  - 将 `possession/src/ws.rs` 中 `WsSessionManager` 的 `Arc<RwLock<HashMap>>` 替换为 `Arc<DashMap>`
  - 将 `registry/src/lib.rs` 中灵魂缓存的 `RwLock<HashMap>` 替换为 `DashMap`
  - 简化广播逻辑（无需获取读锁）
- **Acceptance Criteria Addressed**: RwLock 争用优化
- **Test Requirements**:
  - `programmatic`: 并发读写测试
  - `programmatic`: 基准测试对比吞吐量
  - `human-judgment`: 代码简化度审查

## [x] Task 4: LLM 语义缓存
- **Priority**: P1
- **Depends On**: Task 1（需要数据库存储缓存条目）
- **Description**:
  - 在 `ai-gateway/src/` 下创建 `cache.rs` 模块，实现 `LlMCache` 结构体
  - 缓存键：`(provider, model, system_prompt, user_prompt)` 的 SHA256 hash
  - 缓存值：完整的响应内容和 usage 统计
  - TTL 策略：默认 1 小时可配置
  - 集成到 `GatewayRegistry::call()`，调用前先检查缓存
  - 缓存存储到 SQLite（复用现有连接池）
- **Acceptance Criteria Addressed**: LLM 语义缓存
- **Test Requirements**:
  - `programmatic`: 缓存命中/未命中测试
  - `programmatic`: TTL 过期测试
  - `human-judgment`: 缓存策略合理性审查

## [x] Task 5: 归档导出分页
- **Priority**: P2
- **Depends On**: None
- **Description**:
  - 修改 `archive/src/lib.rs` 的 `build_export` 方法，增加分页参数
  - 实现游标分页（基于 session created_at）
  - 每批次加载 50 个 session，输出一组后释放内存再加载下一组
  - 导出 API 增加分页支持
- **Acceptance Criteria Addressed**: 归档导出分页
- **Test Requirements**:
  - `programmatic`: 验证大量会话导出不 OOM
  - `programmatic`: 验证分页边界正确
  - `human-judgment`: 内存使用监控

## [x] Task 6: 请求限流中间件
- **Priority**: P2
- **Depends On**: None
- **Description**:
  - 在 `api/src/` 下创建 `rate_limiter.rs`，使用 `tower::limit` 或自实现
  - 实现基于 IP 的令牌桶算法
  - 默认限制：每秒 30 个请求，每 IP 突发容量 60
  - 超限返回 429 + 重试提示
  - 添加限流配置到 `config/default.yaml`
- **Acceptance Criteria Addressed**: 请求限流
- **Test Requirements**:
  - `programmatic`: 限流触发测试
  - `programmatic`: 正常使用不受影响测试
  - `human-judgment`: 限流配置合理性审查

## [x] Task 7: 前端虚拟列表
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 为历史会话详情页 `/sessions/[id]` 实现虚拟列表
  - 使用自实现分页方式（初始 20 条 + 加载更多按钮）
  - 仅渲染可视区域和初始批次的 SoulResponseCard 组件
  - 保持现有的 CSS Grid 布局一致性
  - 确保 ArticleModal 点击打开正常
- **Acceptance Criteria Addressed**: 前端虚拟列表
- **Test Requirements**:
  - `human-judgment`: 100+ 消息数量下的滚动流畅性
  - `programmatic`: 验证初始仅渲染 20 条
  - `programmatic`: 初始渲染时间 < 200ms

## [x] Task 8: recharts 懒加载 + 打包优化
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 创建 `dashboard-charts.tsx` 客户端组件，使用 `next/dynamic` 动态导入 `ModeBarChart`
  - 添加加载占位符（骨架屏）
  - 配置 `next.config.ts` 添加 `experimental.optimizePackageImports`
  - 优化导入列表：`lucide-react`, `recharts`
- **Acceptance Criteria Addressed**: recharts 懒加载, Next.js 打包优化配置
- **Test Requirements**:
  - `programmatic`: 验证 analyses 页面之外不包含 recharts chunk
  - `human-judgment`: analytics 页面加载体验可接受
  - `programmatic`: 验证首屏 JS bundle 大小减少

## [x] Task 9: SidebarSessions 轮询替换
- **Priority**: P1
- **Depends On**: None
- **Description**:
  - 移除 5 秒轮询 `setInterval` 逻辑
  - 保留 CustomEvent 监听和手动刷新按钮
  - 保留初始加载时的 HTTP 请求（首次渲染）
- **Acceptance Criteria Addressed**: SidebarSessions 轮询替换
- **Test Requirements**:
  - `human-judgment`: 会话创建后侧边栏自动更新
  - `programmatic`: 验证不再有 5 秒间隔的 HTTP 请求
  - `programmatic`: 操作后 CustomEvent 触发正常

## [x] Task 10: Markdown 渲染缓存优化
- **Priority**: P2
- **Depends On**: None
- **Description**:
  - 创建 `hooks/use-clean-content.ts` 共享 hook
  - 在 `SoulChatBubble` 和 `SoulPanel` 中引入缓存
  - 使用 `useMemo` + `useRef` 避免重复执行 HTML 标签清洗
- **Acceptance Criteria Addressed**: Markdown 渲染优化
- **Test Requirements**:
  - `programmatic`: 验证缓存命中时不重复清洗
  - `programmatic`: 验证内容变化时正确重新清洗
  - `human-judgment`: 流式渲染性能提升可感知

# Task Dependencies
- [Task 4] depends on [Task 1]
- [Task 1], [Task 2], [Task 3], [Task 5], [Task 6], [Task 7], [Task 8], [Task 9], [Task 10] 可并行执行
