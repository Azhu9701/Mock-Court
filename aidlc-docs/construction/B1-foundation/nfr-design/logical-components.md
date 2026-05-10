# Logical Components — B1: Foundation

## Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     B1: Foundation                       │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │ ConfigLoader │  │  Migrator    │  │ HealthChecker │  │
│  │              │  │              │  │               │  │
│  │ load()       │  │ migrate()    │  │ check()       │  │
│  │ reload()     │  │ rollback()   │  │ repair()      │  │
│  └──────┬───────┘  └──────┬───────┘  └───────┬───────┘  │
│         │                 │                   │          │
│         ▼                 ▼                   ▼          │
│  ┌──────────────────────────────────────────────────┐   │
│  │                 StorageImpl                       │   │
│  │  (implements Storage trait)                       │   │
│  │                                                   │   │
│  │  ┌────────────┐  ┌──────────────┐  ┌──────────┐  │   │
│  │  │  SqliteDb  │  │  FileStore   │  │ Registry │  │   │
│  │  │            │  │              │  │ Cache    │  │   │
│  │  │ pool       │  │ souls_dir    │  │          │  │   │
│  │  │ wal_mode   │  │ archive_dir  │  │ HashMap  │  │   │
│  │  └────────────┘  └──────────────┘  └──────────┘  │   │
│  │                                                   │   │
│  │  ┌────────────┐  ┌──────────────┐                 │   │
│  │  │ FileLock   │  │ AtomicWrite  │                 │   │
│  │  │ Manager    │  │              │                 │   │
│  │  │ flock()    │  │ write_tmp()  │                 │   │
│  │  └────────────┘  └──────────────┘                 │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Component Details

### ConfigLoader
- **Purpose**: 加载和管理应用配置
- **Sources**: `config/default.yaml` → `config/local.yaml` → `WANMINFAN_*` 环境变量
- **Interface**:
```rust
struct ConfigLoader;
impl ConfigLoader {
    fn load() -> Result<Config>;
    fn reload(&mut self) -> Result<Config>;
    fn get_data_dir(&self) -> &Path;
    fn get_db_path(&self) -> &Path;
}
```

### Migrator
- **Purpose**: 数据库表创建（启动时自动执行）
- **Strategy**: `CREATE TABLE IF NOT EXISTS` — 幂等，无版本号
- **Interface**:
```rust
struct Migrator { conn: Connection }
impl Migrator {
    fn new(conn: Connection) -> Self;
    fn run(&self) -> Result<()>;       // 执行全部 DDL
    fn ensure_schema(&self) -> Result<()>;  // 检查表是否存在
}
```

### SqliteDb
- **Purpose**: SQLite 连接管理
- **Features**: WAL 模式、连接池（1 写 + N 读）、自动外键
- **Interface**:
```rust
struct SqliteDb { pool: r2d2::Pool<ConnectionManager> }
impl SqliteDb {
    fn open(path: &Path) -> Result<Self>;
    fn write<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T>;
    fn read<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T>;
    fn vacuum(&self) -> Result<()>;
}
```

### FileStore
- **Purpose**: 文件系统操作抽象
- **Directories**: `data/souls/`, `data/archive/{YYYY}/{MM}/{DD}/{session_id}/`
- **Interface**:
```rust
struct FileStore { data_dir: PathBuf }
impl FileStore {
    async fn read_soul(&self, name: &str) -> Result<SoulProfile>;
    async fn write_soul(&self, profile: &SoulProfile) -> Result<()>;
    async fn delete_soul(&self, name: &str) -> Result<()>;
    async fn list_souls(&self) -> Result<Vec<String>>;
    async fn archive_output(&self, session_id: &str, soul: &str, content: &str) -> Result<String>;
    async fn atomic_write(&self, path: &Path, content: &str) -> Result<()>;
}
```

### RegistryCache
- **Purpose**: Registry 内存缓存，减少 YAML 解析
- **Strategy**: 启动时加载，变更时同步更新 YAML + 缓存
- **Interface**:
```rust
struct RegistryCache { cache: RwLock<HashMap<String, RegistryEntry>> }
impl RegistryCache {
    fn load_from_yaml(path: &Path) -> Result<Self>;
    fn get(&self, name: &str) -> Option<RegistryEntry>;
    fn list(&self, filter: Option<IsmismFilter>) -> Vec<RegistryEntry>;
    fn upsert(&self, name: &str, entry: RegistryEntry) -> Result<()>;
    fn remove(&self, name: &str) -> Result<()>;
    fn sync_to_yaml(&self, path: &Path) -> Result<()>;
}
```

### FileLockManager
- **Purpose**: 文件级并发控制
- **Features**: flock/exclusive lock
- **Interface**:
```rust
struct FileLockManager;
impl FileLockManager {
    fn lock_soul(name: &str) -> Result<FileLock>;
    fn lock_registry() -> Result<FileLock>;
}
```

### HealthChecker
- **Purpose**: 数据完整性检查
- **Checks**: SQLite vs YAML 记录数比对、魂文件数与 registry 条目数比对
- **Interface**:
```rust
struct HealthChecker { storage: Arc<StorageImpl> }
impl HealthChecker {
    async fn check(&self) -> HealthStatus;
    async fn repair(&self) -> Result<()>;
}
```

## Component Integration

```
App Startup:
  1. ConfigLoader::load()
  2. SqliteDb::open(db_path)
  3. Migrator::run()
  4. FileStore::new(data_dir)
  5. RegistryCache::load_from_yaml(registry_path)
  6. → StorageImpl ready, pass to axum state

App Shutdown:
  1. RegistryCache::sync_to_yaml()
  2. SqliteDb::vacuum()
  3. Drop connection pool
```
