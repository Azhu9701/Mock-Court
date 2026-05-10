# NFR Requirements — B5: Archive & Analytics

## Performance

| 指标 | 目标 | 依据 |
|------|------|------|
| `record_call()` 延迟 | < 50ms | SQLite INSERT + YAML append |
| `get_summon_stats()` 首次查询 | < 200ms | SQLite GROUP BY 聚合（缓存后 < 1ms） |
| `get_summon_stats()` 缓存命中 | < 5ms | 从 HashMap 读取（Q1: B 缓存策略） |
| `export_archive()` 响应 | 立即返回 task_id | Q2: C 异步导出 |
| `verify_archive()` | < 100ms | 遍历文件系统检查存在性 |

## 统计缓存策略 (Q1: B)

```
StatsCache {
    summon_stats: RwLock<Option<(SummonStats, Instant)>>,
    mode_distribution: RwLock<Option<(HashMap<PossessionMode, usize>, Instant)>>,
    ttl: Duration,  // 默认 5 分钟
}

get_summon_stats():
  if cache hit (exists && age < ttl) → return cached
  else → query SQLite → cache → return
```

## 异步导出策略 (Q2: C)

```
export_archive() -> Result<String>  // 返回 task_id
  1. 生成 task_id = uuid
  2. tokio::spawn(async { build_export_bundle(); write to file; update status })
  3. 返回 task_id 供前端轮询

export_status(task_id) -> ExportStatus
  Pending / Running / Complete(path) / Failed(error)
```

## 存储

| 指标 | 目标 |
|------|------|
| 存档文件路径 | data/archive/YYYY/MM/DD/{session_id}/ |
| 单次会话存档大小 | ~50KB（含 N 魂输出） |
| 导出 JSON 大小 | ~5MB（100 次会话估算） |
| 导出文件路径 | data/exports/{task_id}.json |

## 可靠性

| 要求 | 描述 |
|------|------|
| Dual-write 保证 | call_records 同时写入 SQLite + YAML，继承 B1 |
| 原子文件写入 | 存档文件继承 B1 atomic_write（tmp → rename） |
| 导出失败恢复 | 导出文件先写 tmp，成功后 rename |

## Maintainability

| 要求 | 描述 |
|------|------|
| 统计缓存 TTL | 可配置（环境变量 STATS_CACHE_TTL_MINUTES，默认 5） |
| 导出清理 | 导出文件保留 7 天后自动删除 |
