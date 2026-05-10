# Business Logic Model — B1: Foundation

## Storage Trait

核心抽象：所有上层组件通过 `Storage` trait 访问数据。

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    // === Soul (File System) ===
    async fn read_soul(&self, name: &str) -> Result<SoulProfile>;
    async fn write_soul(&self, profile: &SoulProfile) -> Result<()>;
    async fn delete_soul(&self, name: &str) -> Result<()>;
    async fn list_soul_names(&self) -> Result<Vec<String>>;

    // === Registry (FS + SQLite cache) ===
    async fn read_registry(&self) -> Result<Registry>;
    async fn write_registry(&self, registry: &Registry) -> Result<()>;

    // === Session (SQLite) ===
    async fn create_session(&self, session: &Session) -> Result<()>;
    async fn update_session(&self, session: &Session) -> Result<()>;
    async fn get_session(&self, id: &str) -> Result<Session>;
    async fn list_sessions(&self, filter: SessionFilter) -> Result<Vec<SessionSummary>>;

    // === Messages (SQLite) ===
    async fn append_message(&self, msg: &Message) -> Result<()>;
    async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>>;

    // === Call Records (SQLite + YAML) ===
    async fn record_call(&self, record: &CallRecord) -> Result<()>;
    async fn query_call_records(&self, filter: CallFilter) -> Result<Vec<CallRecord>>;

    // === Archive (File System) ===
    async fn archive_soul_output(&self, session_id: &str, soul: &str, content: &str) -> Result<String>;
    async fn archive_synthesis(&self, session_id: &str, content: &str) -> Result<String>;
    async fn read_archive(&self, path: &str) -> Result<String>;
}
```

## Data Access Patterns

### Read Path: 获取魂列表（registry 摘要）
```
API Request → read_registry() → parse YAML → return Registry.souls
(tokens: ~28KB for 24 souls, no soul YAML loaded)
```

### Read Path: 获取魂详情（完整档案）
```
API Request → read_soul(name) → parse YAML frontmatter + MD body → return SoulProfile
(includes full summon_prompt, ~20KB+ per soul)
```

### Write Path: 存档魂输出（合议模式）
```
Possession complete →
  1. archive_soul_output(session_id, "列宁", output)   → data/archive/YYYY/MM/DD/{session_id}/列宁.md
  2. archive_soul_output(session_id, "费曼", output)   → data/archive/YYYY/MM/DD/{session_id}/费曼.md
  3. archive_soul_output(session_id, "毛泽东", output) → data/archive/YYYY/MM/DD/{session_id}/毛泽东.md
  4. archive_synthesis(session_id, synthesis_report)   → data/archive/YYYY/MM/DD/{session_id}/synthesis.md
  5. append_message() × 4 → SQLite
  6. record_call() × 3    → SQLite + call-records.yaml
```

### Write Path: 炼化新魂
```
SoulManager.refine() →
  1. write_soul(profile)       → data/souls/{name}.md
  2. read_registry()           → parse registry.yaml
  3. registry.souls.insert(name, entry)
  4. write_registry(registry)  → data/registry.yaml
```

## Two-Level Data Access (Token Optimization)

匹配现有万民幡的分层访问策略：

| 阶段 | 数据源 | 读取内容 |
|------|--------|---------|
| 匹配魂列表 | SQLite registry_cache | name, ismism_code, domains, grade |
| 魂详情 | FS souls/{name}.md | 完整 SoulProfile + summon_prompt |
| 会话列表 | SQLite sessions | id, title, mode, status, created_at |
| 会话详情 | SQLite messages | 全部 message，含 content |

**规则**：列表视图只加载摘要字段，详情才加载完整内容。

## Config

```rust
struct Config {
    data_dir: PathBuf,          // ./data/
    souls_dir: PathBuf,         // ./data/souls/
    archive_dir: PathBuf,       // ./data/archive/
    db_path: PathBuf,           // ./data/wanminfan.db
    registry_path: PathBuf,     // ./data/registry.yaml
    call_records_path: PathBuf, // ./data/call-records.yaml
    server_host: String,        // 127.0.0.1
    server_port: u16,           // 3001
    nextjs_port: u16,           // 3000
}
```

## SQLite Schema (Full)

```sql
-- 应用启动时自动创建（CREATE TABLE IF NOT EXISTS）

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    mode        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS messages (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    role        TEXT NOT NULL,
    soul_name   TEXT,
    content     TEXT NOT NULL,
    seq         INTEGER NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS call_records (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES sessions(id),
    soul_name       TEXT NOT NULL,
    mode            TEXT NOT NULL,
    task_summary    TEXT NOT NULL,
    effectiveness   TEXT NOT NULL,
    notes           TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, seq);
CREATE INDEX IF NOT EXISTS idx_call_records_soul ON call_records(soul_name);
CREATE INDEX IF NOT EXISTS idx_call_records_session ON call_records(session_id);
CREATE INDEX IF NOT EXISTS idx_sessions_mode ON sessions(mode);
CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at);
```
