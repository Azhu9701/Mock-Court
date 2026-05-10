# Code Generation Plan — B2: Soul Registry

## Unit Context
- **Unit**: B2 Soul Registry
- **Crate**: `rust/registry/`
- **Dependencies**: `foundation`
- **Stories covered**: FR1.1-1.4, FR3.2-3.5

## Plan Steps

### Foundation 类型扩展
- [x] Step 1: 更新 `rust/foundation/src/models.rs` — 添加 `SoulListEntry`, `SoulMatch`, `IsmismStats`, `IsmismSearch`, `IsmismCode::distance()`, 扩展 `IsmismFilter`

### Crate 初始化
- [x] Step 2: 创建 `rust/registry/Cargo.toml` — 依赖 foundation + workspace deps
- [x] Step 3: 更新 workspace `Cargo.toml` — 添加 `rust/registry` 成员

### 核心代码
- [x] Step 4: 创建 `rust/registry/src/ismism.rs` — IsmismUtils: parse, distance, distribution 计算
- [x] Step 5: 创建 `rust/registry/src/search.rs` — Tokenizer + SearchEngine (fulltext_search, nearest_search, relevance)
- [x] Step 6: 创建 `rust/registry/src/lib.rs` — SoulRegistry struct + 公共 API (new, list, get, search, CRUD, reload, distribution)

### 文档
- [x] Step 7: 创建 `aidlc-docs/construction/B2-soul-registry/code/code-summary.md`

### 验证
- [x] Step 8: `cargo check` 验证编译通过

## File List

| File | Purpose |
|------|---------|
| `rust/foundation/src/models.rs` | 新增类型（修改已有文件） |
| `rust/registry/Cargo.toml` | Crate 依赖声明 |
| `Cargo.toml` | Workspace 成员更新 |
| `rust/registry/src/lib.rs` | SoulRegistry 入口 |
| `rust/registry/src/search.rs` | 搜索 + 分词 |
| `rust/registry/src/ismism.rs` | ismism 工具函数 |
| `aidlc-docs/construction/B2-soul-registry/code/code-summary.md` | 代码总结 |
