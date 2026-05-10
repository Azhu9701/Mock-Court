# NFR Design Patterns — B4: Possession Core

## Pattern 1: Mode Dispatch（模式路由 — 策略模式）

**问题**: 六种附体模式入口统一但处理逻辑各异。

**方案**: `classify_entry()` 返回 `EntryType`，`dispatch()` 按类型路由到对应 handler。

```
classify_entry(input: &PossessionInput) -> EntryType:
  if input.mode is set → direct map
  else → rule-based classification (soul_count, topic, etc.)

dispatch(engine, entry: EntryType, input, ws_tx) -> Result<()>:
  match entry:
    Single → single_handler(engine, input, ws_tx)
    Conference → conference_handler(engine, input, ws_tx)
    Debate → debate_handler(engine, input, ws_tx)
    Relay → relay_handler(engine, input, ws_tx)
    Learn → learn_handler(engine, input, ws_tx)
    PracticeOpening → practice_handler(engine, input, ws_tx)
```

**关键设计**: 所有 handler 共享 `PossessionEngine` 引用（store + registry + gateway），通过 `ws_tx` 发送事件。

## Pattern 2: Parallel Soul Execution（并行魂调用）

**问题**: Conference/Debate 模式需并行调用多魂 LLM，需控制并发和超时。

**方案**: `tokio::spawn` + `JoinSet` + 300s 全局超时。

```
run_parallel_souls(souls: Vec<String>, task: &str) -> Vec<SoulOutput>:
  let mut set = JoinSet::new();
  for soul in souls:
    set.spawn(async { call_single_soul(soul, task).await })
  
  let timeout = tokio::time::timeout(Duration::from_secs(300), async {
    let mut outputs = vec![];
    while let Some(result) = set.join_next().await {
      outputs.push(match result { Ok(o) => o, Err(e) => SoulOutput::error(e) });
    }
    outputs
  });
  
  match timeout.await:
    Ok(outputs) → outputs
    Err(_) → 剩余未完成的 abort，已完成的返回，未完成的标记 error
```

**限制**: 最多 10 魂并行（Q2: B），超过截断。

## Pattern 3: Session Recovery（会话恢复 — Q1: B 轻量续传）

**问题**: PossessionEngine 重启后，活跃 session 如何恢复。

**方案**: 启动时 SQLite 扫描 active sessions → 重建 WS sessions → 续传当前流。

```
recover_sessions(engine):
  1. active_sessions = store.list_sessions(status=Active)
  2. for each session:
     a. 从 messages 表读取已完成的消息元数据（不重放内容）
     b. 判断当前阶段（是否有未完成的魂调用）
     c. 如有未完成调用 → 重新发起 LLM 请求 → 等待 WS 重连后续传
     d. 如全部完成但 synthesis 未做 → 触发 synthesis
     e. 如全部完成 → 标记 Completed

handle_ws_reconnect(session_id, new_ws_tx):
  1. 查找活跃 session 的 ws_senders
  2. 替换或追加 new_ws_tx
  3. 如果有正在进行的流式输出 → 继续通过 new_ws_tx 发送后续 chunk
  4. 不重放历史消息（Q1: B）
```

**关键**: 历史消息从 SQLite messages 表按需查询（客户端通过 REST API 获取），WS 只负责实时流式。

## Pattern 4: WebSocket Channel（频道模型）

**问题**: 多魂并行时如何组织 WS 消息，避免混乱。

**方案**: per-soul `mpsc::unbounded_channel` + system 统一频道。

```
WsSessionManager {
    sessions: RwLock<HashMap<String, WsSessionState>>,
}

WsSessionState {
    soul_channels: HashMap<String, Vec<UnboundedSender<WsEvent>>>,  // "soul/{name}"
    system_channel: Vec<UnboundedSender<WsEvent>>,                   // "system"
}

broadcast_soul(session_id, soul_name, event: WsEvent):
  for tx in sessions[session_id].soul_channels[soul_name]:
    tx.send(event).ok();

broadcast_system(session_id, event: WsEvent):
  for tx in sessions[session_id].system_channel:
    tx.send(event).ok();

// 客户端断连清理
on_disconnect(session_id, ws_tx):
  从所有频道中移除该 tx
  如果所有 tx 都移除且 session 不在恢复中 → 清理 session
```

## Pattern 5: Graceful Shutdown（优雅关闭 — Q2: B）

**问题**: 服务关闭时如何处理活跃 session。

**方案**: Drain mode + wait for active calls + timeout。

```
graceful_shutdown(engine):
  1. shutdown_flag.store(true, Ordering::SeqCst)
  2. 不接受新的 start_possession 请求
  3. 向所有活跃 WS 连接发送 SystemMessage("server shutting down")
  4. 等待活跃 LLM 调用完成（已有 300s 超时保护）
  5. 300s 后强制关闭剩余连接
  6. 落盘所有未保存数据
  7. 退出
```

## Pattern 6: Stream Bridge（SSE → WS 桥接）

**问题**: B3 Gateway 返回 `UnboundedReceiver<Chunk>`，需要桥接到 WS。

**方案**: 继承 B3 的 mpsc 模式，直接桥接。

```
stream_to_ws(session_id, soul_name, mut rx: UnboundedReceiver<Chunk>, ws_tx_set):
  loop:
    match rx.recv().await:
      Some(chunk) →
        broadcast_soul(session_id, soul_name, WsEvent::SoulChunk { chunk })
      None →
        broadcast_soul(session_id, soul_name, WsEvent::SoulDone)
        return
```
