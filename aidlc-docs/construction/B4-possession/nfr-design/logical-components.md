# Logical Components — B4: Possession Core

## Component Architecture

```
PossessionEngine (src/lib.rs)
├── EntryClassifier (src/classifier.rs)
│   └── classify_entry() → EntryType
├── ModeDispatcher (src/lib.rs)
│   └── dispatch(entry_type, input, ws_tx) → Result<()>
├── SingleHandler (src/modes/single.rs)
│   └── run_single(session_id, soul, task, ws_tx) → Result<SoulOutput>
├── ConferenceHandler (src/modes/conference.rs)
│   ├── run_parallel_souls() → Vec<SoulOutput>
│   └── run_synthesis(outputs, ws_tx) → Result<SynthesisReport>
├── DebateHandler (src/modes/debate.rs)
│   ├── run_debate_rounds() → (SoulOutput, SoulOutput)
│   └── run_verdict(output_a, output_b, ws_tx) → Result<Verdict>
├── RelayHandler (src/modes/relay.rs)
│   └── run_relay_chain(soul_chain, task, ws_tx) → Result<Vec<SoulOutput>>
├── LearnHandler (src/modes/learn.rs)
│   └── run_learn(soul, task, ws_tx) → Result<SoulOutput>
├── PracticeOpeningHandler (src/modes/practice_opening.rs)
│   ├── run_p1_collect(input, ws_tx) → Result<FieldData>
│   ├── run_p2_digest(field_data, souls, ws_tx) → Result<Vec<DigestionReport>>
│   ├── run_p3_revise(reports, ws_tx) → Result<Vec<RevisionRecord>>
│   └── run_p4_action(revisions, ws_tx) → Result<ActionMemo>
├── WsSessionManager (src/ws.rs)
│   ├── create_session(session_id, tx)
│   ├── remove_session(session_id)
│   ├── broadcast_soul(session_id, soul, event)
│   ├── broadcast_system(session_id, event)
│   └── handle_reconnect(session_id, new_tx)
└── RecoveryManager (src/recovery.rs)
    ├── recover_active_sessions() → Vec<RecoveredSession>
    └── resume_session(session_id) → Result<()>
```

## Component: PossessionEngine (`src/lib.rs`)

```rust
pub struct PossessionEngine {
    store: Arc<dyn Storage>,
    registry: Arc<SoulRegistry>,
    gateway: Arc<GatewayRegistry>,
    ws_manager: WsSessionManager,
    shutdown_flag: AtomicBool,
    active_calls: RwLock<HashSet<String>>,
}

impl PossessionEngine {
    pub fn new(store, registry, gateway) -> Self
    pub async fn recover_sessions(&self) -> Result<Vec<String>>
    pub async fn start_possession(&self, input: PossessionInput) -> Result<SessionHandle>
    pub async fn graceful_shutdown(&self)
}
```

## File Structure

```
rust/possession/
├── Cargo.toml
└── src/
    ├── lib.rs              # PossessionEngine + ModeDispatcher
    ├── classifier.rs        # EntryClassifier
    ├── ws.rs                # WsSessionManager
    ├── recovery.rs          # RecoveryManager
    ├── stream_bridge.rs     # SSE → WS bridge
    └── modes/
        ├── mod.rs
        ├── single.rs
        ├── conference.rs
        ├── debate.rs
        ├── relay.rs
        ├── learn.rs
        └── practice_opening.rs
```

## Component Dependencies

```
possession
├── foundation (Storage trait, models)
├── registry (SoulRegistry → soul lookup)
└── ai-gateway (GatewayRegistry → LLM call, PromptBuilder)
```

## Key Interfaces

| Component | Method | Async | Description |
|-----------|--------|-------|-------------|
| PossessionEngine | `new()` | no | 构造 |
| PossessionEngine | `recover_sessions()` | yes | 启动恢复 |
| PossessionEngine | `start_possession()` | yes | 开始附体 |
| PossessionEngine | `graceful_shutdown()` | yes | 优雅关闭 |
| EntryClassifier | `classify_entry()` | no | 入口分流（纯规则） |
| WsSessionManager | `create_session()` | no | 创建 WS session |
| WsSessionManager | `broadcast_soul()` | no | 魂频道广播 |
| WsSessionManager | `handle_reconnect()` | no | WS 重连续传 |
| RecoveryManager | `recover_active_sessions()` | yes | 扫 active sessions |
| RecoveryManager | `resume_session()` | yes | 恢复单个 session |
