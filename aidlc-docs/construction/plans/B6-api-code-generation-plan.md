# Code Generation Plan — B6: API Layer

## Unit Context

- **Crate**: `api` (binary, depends on `foundation`, `registry`, `possession`, `archive`)
- **Stories**: HTTP REST API + WebSocket 流式推送
- **Crate type**: binary (main.rs entry point)

## Plan Steps

### Project Setup
- [x] Step 1: 创建 `rust/api/Cargo.toml` + 更新 workspace `Cargo.toml`
- [x] Step 2: 创建 `rust/api/src/state.rs` — AppState
- [x] Step 3: 创建 `rust/api/src/error.rs` — ApiError + map_api_error

### Infrastructure
- [x] Step 4: 创建 `rust/api/src/middleware.rs` — middleware_stack()
- [x] Step 5: 创建 `rust/api/src/ws.rs` — ws_handler (WebSocket upgrade + mpsc relay)

### Route Handlers
- [x] Step 6: 创建 `rust/api/src/routes/mod.rs` — 路由模块声明 + api_router()
- [x] Step 7: 创建 `rust/api/src/routes/souls.rs` — 7 handlers (list/search/get/create/update/delete/ismism)
- [x] Step 8: 创建 `rust/api/src/routes/possess.rs` — 2 handlers (start_possession/status)
- [x] Step 9: 创建 `rust/api/src/routes/sessions.rs` — 2 handlers (list/get_detail)
- [x] Step 10: 创建 `rust/api/src/routes/analytics.rs` — 5 handlers (summon_stats/effectiveness/distribution/unsummoned/low_effectiveness)
- [x] Step 11: 创建 `rust/api/src/routes/archive.rs` — 4 handlers (verify/export/export_status)

### Entry Point
- [x] Step 12: 创建 `rust/api/src/main.rs` — main() + build_router() + graceful_shutdown
- [x] Step 13: 创建 `rust/api/src/store.rs` — AppStore (FileStore + SqliteDb implementing Storage trait)

### Verification
- [x] Step 14: `cargo check` 验证 (0 errors, 0 warnings)

## Additional Changes
- **Modified**: `rust/possession/src/ws.rs` — 添加 `subscribe`/`unsubscribe`/`has_session` 方法
- **Modified**: `rust/possession/src/lib.rs` — 添加 `set_shutdown()` 公共方法
- **Modified**: `rust/archive/src/lib.rs` — 为 7 个类型添加 `Serialize` derive (SummonStats, SoulAlert, BoundaryReview, EffectivenessTrend, SoulCallStats, ArchiveVerification, AlertType, ExportStatus)
