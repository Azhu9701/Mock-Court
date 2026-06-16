# 自定义推理模式

## 内置 6 种模式

| 模式 | 文件 | 流程 |
|------|------|------|
| Single | `modes/single.rs` | 单 Agent 问答 |
| Conference | `modes/conference.rs` | 并行 → 碰撞 → 综合 |
| Debate | `modes/debate.rs` | 对立 → 多轮 → 裁决 |
| Relay | `modes/relay.rs` | 串行阶段传递 |
| Learn | `modes/learn.rs` | 费曼教学法 |
| PracticeOpening | `modes/practice_opening.rs` | 四阶段循环 |

如果这 6 种不够，你可以注册自定义模式。

## 模式接口

每个模式实现以下函数签名：

```rust
pub async fn run(
    engine: &PossessionEngine,
    session_id: &str,
    agents: &[AgentProfile],
    task: &str,
    input: &PossessionInput,
    ws_manager: &WsSessionManager,
) -> Result<Vec<AgentOutput>>
```

## 示例：添加「投票」模式

### 1. 定义模式枚举

```rust
// rust/my-agent-app/src/modes/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PossessionMode {
    Single,
    Conference,
    Debate,
    Relay,
    Learn,
    PracticeOpening,
    Voting,  // 新增
}
```

### 2. 实现模式逻辑

```rust
// rust/my-agent-app/src/modes/voting.rs

use foundation::models::*;
use possession::ws::WsSessionManager;
use possession::PossessionEngine;
use ai_gateway::GatewayRegistry;

pub async fn run(
    engine: &PossessionEngine,
    session_id: &str,
    agents: &[AgentProfile],
    task: &str,
    input: &PossessionInput,
    ws: &WsSessionManager,
) -> Result<Vec<AgentOutput>> {
    let mut outputs = Vec::new();

    // 1. 让每个 Agent 独立投票
    for agent in agents {
        let prompt = format!(
            "请对以下问题投票（支持/反对/弃权），并给出理由：
             问题：{}
             
             输出格式：
             投票：支持/反对/弃权
             理由：...", 
            task
        );

        let stream = engine.gateway().call(&LLMRequest {
            provider: Provider::DeepSeek,  // 或从 agent.model 获取
            prompt: Prompt::user(&prompt),
            config: CallConfig::default(),
        })?;

        // 流式输出
        let output = stream_agent_output(agent, stream, ws, session_id).await?;
        outputs.push(output);
    }

    // 2. 统计投票
    let supports = outputs.iter().filter(|o| o.content.contains("支持")).count();
    let opposes = outputs.iter().filter(|o| o.content.contains("反对")).count();

    // 3. 广播结果
    let result = format!(
        "投票结果：支持 {} 票，反对 {} 票",
        supports, opposes
    );
    ws.broadcast_system(session_id, &result);

    Ok(outputs)
}
```

### 3. 注册模式到引擎

```rust
// rust/my-agent-app/src/main.rs

use possession::modes;

fn build_engine() -> PossessionEngine {
    // ... 初始化 engine ...

    // 注册自定义模式
    modes::register(
        PossessionMode::Voting,
        Arc::new(voting::run),
    );

    engine
}
```

### 4. 更新 Triage

```rust
// 在 domain.yaml 中添加触发词
trigger_markers:
  voting: ["投票", "表决", "举手表决"]
```

## 模式中的最佳实践

1. **始终使用 `WsSessionManager` 广播进度**：让前端知道 Agent 正在进行中
2. **Agent 调用之间发送 `soul_started` / `soul_done`**：前端依此切换 loading 状态
3. **错误时发送 `soul_error` 而不是 panic**：保证其他 Agent 不受影响
4. **大量 Agent 时考虑并发控制**：用 `tokio::spawn` + `JoinSet` 限制并发数
5. **输出完成后调用 `memory_graph.add_memory()`**：让记忆图谱记录本轮输出
