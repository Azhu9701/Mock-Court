# Functional Design Plan — B4: Possession Core

## Plan Steps

- [x] Step 1: 创建 `domain-entities.md` — PossessionInput, SoulOutput, ConferenceSession, DebateSession, RelaySession, SynthesisReport, Verdict, PractitionerInput, FieldData, DigestionReport, RevisionRecord, ActionMemo, WsEvent, EntryType
- [x] Step 2: 创建 `business-logic-model.md` — PossessionEngine 各模式编排流程
- [x] Step 3: 创建 `business-rules.md` — 魂选择、模式验证、合成触发、接力链、实践开口序列、入口分流规则

## Design Questions

### Q1: 入口分流（Entry Classification）
- [Answer]: A — EntryType = Single / Conference / Debate / Relay / Learn / PracticeOpening

### Q2: 合议模式 Synthesis 触发时机
- [Answer]: A — 所有魂回复完成后自动触发

### Q3: 接力模式（Relay）链管理
- [Answer]: A — 用户启动时指定固定链，不可中途修改

### Q4: Practice Opening P1 追问轮次
- [Answer]: B — AI 自动判断信息充分后停止（无固定轮次）

### Q5: WebSocket 事件频道
- [Answer]: C — 只有 soul 输出按频道分离，synthesis/system 统一广播
