# 错误类型与处理建议

## 错误结构

```rust
pub enum FoundationError {
    AgentNotFound(String),
    SessionNotFound(String),
    NotFound(String),
    InvalidState(String),
    Validation(String),
    Storage(String),
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    LLM(String),
    Archive(String),
    Knowledge(String),
}
```

## 各类错误的触发条件与处理

### AgentNotFound

**触发**: 请求的 Agent 名称在注册表和文件系统中都不存在。

**HTTP 状态码**: 404

**原因与处理**:

| 原因 | 处理 |
|------|------|
| Agent 名称拼写错误 | 检查 `data/agents/` 目录下的文件名 |
| Agent 文件格式错误 | 检查 YAML frontmatter 是否合法 |
| 文件未被加载 | 调用 `POST /souls/reload` |

### SessionNotFound

**触发**: 请求的会话 ID 在数据库中不存在。

**HTTP 状态码**: 404

**原因与处理**:

| 原因 | 处理 |
|------|------|
| 会话已过期/已删除 | 重新创建会话 |
| 数据库文件损坏 | 检查 `data/app.db` 完整性 |

### InvalidState

**触发**: 操作与会话当前状态冲突（例如向已完成会话发送消息）。

**HTTP 状态码**: 409

**原因与处理**:

| 原因 | 处理 |
|------|------|
| 并发操作冲突 | 等待前一个操作完成后重试 |
| 对已完成会话操作 | 创建新会话或 fork 旧会话 |

### Validation

**触发**: 输入数据不符合约束（必填字段缺失、类型错误等）。

**HTTP 状态码**: 400

**示例**:

```
Validation("Agent name cannot contain special characters")
Validation("Session mode must be one of: single, conference, debate, relay, learn, practice_opening")
```

### Storage / Sqlite

**触发**: 数据库读写错误。

**HTTP 状态码**: 500

**原因与处理**:

| 原因 | 处理 |
|------|------|
| 磁盘空间不足 | 清理磁盘 |
| SQLite 文件锁定 | 确保 WAL 模式开启，检查 busy_timeout 配置 |
| Schema 版本不匹配 | 运行数据库迁移 |

### LLM

**触发**: AI 提供商调用失败。

**HTTP 状态码**: 502

**原因与处理**:

| 原因 | 处理 |
|------|------|
| API Key 无效/过期 | 检查 `data/apikeys.json` 或环境变量 |
| 提供商服务不可用 | 检查提供商状态页面，切换其他提供商 |
| 请求超时 | 增大 `max_tokens` 或切换更快的模型 |
| 速率限制 | 等待后重试，降低并发 Agent 数量 |

### Archive

**触发**: 归档操作失败（磁盘写入错误、格式错误等）。

**HTTP 状态码**: 500

### Knowledge

**触发**: 知识库操作失败（FTS5 索引损坏、搜索语法错误等）。

**HTTP 状态码**: 500

**处理**: 调用 `POST /knowledge/rebuild` 重建全文索引。

## API 响应格式

### 成功
```json
{ "data": { ... } }
```
或直接返回数据（集合类型通常包裹在 `data` 中）。

### 错误
```json
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent '合同审查员' not found. Check data/agents/ directory."
  }
}
```

## 前端错误处理

```typescript
// lib/api.ts 中的通用错误处理
try {
  const data = await apiRequest("/souls/some-agent");
} catch (error) {
  if (error instanceof NetworkError) {
    // 网络不通 → 提示用户检查连接
    showToast("无法连接到服务器，请检查网络");
  } else if (error.status === 404) {
    // 资源不存在 → 跳转或刷新
    showToast("请求的资源不存在");
  } else if (error.status === 502) {
    // LLM 调用失败 → 提示切换提供商
    showToast("AI 服务暂时不可用，请稍后重试或切换提供商");
  } else {
    showToast(error.message);
  }
}
```
