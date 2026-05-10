# Logical Components — B6: API Layer

## Component Architecture

```
main.rs
├── AppState (构造 + 注入)
├── build_router() → axum::Router
│   ├── souls_router() → 6 endpoints
│   ├── possess_router() → 2 endpoints
│   ├── sessions_router() → 2 endpoints
│   ├── analytics_router() → 5 endpoints
│   ├── archive_router() → 4 endpoints
│   └── ws_handler() → WebSocket upgrade
├── middleware_stack() → ServiceBuilder
├── error_mapping(map_api_error) → (StatusCode, Json<ApiError>)
└── graceful_shutdown(engine, signal, listener)
```

## Component: AppState

```rust
pub struct AppState {
    pub registry: Arc<SoulRegistry>,
    pub engine: Arc<PossessionEngine>,
    pub archive: Arc<ArchiveSystem>,
    pub analytics: Arc<AnalyticsEngine>,
}
```

**生命周期**: 在 `main()` 中构造所有依赖后一次性创建，通过 `Arc` 共享给 axum `State`。

## Component: Middleware Stack

```
tower::ServiceBuilder
├── CorsLayer::permissive()       // allow all origins
├── TraceLayer::new_for_http()    // method + path + status + latency
├── TimeoutLayer::new(Duration::from_secs(30))
└── HandleErrorLayer::new(panic_recovery)
```

**panic_recovery**:

```rust
fn panic_recovery(err: BoxError) -> (StatusCode, Json<ApiError>) {
    tracing::error!("Panic recovered: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: "Internal server error".into() }),
    )
}
```

## Component: Router

### Top-level

```rust
Router::new()
    .nest("/api/v1", api_router())                                    // Q1: A
    .route("/ws/possess/:session_id/:channel", get(ws_handler))       // Q2: C
    .with_state(state)                                                // Q2: A
```

### Soul Router (`souls_router()`)

```
GET    /                           → list_souls
GET    /search                     → search_souls
GET    /ismism/distribution        → ismism_distribution
GET    /:name                      → get_soul
POST   /                           → create_soul
PUT    /:name                      → update_soul
DELETE /:name                      → delete_soul
```

### Possess Router (`possess_router()`)

```
POST   /                           → start_possession
GET    /:session_id/status         → possession_status
```

### Sessions Router (`sessions_router()`)

```
GET    /                           → list_sessions
GET    /:id                        → get_session_detail
```

### Analytics Router (`analytics_router()`)

```
GET    /summon-stats               → summon_stats
GET    /soul-effectiveness/:name   → soul_effectiveness
GET    /mode-distribution          → mode_distribution
GET    /unsummoned                 → unsummoned_souls
GET    /low-effectiveness          → low_effectiveness
```

### Archive Router (`archive_router()`)

```
GET    /verify/:session_id         → verify_archive
POST   /export                     → export_archive
GET    /export/:task_id            → export_status
POST   /import                     → import_archive
```

## Component: Error Handler

```rust
fn map_api_error(e: FoundationError) -> (StatusCode, Json<ApiError>) {
    let status = match &e {
        FoundationError::SoulNotFound(_) => StatusCode::NOT_FOUND,
        FoundationError::Validation(_) => StatusCode::BAD_REQUEST,
        _ => {
            tracing::error!("Unhandled error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (status, Json(ApiError { error: e.to_string() }))
}
```

## File Structure

```
rust/api/
├── Cargo.toml
├── src/
│   ├── main.rs          # main() + build_router() + graceful_shutdown
│   ├── state.rs         # AppState
│   ├── error.rs         # map_api_error + ApiError
│   ├── middleware.rs    # middleware_stack() + panic_recovery
│   ├── routes/
│   │   ├── mod.rs       # 路由模块声明
│   │   ├── souls.rs     # 7 handlers
│   │   ├── possess.rs   # 2 handlers
│   │   ├── sessions.rs  # 2 handlers
│   │   ├── analytics.rs # 5 handlers
│   │   └── archive.rs   # 4 handlers
│   └── ws.rs            # ws_handler + stream relay
```

## Component Dependencies

```
api (binary crate)
├── foundation  (models + Storage trait)
├── registry    (SoulRegistry)
├── possession  (PossessionEngine, WsSessionManager, WsEvent)
├── archive     (ArchiveSystem + AnalyticsEngine)
├── axum        (HTTP framework)
├── tower       (ServiceBuilder)
├── tower-http  (cors, trace, timeout, handle-error)
├── tokio       (runtime, signal)
├── serde_json  (JSON ser)
└── tracing-subscriber (log output)
```

## Key Interfaces

| Component | Method | Description |
|-----------|--------|-------------|
| `build_router()` | `fn(Arc<AppState>) -> Router` | 构造完整路由树 |
| `middleware_stack()` | `fn() -> ServiceBuilder` | 构造中间件栈 |
| `map_api_error()` | `fn(FoundationError) -> (StatusCode, Json<ApiError>)` | 错误映射 |
| `panic_recovery()` | `fn(BoxError) -> (StatusCode, Json<ApiError>)` | panic 恢复 |
| `ws_handler()` | `async fn(WebSocketUpgrade, State, Path)` | WebSocket 升级 |
| `health_check()` | `async fn() -> Json<Value>` | 健康检查 |
