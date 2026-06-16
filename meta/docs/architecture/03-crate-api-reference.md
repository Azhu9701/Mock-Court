# 模块间接口契约

## 核心 Trait

### Storage（foundation）

```rust
pub trait Storage: Send + Sync {
    // Agent CRUD
    async fn list_agent_names(&self) -> Result<Vec<String>>;
    async fn read_agent(&self, name: &str) -> Result<AgentProfile>;
    async fn write_agent(&self, profile: &AgentProfile) -> Result<()>;
    async fn delete_agent(&self, name: &str) -> Result<()>;

    // Session CRUD
    async fn create_session(&self, session: &Session) -> Result<()>;
    async fn get_session(&self, id: &str) -> Result<Session>;
    async fn update_session(&self, session: &Session) -> Result<()>;
    async fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>>;
    async fn delete_session(&self, id: &str) -> Result<()>;

    // Message CRUD
    async fn append_message(&self, msg: &Message) -> Result<()>;
    async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>>;

    // Call Records
    async fn record_call(&self, record: &CallRecord) -> Result<()>;
    async fn query_call_records(&self, filter: &CallFilter) -> Result<Vec<CallRecord>>;

    // Archive
    async fn archive_agent_output(&self, session_id: &str, agent: &str, content: &str) -> Result<String>;
    async fn archive_synthesis(&self, session_id: &str, content: &str) -> Result<String>;

    // Knowledge
    async fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeResult>>;
    async fn rebuild_fts(&self) -> Result<usize>;
}
```

### Gateway（ai-gateway）

```rust
pub trait Gateway: Send + Sync {
    fn provider(&self) -> Provider;
    fn is_available(&self) -> bool;
    /// 返回 mpsc::Receiver，支持流式消费
    fn call(&self, prompt: &Prompt, config: &CallConfig) -> mpsc::Receiver<Result<Chunk>>;
}
```

### CoordinateMatcher（registry）

```rust
pub trait CoordinateMatcher: Send + Sync {
    /// 计算两个坐标向量间的距离
    fn distance(&self, a: &[f64], b: &[f64]) -> f64;
    
    /// 从候选集中找 top_k 近邻
    fn nearest(
        &self, 
        target: &[f64], 
        candidates: &[(String, Vec<f64>)], 
        top_k: usize
    ) -> Vec<(String, f64)>;
}
```

### MemoryStore（foundation）

```rust
pub trait MemoryStore: Send + Sync {
    async fn save_node(&self, node: MemoryNode) -> Result<String>;
    async fn save_edge(&self, edge: MemoryEdge) -> Result<()>;
    async fn query_by_agent(&self, agent_id: &str, window: TimeWindow) -> Result<Vec<MemoryNode>>;
    async fn query_semantic(&self, query: &str, top_k: usize) -> Result<Vec<(MemoryNode, f64)>>;
    async fn subgraph(&self, session_id: &str) -> Result<MemoryGraph>;
}
```

## 核心类型

### AgentProfile

```rust
pub struct AgentProfile {
    pub name: String,
    pub description: String,
    pub model: String,
    pub tools: Vec<String>,             // 可用工具列表
    pub dimensions: HashMap<String, f64>, // 坐标维度值
    pub domains: Vec<String>,           // 擅长领域
    pub system_prompt: String,          // 系统 Prompt
    pub trigger_keywords: Vec<String>,  // 触发关键词
    pub compat: Vec<String>,            // 兼容 Agent
    pub incompat: Vec<String>,          // 不兼容 Agent
}
```

### Session

```rust
pub struct Session {
    pub id: String,
    pub title: String,
    pub mode: PossessionMode,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PossessionMode { Single, Conference, Debate, Relay, Learn, PracticeOpening }
pub enum SessionStatus { Active, Completed, Archived }
```

### Message

```rust
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,              // User | Agent | Synthesis | System
    pub agent_name: Option<String>,
    pub content: String,
    pub seq: u32,
    pub created_at: DateTime<Utc>,
}
```

### LLM 请求/响应

```rust
pub struct LLMRequest {
    pub provider: Provider,
    pub prompt: Prompt,
    pub config: CallConfig,
}

pub struct CallConfig {
    pub temperature: f64,
    pub max_tokens: u32,
    pub stream: bool,
    pub model: Option<String>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub tools: Option<Vec<ToolDefinition>>,
}

pub struct Chunk {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub finish_reason: Option<String>,
    pub index: u32,
    pub usage: Option<UsageStats>,
}
```

### DomainProfile

```rust
pub struct DomainProfile {
    pub name: String,
    pub system_name: String,            // e.g. "法律智囊团"
    pub agent_noun: String,             // e.g. "顾问"
    pub user_title: String,             // e.g. "委托人"
    pub dimensions: Vec<DomainDimension>,
    pub synthesis_template: String,
    pub collect_intro: String,
}

pub struct DomainDimension {
    pub id: String,
    pub label: String,
    pub values: Vec<String>,
    pub weight: f64,
}
```

### WsEvent

```rust
pub struct WsEvent {
    pub event_type: WsEventType,
    pub session_id: String,
    pub agent_name: Option<String>,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

pub enum WsEventType {
    SessionStarted, SessionComplete,
    SoulStarted, SoulToken, SoulDone, SoulError,
    SynthesisStarted, SynthesisChunk, SynthesisDone,
    Collision, Cost,
    ToolCallStarted, ToolResult,
    SystemMessage, Error,
}
```

## 模块间调用关系

```
api::routes::possess
    → possession::PossessionEngine::start_possession()
        → registry::AgentRegistry::get_agent()       // 获取 Agent 配置
        → possession::triage::triage()                // 分析任务 → 匹配模式
        → possession::modes::conference::run()        // 执行具体模式
            → ai_gateway::GatewayRegistry::call()     // 调用 LLM
                → possession::stream::stream_agent()  // 流式输出
                    → possession::ws::WsSessionManager::broadcast() // WS 广播
            → possession::cross_detector::detect()    // 碰撞检测
            → possession::soul::memory_graph::add()   // 记忆记录
        → archive::ArchiveSystem::archive_session()   // 归档
```

## 错误类型

```rust
pub enum FoundationError {
    AgentNotFound(String),
    SessionNotFound(String),
    InvalidState(String),
    Validation(String),
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    Storage(String),
    LLM(String),
    Archive(String),
    Knowledge(String),
}
```
