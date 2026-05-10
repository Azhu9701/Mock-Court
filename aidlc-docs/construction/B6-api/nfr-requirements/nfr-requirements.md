# NFR Requirements — B6: API Layer

## Performance

| 指标 | 目标 | 依据 |
|------|------|------|
| 请求延迟（P50） | < 10ms | 本地 loopback，纯内存/CPU 操作 |
| 请求延迟（P99） | < 100ms | 含 SQLite 查询的端点 |
| `/api/v1/possess` 响应时间 | < 50ms | 仅创建 session + spawn task，不等待 LLM |
| `/api/v1/archive/export` 响应时间 | < 50ms | 仅创建 task_id + spawn task |
| 静态文件加载 | N/A | B6 不 serve 前端静态文件（Next.js 负责） |

## 并发 (Q1: A — 30s 超时)

| 指标 | 目标 |
|------|------|
| 最大并发 HTTP 连接 | 50（本地单用户 + 最多 10 个 WS） |
| HTTP 请求超时 | 30s（Q1: A） |
| WS 空闲超时 | 300s（无消息则断开） |
| WS Ping 间隔 | 30s（axum 内置） |
| Graceful shutdown 等待 | 10s |

## Body Size (Q2: D — 不限制)

| 指标 | 目标 |
|------|------|
| Request body 大小 | 不限制（Q2: D — 本地使用无限流必要） |
| Response body 大小 | 不限制 |
| `ImportArchive` 最大包 | 由系统内存决定 |

## 安全 (Q3: A, Q4: A)

| 要求 | 描述 |
|------|------|
| 认证 | 无需（B6 Functional Q5: A） |
| TLS | 不需要（Q4: A — 本地 loopback） |
| 速率限制 | 不需要（Q3: A — 本地单用户无滥用风险） |
| CORS | 全开放（B6 Functional Q3: A） |
| 绑定地址 | `127.0.0.1:3096`（仅本地 loopback，不对外暴露） |

## 可靠性

| 要求 | 描述 |
|------|------|
| 错误恢复 | axum middleware catch panic → 500，不 crash 进程 |
| WS 断连 | WS 断连不传播到 PossessionEngine，LLM 调用继续 |
| 启动顺序 | API 服务最后启动（所有依赖 crate 初始化完毕后绑定端口） |
| 优雅关闭 | SIGTERM → 停止接受新请求 → 等待活跃请求完成（30s 超时）→ shutdown engine → exit |

## Logging (Q5: A — info)

| 要求 | 描述 |
|------|------|
| 日志级别 | `info`（Q5: A） |
| 请求日志 | method + path + status + latency_ms |
| 错误日志 | method + path + error_message + backtrace |
| WS 日志 | connect / disconnect + session_id + channel |
| 日志格式 | tracing-subscriber fmt layer 单行输出 |
