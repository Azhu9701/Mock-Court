# NFR Design Patterns — B6: API Layer

## Pattern 1: Middleware Stack（中间件栈）

**问题**: 请求需要依次经过 CORS、Tracing、Timeout、Error Recovery，顺序和组合需明确。

**方案**: tower `ServiceBuilder` 链式组合。

```
ServiceBuilder::new()
    .layer(CorsLayer::permissive())     // Q3: A — 全开放
    .layer(TraceLayer::new_for_http())  // 自动 method + path + status + latency
    .layer(TimeoutLayer::new(30s))      // Q1: A — 30s 超时
    .layer(HandleErrorLayer::new(panic_recovery))  // catch panic → 500
    .into_inner()
```

**顺序理由**: CORS 在最外层（处理 OPTIONS 预检不经过后续层），Tracing 记录完整请求（含 timeout 触发的错误），Timeout 保护内层 handler，Panic Recovery 是最后一道防线。

## Pattern 2: Nested Router（嵌套路由 — Q1: A）

**问题**: 12+ 端点需要组织成可维护的结构。

**方案**: axum `Router::nest()` 按资源分组。

```
Router::new()
    .route("/health", get(health_check))
    .nest("/souls", souls_router())
    .nest("/possess", possess_router())
    .nest("/sessions", sessions_router())
    .nest("/analytics", analytics_router())
    .nest("/archive", archive_router())
    .layer(middleware_stack)
    .with_state(app_state)

// 合并到主 router
let app = Router::new()
    .nest("/api/v1", api_router)        // REST 前缀
    .route("/ws/possess/:session_id/:channel", get(ws_handler));  // WS 独立
```

## Pattern 3: Error Mapping（错误映射 — Q1: A）

**问题**: `FoundationError` 需转换为 HTTP 响应。

**方案**: 在 handler 中统一 `map_err` 函数。

```rust
fn map_api_error(e: FoundationError) -> (StatusCode, Json<ApiError>) {
    let status = match &e {
        FoundationError::SoulNotFound(_) => StatusCode::NOT_FOUND,
        FoundationError::Validation(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ApiError { error: e.to_string() };
    (status, Json(body))
}

// handler 中使用
async fn get_soul(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<SoulProfile>, (StatusCode, Json<ApiError>)> {
    state.registry.get_soul(&name)
        .map(Json)
        .map_err(map_api_error)
}
```

**粗粒度映射**: SoulNotFound → 404, Validation → 400, 其他 → 500。

## Pattern 4: Graceful Shutdown（优雅关闭）

**问题**: SIGTERM 时需有序关闭 HTTP server + PossessionEngine。

**方案**: `tokio::signal` + `axum::serve` graceful shutdown。

```
let (tx, rx) = tokio::sync::oneshot::channel();
let listener = tokio::net::TcpListener::bind("127.0.0.1:3096").await?;

tokio::spawn(async move {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("Shutdown signal received");
    engine.shutdown_flag.store(true, Ordering::SeqCst);
    // axum graceful 会自动等待活跃连接完成（配合 TimeoutLayer）
    tx.send(()).ok();
});

axum::serve(listener, app)
    .with_graceful_shutdown(async { rx.await.ok(); })
    .await?;
```

**关闭顺序**:
1. 设置 shutdown_flag → 拒绝新 possess 请求
2. axum graceful 等待活跃连接完成（TimeoutLayer 保证最长 30s）
3. 进程退出

## Pattern 5: WS Stream Relaying（WebSocket 流转发）

**问题**: PossessionEngine 通过 `WsSessionManager` 管理 mpsc channel，B6 只需将 WebSocket 连接桥接到这个 channel。

**方案**: spawn per-connection task 桥接 `WsSessionManager` 的 mpsc receiver → WS sink。

```rust
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path((session_id, channel)): Path<(String, String)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (ws_tx, mut ws_rx) = socket.split();

        // 创建 mpsc channel 订阅 WsSessionManager
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // 注册到 WsSessionManager
        state.engine.ws_manager().subscribe(&session_id, &channel, tx);

        // 转发 task: mpsc → WS
        let send_task = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if ws_tx.send(Message::Text(serde_json::to_string(&event).unwrap())).await.is_err() {
                    break;  // WS 断开
                }
            }
        });

        // 接收 task: 忽略客户端消息
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(_)) = ws_rx.next().await {
                // 客户端不发消息，忽略
            }
        });

        // 等待任一 task 结束
        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
        }

        // 取消订阅
        state.engine.ws_manager().unsubscribe(&session_id, &channel);
    })
}
```

## Pattern 6: Request Logging（请求日志 — Q5: A）

**问题**: 需要记录所有 HTTP 请求的 method + path + status + latency。

**方案**: `tower_http::trace::TraceLayer` 开箱即用，配合 `tracing-subscriber` fmt layer。

```rust
tracing_subscriber::fmt()
    .with_env_filter("info")  // Q5: A
    .with_target(false)
    .init();

// TraceLayer 自动记录（无需手写 on_request/on_response callback）
let trace_layer = TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
    .on_response(DefaultOnResponse::new().level(Level::INFO));
```
