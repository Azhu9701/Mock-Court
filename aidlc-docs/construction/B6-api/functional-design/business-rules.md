# Business Rules — B6: API Layer

## 1. 路由规则 (Q1: A)

| 规则 | 描述 |
|------|------|
| BR1.1 | 所有 REST 路由必须以 `/api/v1/` 为前缀 |
| BR1.2 | WebSocket 路由为 `/ws/possess/{session_id}/{channel}`，不参与 REST 前缀 |
| BR1.3 | 路由变量使用 axum `Path<T>` 提取，类型不匹配直接返回 400 |
| BR1.4 | 不存在的路由返回 404 `{ "error": "Not found" }` |

## 2. CORS 规则 (Q3: A)

| 规则 | 描述 |
|------|------|
| BR2.1 | 允许所有来源 (`Any` origin) |
| BR2.2 | 允许的 HTTP 方法: GET, POST, PUT, DELETE, OPTIONS |
| BR2.3 | 允许的请求头: Content-Type, Authorization（预留） |
| BR2.4 | 预检请求（OPTIONS）直接返回 204，不经过业务逻辑 |

## 3. 响应格式规则 (Q4: A)

| 规则 | 描述 |
|------|------|
| BR3.1 | 成功响应：直接返回 JSON 对象，HTTP 200/201/204 |
| BR3.2 | 错误响应格式统一为 `{ "error": "message" }` |
| BR3.3 | 4xx 错误（客户端错误）：400 Bad Request / 404 Not Found |
| BR3.4 | 5xx 错误（服务端错误）：500 Internal Server Error，错误信息不泄露内部细节 |
| BR3.5 | `FoundationError::SoulNotFound` 映射为 HTTP 404 |
| BR3.6 | `FoundationError::Validation(msg)` 映射为 HTTP 400 |
| BR3.7 | 其他 `FoundationError` 映射为 HTTP 500 |
| BR3.8 | 所有响应必须设置 `Content-Type: application/json` |

## 4. 认证/授权规则 (Q5: A)

| 规则 | 描述 |
|------|------|
| BR4.1 | 无需任何认证 token/header/cookie |
| BR4.2 | 所有端点对本地访问完全开放 |
| BR4.3 | 不在代码中预埋任何认证中间件（方便未来通过中间件注入） |

## 5. Request Body 校验

| 规则 | 描述 |
|------|------|
| BR5.1 | `POST /api/v1/possess` 要求 `task` 字段非空，否则 400 `{ "error": "task is required" }` |
| BR5.2 | `POST /api/v1/souls` 要求 `name` 和 `summon_prompt` 非空 |
| BR5.3 | JSON body 解析失败时返回 400 `{ "error": "Invalid JSON: ..." }` |
| BR5.4 | Query 参数解析失败使用默认值，不报错 |

## 6. WebSocket 规则 (Q2: C)

| 规则 | 描述 |
|------|------|
| BR6.1 | WS 升级失败返回 400（非 WebSocket 请求） |
| BR6.2 | 每个 session_id + channel 组合最多一个连接（后连踢前连） |
| BR6.3 | WS 客户端不应发送业务消息（只读流），收到的消息被忽略 |
| BR6.4 | WS 连接断开时自动从 session 取消订阅 |
| BR6.5 | channel 固定值：`main`（全部流动消息）、`synthesis`（仅辩证综合流） |
| BR6.6 | WS Ping/Pong 由 axum 内置处理，间隔 30s |

## 7. 大响应控制

| 规则 | 描述 |
|------|------|
| BR7.1 | `GET /api/v1/export/:task_id` 在任务进行中返回 `{ "status": "processing" }` |
| BR7.2 | `list_sessions()` 默认 limit=50，最大 200 |
| BR7.3 | `list_souls()` 全量返回（魂数量不超过 50） |

## 8. 启动规则

| 规则 | 描述 |
|------|------|
| BR8.1 | axum 绑定地址 `0.0.0.0:3096`（与 Next.js 3000 不冲突） |
| BR8.2 | 启动时加载所有 soul profiles → SoulRegistry |
| BR8.3 | 启动时初始化 SQLite 数据库 → ArchiveSystem + AnalyticsEngine |
| BR8.4 | 优雅关闭信号处理：SIGTERM/SIGINT → mark engine shutdown → drain connections → exit |

## 9. Logging 规则

| 规则 | 描述 |
|------|------|
| BR9.1 | 每个请求记录：method + path + status + latency_ms |
| BR9.2 | 错误请求记录：method + path + error_message |
| BR9.3 | WS 连接/断开记录 session_id + channel |
| BR9.4 | 使用 `tracing` crate + `tracing-subscriber` fmt layer |
