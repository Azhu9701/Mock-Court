# Code Summary — B1: Foundation

## Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `Cargo.toml` | 18 | Workspace root + shared deps |
| `rust/foundation/Cargo.toml` | 15 | Crate dependencies |
| `rust/foundation/src/lib.rs` | 12 | Crate root, re-exports |
| `rust/foundation/src/models.rs` | 215 | All domain types |
| `rust/foundation/src/error.rs` | 27 | Error type + Result alias |
| `rust/foundation/src/config.rs` | 36 | Layered config loader |
| `rust/foundation/src/storage.rs` | 42 | Storage trait (async) |
| `rust/foundation/src/sqlite.rs` | 235 | SQLite impl + migrations |
| `rust/foundation/src/fs_store.rs` | 148 | File store + YAML+MDFormat |
| `rust/foundation/src/health.rs` | 25 | Health checker |
| `config/default.yaml` | 8 | Default config |

## Data Models

- `SoulProfile` / `SoulFrontmatter` — YAML frontmatter + Markdown body
- `Session` / `Message` — 扁平消息模型
- `CallRecord` — 枚举有效性 + 文本判据
- `Registry` / `RegistryEntry` — in-memory cached
- `IsmismFilter` / `IsmismCode` — 四维坐标查询

## Storage Trait

- `Storage` trait: 16 async methods covering souls, sessions, messages, call-records, archive, health
- `HealthStatus`: Cross-verify SQLite vs YAML consistency

## Key Patterns

- Atomic write (tmp → rename)
- SQLite WAL mode + foreign keys
- Layered config (YAML default → YAML local → env)
- Soul MD format: YAML frontmatter + Markdown body
- Registry in-memory cache (RwLock<HashMap>)
