# Logical Components — B5: Archive & Analytics

## Component Architecture

```
ArchiveSystem (src/lib.rs)
├── SessionManager
│   ├── list_sessions()    → delegate to store
│   └── get_session_detail()  → SessionDetail
├── Archiver
│   ├── archive_soul()     → atomic_write
│   ├── archive_synthesis()
│   ├── archive_debate()
│   └── verify_archive()   → expected_files check
├── CallRecordManager (src/call_records.rs)
│   ├── record_call()      → delegate to store
│   └── query()           → delegate to store
├── Exporter (src/archive.rs)
│   ├── export_archive()   → spawn async task → task_id
│   └── export_status()   → status lookup
└── AnalyticsEngine (src/analytics.rs)
    ├── StatsCache         → RwLock TTL cache
    ├── get_summon_stats() → SQL + cache
    ├── get_soul_effectiveness()
    ├── detect_unsummoned_souls()
    └── detect_low_effectiveness()
```

## Component: ArchiveSystem (`src/lib.rs`)

```rust
pub struct ArchiveSystem {
    store: Arc<dyn Storage>,
    export_statuses: RwLock<HashMap<String, ExportStatus>>,
    summon_stats_cache: RwLock<Option<(SummonStats, Instant)>>,
    stats_ttl: Duration,
}

impl ArchiveSystem {
    // 生命周期
    pub fn new(store: Arc<dyn Storage>) -> Self

    // 存档
    pub async fn archive_soul_output(&self, session_id, soul, content) -> Result<String>
    pub async fn archive_synthesis(&self, session_id, content) -> Result<String>
    pub async fn archive_debate(&self, session_id, soul_a, soul_b, out_a, out_b) -> Result<(String, String)>

    // Call Record
    pub async fn record_call(&self, record: &CallRecord) -> Result<()>
    pub async fn query_call_records(&self, filter: &CallFilter) -> Result<Vec<CallRecord>>

    // 会话
    pub async fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>>
    pub async fn get_session_detail(&self, id: &str) -> Result<SessionDetail>

    // 完整性
    pub fn verify_archive(&self, session_id: &str) -> Result<ArchiveVerification>

    // 导出
    pub fn export_archive(&self) -> Result<String>  // returns task_id
    pub fn export_status(&self, task_id: &str) -> Option<ExportStatus>

    // 统计
    pub fn get_summon_stats(&self, period: Period) -> Result<SummonStats>
    pub fn get_soul_effectiveness(&self, soul: &str) -> Result<EffectivenessTrend>
    pub fn detect_unsummoned_souls(&self, threshold_days: u32) -> Result<Vec<SoulAlert>>
    pub fn detect_low_effectiveness(&self, threshold: f64) -> Result<Vec<BoundaryReview>>
}
```

## File Structure

```
rust/archive/
├── Cargo.toml
└── src/
    ├── lib.rs             # ArchiveSystem struct
    ├── call_records.rs    # CallRecord CRUD wrapper
    ├── archive.rs         # 存档 + 校验 + 导出
    └── analytics.rs       # StatsCache + 统计 + 检测
```
