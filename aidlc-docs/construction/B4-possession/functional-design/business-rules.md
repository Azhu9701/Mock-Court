# Business Rules — B4: Possession Core

## 1. 魂选择规则

| 规则 | 描述 |
|------|------|
| BR1.1 | 单魂模式 soul 必须在 registry 中存在，否则返回 SoulNotFound |
| BR1.2 | 合议模式 souls 数量 ≥ 2，否则降级为 Single |
| BR1.3 | 辩论模式固定使用 2 个魂，多余的忽略 |
| BR1.4 | 接力模式 soul_chain 不能为空 |
| BR1.5 | 已散魂（deleted_soul）不能被召唤 |

## 2. 模式验证规则

| 规则 | 描述 |
|------|------|
| BR2.1 | `classify_entry()` 如果 input.mode 已指定，直接返回对应 EntryType（优先级最高） |
| BR2.2 | 未指定 mode 时，soul_count==1 且无 topic → Single |
| BR2.3 | 未指定 mode 时，soul_count ≥ 2 且有 topic → Debate |
| BR2.4 | 未指定 mode 时，soul_count ≥ 2 且无 topic → Conference |
| BR2.5 | PracticeOpening 由 UI 层显式触发，不在 classify_entry 中自动推断 |

## 3. 合成触发规则（Q2: A）

| 规则 | 描述 |
|------|------|
| BR3.1 | 合议 synthesis 在所有魂并行调用完成后自动触发 |
| BR3.2 | 如果某个魂调用失败，使用错误信息占位，不阻塞 synthesis |
| BR3.3 | 辩论 verdict 在双方辩论完成后自动触发 |
| BR3.4 | synthesis/verdict 的内容也必须落盘 |

## 4. 接力链规则（Q3: A）

| 规则 | 描述 |
|------|------|
| BR4.1 | 接力链在 start_relay 时指定，运行期间不可修改 |
| BR4.2 | 每步完成后，当前魂输出作为下一魂的上下文（prev_output） |
| BR4.3 | 如果某一步魂调用失败，整个接力链中断，后续魂不执行 |
| BR4.4 | 每步输出独立落盘，失败的不落盘 |

## 5. 实践开口序列规则（Q4: B）

| 规则 | 描述 |
|------|------|
| BR5.1 | P1 由 AI 自动判断信息是否充分，无固定轮次限制 |
| BR5.2 | P1 信息充分判断标准：现场现象 + 约束条件 + 利益相关方 + 紧迫程度均已被覆盖 |
| BR5.3 | P2 只选择与现场数据 domain 匹配的魂参与消化 |
| BR5.4 | P3 每个魂独立进行自我审查和修正 |
| BR5.5 | P4 行动备忘基于所有修正记录的综合 |
| BR5.6 | P1-P4 为不可跳过的顺序流程 |

## 6. 入口分流规则

| 规则 | 描述 |
|------|------|
| BR6.1 | classify_entry 返回六种 EntryType 之一（Q1: A） |
| BR6.2 | 如果 input 中的 souls 包含不存在的魂名，在 dispatch 阶段报错（不在 classify 阶段） |
| BR6.3 | classify_entry 不访问 LLM，纯规则匹配 |

## 7. WebSocket 广播规则（Q5: C）

| 规则 | 描述 |
|------|------|
| BR7.1 | SoulChunk/SoulDone 事件按 soul_name 分发到独立频道 "soul/{name}" |
| BR7.2 | SynthesisChunk/SynthesisDone/SystemMessage/Error 统一广播到 "system" 频道 |
| BR7.3 | 每个 session 维护订阅者列表，session 结束时清理 |
| BR7.4 | 客户端断连时自动从订阅者列表移除 |

## 8. 落盘约束

| 规则 | 描述 |
|------|------|
| BR8.1 | 所有魂输出必须在流式返回完成时立即落盘（落盘先于 WsEvent::SoulDone） |
| BR8.2 | 落盘失败时不阻塞流式输出，但需 broadcast WsEvent::Error |
| BR8.3 | 每次魂调用记录一条 CallRecord（effectiveness 初始为 Invalid） |
| BR8.4 | Session 状态在全部流程完成后更新为 Completed |

## 9. 错误处理规则

| 规则 | 描述 |
|------|------|
| BR9.1 | LLM 调用失败 → broadcast WsEvent::Error，记录到 CallRecord.notes |
| BR9.2 | 落盘失败 → broadcast WsEvent::Error，不影响已返回的流式内容 |
| BR9.3 | Single/Debate 单个魂失败 → 整个 session 标记为 Inconsistent |
| BR9.4 | Conference 部分魂失败 → 不阻塞 synthesis，用错误占位符 |
| BR9.5 | Relay 中任一步失败 → 链中断，后续不执行 |
