# 核心类型索引

## 数据模型

### AgentProfile
Agent 的完整定义。

```rust
pub struct AgentProfile {
    pub name: String,                     // 唯一标识
    pub title: String,                    // 显示标题
    pub description: String,              // 描述
    pub model: String,                    // 推荐模型
    pub tools: Vec<String>,               // 工具列表
    pub dimensions: HashMap<String, f64>, // 坐标维度值
    pub domains: Vec<String>,             // 擅长领域
    pub system_prompt: String,            // 系统 Prompt
    pub trigger_keywords: Vec<String>,    // 触发关键词
    pub compat: Vec<String>,              // 兼容 Agent
    pub incompat: Vec<String>,            // 不兼容 Agent
    pub voice: String,                    // 语气
    pub mind: String,                     // 思维模式
    pub summon_count: u32,                // 被调用次数
    pub exclude_scenarios: Vec<String>,   // 排除场景
}
```

### Session
一次多 Agent 会话。

```rust
pub struct Session {
    pub id: String,
    pub title: String,
    pub mode: PossessionMode,             // 模式枚举
    pub status: SessionStatus,            // Active | Completed | Archived
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Message
会话中的一条消息。

```rust
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,                // User | Agent | Synthesis | System
    pub agent_name: Option<String>,
    pub content: String,
    pub seq: u32,
    pub created_at: DateTime<Utc>,
}
```

### PossessionMode
推理模式枚举。

```rust
pub enum PossessionMode {
    Single,          // 单 Agent
    Conference,      // 合议（并行+碰撞+综合）
    Debate,          // 辩论（对立+多轮+裁决）
    Relay,           // 接力（串行阶段）
    Learn,           // 学习（费曼法）
    PracticeOpening, // 实践开口（四阶段）
}
```

### SessionStatus
```rust
pub enum SessionStatus { Active, Completed, Archived }
```

### MessageRole
```rust
pub enum MessageRole { User, Agent, Synthesis, System }
```

## LLM 相关

### LLMRequest
```rust
pub struct LLMRequest {
    pub provider: Provider,
    pub prompt: Prompt,
    pub config: CallConfig,
}
```

### Prompt
```rust
pub struct Prompt {
    pub system: Option<String>,
    pub messages: Vec<PromptMessage>,
}

pub struct PromptMessage {
    pub role: String,     // "user" | "assistant"
    pub content: String,
}
```

### CallConfig
```rust
pub struct CallConfig {
    pub temperature: f64,                    // 默认 0.7
    pub max_tokens: u32,                     // 默认 4096
    pub stream: bool,                        // 默认 true
    pub model: Option<String>,               // 覆盖默认模型
    pub reasoning_effort: Option<ReasoningEffort>,
    pub tools: Option<Vec<ToolDefinition>>,
}
```

### ReasoningEffort
```rust
pub enum ReasoningEffort {
    NonThink,    // 不使用深度推理
    Think,       // 标准推理
    ThinkHigh,   // 深度推理
    ThinkMax,    // 最大推理
}
```

### Chunk
LLM 流式输出的最小单位。

```rust
pub struct Chunk {
    pub content: String,
    pub reasoning_content: Option<String>,   // 思考过程（DeepSeek Reasoner 等）
    pub finish_reason: Option<String>,
    pub index: u32,
    pub usage: Option<UsageStats>,
}
```

### UsageStats
```rust
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

### Provider
```rust
pub enum Provider {
    Claude,
    OpenAI,
    DeepSeek,
    LMStudio,
    Custom(String),
}
```

## 领域配置

### DomainProfile
```rust
pub struct DomainProfile {
    pub name: String,
    pub icon: String,
    pub system_name: String,
    pub agent_noun: String,
    pub user_title: String,
    pub synthesis_verb: String,
    pub dimensions: Vec<DomainDimension>,
    pub synthesis_template: String,
    pub collect_intro: String,
    pub trigger_markers: HashMap<String, Vec<String>>,
    pub memory_config: MemoryConfig,
}
```

### DomainDimension
```rust
pub struct DomainDimension {
    pub id: String,
    pub label: String,
    pub description: String,
    pub values: Vec<String>,
    pub weight: f64,
}
```

### MemoryConfig
```rust
pub struct MemoryConfig {
    pub default_share: ShareMode,       // None | Session | All
    pub max_turns: usize,
    pub persist: bool,
}
```

## 记忆图谱

### MemoryNode
```rust
pub struct MemoryNode {
    pub id: String,
    pub session_id: String,
    pub agent_name: String,
    pub content: String,
    pub turn: u32,
    pub embedding: Option<Vec<f64>>,
    pub created_at: DateTime<Utc>,
}
```

### MemoryEdge
```rust
pub struct MemoryEdge {
    pub relation: RelationType,         // Reference | Contradiction | Complement | Summary
    pub weight: f64,
}

pub enum RelationType { Reference, Contradiction, Complement, Summary }
```

### MemoryGraph
```rust
pub struct MemoryGraph {
    pub nodes: StableGraph<MemoryNode, MemoryEdge>,
    pub agent_index: HashMap<String, Vec<NodeIndex>>,
}
```

### TimeWindow
```rust
pub struct TimeWindow {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub last_n_turns: Option<usize>,
}
```

## 工具系统

### ToolDefinition
```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,     // JSON Schema
}
```

### ToolCall
```rust
pub struct ToolCall {
    pub name: String,
    pub params: serde_json::Value,
}
```

### ToolResult
```rust
pub struct ToolResult {
    pub success: bool,
    pub content: String,
    pub metadata: Option<HashMap<String, String>>,
}
```

## WebSocket

### WsEvent
```rust
pub struct WsEvent {
    pub event_type: WsEventType,
    pub session_id: String,
    pub agent_name: Option<String>,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
```

### WsEventType
```rust
pub enum WsEventType {
    SessionStarted, SessionComplete,
    SoulStarted, SoulToken, SoulDone, SoulError, SoulCalling,
    SynthesisStarted, SynthesisChunk, SynthesisDone,
    Collision, Cost,
    ToolCallStarted, ToolResult,
    SoulRecommendations, ProcessStep,
    SystemMessage, Error,
}
```

## 错误

### FoundationError
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
