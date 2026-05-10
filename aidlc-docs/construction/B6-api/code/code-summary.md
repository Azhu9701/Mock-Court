# Code Summary — B6: API Layer

## Created Files

| 文件 | 行数 | 说明 |
|------|------|------|
| `rust/api/Cargo.toml` | 22 | 依赖 axum 0.7, tower-http 0.6, futures-util |
| `rust/api/src/main.rs` | 70 | main() 入口 + build_router() + graceful shutdown |
| `rust/api/src/state.rs` | 13 | AppState (registry + engine + archive) |
| `rust/api/src/error.rs` | 20 | ApiError + map_api_error |
| `rust/api/src/middleware.rs` | 38 | apply_middleware + panic_recovery |
| `rust/api/src/ws.rs` | 60 | ws_handler + handle_ws (mpsc relay) |
| `rust/api/src/store.rs` | 105 | AppStore impl Storage (FileStore + SqliteDb) |
| `rust/api/src/routes/mod.rs` | 25 | api_router() 路由组装 + health_check |
| `rust/api/src/routes/souls.rs` | 145 | 7 handlers (CRUD + search + ismism) |
| `rust/api/src/routes/possess.rs` | 82 | 2 handlers (start_possession + status) |
| `rust/api/src/routes/sessions.rs` | 67 | 2 handlers (list + get_detail) |
| `rust/api/src/routes/analytics.rs` | 87 | 5 handlers (stats / effectiveness / distribution / alerts) |
| `rust/api/src/routes/archive.rs` | 52 | 3 handlers (verify / export / export_status) |

**总计**: 13 个源文件, ~786 行

## Modified Files

| 文件 | 变更 |
|------|------|
| `Cargo.toml` | 启用 `rust/api` workspace member |
| `rust/possession/src/ws.rs` | 添加 `subscribe()`/`unsubscribe()`/`has_session()` |
| `rust/possession/src/lib.rs` | 添加 `set_shutdown()` 公共方法 |
| `rust/archive/src/lib.rs` | 为 8 个类型添加 Serialize derive |

## API Endpoints (20 REST + 1 WS)

```
REST (20 endpoints):
  GET    /api/v1/health
  GET    /api/v1/souls
  GET    /api/v1/souls/search?q=...
  GET    /api/v1/souls/ismism/distribution
  GET    /api/v1/souls/:name
  POST   /api/v1/souls
  PUT    /api/v1/souls/:name
  DELETE /api/v1/souls/:name
  POST   /api/v1/possess
  GET    /api/v1/possess/:session_id/status
  GET    /api/v1/sessions
  GET    /api/v1/sessions/:id
  GET    /api/v1/analytics/summon-stats
  GET    /api/v1/analytics/soul-effectiveness/:name
  GET    /api/v1/analytics/mode-distribution
  GET    /api/v1/analytics/unsummoned
  GET    /api/v1/analytics/low-effectiveness
  GET    /api/v1/archive/verify/:session_id
  POST   /api/v1/archive/export
  GET    /api/v1/archive/export/:task_id

WebSocket (1 endpoint):
  WS     /ws/possess/:session_id/:channel
```

## Verification

```
cargo check --offline: 0 errors, 0 warnings
```
