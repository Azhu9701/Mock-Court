# Unit Test Instructions

## Current Status

本项目各 crate 在代码生成阶段未创建单元测试文件。以下是各 crate 的测试策略和待补充的测试覆盖建议。

## Run Existing Tests

```bash
cargo test
```

**预期**: 0 tests（目前无测试文件）

## Test Coverage Plan (Per Crate)

### Foundation (`rust/foundation/`)
| 测试目标 | 覆盖内容 | 优先级 |
|---------|---------|--------|
| `models.rs` | PossessionMode::from_str() 序列化/反序列化 | P0 |
| `fs_store.rs` | atomic_write, parse_soul_md, serialize_soul_md | P0 |
| `sqlite.rs` | CRUD operations, migration | P0 |
| `error.rs` | FoundationError Display/Serialize | P1 |

**测试文件**: `rust/foundation/tests/unit/`

```bash
cargo test -p foundation
```

### Registry (`rust/registry/`)
| 测试目标 | 覆盖内容 | 优先级 |
|---------|---------|--------|
| `search.rs` | fulltext_search, nearest_search, build_inverted_index | P0 |
| `ismism.rs` | compute_distribution | P1 |
| `lib.rs` | SoulRegistry CRUD 集成测试 | P0 |

**测试文件**: `rust/registry/tests/unit/`

```bash
cargo test -p registry
```

### Possession (`rust/possession/`)
| 测试目标 | 覆盖内容 | 优先级 |
|---------|---------|--------|
| `classifier.rs` | classify_entry 规则匹配 | P0 |
| `ws.rs` | WsSessionManager create/subscribe/broadcast | P0 |
| `recovery.rs` | recover_active_sessions mock | P1 |

**测试文件**: `rust/possession/tests/unit/`

```bash
cargo test -p possession
```

### Archive (`rust/archive/`)
| 测试目标 | 覆盖内容 | 优先级 |
|---------|---------|--------|
| `lib.rs` | verify_archive, expected_files | P0 |
| `analytics.rs` | compute_summon_stats, detect_* functions | P0 |

**测试文件**: `rust/archive/tests/unit/`

```bash
cargo test -p archive
```

### API (`rust/api/`)
| 测试目标 | 覆盖内容 | 优先级 |
|---------|---------|--------|
| `error.rs` | map_api_error 状态码映射 | P1 |
| `state.rs` | AppState 构造 | P1 |

**测试文件**: `rust/api/tests/unit/`

```bash
cargo test -p api
```

## Test Dependency

单元测试需要 mock `Storage` trait。建议创建一个 `TestStore` 实现：

```rust
// rust/foundation/tests/common/mod.rs
pub struct TestStore { /* 内存实现 */ }
```
