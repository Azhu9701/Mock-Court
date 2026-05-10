# Business Rules — B5: Archive & Analytics

## 1. 落盘约束（硬规则）

| 规则 | 描述 |
|------|------|
| BR1.1 | 落盘先于呈现 — `archive_soul_output()` 必须在数据返回给用户之前调用完成 |
| BR1.2 | 存档路径遵循 `YYYY/MM/DD/{session_id}/{filename}` 日期分组约定 |
| BR1.3 | 文件写入使用原子写（tmp → rename），继承 B1 的 `atomic_write` 模式 |
| BR1.4 | 存档文件命名：`{soul_name}.md`（魂输出）、`synthesis.md`（辩证综合）、`debate_{a}_vs_{b}.md`（辩论） |

## 2. Call Record 双写约束

| 规则 | 描述 |
|------|------|
| BR2.1 | 每次召唤必须记录 CallRecord（同步写入 SQLite + YAML） |
| BR2.2 | CallRecord.effectiveness 初始值为 Invalid，后续由审查流程更新 |
| BR2.3 | CallRecord 不可删除（审计追踪），只能通过新增 call_record 修正 |
| BR2.4 | SQLite 和 YAML 中的 CallRecord 必须保持一致（由 B1 dual-write 保证） |

## 3. 统计计算规则

| 规则 | 描述 |
|------|------|
| BR3.1 | `get_summon_stats()` 直接对 SQLite 执行聚合查询（Q1: A），无缓存 |
| BR3.2 | 时间范围统计使用 `created_at` 字段过滤 |
| BR3.3 | 魂有效性 = effective_count / total_calls（total_calls >= 5 时才有统计意义） |
| BR3.4 | 无调用记录的魂在按魂统计中不出现（count = 0） |

## 4. 检测规则

| 规则 | 描述 |
|------|------|
| BR4.1 | `detect_unsummoned_souls(threshold_days)` 检测两类：从未召唤 + 超过阈值天数未召唤 |
| BR4.2 | `detect_low_effectiveness(threshold)` 只对调用次数 ≥ 5 的魂进行评估 |
| BR4.3 | NeverSummoned 告警不可自动解除，需人工处理（召唤或散魂） |
| BR4.4 | UnsummonedLongDuration 在再次召唤后自动解除 |

## 5. 完整性校验规则

| 规则 | 描述 |
|------|------|
| BR5.1 | 校验基于存在性检查（Q4: A），不计算哈希 |
| BR5.2 | 预期文件数 = mode 对应产出数（Single: 2, Conference: N+1, Debate: 3, Relay: N, Learn: 1, PracticeOpening: 1+4） |
| BR5.3 | 校验失败不阻塞访问，仅返回 ArchiveVerification.ok = false |
| BR5.4 | `verify_archive()` 返回 `missing_files` 列表供 UI 展示 |

## 6. 导出/导入规则

| 规则 | 描述 |
|------|------|
| BR6.1 | 导出格式为 JSON（Q2: A） |
| BR6.2 | 导出包含 sessions + messages + call_records，不包含魂档案文件 |
| BR6.3 | 导入时不覆盖已有 session（按 session.id 判断），重复时跳过 |
| BR6.4 | 导入后需调用 `reload_registry()` 刷新 registry 缓存 |
