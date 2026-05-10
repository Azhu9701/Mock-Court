# Business Rules — B3: AI Gateway

## 1. Provider 选择规则

| 规则 | 描述 |
|------|------|
| BR1.1 | 请求中的 `Provider` 必须在 GatewayRegistry 中存在已注册的 client |
| BR1.2 | 请求的 Provider 不可用（无 API key）时，返回 `FoundationError::Validation("Provider X not configured")` |
| BR1.3 | `call_parallel()` 中，任一 provider 失败不影响其他 provider 的调用 |

## 2. 错误处理与重试

| 规则 | 描述 |
|------|------|
| BR2.1 | HTTP 4xx 错误（除 429）不重试，直接返回错误 |
| BR2.2 | HTTP 429（Rate Limit）重试 1 次，等待 `Retry-After` header 指定时间或默认 5 秒，再失败则向上返回错误 |
| BR2.3 | HTTP 5xx 错误重试 1 次，间隔 2 秒 |
| BR2.4 | 网络错误（连接超时/重置）重试 1 次 |
| BR2.5 | 所有重试总计不超过 1 次，总超时不超过 120 秒 |
| BR2.6 | 流式传输中途断连，已接收的 chunks 保留，错误附加在最后 |

## 3. Prompt 构建规则

| 规则 | 描述 |
|------|------|
| BR3.1 | `build_summon_prompt()` 必须包含 `soul.summon_prompt`（魂的核心行为指令） |
| BR3.2 | System message 包含魂的 ismism 坐标、领域、排除场景等信息，格式由模板定义 |
| BR3.3 | User message 包含实际任务/问题 |
| BR3.4 | 模板渲染失败（缺失变量）时，返回 `FoundationError::Validation` 并指明缺失变量名 |
| BR3.5 | 魂的 `exclude_scenarios` 必须在 system prompt 中作为约束条件出现 |

## 4. Provider API 兼容规则

| 规则 | 描述 |
|------|------|
| BR4.1 | 内部 `Prompt` 格式与 provider 无关，由各 client 负责格式转换 |
| BR4.2 | Claude Messages API: 不支持 system role 作为 message → 需用 `system` 顶层参数 |
| BR4.3 | OpenAI/DeepSeek Chat Completion API: system role 作为 messages[0] |
| BR4.4 | 每个 provider 的 `model` 可通过环境变量覆盖（`{PROVIDER}_MODEL`） |

## 5. Stream 生命周期

| 规则 | 描述 |
|------|------|
| BR5.1 | `call()` 返回 `mpsc::Receiver`，调用方负责消费 |
| BR5.2 | 当 stream 正常结束（收到 `finish_reason: "stop"`），sender 在发送最后一帧后关闭 |
| BR5.3 | 当 stream 出错，sender 发送 `Err(...)` 后关闭 |
| BR5.4 | 调用方 drop receiver 时，底层 HTTP 连接应被取消（通过 AbortController） |
