# Domain Entities — B6: API Layer

## Request Types

### StartPossessionRequest — 发起附体

```rust
#[derive(Debug, Deserialize)]
pub struct StartPossessionRequest {
    pub mode: Option<String>,      // "single"|"conference"|"debate"|"relay"|"learn"|"practice_opening"
    pub task: String,              // 用户任务描述
    pub souls: Vec<String>,        // 指定魂名列表
    pub topic: Option<String>,     // 辩论主题（仅 debate 模式）
}
```

### CreateSoulRequest — 创建魂

```rust
#[derive(Debug, Deserialize)]
pub struct CreateSoulRequest {
    pub name: String,
    pub ismism_code: String,
    pub field: String,
    pub ontology: String,
    pub epistemology: String,
    pub teleology: String,
    pub grade: SoulGrade,
    pub domains: Vec<String>,
    pub tags: Vec<String>,
    pub summon_prompt: String,
}
```

### UpdateSoulRequest — 更新魂

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateSoulRequest {
    pub ismism_code: Option<String>,
    pub field: Option<String>,
    pub grade: Option<SoulGrade>,
    pub domains: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub summon_prompt: Option<String>,
}
```

### ImportArchiveRequest — 导入存档

```rust
#[derive(Debug, Deserialize)]
pub struct ImportArchiveRequest {
    pub bundle: ExportBundle,  // 复用 archive crate 的 ExportBundle
}
```

## Response Types

### ApiError — 统一错误响应 (Q4: A)

```rust
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}
```

### StartPossessionResponse — 附体启动响应

```rust
#[derive(Debug, Serialize)]
pub struct StartPossessionResponse {
    pub session_id: String,
    pub mode: String,
    pub ws_url: String,  // WS 连接地址，如 /ws/possess/{session_id}/main
}
```

### ExportResponse — 导出响应

```rust
#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub task_id: String,
    pub status: String,  // "started"
}
```

### ExportStatusResponse — 导出状态响应

```rust
#[derive(Debug, Serialize)]
pub struct ExportStatusResponse {
    pub task_id: String,
    pub status: ExportStatus,  // 复用 archive crate
}
```

## Query Parameters

### SoulListQuery — 魂列表查询

```rust
#[derive(Debug, Deserialize)]
pub struct SoulListQuery {
    pub grade: Option<String>,
    pub field: Option<String>,
    pub nearest: Option<String>,   // 1-2-3-3 格式 ismism code
    pub limit: Option<usize>,
}
```

### SessionListQuery — 会话列表查询

```rust
#[derive(Debug, Deserialize)]
pub struct SessionListQuery {
    pub mode: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
```

### AnalyticsQuery — 统计查询

```rust
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub period_start: Option<String>,  // ISO 8601
    pub period_end: Option<String>,
    pub threshold_days: Option<u32>,
    pub threshold: Option<f64>,        // 低效阈值
}
```

## WebSocket Types

### WsEvent — 复用 possession crate 的 WsEvent

```rust
// 定义在 possession crate，B6 只做序列化透传
#[derive(Debug, Serialize, Deserialize)]
pub struct WsEvent {
    pub event_type: WsEventType,
    pub payload: String,
    pub soul_name: Option<String>,
    pub seq: u32,
}
```

## Relations

```
HTTP Routes
├── POST   /api/v1/possess         → StartPossessionRequest → StartPossessionResponse
├── GET    /api/v1/souls           → SoulListQuery → Vec<SoulListEntry>
├── GET    /api/v1/souls/:name     → SoulProfile
├── POST   /api/v1/souls           → CreateSoulRequest → SoulListEntry
├── PUT    /api/v1/souls/:name     → UpdateSoulRequest → SoulListEntry
├── DELETE /api/v1/souls/:name     → ()
├── GET    /api/v1/sessions        → SessionListQuery → Vec<SessionSummary>
├── GET    /api/v1/sessions/:id    → SessionDetail
├── GET    /api/v1/analytics/*     → AnalyticsQuery → various stats
├── POST   /api/v1/archive/export  → () → ExportResponse
└── POST   /api/v1/archive/import  → ImportArchiveRequest → ()

WebSocket
└── WS /ws/possess/{session_id}/{channel}  → WsEvent stream
```
