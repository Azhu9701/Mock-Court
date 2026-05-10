# NFR Requirements Plan — B5: Archive & Analytics

## NFR Questions

B5 基于 foundation Storage trait，大部分 NFR 继承 B1（SQLite WAL、原子写、dual-write）。以下为 B5 特有问题：

### Question 1: 统计查询性能策略
`get_summon_stats()` 执行 SQL 聚合查询时的性能策略？

A) 直接查询 — 对 call_records 表执行 GROUP BY 聚合，不缓存
B) 查询 + 应用层缓存 — 首次查询后缓存 N 分钟
C) 物化视图 — 在 SQLite 中创建统计汇总表，每次 record_call 时更新

[Answer]: B

### Question 2: 导出规模
`export_archive()` 一次性导出全部数据时的预期规模？

A) 全量导出 — 导出所有历史数据，可能很大，同步处理
B) 分页导出 — 支持按时间范围/数量分批导出
C) 异步导出 — spawn 后台任务生成，完成后通知下载

[Answer]: C
