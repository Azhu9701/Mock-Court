# Business Logic Model — B6: API Layer

## AppState — 全局共享状态

```rust
pub struct AppState {
    pub registry: Arc<SoulRegistry>,
    pub engine: Arc<PossessionEngine>,
    pub archive: Arc<ArchiveSystem>,
    pub analytics: Arc<AnalyticsEngine>,
}
```

**组装**: 在 `main.rs` 中创建所有服务后注入 `AppState`，通过 axum `Extension<AppState>` 传递到各 handler。

## Router 结构 (Q1: A — `/api/v1/...`)

```
axum::Router
├── /api/v1
│   ├── /health                           GET  → health_check
│   ├── /souls                            GET  → list_souls
│   │   ├── /search                       GET  → search_souls
│   │   ├── /ismism/distribution          GET  → ismism_distribution
│   │   ├── /:name                        GET  → get_soul
│   │   │   ├── PUT                       → update_soul
│   │   │   └── DELETE                    → delete_soul
│   │   └── /                             POST → create_soul
│   ├── /possess                          POST → start_possession
│   │   └── /:session_id/status           GET  → possession_status
│   ├── /sessions                         GET  → list_sessions
│   │   └── /:id                          GET  → get_session_detail
│   ├── /analytics
│   │   ├── /summon-stats                 GET  → summon_stats
│   │   ├── /soul-effectiveness/:name     GET  → soul_effectiveness
│   │   ├── /mode-distribution            GET  → mode_distribution
│   │   ├── /unsummoned                   GET  → unsummoned_souls
│   │   └── /low-effectiveness            GET  → low_effectiveness
│   └── /archive
│       ├── /verify/:session_id           GET  → verify_archive
│       ├── /export                       POST → export_archive
│       ├── /export/:task_id              GET  → export_status
│       └── /import                       POST → import_archive
└── /ws
    └── /possess/:session_id/:channel     GET  → ws_handler (WebSocket upgrade)
```

## 中间件链（按顺序）

```
Request
  → CORS (allow all origins — Q3: A)
  → Tracing (request_id + latency)
  → Error Recovery (catch panics → 500)
  → Extension<AppState> injection
  → Router dispatch
```

## 各 Route 处理流程

### Health Check

```
GET /api/v1/health
  1. 返回 {"status": "ok"}
```

### List Souls

```
GET /api/v1/souls?grade=S&field=哲学
  1. 解析 query → IsmismFilter
  2. state.registry.list_souls(&filter)
  3. 返回 JSON Vec<SoulListEntry>
```

### Search Souls

```
GET /api/v1/souls/search?q=马克思
  1. 解析 query string
  2. state.registry.search_souls(query)
  3. 返回 JSON Vec<SoulMatch>
```

### Get Soul

```
GET /api/v1/souls/:name
  1. state.registry.get_soul(name)
  2. SoulNotFound → 404
  3. 返回 JSON SoulProfile
```

### Create Soul

```
POST /api/v1/souls  { body: CreateSoulRequest }
  1. 从 request body 构造 SoulProfile
  2. state.registry.create_soul(profile).await
  3. 返回 201 + SoulListEntry
```

### Update Soul

```
PUT /api/v1/souls/:name  { body: UpdateSoulRequest }
  1. state.registry.get_soul(name) → 现有 profile
  2. 合并 update fields → 新 profile
  3. state.registry.update_soul(profile).await
  4. 返回 SoulListEntry
```

### Delete Soul

```
DELETE /api/v1/souls/:name
  1. state.registry.delete_soul(name).await
  2. 返回 204 No Content
```

### Start Possession

```
POST /api/v1/possess  { body: StartPossessionRequest }
  1. 解析 JSON → PossessionInput { mode, task, souls, topic }
  2. 创建 mpsc::unbounded_channel() → (tx, rx)
  3. state.engine.start_possession(input, tx).await → session_id
  4. spawn task: forward rx to engine's WsSessionManager (this creates the WS session)
  5. 返回 { session_id, mode, ws_url }
```

### Possession Status

```
GET /api/v1/possess/:session_id/status
  1. state.engine.ws_manager().get_session(session_id)
  2. 返回 { session_id, active_souls, status }
```

### List Sessions

```
GET /api/v1/sessions?mode=conference&limit=20
  1. 解析 query → SessionFilter
  2. state.archive的store.list_sessions(&filter).await
  3. 返回 JSON Vec<SessionSummary>
```

### Get Session Detail

```
GET /api/v1/sessions/:id
  1. state.archive的store.get_session(id).await
  2. 返回 JSON SessionDetail { session, messages }
```

### Analytics Endpoints

```
GET /api/v1/analytics/summon-stats?period_start=...&period_end=...
  1. 解析 query period
  2. state.analytics.get_summon_stats(period_start, period_end).await
  3. 返回 JSON SummonStats

GET /api/v1/analytics/soul-effectiveness/:name
  1. state.analytics.get_soul_effectiveness(name).await
  2. 返回 JSON EffectivenessStats

GET /api/v1/analytics/mode-distribution
  1. state.analytics.get_mode_distribution().await
  2. 返回 JSON HashMap<PossessionMode, usize>

GET /api/v1/analytics/unsummoned?threshold_days=30
  1. 解析 query threshold_days (default: 30)
  2. state.analytics.detect_unsummoned_souls(threshold_days).await
  3. 返回 JSON Vec<SoulAlert>

GET /api/v1/analytics/low-effectiveness?threshold=0.3
  1. 解析 query threshold (default: 0.3)
  2. state.analytics.detect_low_effectiveness(threshold).await
  3. 返回 JSON Vec<BoundaryReview>
```

### Archive Export/Import

```
POST /api/v1/archive/export
  1. state.archive.export_archive() → task_id (async spawn)
  2. 返回 { task_id, status: "started" }

GET /api/v1/archive/export/:task_id
  1. state.archive.export_status(&task_id)
  2. 返回 ExportStatusResponse

POST /api/v1/archive/import  { body: ImportArchiveRequest }
  1. state.archive.import_archive(&bundle).await
  2. state.registry.reload().await  // 刷新 registry
  3. 返回 200 OK
```

### WebSocket Handler (Q2: C)

```
WS /ws/possess/:session_id/:channel
  1. axum ws upgrade
  2. on_connect:
     a. ws_manager.subscribe(session_id, channel, tx).await
     b. 注册该 WebSocket 到 session
  3. on_message: 忽略（只读流，客户端不应发送消息）
  4. on_disconnect:
     a. ws_manager.unsubscribe(session_id, channel).await
  5. 流式转发: WsSessionManager 的 broadcast 自动发送到所有订阅 channel 的 WS
```
