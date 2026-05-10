# Business Logic Model — B4: Possession Core

## PossessionEngine

```rust
pub struct PossessionEngine {
    store: Arc<dyn Storage>,
    registry: Arc<SoulRegistry>,
    gateway: Arc<GatewayRegistry>,
    ws_sessions: RwLock<HashMap<String, Vec<UnboundedSender<WsEvent>>>>,
}
```

**依赖**: `Storage`（持久化）、`SoulRegistry`（魂查询）、`GatewayRegistry`（LLM 调用）

### 入口分流

```
classify_entry(input: &PossessionInput) -> EntryType
  1. 如果 input.mode 已指定 → 直接返回对应 EntryType
  2. 如果 input.mode 未指定（由 UI 层推断）：
     - 检查 input 是否包含"在场者"信号 → PracticeOpening
     - 检查 soul_count == 1 → Single
     - 检查 soul_count > 1 && 无 topic → Conference
     - 检查 soul_count == 2 && 有 topic → Debate
     - 其他 → Conference
```

### 单魂附体 (Single)

```
start_single(session_id, soul, task) -> Result<()>
  1. session = store.create_session(Single, ...).await
  2. profile = registry.get_soul(&soul)?
  3. prompt = PromptBuilder::build_summon_prompt(&profile, &task)
  4. for event in gateway.call_llm(provider, prompt, config):
       broadcast to ws channel "soul/{soul_name}" as WsEvent::SoulChunk
  5. 落盘: store.archive_soul_output(session_id, &soul, &full_content).await
  6. broadcast WsEvent::SoulDone
  7. record_call(CallRecord { effectiveness: Invalid }).await
  8. store.update_session(status: Completed).await
```

### 合议 (Conference) — Q2: A

```
start_conference(session_id, task, souls) -> Result<()>
  1. session = store.create_session(Conference, ...).await
  2. 并行: for each soul in souls:
       tokio::spawn(run_single_soul(session_id, soul, task))
  3. 等待所有魂完成（失败的魂记录 error 占位继续）
  4. 自动触发辩证综合:
       synthesis_prompt = PromptBuilder::build_synthesis_prompt(all_outputs)
       call_llm(synthesis_prompt) → broadcast to ws channel "synthesis"
  5. 落盘: store.archive_synthesis(session_id, &synthesis_content).await
  6. store.update_session(status: Completed).await
```

### 辩论 (Debate)

```
start_debate(session_id, topic, soul_a, soul_b) -> Result<()>
  1. session = store.create_session(Debate, ...).await
  2. 并行: call_llm(soul_a, debate_prompt_a) + call_llm(soul_b, debate_prompt_b)
      各自 broadcast 到 ws channel "soul/{soul_a}" 和 "soul/{soul_b}"
  3. 双方完成后，构建裁决 prompt:
       verdict = call_llm(verdict_prompt(output_a, output_b))
       broadcast 到 ws channel "synthesis"
  4. 落盘: store.archive_soul_output(session_id, soul_a, output_a)
           store.archive_soul_output(session_id, soul_b, output_b)
  5. 记录两条 CallRecord
  6. store.update_session(status: Completed).await
```

### 接力 (Relay) — Q3: A

```
start_relay(session_id, task, soul_chain: Vec<String>) -> Result<()>
  1. session = store.create_session(Relay, ...).await
  2. 固定链顺序: for i in 0..soul_chain.len():
       prev = if i > 0 { Some(outputs[i-1]) } else { None }
       prompt = build_relay_prompt(soul_chain[i], task, prev)
       output = call_llm(soul_chain[i], prompt)
       broadcast to ws channel "soul/{soul_chain[i]}"
       落盘: store.archive_soul_output(...)
       record_call(...)
  3. store.update_session(status: Completed).await
```

### 学习 (Learn)

```
start_learn(session_id, soul, task) -> Result<()>
  1. 同 Single 模式，但 prompt 附加 "作为学习伙伴，解释你的思考过程"
  2. 其他流程同 Single
```

### 实践开口 (PracticeOpening) — Q4: B

```
start_practice_opening(session_id, user_input) -> Result<()>
  1. session = store.create_session(PracticeOpening, ...).await

  P1 — 现场收集（AI 自动判断信息充分后停止）:
     loop:
       response = call_llm(collect_prompt, user_input)
       broadcast to ws channel "synthesis"
       if AI 判断信息充分 → break
       user_input = 等待用户下一轮输入

  P2 — 魂消化（并行多魂）:
     for each soul in registry.list_souls(domain_match):
       report = call_llm(soul, digest_prompt(field_data))
       落盘: store.archive_soul_output(session_id, soul, report)
       broadcast to ws channel "soul/{soul}"
     → Vec<DigestionReport>

  P3 — 修正记录:
     for each DigestionReport:
       调用魂自我审查: "你的分析中哪些可能偏颇？请修正"
       → RevisionRecord
     broadcast to ws channel "synthesis"

  P4 — 行动备忘:
     action = call_llm(synthesis_prompt(all_revisions))
     broadcast to ws channel "synthesis"
     落盘: store.archive_synthesis(session_id, &action)

  4. store.update_session(status: Completed).await
```

## WebSocket 事件广播 — Q5: C

```
broadcast(session_id, event: WsEvent)
  1. 根据 event.soul_name 确定频道:
     - Some(name) → 频道 "soul/{name}"   （按魂分离）
     - None → 频道 "system"              （synthesis/system 统一广播）
  2. 将 event JSON 发送到该 session 的所有订阅者
  3. 客户端侧: 根据 event_type 过滤渲染
```

## 数据流

```
用户输入 → PossessionInput
       → classify_entry() → EntryType
       → dispatch to mode handler
       → Gateway.call_llm() → stream chunks
       → WsEvent broadcast to ws channels
       → 落盘 (archive_soul_output / archive_synthesis)
       → CallRecord 写入
       → Session 状态更新
```
