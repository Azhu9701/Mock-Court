# Code Generation Plan — B1: Foundation

## Unit Context
- **Unit**: B1 Foundation
- **Crate**: `rust/foundation/`
- **Dependencies**: None (first unit)
- **Stories covered**: FR1.4
- **Database entities**: sessions, messages, call_records

## Plan Steps

### Project Structure Setup
- [ ] Step 1: Create root Cargo.toml (workspace)
- [ ] Step 2: Create rust/foundation/Cargo.toml
- [ ] Step 3: Create config/default.yaml

### Data Models
- [ ] Step 4: Create rust/foundation/src/lib.rs (crate root, re-exports)
- [ ] Step 5: Create rust/foundation/src/models.rs (Soul, Session, Message, CallRecord, Registry, Ismism, Effectiveness, Config)

### Storage Trait
- [ ] Step 6: Create rust/foundation/src/storage.rs (Storage trait + async methods)

### SQLite Implementation
- [ ] Step 7: Create rust/foundation/src/sqlite.rs (SqliteDb: open, WAL, pool, migrations)

### File Store Implementation
- [ ] Step 8: Create rust/foundation/src/fs_store.rs (FileStore: read/write soul, archive, atomic write, YAML)

### Config Loader
- [ ] Step 9: Create rust/foundation/src/config.rs (ConfigLoader: layered YAML + env)

### Error & Health
- [ ] Step 10: Create rust/foundation/src/error.rs (FoundationError enum)
- [ ] Step 11: Create rust/foundation/src/health.rs (HealthChecker)

### Documentation
- [ ] Step 12: Create aidlc-docs/construction/B1-foundation/code/code-summary.md

## File List

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace root |
| `rust/foundation/Cargo.toml` | Crate deps |
| `rust/foundation/src/lib.rs` | Crate root |
| `rust/foundation/src/models.rs` | All data types |
| `rust/foundation/src/storage.rs` | Storage trait |
| `rust/foundation/src/sqlite.rs` | SQLite impl |
| `rust/foundation/src/fs_store.rs` | File store impl |
| `rust/foundation/src/config.rs` | Config loader |
| `rust/foundation/src/error.rs` | Error types |
| `rust/foundation/src/health.rs` | Health checker |
| `config/default.yaml` | Default config |
