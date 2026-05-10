# Business Logic Model — B5: Archive & Analytics

## ArchiveSystem

```rust
pub struct ArchiveSystem {
    store: Arc<dyn Storage>,
}
```

**依赖**: `Storage` trait（通过 `Arc<dyn Storage>` 注入），所有持久化委托给它。

### 存档操作（落盘先于呈现）

```
archive_soul_output(session_id, soul_name, content) -> Result<String>
  1. store.archive_soul_output(session_id, soul_name, content)
  2. 返回存档路径

archive_synthesis(session_id, content) -> Result<String>
  1. store.archive_synthesis(session_id, "synthesis", content)
  2. 返回存档路径

archive_debate(session_id, soul_a, soul_b, output_a, output_b) -> Result<(String, String)>
  1. path_a = store.archive_soul_output(session_id, soul_a, output_a)
  2. path_b = store.archive_soul_output(session_id, soul_b, output_b)
  3. 返回 (path_a, path_b)
```

### Call Record（双写）

```
record_call(record: &CallRecord) -> Result<()>
  1. store.record_call(record).await  // SQLite INSERT + YAML append
  2. 日志记录

query_call_records(filter: &CallFilter) -> Result<Vec<CallRecord>>
  1. store.query_call_records(filter).await
```

### 会话查询

```
list_sessions(filter: &SessionFilter) -> Result<Vec<SessionSummary>>
  1. store.list_sessions(filter).await

get_session(id: &str) -> Result<SessionDetail>
  1. session = store.get_session(id).await
  2. messages = store.get_messages(id).await
  3. return SessionDetail { session, messages }
```

### 完整性校验

```
verify_archive(session_id: &str) -> Result<ArchiveVerification>
  1. 获取 session 信息
  2. 根据 session.mode 确定应有的文件列表
     - Single: {soul_name}.md, {soul_name}_record.md
     - Conference: N × {soul_name}.md + synthesis.md
     - Debate: {soul_a}.md + {soul_b}.md + verdict.md
  3. 检查每个文件是否存在于 data/archive/.../session_id/ 下
  4. 返回 ArchiveVerification { ok, expected_files, found_files, missing_files }
```

### 导出/导入

```
export_archive() -> Result<ExportBundle>
  1. sessions = store.list_sessions(default).await
  2. for each session: 获取 messages
  3. call_records = store.query_call_records(default).await
  4. return ExportBundle { exported_at, sessions, call_records }

import_archive(bundle: &ExportBundle) -> Result<()>
  1. for each session in bundle.sessions:
      store.create_session(&session.session).await
      for msg in session.messages:
         store.append_message(&msg).await
  2. for each record in bundle.call_records:
      store.record_call(&record).await
```

## AnalyticsEngine

```rust
pub struct AnalyticsEngine {
    store: Arc<dyn Storage>,
}
```

### 召唤统计（Q1: A — SQL 聚合）

```
get_summon_stats(period_start, period_end) -> Result<SummonStats>
  1. 查询 SQLite call_records WHERE created_at BETWEEN period_start AND period_end
  2. GROUP BY soul_name → 每个魂的调用次数
  3. 按 mode 分布: GROUP BY mode
  4. 按 effectiveness 分布: GROUP BY soul_name, effectiveness
  5. 获取总魂数: registry_entry_count()
```

### 魂有效性趋势

```
get_soul_effectiveness(soul_name: &str) -> Result<EffectivenessStats>
  1. 查询 call_records WHERE soul_name = ?
  2. 聚合 effective/partial/invalid 计数
  3. 返回 EffectivenessStats
```

### 模式分布

```
get_mode_distribution() -> Result<HashMap<PossessionMode, usize>>
  1. 查询 sessions GROUP BY mode
  2. 返回各模式会话数量
```

### 未召唤魂检测（Q3: C — 组合检测）

```
detect_unsummoned_souls(threshold_days: u32) -> Result<Vec<SoulAlert>>
  1. all_souls = store 读取 registry 获取所有魂名
  2. 查询 call_records:
     - 从未有记录的魂（不在 call_records 中）→ SoulAlert { NeverSummoned }
     - MAX(created_at) < now - threshold_days → SoulAlert { UnsummonedLongDuration }
  3. 返回告警列表
```

### 低效检测

```
detect_low_effectiveness(threshold: f64) -> Result<Vec<BoundaryReview>>
  1. 对所有魂计算 effective / (effective + partial + invalid)
  2. 如果 effective_rate < threshold 且 total_calls > 5
     → BoundaryReview { recommendation: "请进行实践审查" }
  3. 按 effective_rate 升序排列
```
