# 性能优化验证检查清单

## Rust 后端优化验证

### [ ] 1. SQLite WAL 模式 + 连接池
- [ ] WAL 模式已启用（`PRAGMA journal_mode` 返回 `wal`）
- [ ] 连接池正常工作（并发请求不出现 `database is locked`）
- [ ] 数据库查询响应时间 < 30ms
- [ ] 现有所有数据库操作功能正常

### [ ] 2. 有界 Channel 替换
- [ ] 所有 `mpsc::unbounded_channel` 已替换为 `mpsc::channel`（有界）
- [ ] Channel 满时不会 panic，正确处理 `try_send` 失败
- [ ] 消费者断开后消息不会无限积压
- [ ] 正常流式传输不受影响（token 不丢失）
- [ ] WebSocket 广播正常工作

### [ ] 3. DashMap 替换 RwLock<HashMap>
- [ ] `WsSessionManager` 使用 `DashMap`，广播操作无读写锁
- [ ] `Registry` 灵魂缓存使用 `DashMap`
- [ ] 并发广播吞吐量有可测量提升
- [ ] 魂加载/重载功能正常
- [ ] 代码简化，减少了锁管理样板代码

### [ ] 4. LLM 语义缓存
- [ ] 相同 prompt 第二次调用命中缓存
- [ ] 命中缓存时不产生额外 LLM API 调用
- [ ] TTL 过期后缓存正确失效
- [ ] 不同 prompt 不会错误命中缓存
- [ ] 缓存键正确包含 provider/model/system_prompt/user_prompt

### [ ] 5. 归档导出分页
- [ ] 大量会话导出时不发生 OOM
- [ ] 分页游标正确，不丢失也不重复数据
- [ ] 导出结果与全量导出一致
- [ ] 内存使用在导出过程中保持稳定

### [ ] 6. 请求限流中间件
- [ ] 超过阈值的请求返回 429
- [ ] 正常使用不受影响
- [ ] 429 响应包含 `Retry-After` 头
- [ ] 限流配置可通过 config 文件调整
- [ ] 不同 IP 之间的限流相互独立

## 前端优化验证

### [ ] 7. 虚拟列表
- [ ] 历史会话详情页在 100+ 消息时仅渲染可视区域卡片
- [ ] 滚动流畅无卡顿
- [ ] 初始渲染时间 < 200ms
- [ ] ArticleModal 点击打开/关闭正常
- [ ] CSS Grid 布局保持一致
- [ ] 窗口缩放时虚拟列表正确自适应

### [ ] 8. recharts 懒加载 + 打包优化
- [ ] analytics 之外页面不包含 recharts chunk
- [ ] analytics 页面有加载占位符
- [ ] `optimizePackageImports` 配置已添加
- [ ] 首屏 JS bundle gzipped < 300KB
- [ ] lucide-react 图标按需导入

### [ ] 9. SidebarSessions 轮询替换
- [ ] 会话创建/完成后侧边栏自动更新
- [ ] 不再有 5 秒轮询 HTTP 请求
- [ ] 初始加载时首次 HTTP 请求正常
- [ ] WebSocket 断开时侧边栏状态合理（保持不变或显示最后状态）

### [ ] 10. Markdown 渲染缓存
- [ ] 相同 content 不重复执行 HTML 清洗
- [ ] content 变化时正确重新清洗
- [ ] 流式渲染性能有可感知提升
- [ ] SoulChatBubble 和 SoulPanel 均受益

## 性能指标汇总

### [ ] 11. 核心性能指标
- [ ] 合议启动时间 < 3 秒
- [ ] Token 流延迟 < 100ms
- [ ] 支持同时 10+ 魂并行输出
- [ ] 数据库查询响应 < 30ms
- [ ] 页面切换响应 < 150ms
- [ ] 前端首屏 JS bundle gzipped < 300KB
- [ ] 长历史会话详情页渲染 < 200ms

## 回归验证

### [ ] 12. 功能回归
- [ ] 单魂附体功能正常
- [ ] 合议模式功能正常
- [ ] 辩论模式功能正常
- [ ] 接力模式功能正常
- [ ] 学习模式功能正常
- [ ] 实践开口模式功能正常
- [ ] 魂管理 CRUD 正常
- [ ] 知识库搜索正常
- [ ] 归档导出功能正常
- [ ] 成本统计正常
