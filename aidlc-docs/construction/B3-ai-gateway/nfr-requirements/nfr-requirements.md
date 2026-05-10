# NFR Requirements — B3: AI Gateway

## Performance

| 指标 | 目标 | 依据 |
|------|------|------|
| 首次 chunk 延迟 | < 2s | LLM API 首 token 时间 + 网络 |
| chunk 间延迟 | < 100ms | 本地消费 mpsc channel，零网络 |
| `call_parallel()` N 路并发 | N 路同时发起，无需互相等待 | `tokio::spawn` 独立 task |
| HTTP 连接建立 | < 500ms | reqwest 连接池复用，冷启动含 DNS + TLS |

## Availability

| 要求 | 描述 |
|------|------|
| Provider 降级 | 一个 provider 不可用不影响其他 provider |
| 超时 | 连接超时 30s，读取超时 120s |
| 重试 | 429/5xx/网络错误各重试 1 次（见 Business Rules） |
| API Key 缺失 | 启动时不阻塞，`is_available()` 返回 false，调用时报错 |

## Connection Management (Q3: A)

使用 reqwest 默认连接池：
- 每个 host 最多 1 个空闲连接
- 连接空闲 90s 后回收
- 连接在 HTTP/1.1 Keep-Alive 下复用

## Async Runtime (Q4: B)

每个 LLM 调用通过 `tokio::spawn` 在独立 task 中执行：
- 调用方获取 `mpsc::Receiver` 接收 chunk
- `tokio::spawn` 返回 `JoinHandle`，可调用 `.abort()` 取消
- 调用方 drop receiver 时，task 检测到 send 失败并退出

## Security

| 要求 | 描述 |
|------|------|
| API Key 传输 | HTTPS 加密，Key 通过 `Authorization` header 发送 |
| API Key 存储 | 仅环境变量，不在日志中打印 |
| 输入校验 | Prompt 长度不限制（由 LLM API 自行处理） |

## Maintainability

| 要求 | 描述 |
|------|------|
| Provider 扩展 | 新增 provider 只需实现 `Gateway` trait + 注册到 `GatewayRegistry` |
| 模板管理 | 模板文件放在 `rust/ai-gateway/src/prompts/`，代码与模板分离 |
| 日志 | 每次调用记录 provider、tokens、延迟、状态（使用 tracing） |
