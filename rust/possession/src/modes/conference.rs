use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ai_gateway::model_router::{ModelRouter, RoutingRole};
use ai_gateway::prompt::PromptBuilder;
use ai_gateway::GatewayRegistry;
use foundation::{
    CallConfig, KnowledgeCard, LLMRequest, Message, Prompt, PromptMessage, ReasoningEffort, Result, Storage,
};
use registry::SoulRegistry;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::sync::broadcast;
use crate::cross_detector::{CrossDetector, CollisionEvent};
use crate::soul::intervention::{InterventionGate, InterventionDecision};

use crate::stream;
use crate::tools::ToolRegistry;
use crate::{SoulOutput, UserPresets, WsEvent, WsEventType, WsSessionManager};

const MAX_PARALLEL_SOULS: usize = 10;
const SOUL_TIMEOUT_SECS: u64 = 300;
const MAX_INTERVENTION_ROUNDS: usize = 3;

// 用于魂流式输出的消息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SoulStreamMessage {
    Chunk { soul_name: String, token: String },
    Done { soul_name: String, output: SoulOutput },
    Error { soul_name: String, error: String },
}

/// 增强的合议模式，支持流式交叉检测
pub async fn run(
    store: &dyn Storage,
    registry: &SoulRegistry,
    gateway: &GatewayRegistry,
    ws: &WsSessionManager,
    session_id: &str,
    task: &str,
    souls: &[String],
    task_cards: &std::collections::HashMap<String, String>,
    presets: &UserPresets,
    system_tx: &mpsc::Sender<WsEvent>,
    tool_registry: &ToolRegistry,
) -> Result<Vec<SoulOutput>> {
    let limited: Vec<String> = souls.iter().take(MAX_PARALLEL_SOULS).cloned().collect();
    let providers = gateway.list_providers();
    let prompt_builder = PromptBuilder::new();
    let task_arc = std::sync::Arc::new(task.to_string());

    // 创建交叉检测器
    let cross_detector = CrossDetector::new();
    
    // 创建广播通道用于传输魂的流式输出给检测器
    let (chunk_tx, chunk_rx) = broadcast::channel(100);

    let mut requests: Vec<(String, LLMRequest)> = Vec::with_capacity(limited.len());
    for soul_name in &limited {
        let _ = system_tx.try_send(WsEvent {
            event_type: WsEventType::SoulStarted,
            payload: format!("正在召唤 {} ...", soul_name),
            reasoning_content: None,
            soul_name: Some(soul_name.clone()),
            seq: 0,
        }).ok();

        // 注册魂到检测器
        cross_detector.register_soul(soul_name.clone());

        match registry.get_soul(soul_name) {
            Ok(profile) => {
                // 使用模型路由器选择合适的配置
                let soul_decision = ModelRouter::route(&providers, RoutingRole::Soul);
                let (use_cache, mut config) = if let Some(decision) = &soul_decision {
                    (decision.use_cache_hint, ModelRouter::create_call_config(decision))
                } else {
                    (false, CallConfig::default())
                };
                let provider = soul_decision.as_ref().map(|d| d.provider.clone()).unwrap_or_else(|| stream::pick_provider_info(gateway).provider);
                let tier = soul_decision.as_ref().map(|d| d.tier.clone()).unwrap_or_else(|| stream::pick_provider_info(gateway).tier);

                let tool_names = crate::tools::parse_soul_tools(&profile.tools);
                if !tool_names.is_empty() {
                    let definitions = tool_registry.filter_definitions(&tool_names);
                    if !definitions.is_empty() {
                        config = config.with_tools(definitions);
                    }
                }

                let prompt = if let Some(card) = task_cards.get(soul_name) {
                    // 有差异化子任务——使用专属 task card
                    prompt_builder.build_summon_with_task_card(
                        &profile, &task_arc, card,
                        presets.judgment.as_deref(),
                        presets.worry.as_deref(),
                        presets.unknown.as_deref(),
                        tier,
                        presets.search_results.as_deref(),
                    )
                } else if use_cache {
                    prompt_builder.build_summon_cached(
                        &profile, &task_arc,
                        presets.judgment.as_deref(),
                        presets.worry.as_deref(),
                        presets.unknown.as_deref(),
                        tier,
                        presets.search_results.as_deref(),
                    )
                } else {
                    prompt_builder.build_summon_prompt(
                        &profile, &task_arc,
                        presets.judgment.as_deref(),
                        presets.worry.as_deref(),
                        presets.unknown.as_deref(),
                        tier,
                        presets.search_results.as_deref(),
                    )
                };
                requests.push((soul_name.clone(), LLMRequest {
                    provider, prompt, config,
                }));
            }
            Err(e) => {
                let _ = system_tx.try_send(WsEvent {
                    event_type: WsEventType::SoulError,
                    payload: e.to_string(),
                    reasoning_content: None,
                    soul_name: Some(soul_name.clone()),
                    seq: 0,
                }).ok();
            }
        }
    }

    let gateway_owned = GatewayRegistry::clone(gateway);
    let ws_c = ws.clone();
    let s_id = session_id.to_string();

    // ── 创建 InterventionGate + 干预通道 ──
    let intervention_gate = InterventionGate::new(Some(Arc::new(gateway_owned.clone())));
    let mut intervention_txs: HashMap<String, mpsc::Sender<InterventionDecision>> = HashMap::new();
    let mut intervention_rxs: HashMap<String, mpsc::Receiver<InterventionDecision>> = HashMap::new();
    for soul_name in &limited {
        let (tx, rx) = mpsc::channel::<InterventionDecision>(8);
        intervention_txs.insert(soul_name.clone(), tx);
        intervention_rxs.insert(soul_name.clone(), rx);
    }

    // 启动碰撞检测任务（带实时干预门控）
    let detector = cross_detector.clone();
    let ws_clone = ws_c.clone();
    let session_id_clone = s_id.clone();
    let system_tx_clone = system_tx.clone();
    let _collision_handle = tokio::spawn(async move {
        detect_collisions_async(
            detector, chunk_rx, ws_clone, session_id_clone,
            system_tx_clone, intervention_gate, intervention_txs,
        ).await;
    });

    let mut set = JoinSet::new();
    for (soul_name, req) in requests {
        let s_id = session_id.to_string();
        let ws_c = ws.clone();
        let gw = gateway_owned.clone();
        let chunk_tx_clone = chunk_tx.clone();
        let tr = tool_registry.clone();
        let provider = req.provider.clone();
        let prompt = req.prompt.clone();
        let config = req.config.clone();
        let irx = intervention_rxs.remove(&soul_name).unwrap_or_else(|| {
            let (_, rx) = mpsc::channel(1);
            rx
        });
        set.spawn(async move {
            run_soul_with_tools(
                &gw, &provider, &prompt, &config,
                &s_id, &soul_name, &ws_c, &tr,
                chunk_tx_clone, irx,
            ).await
        });
    }

    let collect = async {
        let mut acc = Vec::with_capacity(limited.len());
        while let Some(r) = set.join_next().await {
            match r {
                Ok(output) => acc.push(output),
                Err(e) => acc.push(SoulOutput::error("unknown".into(), e.to_string())),
            }
        }
        acc
    };

    let outputs = match tokio::time::timeout(Duration::from_secs(SOUL_TIMEOUT_SECS), collect).await {
        Ok(acc) => acc,
        Err(_) => {
            set.abort_all();
            tracing::warn!("Conference timed out after {}s", SOUL_TIMEOUT_SECS);
            let _ = system_tx.send(WsEvent {
                event_type: WsEventType::SystemMessage,
                payload: format!("⚠️ 合议超时（{}秒），{} 个魂的回应可能不完整", SOUL_TIMEOUT_SECS, limited.len()),
                reasoning_content: None,
                soul_name: None,
                seq: 0,
            });
            vec![]
        }
    };

    for output in &outputs {
        crate::emit_soul_cost(system_tx, &output.soul_name, &output.usage, None);
        if let Err(e) = crate::finalize_output(store, session_id, output, foundation::PossessionMode::Conference, task).await {
            tracing::error!("Failed to finalize {} output: {}", output.soul_name, e);
        }
    }

    let synthesis_outputs: Vec<(String, String)> = outputs
        .iter()
        .filter(|o| o.error.is_none())
        .map(|o| (o.soul_name.clone(), o.content.clone()))
        .collect();

    if !synthesis_outputs.is_empty() {
        let _ = system_tx.try_send(WsEvent {
            event_type: WsEventType::SynthesisStarted,
            payload: "辩证综合开始...".into(),
            reasoning_content: None,
            soul_name: None,
            seq: 0,
        }).ok();

        // 收集碰撞检测结果
        let collisions = cross_detector.get_collisions();
        let collision_summary = if collisions.is_empty() {
            String::new()
        } else {
            let mut summary = String::from("## 检测到的碰撞事件\n\n");
            for c in &collisions {
                summary.push_str(&format!(
                    "- **{}** → **{}**（{:?}）：{}\n",
                    c.from_soul, c.to_soul, c.collision_type, c.content
                ));
            }
            summary.push_str("\n请综合考虑以上碰撞信息——它们可能揭示了各魂在思考过程中真实发生的结构冲突。\n");
            summary
        };

        // 为综合官选择合适的配置
        let synthesis_decision = ModelRouter::route(&providers, RoutingRole::Synthesizer);
        let (synthesis_provider, synthesis_config) = if let Some(decision) = &synthesis_decision {
            (decision.provider.clone(), ModelRouter::create_call_config(decision))
        } else {
            (stream::pick_provider_info(gateway).provider, CallConfig::default().with_reasoning_effort(ReasoningEffort::ThinkMax))
        };
        let card_decision = ModelRouter::route(&providers, RoutingRole::KnowledgeCard);
        let card_provider = card_decision.as_ref().map(|d| d.provider.clone()).unwrap_or_else(|| synthesis_provider.clone());

        let synthesis_prompt = if collision_summary.is_empty() {
            prompt_builder.build_synthesis_prompt(task, &synthesis_outputs)
        } else {
            prompt_builder.build_synthesis_with_collisions(task, &synthesis_outputs, &collision_summary)
        };
        let synthesis_req = LLMRequest { provider: synthesis_provider, prompt: synthesis_prompt, config: synthesis_config };

        if let Ok(rx) = gateway.call(&synthesis_req) {
            if let Ok((content, synth_usage)) = stream::stream_synthesis(rx, session_id, ws).await {
                let per_soul_costs: Vec<(String, foundation::UsageStats, Option<String>)> = outputs.iter()
                    .filter(|o| o.error.is_none())
                    .map(|o| (o.soul_name.clone(), o.usage.clone(), None))
                    .collect();
                crate::emit_session_cost(system_tx, &per_soul_costs, Some(synth_usage.total_tokens));

                if let Err(e) = store.archive_synthesis(session_id, &content).await {
                    tracing::error!("Failed to archive synthesis: {}", e);
                }
                let msg = Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    role: foundation::MessageRole::Synthesis,
                    soul_name: None,
                    content: content.clone(),
                    seq: 0,
                    created_at: chrono::Utc::now(),
                };
                if let Err(e) = store.append_message(&msg).await {
                    tracing::error!("Failed to store synthesis message: {}", e);
                }

                let card_content = content;
                let card_prompt = Prompt {
                    messages: vec![PromptMessage {
                        role: "user".into(),
                        content: format!("从以下辩证综合报告中提取最核心的 ≤500 字的卡片：\n\n{}", if card_content.len() > 3000 { &card_content[..3000] } else { &card_content }),
                        reasoning_content: None,
                        ..Default::default()
                    }],
                };
                let mut card_config = CallConfig { temperature: 0.3, max_tokens: 256, stream: false, ..Default::default() };
                if let Some(decision) = &card_decision {
                    card_config = ModelRouter::create_call_config(decision);
                    card_config.stream = false;
                }
                let card_req = LLMRequest { provider: card_provider, prompt: card_prompt, config: card_config };
                if let Ok(mut card_rx) = gateway.call(&card_req) {
                    let mut card = String::new();
                    while let Some(r) = card_rx.recv().await {
                        if let Ok(c) = r { card.push_str(&c.content); }
                    }
                    if !card.is_empty() {
                        let card_entity = KnowledgeCard {
                            id: uuid::Uuid::new_v4().to_string(),
                            title: task.to_string(),
                            content: card.clone(),
                            source_soul: None,
                            source_session: Some(session_id.to_string()),
                            tags: souls.to_vec(),
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        };
                        let _ = store.insert_knowledge_card(&card_entity).await;
                    }
                }
            }
        }

        // ── 24小时可检验项 ──
        let verify_msg = Message {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            role: foundation::MessageRole::System,
            soul_name: Some("实践检验".into()),
            content: "## ⏳ 24小时可检验项\n\n综合以上分析，请设定一个**在未来24小时内可以实际检验的具体行动**：\n\n- 你准备做什么来验证（或挑战）这次分析的结论？\n- 你预计什么信号表示\"分析有效\"？什么信号表示\"分析需要修正\"？\n- 检验后，请在下次附体时通过实践开口带回现场数据。\n\n**如果24小时内不检验——本次分析的结论标记为\"待验证\"而非\"已确认\"。**".into(),
            seq: 2,
            created_at: chrono::Utc::now(),
        };
        let _ = store.append_message(&verify_msg).await;
        let _ = system_tx.try_send(WsEvent {
            event_type: WsEventType::SystemMessage,
            payload: "⏳ 请设定一个24小时内可检验的具体行动，验证本次分析的结论。".into(),
            reasoning_content: None,
            soul_name: None,
            seq: 0,
        }).ok();
    }

    Ok(outputs)
}

async fn run_soul_with_tools(
    gw: &GatewayRegistry,
    provider: &foundation::Provider,
    prompt: &foundation::Prompt,
    config: &foundation::CallConfig,
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
    tool_registry: &crate::tools::ToolRegistry,
    chunk_tx: broadcast::Sender<SoulStreamMessage>,
    mut intervention_rx: mpsc::Receiver<InterventionDecision>,
) -> SoulOutput {
    let max_rounds = crate::tools::max_tool_rounds();
    let mut history: Vec<foundation::PromptMessage> = prompt.messages.clone();
    let name = soul_name.to_string();
    let mut intervention_count = 0usize;

    for _round in 0..max_rounds {
        let req = foundation::LLMRequest {
            provider: provider.clone(),
            prompt: foundation::Prompt { messages: history.clone() },
            config: config.clone(),
        };

        let rx = match gw.call(&req) {
            Ok(rx) => rx,
            Err(e) => {
                let _ = chunk_tx.send(SoulStreamMessage::Error {
                    soul_name: name.clone(),
                    error: e.to_string(),
                });
                return SoulOutput::error(name, e.to_string());
            }
        };

        let outcome = stream_single_soul_with_detection(
            rx, session_id, &name, ws, chunk_tx.clone(), &mut intervention_rx,
        ).await;

        match outcome {
            StreamOutcome::Interrupted { partial_content, partial_tool_calls, intervention } => {
                intervention_count += 1;

                // 注入部分输出到历史
                if !partial_content.is_empty() {
                    history.push(foundation::PromptMessage {
                        role: "assistant".to_string(),
                        content: partial_content,
                        reasoning_content: None,
                        tool_calls: if partial_tool_calls.is_empty() { None } else { Some(partial_tool_calls) },
                        tool_call_id: None,
                    });
                }

                // 将干预转为 user 消息注入，迫使魂在下轮推理中回应
                let intervention_msg = intervention_to_message(&intervention);
                history.push(foundation::PromptMessage {
                    role: "user".to_string(),
                    content: intervention_msg,
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                });

                // 干预轮次上限保护，避免死循环
                if intervention_count >= MAX_INTERVENTION_ROUNDS {
                    tracing::warn!(
                        "Soul '{}' reached max intervention rounds ({}), forcing completion",
                        name, MAX_INTERVENTION_ROUNDS
                    );
                    // 最后再调一次 LLM 并正常返回
                    let final_req = foundation::LLMRequest {
                        provider: provider.clone(),
                        prompt: foundation::Prompt { messages: history.clone() },
                        config: config.clone(),
                    };
                    if let Ok(final_rx) = gw.call(&final_req) {
                        let final_outcome = stream_single_soul_with_detection(
                            final_rx, session_id, &name, ws, chunk_tx.clone(), &mut intervention_rx,
                        ).await;
                        if let StreamOutcome::Completed(output) = final_outcome {
                            return output;
                        }
                    }
                    return SoulOutput {
                        soul_name: name,
                        content: history.iter()
                            .filter(|m| m.role == "assistant")
                            .map(|m| m.content.clone())
                            .collect::<Vec<_>>()
                            .join("\n\n"),
                        usage: foundation::UsageStats::default(),
                        error: None,
                        tool_calls: Vec::new(),
                    };
                }

                // 继续循环，用带干预消息的 history 重新调 LLM
                tracing::info!(
                    "Soul '{}' restarting inference after intervention #{}, history_len={}",
                    name, intervention_count, history.len()
                );
                continue;
            }

            StreamOutcome::Completed(output) => {
                if output.error.is_some() {
                    return output;
                }

                if output.tool_calls.is_empty() {
                    return output;
                }

                // 处理工具调用
                for tc in &output.tool_calls {
                    let payload = crate::ToolCallPayload {
                        tool_call_id: tc.id.clone(),
                        tool_name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                        soul_name: name.clone(),
                    };
                    let json = serde_json::to_string(&payload).unwrap_or_default();
                    ws.broadcast_soul(session_id, &name, &WsEvent {
                        event_type: WsEventType::ToolCallStarted,
                        payload: json,
                        reasoning_content: None,
                        soul_name: Some(name.clone()),
                        seq: 0,
                    });

                    match tool_registry.execute(tc).await {
                        Ok(result) => {
                            let result_payload = crate::ToolResultPayload {
                                tool_call_id: tc.id.clone(),
                                tool_name: tc.function.name.clone(),
                                result: result.clone(),
                                soul_name: name.clone(),
                            };
                            let result_json = serde_json::to_string(&result_payload).unwrap_or_default();
                            ws.broadcast_soul(session_id, &name, &WsEvent {
                                event_type: WsEventType::ToolResult,
                                payload: result_json,
                                reasoning_content: None,
                                soul_name: Some(name.clone()),
                                seq: 0,
                            });

                            history.push(foundation::PromptMessage {
                                role: "assistant".to_string(),
                                content: String::new(),
                                reasoning_content: None,
                                tool_calls: Some(output.tool_calls.clone()),
                                tool_call_id: None,
                            });
                            history.push(foundation::PromptMessage {
                                role: "tool".to_string(),
                                content: result,
                                reasoning_content: None,
                                tool_calls: None,
                                tool_call_id: Some(tc.id.clone()),
                            });
                        }
                        Err(e) => {
                            return SoulOutput::error(name, format!("Tool {} failed: {}", tc.function.name, e));
                        }
                    }
                }
            }
        }
    }

    SoulOutput {
        soul_name: name,
        content: String::new(),
        usage: foundation::UsageStats::default(),
        error: Some(format!("Tool call loop exceeded {} rounds", max_rounds)),
        tool_calls: Vec::new(),
    }
}

/// 将干预决策转为注入魂对话的 user 消息
fn intervention_to_message(decision: &InterventionDecision) -> String {
    match decision {
        InterventionDecision::InjectQuestion { question } => {
            format!(
                "⚠️ [系统干预] 检测到同行魂对你的立场有冲突。{}\n\n请在你的下一段输出中回应这个问题——不要无视它。",
                question
            )
        }
        InterventionDecision::Redirect { target } => {
            format!(
                "⚠️ [系统干预] 你的输出与同行魂高度重叠（冗余）。{}\n\n请从不同角度重新切入，避免重复已有观点。",
                target
            )
        }
        InterventionDecision::DeepenRequest { aspect, reason } => {
            format!(
                "⚠️ [系统干预] 同行魂尚未覆盖维度「{}」。{}\n\n请展开这一维度，提供更深的洞察。",
                aspect, reason
            )
        }
        InterventionDecision::NoAction => String::new(),
    }
}

/// 流式输出结果：完成或被干预打断
enum StreamOutcome {
    Completed(SoulOutput),
    Interrupted {
        partial_content: String,
        partial_tool_calls: Vec<foundation::ToolCall>,
        intervention: InterventionDecision,
    },
}

/// 流式输出单个魂，同时发送给交叉检测器，并监听实时干预信号
async fn stream_single_soul_with_detection(
    mut rx: mpsc::Receiver<foundation::Result<foundation::Chunk>>,
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
    chunk_tx: broadcast::Sender<SoulStreamMessage>,
    intervention_rx: &mut mpsc::Receiver<InterventionDecision>,
) -> StreamOutcome {
    let mut content = String::new();
    let mut usage = foundation::UsageStats::default();
    let mut seq: u32 = 0;
    let name = soul_name.to_string();
    let mut tool_calls: Vec<foundation::ToolCall> = Vec::new();

    loop {
        tokio::select! {
            // 偏向 LLM token 流（保证推理不被干预饿死）
            biased;

            result = rx.recv() => {
                match result {
                    Some(Ok(chunk)) => {
                        if let Some(u) = chunk.usage {
                            usage = u;
                        }
                        if !chunk.tool_calls.is_empty() {
                            tool_calls.extend(chunk.tool_calls);
                        }
                        if !chunk.content.is_empty() {
                            content.push_str(&chunk.content);

                            ws.broadcast_soul(
                                session_id,
                                &name,
                                &WsEvent {
                                    event_type: WsEventType::SoulChunk,
                                    payload: chunk.content.clone(),
                                    reasoning_content: chunk.reasoning_content.clone(),
                                    soul_name: Some(name.clone()),
                                    seq,
                                },
                            );

                            let _ = chunk_tx.send(SoulStreamMessage::Chunk {
                                soul_name: name.clone(),
                                token: chunk.content,
                            });

                            seq += 1;
                        } else if let Some(ref reasoning) = chunk.reasoning_content {
                            if !reasoning.is_empty() && seq == 0 {
                                ws.broadcast_soul(
                                    session_id,
                                    &name,
                                    &WsEvent {
                                        event_type: WsEventType::SoulChunk,
                                        payload: String::new(),
                                        reasoning_content: Some(reasoning.clone()),
                                        soul_name: Some(name.clone()),
                                        seq,
                                    },
                                );
                                seq += 1;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        ws.broadcast_soul(
                            session_id,
                            &name,
                            &WsEvent {
                                event_type: WsEventType::SoulError,
                                payload: e.to_string(),
                                reasoning_content: None,
                                soul_name: Some(name.clone()),
                                seq,
                            },
                        );
                        let _ = chunk_tx.send(SoulStreamMessage::Error {
                            soul_name: name.clone(),
                            error: e.to_string(),
                        });
                        let output = SoulOutput::error(name, e.to_string());
                        return StreamOutcome::Completed(output);
                    }
                    None => {
                        // LLM 流正常结束
                        break;
                    }
                }
            }

            intervention = intervention_rx.recv() => {
                match intervention {
                    Some(decision) => {
                        tracing::info!(
                            "Soul '{}' interrupted mid-stream with {:?}, partial_content={} chars",
                            name, decision, content.len()
                        );
                        let system_tx = chunk_tx.clone();
                        let _ = system_tx.send(SoulStreamMessage::Chunk {
                            soul_name: name.clone(),
                            token: format!("\n\n[干预信号: {:?}]\n", decision),
                        });
                        return StreamOutcome::Interrupted {
                            partial_content: content,
                            partial_tool_calls: tool_calls,
                            intervention: decision,
                        };
                    }
                    None => {
                        // 干预通道关闭，继续等待 LLM
                        continue;
                    }
                }
            }
        }
    }

    ws.broadcast_soul(
        session_id,
        &name,
        &WsEvent {
            event_type: WsEventType::SoulDone,
            payload: String::new(),
            reasoning_content: None,
            soul_name: Some(name.clone()),
            seq,
        },
    );

    let output = SoulOutput { soul_name: name.clone(), content, usage, error: None, tool_calls };
    let _ = chunk_tx.send(SoulStreamMessage::Done {
        soul_name: name,
        output: output.clone(),
    });

    StreamOutcome::Completed(output)
}

/// 异步碰撞检测任务 — 碰撞检出后实时触发 L1→L2→L3 门控干预
async fn detect_collisions_async(
    detector: CrossDetector,
    mut chunk_rx: broadcast::Receiver<SoulStreamMessage>,
    ws: WsSessionManager,
    session_id: String,
    system_tx: mpsc::Sender<WsEvent>,
    intervention_gate: InterventionGate,
    intervention_txs: HashMap<String, mpsc::Sender<InterventionDecision>>,
) {
    loop {
        match chunk_rx.recv().await {
            Ok(msg) => {
                match msg {
                    SoulStreamMessage::Chunk { soul_name, token } => {
                        detector.add_token(&soul_name, &token);

                        let collisions = detector.detect_collisions();
                        for collision in collisions {
                            broadcast_collision(&ws, &session_id, &collision, &system_tx);

                            // ── 实时干预：碰撞检出后跑三级门控 ──
                            if let Some(tx) = intervention_txs.get(&collision.to_soul) {
                                let from_ctx = detector.get_soul_context(&collision.from_soul);
                                let to_ctx = detector.get_soul_context(&collision.to_soul);

                                let from_text = from_ctx.unwrap_or_default();
                                let to_text = to_ctx.unwrap_or_default();

                                let peer_outputs: Vec<String> = vec![from_text];
                                let gate = intervention_gate.clone();
                                let tx = tx.clone();
                                let from_soul = collision.from_soul.clone();
                                let contradiction = collision.content.clone();

                                tokio::spawn(async move {
                                    let decision = gate.gate(&to_text, &peer_outputs).await;
                                    match &decision {
                                        InterventionDecision::InjectQuestion { .. }
                                        | InterventionDecision::Redirect { .. }
                                        | InterventionDecision::DeepenRequest { .. } => {
                                            tracing::info!(
                                                "Intervention triggered: {:?} → soul '{}'",
                                                decision,
                                                collision.to_soul,
                                            );
                                            let _ = tx.send(decision).await;
                                        }
                                        InterventionDecision::NoAction => {}
                                    }
                                });
                            }
                        }
                    }
                    SoulStreamMessage::Done { .. } => {}
                    SoulStreamMessage::Error { .. } => {}
                }
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                tracing::warn!("Collision detector lagged behind, some chunks may be missed");
            }
        }
    }
}

/// 广播碰撞事件到前端
fn broadcast_collision(
    ws: &WsSessionManager,
    session_id: &str,
    collision: &CollisionEvent,
    system_tx: &mpsc::Sender<WsEvent>,
) {
    let payload = serde_json::json!({
        "collision_type": collision.collision_type,
        "from": collision.from_soul,
        "to": collision.to_soul,
        "content": collision.content,
        "trigger_keywords": collision.trigger_keywords,
        "injected": collision.injected,
    });
    
    let event = WsEvent {
        event_type: WsEventType::Collision,
        payload: payload.to_string(),
        reasoning_content: None,
        soul_name: None,
        seq: 0,
    };
    
    ws.broadcast_system(session_id, &event);
    let _ = system_tx.try_send(event).ok();
    
    tracing::info!("Broadcast collision: {} -> {} ({:?})", collision.from_soul, collision.to_soul, collision.collision_type);
}

fn estimate_cost(provider: foundation::Provider, total_tokens: u32, apply_cache_discount: bool) -> String {
    let price_per_1m: f64 = match provider {
        foundation::Provider::Claude => 15.0,     // Sonnet pricing
        foundation::Provider::OpenAI => 10.0,      // GPT-4o pricing
        foundation::Provider::DeepSeek => 2.0,     // DeepSeek pricing
    };
    
    let mut cost = total_tokens as f64 / 1_000_000.0 * price_per_1m;
    
    // 应用缓存折扣（DeepSeek 有缓存优惠）
    if apply_cache_discount && matches!(provider, foundation::Provider::DeepSeek) {
        // 假设 50% 的 token 命中缓存，缓存价格是原价的 10%
        cost = cost * 0.55; // 0.5 * 0.1 + 0.5 * 1.0 = 0.55
    }
    
    format!("${:.4}", cost)
}
