# 添加自定义工具

## 工具系统架构

```
Agent 输出 → 解析 tool_call → ToolRegistry.execute() → 工具执行 → 结果注入 Prompt → Agent 继续
```

## 工具定义

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}
```

## 注册工具

### 方式 1：Rust 实现

```rust
// rust/my-agent-app/src/tools/currency_converter.rs

use possession::tools::{ToolRegistry, ToolDefinition, ToolResult};
use async_trait::async_trait;

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult>;
}

struct CurrencyConverter;

#[async_trait]
impl Tool for CurrencyConverter {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "convert_currency".into(),
            description: "转换货币金额".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "amount": {"type": "number", "description": "金额"},
                    "from": {"type": "string", "description": "源货币代码（如 USD）"},
                    "to": {"type": "string", "description": "目标货币代码（如 CNY）"}
                },
                "required": ["amount", "from", "to"]
            }),
        }
    }

    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult> {
        let amount: f64 = params["amount"].as_f64().unwrap();
        let from = params["from"].as_str().unwrap();
        let to = params["to"].as_str().unwrap();

        // 调用汇率 API
        let rate = fetch_exchange_rate(from, to).await?;
        let result = amount * rate;

        Ok(ToolResult {
            success: true,
            content: format!("{} {} = {} {}", amount, from, result, to),
            metadata: None,
        })
    }
}

// main.rs 中注册
engine.tool_registry().register(Arc::new(CurrencyConverter));
```

### 方式 2：HTTP 工具（无需写 Rust）

在 `config/tools.yaml` 中声明：

```yaml
tools:
  - name: "convert_currency"
    description: "转换货币金额"
    endpoint: "https://api.exchangerate-api.com/v4/latest/{from}"
    method: "GET"
    parameters:
      type: "object"
      properties:
        amount:
          type: "number"
          description: "金额"
        from:
          type: "string"
          description: "源货币代码"
        to:
          type: "string"
          description: "目标货币代码"
      required: ["amount", "from", "to"]
    result_template: "{{amount}} {{from}} = {{result}} {{to}}"
```

框架自动调用 HTTP 端点并解析结果。

## 工具的 Prompt 格式

当 Agent 配置了工具后，框架自动在系统 Prompt 中注入：

```
你可以使用以下工具：

## convert_currency
转换货币金额

参数：
- amount (number): 金额
- from (string): 源货币代码
- to (string): 目标货币代码

调用格式：
<tool_call>
{"name": "convert_currency", "params": {"amount": 100, "from": "USD", "to": "CNY"}}
</tool_call>
```

Agent 输出 `tool_call` 块 → Engine 拦截 → ToolRegistry 执行 → 结果注入上下文 → Agent 继续。

## 工具结果注入

```rust
// possession/src/tools.rs 中的自动流程
// Agent 输出包含 <tool_call> → 暂停流式输出 → 执行工具 → 构造新 Prompt → 继续
let tool_calls = parse_tool_calls(&agent_output);
for call in tool_calls {
    let result = tool_registry.execute(&call.name, &call.params).await?;
    // 将结果追加到 Agent 会话上下文
    context.push(format!(
        "<tool_result name=\"{}\">\n{}\n</tool_result>",
        call.name, result.content
    ));
}
// Agent 继续推理
```

## 前端工具指示器

工具调用时自动向 WebSocket 发送事件，前端展示：

```
ToolCallIndicator 组件：
  ┌─────────────────────────────────┐
  │ 🔧 正在调用 convert_currency... │
  │   100 USD → 724 CNY            │
  └─────────────────────────────────┘
```
