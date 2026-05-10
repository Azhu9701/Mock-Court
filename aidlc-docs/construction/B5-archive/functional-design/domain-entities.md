# Domain Entities — B5: Archive & Analytics

## New Types

### SessionDetail — 会话详情

```rust
#[derive(Debug, Clone)]
pub struct SessionDetail {
    pub session: Session,
    pub messages: Vec<Message>,
}
```

### ArchiveVerification — 存档校验结果

```rust
#[derive(Debug, Clone)]
pub struct ArchiveVerification {
    pub session_id: String,
    pub ok: bool,
    pub expected_files: usize,
    pub found_files: usize,
    pub missing_files: Vec<String>,
}
```

### SummonStats — 召唤统计

```rust
#[derive(Debug, Clone)]
pub struct SummonStats {
    pub total_calls: usize,
    pub unique_souls_called: usize,
    pub total_souls_available: usize,
    pub by_mode: HashMap<PossessionMode, usize>,
    pub by_soul: Vec<SoulCallStats>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SoulCallStats {
    pub soul_name: String,
    pub call_count: usize,
    pub effective_count: usize,
    pub partial_count: usize,
    pub invalid_count: usize,
}
```

### SoulAlert — 未召唤/低效告警

```rust
#[derive(Debug, Clone)]
pub struct SoulAlert {
    pub soul_name: String,
    pub alert_type: AlertType,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    NeverSummoned,          // 从未被召唤
    UnsummonedLongDuration, // 长期未召唤
    LowEffectiveness,       // 有效性过低
}
```

### BoundaryReview — 边界审查

```rust
#[derive(Debug, Clone)]
pub struct BoundaryReview {
    pub soul_name: String,
    pub effective_rate: f64,
    pub total_calls: usize,
    pub threshold: f64,
    pub recommendation: String,
}
```

### ExportBundle — 导出数据

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBundle {
    pub exported_at: DateTime<Utc>,
    pub sessions: Vec<SessionDetail>,
    pub call_records: Vec<CallRecord>,
}
```

## Extension to Existing Types

B5 不需要修改 foundation models，所有新类型在 `archive` crate 中定义。

## Relations

```
ArchiveSystem
├── archive_soul_output()  → 写入 data/archive/YYYY/MM/DD/{session_id}/{soul}.md
├── archive_synthesis()    → 写入 data/archive/.../{session_id}/synthesis.md
├── archive_debate()       → 写入 debate_A_vs_B.md
├── record_call()          → SQLite + YAML dual-write
├── verify_archive()       → 检查预期文件存在性
└── list_sessions()        → 委托给 SqliteDb.list_sessions()

AnalyticsEngine
├── get_summon_stats()     → SQL 聚合查询
├── get_soul_effectiveness() → call_records 聚合
├── get_mode_distribution()  → sessions 聚合
├── detect_unsummoned_souls() → call_records + registry 组合
└── detect_low_effectiveness() → effective_rate < threshold
```
