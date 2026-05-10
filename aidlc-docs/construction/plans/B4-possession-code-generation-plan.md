# Code Generation Plan — B4: Possession Core

## Unit Context

- **Crate**: `possession` (depends on `foundation`, `registry`, `ai-gateway`)
- **Stories**: FR2.1-2.6（六种附体模式）
- **WS upgrade**: 由 B6 API crate 处理（axum），B4 只负责 mpsc channel 消息逻辑

## Plan Steps

- [x] Step 1: 创建 `rust/possession/Cargo.toml` + 更新 workspace Cargo.toml
- [x] Step 2: 创建 `rust/possession/src/lib.rs` — PossessionEngine + ModeDispatcher + entry types
- [x] Step 3: 创建 `rust/possession/src/classifier.rs` — classify_entry
- [x] Step 4: 创建 `rust/possession/src/ws.rs` — WsSessionManager
- [x] Step 5: 创建 `rust/possession/src/recovery.rs` — session 恢复
- [x] Step 6: 创建 `rust/possession/src/modes/mod.rs` + `single.rs` + `conference.rs` + `debate.rs` + `relay.rs` + `learn.rs` + `practice_opening.rs`
- [x] Step 7: `cargo check` 验证
