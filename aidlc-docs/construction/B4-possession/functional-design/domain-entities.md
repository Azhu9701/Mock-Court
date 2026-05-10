# Domain Entities — B4: Possession Core

## New Types

### PossessionInput — 附体请求

```rust
#[derive(Debug, Clone)]
pub struct PossessionInput {
    pub mode: PossessionMode,
    pub task: String,
    pub souls: Vec<String>,
    pub topic: Option<String>,
}
```

### EntryType — 入口分流结果

```rust
#[derive(Debug, Clone)]
pub enum EntryType {
    Single,
    Conference,
    Debate,
    Relay,
    Learn,
    PracticeOpening,
}
```

### SoulOutput — 魂调用结果

```rust
#[derive(Debug, Clone)]
pub struct SoulOutput {
    pub soul_name: String,
    pub content: String,
    pub usage: UsageStats,
    pub error: Option<String>,
}
```

### ConferenceSession — 合议会话

```rust
#[derive(Debug, Clone)]
pub struct ConferenceSession {
    pub session_id: String,
    pub task: String,
    pub souls: Vec<String>,
    pub outputs: Vec<SoulOutput>,
    pub synthesis: Option<SynthesisReport>,
}
```

### DebateSession — 辩论会话

```rust
#[derive(Debug, Clone)]
pub struct DebateSession {
    pub session_id: String,
    pub topic: String,
    pub soul_a: String,
    pub soul_b: String,
    pub output_a: Option<SoulOutput>,
    pub output_b: Option<SoulOutput>,
    pub verdict: Option<Verdict>,
}
```

### RelaySession — 接力会话

```rust
#[derive(Debug, Clone)]
pub struct RelaySession {
    pub session_id: String,
    pub task: String,
    pub soul_chain: Vec<String>,
    pub current_index: usize,
    pub outputs: Vec<SoulOutput>,
}
```

### SynthesisReport — 辩证综合报告

```rust
#[derive(Debug, Clone)]
pub struct SynthesisReport {
    pub content: String,
    pub source_outputs: Vec<String>,
    pub consensus_points: Vec<String>,
    pub divergence_points: Vec<String>,
    pub blind_spots: Vec<String>,
}
```

### Verdict — 辩论裁决

```rust
#[derive(Debug, Clone)]
pub struct Verdict {
    pub winner: VerdictResult,
    pub reasoning: String,
    pub key_points_a: Vec<String>,
    pub key_points_b: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum VerdictResult {
    SoulAWins,
    SoulBWins,
    Draw,
}
```

### PractitionerInput — 在场者输入

```rust
#[derive(Debug, Clone)]
pub struct PractitionerInput {
    pub user_input: String,
    pub context: String,
    pub round: u32,
}
```

### FieldData — P1 现场数据

```rust
#[derive(Debug, Clone)]
pub struct FieldData {
    pub phenomenon: String,
    pub constraints: Vec<String>,
    pub stakeholders: Vec<String>,
    pub urgency: String,
    pub raw_dialog: Vec<PromptMessage>,
}
```

### DigestionReport — P2 魂消化报告

```rust
#[derive(Debug, Clone)]
pub struct DigestionReport {
    pub soul_name: String,
    pub analysis: String,
    pub blind_spots: Vec<String>,
    pub recommendations: Vec<String>,
}
```

### RevisionRecord — P3 修正记录

```rust
#[derive(Debug, Clone)]
pub struct RevisionRecord {
    pub soul_name: String,
    pub original_claim: String,
    pub revised_claim: String,
    pub revision_reason: String,
}
```

### ActionMemo — P4 行动备忘

```rust
#[derive(Debug, Clone)]
pub struct ActionMemo {
    pub priority_actions: Vec<String>,
    pub responsible_souls: Vec<String>,
    pub timeline: String,
    pub caveats: Vec<String>,
}
```

### WsEvent — WebSocket 事件

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    pub event_type: WsEventType,
    pub payload: String,
    pub soul_name: Option<String>,
    pub seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsEventType {
    SoulChunk,
    SoulDone,
    SynthesisChunk,
    SynthesisDone,
    SystemMessage,
    Error,
}
```

## Relations

```
PossessionEngine
├── classify_entry(PossessionInput) → EntryType → dispatch to mode handler
├── Single: call_llm(soul, task) → stream via WsEvent::SoulChunk → SoulOutput
├── Conference: parallel call_llm(souls) → SynthesisReport via synthesis prompt
├── Debate: dual call_llm(soul_a, soul_b) → Verdict via verdict prompt
├── Relay: sequential call_llm(soul_chain[i]) → next stage with prev_output
├── Learn: call_llm(soul, task) → explain mode
└── PracticeOpening: P1(收集) → P2(消化) → P3(修正) → P4(行动)
```
