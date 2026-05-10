# Tech Stack Decisions — B5: Archive & Analytics

## Decisions

### 1. 统计查询: SQL 聚合 + 内存缓存 (Q1: B)

**决策**: 首次查询执行 SQL 聚合，结果缓存 5 分钟。

**理由**:
- 直接 SQL 聚合避免物化视图的复杂增量更新逻辑
- 缓存减少重复查询开销（同一页面刷新无需重查 SQLite）
- TTL 可配置，满足不同的数据新鲜度需求

### 2. 导出: 异步后台任务 (Q2: C)

**决策**: `export_archive()` spawn 后台 task，返回 task_id 供轮询。

**理由**:
- 导出 100+ 次会话可能耗时数秒，同步阻塞不可取
- 异步任务模型简单：spawn → 写临时文件 → 通知完成
- 前端通过 `GET /api/export/{task_id}/status` 轮询状态

### 3. 无新增依赖

B5 不需要额外 crate：
- 存档委托给 `foundation::Storage`（已实现）
- 统计使用 SQLite 聚合查询（rusqlite 已有）
- JSON 导出使用 `serde_json`（workspace 已有）
- 缓存使用 `std::sync::RwLock` + `std::time::Instant`

## Dependencies

```toml
[dependencies]
foundation = { path = "../foundation" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
```

**无新增外部依赖。**
