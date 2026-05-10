# NFR Design Patterns — B1: Foundation

## Resilience Patterns

### Pattern 1: Atomic File Write
**问题**: 文件写入中断导致损坏  
**方案**: 先写 `.tmp` 文件 → fsync → atomic rename
```rust
async fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, content).await?;
    tokio::fs::sync_all(&tmp).await?;     // fsync
    tokio::fs::rename(&tmp, path).await?; // atomic on same FS
    Ok(())
}
```

### Pattern 2: Dual-Write (SQLite + YAML)
**问题**: call_records 需要同时持久化到 SQLite 和 YAML  
**方案**: 两阶段写 —— SQLite first，成功后再写 YAML。YAML 失败时 SQLite 记录标记为 `yaml_unsynced`，下次同步修复。
```rust
async fn record_call(&self, record: &CallRecord) -> Result<()> {
    let tx = self.db.begin()?;
    insert_call_record(&tx, record)?;     // Phase 1: SQLite
    append_to_yaml(record)?;              // Phase 2: YAML (fail → mark unsynced)
    tx.commit()?;
    Ok(())
}
```

### Pattern 3: Graceful Degradation
**问题**: 单个魂档案损坏不应阻止整个应用启动  
**方案**: 加载时跳过损坏的魂，记录错误日志，返回部分结果。
```rust
async fn load_registry(&self) -> Result<Registry> {
    let mut errors = vec![];
    let mut souls = HashMap::new();
    for name in list_soul_names()? {
        match read_soul(&name).await {
            Ok(profile) => { souls.insert(name, entry_from(profile)); }
            Err(e) => { errors.push((name, e)); }
        }
    }
    log::warn!("{} souls failed to load", errors.len());
    Ok(Registry { souls, load_errors: errors })
}
```

## Performance Patterns

### Pattern 4: SQLite WAL Mode
**问题**: 读阻塞写、写阻塞读  
**方案**: 启用 WAL (Write-Ahead Logging) 模式，读写并发。
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
```

### Pattern 5: Connection Pool
**问题**: 多 handler 共享 SQLite 连接  
**方案**: 单个写连接（互斥）+ 多个读连接（WAL 模式允许）。rusqlite 使用 `r2d2` 连接池。
```rust
struct DbPool {
    writer: Mutex<Connection>,     // 写连接（串行）
    readers: r2d2::Pool<ConnectionManager>,  // 读连接池
}
```

### Pattern 6: Lazy Registry Load
**问题**: 每次列表查询都解析 YAML 浪费 CPU  
**方案**: 启动时加载一次 registry，变更时写入 SQLite 缓存 + 文件。列表查询直接读 SQLite registry_cache。
```
启动: read_registry() → parse YAML → populate SQLite registry_cache
查询: SELECT FROM registry_cache（不读 YAML）
变更: write YAML → UPDATE registry_cache（双写同步）
```

## Data Integrity Patterns

### Pattern 7: Cross-Verify on Health Check
**问题**: SQLite 和 YAML 可能漂移  
**方案**: 健康检查时比对记录数。不一致 → 记录告警，自动从 YAML 修复 SQLite。
```rust
async fn health_check(&self) -> HealthStatus {
    let yaml_count = count_yaml_records()?;
    let sqlite_count = count_sqlite_records()?;
    if yaml_count != sqlite_count {
        repair_sqlite_from_yaml()?;
        return HealthStatus::Repaired;
    }
    HealthStatus::Ok
}
```

## Configuration Patterns

### Pattern 8: Layered Config
**问题**: 不同环境需要不同配置  
**方案**: Config crate 分层加载。
```rust
fn load_config() -> Config {
    Config::builder()
        .add_source(File::from("config/default.yaml"))       // 默认
        .add_source(File::from("config/local.yaml").required(false))  // 本地
        .add_source(Environment::with_prefix("WANMINFAN"))    // 环境变量
        .build()
        .unwrap()
}
```
