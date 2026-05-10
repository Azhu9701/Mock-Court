# Functional Design Plan — B5: Archive & Analytics

## Plan Checklist

- [x] Generate `domain-entities.md` — SessionDetail, ArchiveVerification, ArchivePath, SummonStats, SoulAlert
- [x] Generate `business-logic-model.md` — ArchiveSystem, AnalyticsEngine, 统计聚合算法
- [x] Generate `business-rules.md` — 落盘约束, 完整性校验, 低效检测规则

## Design Questions

### Question 1: 统计计算策略
`get_summon_stats()` 等统计方法如何计算？

A) SQL 聚合查询 — 直接从 SQLite 的 call_records/sessions 表聚合计算
B) 内存缓存 + SQL fallback — 启动时计算并缓存统计，定期刷新
C) SQL + 增量更新 — 每次 record_call 时增量更新内存统计
D) Other

[Answer]: A

### Question 2: 导出格式
`export_archive()` 导出存档使用什么格式？

A) JSON — 标准格式，易于其他系统解析
B) YAML — 与现有魂档案格式一致
C) ZIP 打包 — 多文件打包（sessions.json + 魂输出文件）
D) Other

[Answer]: A

### Question 3: 未召唤魂检测
`detect_unsummoned_souls(threshold)` 如何检测未被召唤的魂？

A) 基于 call_records 表 — 查询指定时间内无调用记录的魂
B) 基于 registry.yaml 的 summon_count — 检查 summon_count 为 0 的魂
C) A + B 组合 — call_records 近期活跃度 + registry 累计计数
D) Other

[Answer]: C

### Question 4: 存档完整性校验
`verify_archive(session_id)` 如何校验存档完整性？

A) 简单存在性检查 — 检查所有预期文件是否存在
B) 文件计数 + 大小校验 — 检查文件数 + 总大小是否匹配
C) 哈希校验 — 对每个文件计算哈希并存储 manifest.json
D) Other

[Answer]: A

### Question 5: SessionDetail 内容
get_session 返回的 SessionDetail 包含什么？

A) Session 元数据 + 消息列表 — Session + Vec<Message>
B) 上述 + call_records — 还包含该 session 的调用记录
C) 上述 + 存档路径 — 还包含关联的存档文件路径列表
D) Other

[Answer]: A
