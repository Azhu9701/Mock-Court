# DeepSeek Tool Calls 集成计划

## 目标
将 DeepSeek 的 Tool Calls（函数调用）能力集成到万民幡系统中，使每个魂（AI 思想家）能够调用外部工具来增强回答能力。

## DeepSeek Tool Calls API 概述
- OpenAI 兼容格式：`tools` 参数传入工具定义，模型返回 `tool_calls`
- 支持流式和非流式模式
- 支持 `strict` 模式（Beta）：严格 JSON Schema 校验
- 思考模式（`deepseek-reasoner`）下不支持 tool calls，会自动回退到 `deepseek-chat`

### 消息流
```
用户消息 → 模型返回 tool_calls → 用户执行工具 → 发送 tool 角色消息 → 模型返回最终回答
```

---

## 实施步骤

### 步骤 1：Rust Foundation 数据模型扩展
**文件：** `rust/foundation/src/models.rs`

1.1 新增 `ToolDefinition` 结构体（OpenAI 兼容格式）
```rust
pub struct ToolDefinition {
    pub r#type: String,          // "function"
    pub function: FunctionDef,
}

pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
    pub strict: Option<bool>,           // Beta strict 模式
}
```

1.2 新增 `ToolCall` 结构体
```rust
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ToolCallFunction,
}

pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,  // JSON string
}
```

1.3 扩展 `Chunk` 结构体 — 新增 `tool_calls` 字段
```rust
pub struct Chunk {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub finish_reason: Option<String>,
    pub index: u32,
    pub usage: Option<UsageStats>,
    pub tool_calls: Vec<ToolCall>,  // 新增
}
```

1.4 扩展 `PromptMessage` 结构体 — 新增 `tool_calls` 和 `tool_call_id` 字段
```rust
pub struct PromptMessage {
    pub role: String,
    pub content: String,
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,     // 新增：assistant 角色的工具调用
    pub tool_call_id: Option<String>,           // 新增：tool 角色的调用 ID
}
```

1.5 扩展 `CallConfig` — 新增 `tools` 字段
```rust
pub struct CallConfig {
    // ... 现有字段
    pub tools: Option<Vec<ToolDefinition>>,  // 新增
    pub tool_choice: Option<String>,          // 新增：auto/none/required
}
```

---

### 步骤 2：DeepSeek 客户端支持 Tool Calls
**文件：** `rust/ai-gateway/src/deepseek.rs`

2.1 在 `call()` 方法中，将 `tools` 添加到请求体
```rust
if let Some(tools) = &config.tools {
    body["tools"] = serde_json::to_value(tools).unwrap();
    if let Some(tool_choice) = &config.tool_choice {
        body["tool_choice"] = serde_json::json!(tool_choice);
    }
}
```

2.2 在 SSE 流解析中，处理 `tool_calls` delta：
```rust
// 在 choices 循环中新增
if let Some(tool_calls) = choice["delta"]["tool_calls"].as_array() {
    for tc in tool_calls {
        let index = tc["index"].as_u64().unwrap_or(0) as usize;
        let function = &tc["function"];
        // 发送 tool_call chunk（增量参数）
        if let Some(args) = function["arguments"].as_str() {
            let _ = tx.send(Ok(Chunk {
                content: String::new(),
                reasoning_content: None,
                finish_reason: None,
                index: chunk_index,
                usage: None,
                tool_calls: vec![ToolCall {
                    id: tc["id"].as_str().unwrap_or("").to_string(),
                    r#type: "function".to_string(),
                    function: ToolCallFunction {
                        name: function["name"].as_str().unwrap_or("").to_string(),
                        arguments: args.to_string(),
                    },
                }],
            })).await;
            chunk_index += 1;
        }
    }
}
```

---

### 步骤 3：升级 Gateway Trait 和 GatewayRegistry
**文件：** `rust/ai-gateway/src/lib.rs`

3.1 `Gateway` trait 的 `call` 方法已通过 `CallConfig` 间接支持 tools，无需修改签名

3.2 确保 `GatewayRegistry::call()` 正确传递 `CallConfig`（包含 tools）到各个 provider

---

### 步骤 4：Possession 引擎新增 WebSocket 事件类型
**文件：** `rust/possession/src/lib.rs`

4.1 新增 WebSocket 事件类型
```rust
pub enum WsEventType {
    // ... 现有事件
    #[serde(rename = "tool_call_started")]
    ToolCallStarted,       // 魂开始调用工具
    #[serde(rename = "tool_call_chunk")]
    ToolCallChunk,         // 工具调用参数流式传输
    #[serde(rename = "tool_result")]
    ToolResult,            // 工具执行结果
}
```

4.2 新增工具调用相关类型
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallPayload {
    pub tool_call_id: String,
    pub tool_name: String,
    pub arguments: String,
    pub soul_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub tool_call_id: String,
    pub tool_name: String,
    pub result: String,
    pub soul_name: String,
}
```

---

### 步骤 5：服务端工具注册与执行
**新建文件：** `rust/possession/src/tools.rs`

5.1 定义 `ToolHandler` trait
```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, arguments: &str) -> Result<String>;
}
```

5.2 实现内置工具（第一期）
- **`web_search`**：联网搜索（调用本地搜索能力或 API）
- **`knowledge_search`**：搜索万民幡知识库
- **`calculate`**：简单计算

5.3 定义 `ToolRegistry`
```rust
pub struct ToolRegistry {
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
}
```

---

### 步骤 6：修改 Possession 流式处理
**文件：** `rust/possession/src/stream.rs`

6.1 修改 `stream_single_soul` 函数：
- 在接收到 `Chunk` 时检测是否有 `tool_calls`
- 如果有 tool_calls，暂存参数，完成后发送 `ToolCallStarted` 事件
- 执行工具 → 发送 `ToolResult` 事件
- 将 tool 结果追加到消息历史，再次调用 LLM
- 继续流式输出最终回答

6.2 新增 `stream_soul_with_tools` 函数
```rust
pub async fn stream_soul_with_tools(
    gateway: &GatewayRegistry,
    provider: Provider,
    prompt: &Prompt,
    config: &CallConfig,
    history: &[PromptMessage],
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
    tool_registry: &ToolRegistry,
) -> SoulOutput
```

工具调用循环逻辑：
```
loop (最多 max_tool_rounds = 3 轮):
    1. 调用 LLM（流式）
    2. 聚合 chunks
    3. 如果模型返回 tool_calls：
       a. 广播 ToolCallStarted 到 WebSocket
       b. 执行工具，广播 ToolResult
       c. 将 assistant(tool_calls) + tool(result) 消息添加到历史
       d. 继续循环
    4. 如果模型返回 content（最终回答）：
       a. 正常流式输出
       b. 结束循环
```

---

### 步骤 7：解析 SoulProfile 中的 tools 字段
**文件：** `rust/possession/src/` 或 `rust/foundation/src/`

7.1 在 summon prompt 构建时，解析 `SoulProfile.tools`（JSON 字符串）为 `Vec<ToolDefinition>`

7.2 将工具定义注入 `CallConfig.tools`

7.3 在 system prompt 中注入可用的工具使用说明

---

### 步骤 8：API 路由扩展
**文件：** `rust/api/src/routes/possess.rs`

无需新增路由（工具在服务端自动执行）。但需要考虑：
- 后续如果需要前端交互式确认工具调用，可新增 `POST /possess/:session_id/tool-result` 端点

---

### 步骤 9：前端 WebSocket Hook 扩展
**文件：** `nextjs/hooks/use-websocket.ts`

9.1 新增接口类型
```typescript
export interface ToolCallEvent {
  toolCallId: string;
  toolName: string;
  arguments: string;
  soulName: string;
  status: 'calling' | 'executing' | 'done';
  result?: string;
}
```

9.2 新增事件处理
```typescript
case "tool_call_started":
  const tcPayload = JSON.parse(event.payload) as ToolCallPayload;
  setToolCalls(prev => [...prev, {
    toolCallId: tcPayload.tool_call_id,
    toolName: tcPayload.tool_name,
    arguments: tcPayload.arguments,
    soulName: tcPayload.soul_name,
    status: 'calling',
  }]);
  break;

case "tool_result":
  const trPayload = JSON.parse(event.payload) as ToolResultPayload;
  setToolCalls(prev => prev.map(tc =>
    tc.toolCallId === trPayload.tool_call_id
      ? { ...tc, status: 'done', result: trPayload.result }
      : tc
  ));
  break;
```

9.3 返回值新增 `toolCalls` 状态

---

### 步骤 10：前端 UI 展示工具调用
**文件：** `nextjs/components/session-runner.tsx`（或新建组件）

10.1 新建 `ToolCallIndicator` 组件
- 在魂的对话流中，展示工具调用状态
- 调用中：显示加载状态 + 工具名称
- 执行完成：显示工具名称 + 结果摘要

10.2 集成到 `ConferenceView` / `SingleView` 等视图组件中

---

### 步骤 11：配置与测试

11.1 准备测试用魂的 `tools` 配置示例（JSON）
```json
[
  {
    "type": "function",
    "function": {
      "name": "web_search",
      "description": "搜索互联网获取最新信息",
      "parameters": {
        "type": "object",
        "properties": {
          "query": {"type": "string", "description": "搜索关键词"}
        },
        "required": ["query"]
      }
    }
  }
]
```

11.2 单元测试：`deepseek.rs` 的 tool call 解析
11.3 集成测试：完整 tool call 流程（魂调用 → 工具执行 → 结果返回）

---

## 数据流总结

```
用户提问
  →
Rust Backend: 构建 summon prompt（含 tools 定义）
  →
DeepSeek API: 模型决定调用工具
  ← 返回 tool_calls (流式 SSE)
  →
Rust Backend: 解析 tool_calls, 聚合参数
  → WebSocket: tool_call_started 事件 → 前端展示
  → ToolRegistry: 执行工具
  → WebSocket: tool_result 事件 → 前端展示结果
  → 将 tool result 追加到消息历史
  →
DeepSeek API: 再次调用（含 tool result）
  ← 返回最终答案 (流式 SSE)
  → WebSocket: soul_token 事件 → 前端展示
```

## 风险与注意事项
1. 工具调用循环需要限制最大轮数（防止死循环）
2. 服务端工具执行需要注意超时处理
3. soul profile 的 `tools` 字段需要向前兼容（空字符串 = 无工具）
4. DeepSeek reasoner 模式不支持 tool calls，需要检测并回退
