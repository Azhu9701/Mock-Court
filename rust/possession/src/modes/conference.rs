use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

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
use super::topology;

const MAX_PARALLEL_SOULS: usize = 10;
const SOUL_TIMEOUT_SECS: u64 = 300;
const _MAX_INTERVENTION_ROUNDS: usize = 3;

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
    let _providers = gateway.list_providers();
    let prompt_builder = PromptBuilder::new();
    let task_arc = std::sync::Arc::new(task.to_string());

    // 创建交叉检测器
    let cross_detector = CrossDetector::new();
    
    // 创建广播通道用于传输魂的流式输出给检测器
    let (chunk_tx, chunk_rx) = broadcast::channel(100);

    let mut requests: Vec<(String, LLMRequest)> = Vec::with_capacity(limited.len());
    let mut profiles: Vec<foundation::SoulProfile> = Vec::with_capacity(limited.len());
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
                // 使用用户配置的 provider，不再硬编码 DeepSeek 优先
                let provider = gateway.pick_provider().unwrap_or(foundation::Provider::DeepSeek);
                let mut config = CallConfig::default().with_reasoning_effort(ReasoningEffort::Think);
                let tier = foundation::ModelTier::for_provider(&provider, config.model.as_deref().unwrap_or("unknown"));

                let tool_names = crate::tools::parse_soul_tools(&profile.tools);
                if !tool_names.is_empty() {
                    let definitions = tool_registry.filter_definitions(&tool_names);
                    if !definitions.is_empty() {
                        config = config.with_tools(definitions);
                    }
                }

                let prompt = if let Some(card) = task_cards.get(soul_name) {
                    prompt_builder.build_summon_with_task_card(
                        &profile, &task_arc, card,
                        presets.judgment.as_deref(),
                        presets.worry.as_deref(),
                        presets.unknown.as_deref(),
                        tier,
                        presets.search_results.as_deref(),
                        presets.interrogation_context.as_deref(),
                    )
                } else {
                    prompt_builder.build_summon_prompt(
                        &profile, &task_arc,
                        presets.judgment.as_deref(),
                        presets.worry.as_deref(),
                        presets.unknown.as_deref(),
                        tier,
                        presets.search_results.as_deref(),
                        presets.interrogation_context.as_deref(),
                    )
                };
                profiles.push(profile.clone());
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

    // ── 拓扑规划：根据任务复杂度和魂多样性选择最优编排策略 ──
    let planner = topology::TopologyPlanner::new();
    let selected_topology = topology::plan_from_profiles(
        &planner, &profiles, task, false,
    );
    let _ = system_tx.try_send(WsEvent {
        event_type: WsEventType::ProcessStep,
        payload: format!(
            "拓扑规划: {} (参与{}魂, 预估{}次LLM调用)",
            selected_topology.describe(),
            selected_topology.soul_count(),
            selected_topology.estimated_calls()
        ),
        reasoning_content: None,
        soul_name: None,
        seq: 0,
    }).ok();

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
    let limited_for_monitor = limited.clone();
    let mut set = JoinSet::new();

    set.spawn(async move {
        detect_collisions_async(
            detector, chunk_rx, ws_clone, session_id_clone,
            system_tx_clone, intervention_gate, intervention_txs,
            selected_topology, limited_for_monitor,
        ).await;
        SoulOutput::error("collision_detector".into(), "task completed".into())
    });

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

    let expected_soul_count = limited.len();
    let collect = async {
        let mut acc = Vec::with_capacity(expected_soul_count);
        while let Some(r) = set.join_next().await {
            match r {
                Ok(output) => {
                    if output.soul_name != "collision_detector" {
                        acc.push(output);
                    }
                    if acc.len() >= expected_soul_count {
                        break;
                    }
                }
                Err(e) => {
                    acc.push(SoulOutput::error("unknown".into(), e.to_string()));
                }
            }
        }
        acc
    };

    let outputs = match tokio::time::timeout(Duration::from_secs(SOUL_TIMEOUT_SECS), collect).await {
        Ok(acc) => {
            set.abort_all();
            acc
        }
        Err(_) => {
            tracing::warn!("Conference timed out after {}s, collecting completed results", SOUL_TIMEOUT_SECS);
            let mut acc = Vec::new();
            loop {
                match tokio::time::timeout(Duration::from_millis(50), set.join_next()).await {
                    Ok(Some(Ok(output))) => {
                        if output.soul_name != "collision_detector" {
                            acc.push(output);
                        }
                    }
                    _ => break,
                }
            }
            set.abort_all();
            let _ = system_tx.send(WsEvent {
                event_type: WsEventType::SystemMessage,
                payload: format!("⚠️ 合议超时（{}秒），已保留 {}/{} 个魂的回应", SOUL_TIMEOUT_SECS, acc.len(), limited.len()),
                reasoning_content: None,
                soul_name: None,
                seq: 0,
            });
            acc
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
        let synthesis_provider = gateway.pick_provider().unwrap_or(foundation::Provider::DeepSeek);
        let synthesis_config = CallConfig::default().with_reasoning_effort(ReasoningEffort::ThinkMax);
        let card_provider = synthesis_provider.clone();

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

                if synth_usage.total_tokens > 0 {
                    let _ = store.record_call(&foundation::CallRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        session_id: session_id.to_string(),
                        soul_name: "综合官".to_string(),
                        mode: foundation::PossessionMode::Conference,
                        task_summary: task.to_string(),
                        effectiveness: foundation::Effectiveness::Effective,
                        notes: "[synthesis]".to_string(),
                        created_at: chrono::Utc::now(),
                        self_negation: None,
                        empty_chair: None,
                        user_feedback: None,
                        usage: synth_usage,
                    }).await;
                }

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

                // ── 提取推荐补充魂 ──
                let recommendations = extract_recommended_souls(&content);
                if !recommendations.is_empty() {
                    let payload = serde_json::json!({
                        "recommendations": recommendations,
                    });
                    let _ = system_tx.try_send(WsEvent {
                        event_type: WsEventType::SoulRecommendations,
                        payload: payload.to_string(),
                        reasoning_content: None,
                        soul_name: None,
                        seq: 0,
                    }).ok();
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
                let card_config = CallConfig { temperature: 0.3, max_tokens: 256, stream: false, ..Default::default() };
                let card_req = LLMRequest { provider: card_provider, prompt: card_prompt, config: card_config };

                // ── Knowledge card + Annotation pass: parallel LLM calls ──
                let annotation_prompt = prompt_builder.build_annotation_prompt(task, &synthesis_outputs);
                let annotation_req = LLMRequest {
                    provider: synthesis_provider.clone(),
                    prompt: annotation_prompt,
                    config: CallConfig { temperature: 0.4, max_tokens: 4096, stream: false, ..Default::default() },
                };

                let card_fut = async {
                    if let Ok(mut card_rx) = gateway.call(&card_req) {
                        let mut card = String::new();
                        let mut card_usage = foundation::UsageStats::default();
                        while let Some(r) = card_rx.recv().await {
                            if let Ok(c) = r {
                                if let Some(u) = c.usage { card_usage = u; }
                                card.push_str(&c.content);
                            }
                        }
                        if !card.is_empty() {
                            let card_entity = KnowledgeCard {
                                id: uuid::Uuid::new_v4().to_string(),
                                title: task.to_string(),
                                content: card,
                                source_soul: None,
                                source_session: Some(session_id.to_string()),
                                tags: souls.to_vec(),
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            };
                            let _ = store.insert_knowledge_card(&card_entity).await;
                        }
                        if card_usage.total_tokens > 0 {
                            let _ = store.record_call(&foundation::CallRecord {
                                id: uuid::Uuid::new_v4().to_string(),
                                session_id: session_id.to_string(),
                                soul_name: "综合官".to_string(),
                                mode: foundation::PossessionMode::Conference,
                                task_summary: task.to_string(),
                                effectiveness: foundation::Effectiveness::Effective,
                                notes: "[knowledge_card]".to_string(),
                                created_at: chrono::Utc::now(),
                                self_negation: None,
                                empty_chair: None,
                                user_feedback: None,
                                usage: card_usage,
                            }).await;
                        }
                    }
                };

                let ann_fut = async {
                    if let Ok(mut ann_rx) = gateway.call(&annotation_req) {
                    let mut raw = String::new();
                    let mut ann_usage = foundation::UsageStats::default();
                    while let Some(r) = ann_rx.recv().await {
                        if let Ok(c) = r {
                            if let Some(u) = c.usage { ann_usage = u; }
                            raw.push_str(&c.content);
                        }
                    }
                    if ann_usage.total_tokens > 0 {
                        let _ = store.record_call(&foundation::CallRecord {
                            id: uuid::Uuid::new_v4().to_string(),
                            session_id: session_id.to_string(),
                            soul_name: "批注官".to_string(),
                            mode: foundation::PossessionMode::Conference,
                            task_summary: task.to_string(),
                            effectiveness: foundation::Effectiveness::Effective,
                            notes: "[annotation]".to_string(),
                            created_at: chrono::Utc::now(),
                            self_negation: None,
                            empty_chair: None,
                            user_feedback: None,
                            usage: ann_usage,
                        }).await;
                    }
                    let trimmed = raw.trim();
                    let json_str = trimmed
                        .trim_start_matches("```json")
                        .trim_start_matches("```")
                        .trim_end_matches("```")
                        .trim();
                    match serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                        Ok(items) => {
                            let now = chrono::Utc::now();
                            let annotations: Vec<foundation::Annotation> = items.iter().filter_map(|v| {
                                Some(foundation::Annotation {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    session_id: session_id.to_string(),
                                    source_soul: v.get("source_soul")?.as_str()?.to_string(),
                                    target_soul: v.get("target_soul")?.as_str()?.to_string(),
                                    target_excerpt: v.get("target_excerpt")?.as_str()?.to_string(),
                                    comment: v.get("comment")?.as_str()?.to_string(),
                                    kind: v.get("kind").and_then(|k| k.as_str()).unwrap_or("nuance").to_string(),
                                    created_at: now,
                                })
                            }).collect();
                            if !annotations.is_empty() {
                                match store.insert_annotations(&annotations).await {
                                    Ok(_) => {
                                        let payload = serde_json::to_string(&annotations)
                                            .unwrap_or_else(|_| "[]".to_string());
                                        let _ = system_tx.try_send(WsEvent {
                                            event_type: WsEventType::AnnotationsReady,
                                            payload,
                                            reasoning_content: None,
                                            soul_name: None,
                                            seq: 0,
                                        }).ok();
                                        tracing::info!(
                                            "Persisted {} marginalia annotations for session {}",
                                            annotations.len(),
                                            session_id
                                        );
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to persist annotations: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse annotation JSON: {} (raw len={})", e, raw.len());
                        }
                    }
                    }
                };

                tokio::join!(card_fut, ann_fut);
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
    let mut _intervention_count = 0usize;
    let mut used_providers: Vec<foundation::Provider> = vec![provider.clone()];
    let mut current_provider = provider.clone();

    for _round in 0..max_rounds {
        let req = foundation::LLMRequest {
            provider: current_provider.clone(),
            prompt: foundation::Prompt { messages: history.clone() },
            config: config.clone(),
        };

        let rx = match gw.call(&req) {
            Ok(rx) => rx,
            Err(e) => {
                let error_msg = e.to_string();
                gw.mark_provider_unhealthy(&current_provider, error_msg.clone());

                if let Some(next_provider) = gw.try_next_provider(&current_provider) {
                    if !used_providers.contains(&next_provider) {
                        tracing::warn!(
                            "Soul '{}' provider {:?} failed, trying fallback {:?}: {}",
                            name, current_provider, next_provider, error_msg
                        );
                        used_providers.push(next_provider);
                        current_provider = next_provider;
                        continue;
                    }
                }

                ws.broadcast_soul(
                    session_id,
                    &name,
                    &WsEvent {
                        event_type: WsEventType::SoulError,
                        payload: error_msg.clone(),
                        reasoning_content: None,
                        soul_name: Some(name.clone()),
                        seq: 0,
                    },
                );
                let _ = chunk_tx.send(SoulStreamMessage::Error {
                    soul_name: name.clone(),
                    error: error_msg.clone(),
                });
                return SoulOutput::error(name, error_msg);
            }
        };

        let outcome = stream_single_soul_with_detection_with_provider(
            rx,
            session_id,
            &name,
            ws,
            chunk_tx.clone(),
            &mut intervention_rx,
            &current_provider,
            gw,
            &used_providers,
        ).await;

        match outcome {
            StreamOutcome::Completed(output) => {
                if output.error.is_some() {
                    if let Some(next_provider) = gw.try_next_provider(&current_provider) {
                        if !used_providers.contains(&next_provider) {
                            tracing::warn!(
                                "Soul '{}' provider {:?} stream failed, trying fallback {:?}",
                                name, current_provider, next_provider
                            );
                            used_providers.push(next_provider);
                            current_provider = next_provider;
                            continue;
                        }
                    }
                    return output;
                }

                if output.tool_calls.is_empty() {
                    gw.mark_provider_healthy(&current_provider);
                    return output;
                }

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
                                reasoning_content: Some(String::new()),
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
                            let error_msg = format!("Tool {} failed: {}", tc.function.name, e);
                            ws.broadcast_soul(
                                session_id,
                                &name,
                                &WsEvent {
                                    event_type: WsEventType::SoulError,
                                    payload: error_msg.clone(),
                                    reasoning_content: None,
                                    soul_name: Some(name.clone()),
                                    seq: 0,
                                },
                            );
                            return SoulOutput::error(name, error_msg);
                        }
                    }
                }
            }
        }
    }

    let error_msg = format!("Tool call loop exceeded {} rounds", max_rounds);
    ws.broadcast_soul(
        session_id,
        &name,
        &WsEvent {
            event_type: WsEventType::SoulError,
            payload: error_msg.clone(),
            reasoning_content: None,
            soul_name: Some(name.clone()),
            seq: 0,
        },
    );
    SoulOutput {
        soul_name: name,
        content: String::new(),
        usage: foundation::UsageStats::default(),
        error: Some(error_msg),
        tool_calls: Vec::new(),
    }
}

/// 将干预决策转为注入魂对话的 user 消息
fn _intervention_to_message(decision: &InterventionDecision) -> String {
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

enum StreamOutcome {
    Completed(SoulOutput),
}

async fn stream_single_soul_with_detection_with_provider(
    mut rx: mpsc::Receiver<foundation::Result<foundation::Chunk>>,
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
    chunk_tx: broadcast::Sender<SoulStreamMessage>,
    intervention_rx: &mut mpsc::Receiver<InterventionDecision>,
    current_provider: &foundation::Provider,
    gw: &GatewayRegistry,
    used_providers: &[foundation::Provider],
) -> StreamOutcome {
    let mut content = String::new();
    let mut usage = foundation::UsageStats::default();
    let mut seq: u32 = 0;
    let name = soul_name.to_string();
    let mut tool_calls: Vec<foundation::ToolCall> = Vec::new();
    let mut truncated = false;
    let mut has_any_content = false;

    loop {
        tokio::select! {
            biased;

            result = rx.recv() => {
                match result {
                    Some(Ok(chunk)) => {
                        has_any_content = true;
                        if let Some(u) = chunk.usage {
                            usage = u;
                        }
                        if !chunk.tool_calls.is_empty() {
                            tool_calls.extend(chunk.tool_calls);
                        }
                        tracing::debug!(
                            "Soul '{}' chunk #{}: content_len={} reasoning_len={} finish_reason={:?}",
                            name, seq, chunk.content.len(),
                            chunk.reasoning_content.as_ref().map(|s| s.len()).unwrap_or(0),
                            chunk.finish_reason
                        );
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
                        if chunk.finish_reason.as_deref() == Some("length") {
                            truncated = true;
                            tracing::warn!("Soul '{}' output truncated by max_tokens limit", name);
                        }
                    }
                    Some(Err(e)) => {
                        let error_msg = e.to_string();
                        gw.mark_provider_unhealthy(current_provider, error_msg.clone());

                        if has_any_content {
                            ws.broadcast_soul(
                                session_id,
                                &name,
                                &WsEvent {
                                    event_type: WsEventType::SoulError,
                                    payload: error_msg.clone(),
                                    reasoning_content: None,
                                    soul_name: Some(name.clone()),
                                    seq,
                                },
                            );
                            let _ = chunk_tx.send(SoulStreamMessage::Error {
                                soul_name: name.clone(),
                                error: error_msg.clone(),
                            });
                            let output = SoulOutput::error(name, error_msg);
                            return StreamOutcome::Completed(output);
                        }

                        if let Some(next_provider) = gw.try_next_provider(current_provider) {
                            if !used_providers.contains(&next_provider) {
                                tracing::warn!(
                                    "Soul '{}' provider {:?} failed before any content, will retry with {:?}: {}",
                                    name, current_provider, next_provider, error_msg
                                );
                            }
                        }

                        ws.broadcast_soul(
                            session_id,
                            &name,
                            &WsEvent {
                                event_type: WsEventType::SoulError,
                                payload: error_msg.clone(),
                                reasoning_content: None,
                                soul_name: Some(name.clone()),
                                seq,
                            },
                        );
                        let _ = chunk_tx.send(SoulStreamMessage::Error {
                            soul_name: name.clone(),
                            error: error_msg.clone(),
                        });
                        let output = SoulOutput::error(name, error_msg);
                        return StreamOutcome::Completed(output);
                    }
                    None => {
                        break;
                    }
                }
            }

            intervention = intervention_rx.recv() => {
                match intervention {
                    Some(decision) => {
                        tracing::info!(
                            "Soul '{}' received intervention {:?} mid-stream — suppressed (marginalia mode)",
                            name, decision
                        );
                        continue;
                    }
                    None => {
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

    if truncated {
        content.push_str("\n\n> ⚠️ [系统提示] 输出因长度限制被截断。如需完整分析，可尝试简化任务或分拆问题。");
    }

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
    current_topology: topology::ConferenceTopology,
    souls: Vec<String>,
) {
    let start = std::time::Instant::now();
    let mut collision_count = 0u32;
    let monitor = topology::TopologyMonitor::new();
    let mut check_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            msg = chunk_rx.recv() => {
                match msg {
                    Ok(msg) => {
                        match msg {
                            SoulStreamMessage::Chunk { soul_name, token } => {
                                detector.add_token(&soul_name, &token);

                                let collisions = detector.detect_collisions();
                                if !collisions.is_empty() {
                                    collision_count += collisions.len() as u32;
                                }
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
                                        let _from_soul = collision.from_soul.clone();
                                        let _contradiction = collision.content.clone();

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
            _ = check_interval.tick() => {
                if monitor.should_downgrade(start.elapsed(), collision_count, 0.0) {
                    let suggestion = topology::TopologyMonitor::suggest_downgrade(
                        &current_topology, &souls
                    );
                    let suggestion_desc = suggestion.map(|t| t.describe().to_string())
                        .unwrap_or_else(|| "保持当前拓扑".to_string());
                    let _ = system_tx.try_send(WsEvent {
                        event_type: WsEventType::SystemMessage,
                        payload: format!(
                            "拓扑监控: 运行{}秒, 碰撞{}次 — 建议降级至「{}」以节省成本",
                            start.elapsed().as_secs(),
                            collision_count,
                            suggestion_desc
                        ),
                        reasoning_content: None,
                        soul_name: None,
                        seq: 0,
                    }).ok();
                }
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

/// 从综合报告中提取"七、推荐补充魂"部分的建议魂名
fn extract_recommended_souls(synthesis: &str) -> Vec<serde_json::Value> {
    let section_start = synthesis.find("## 七、推荐补充魂");
    let section = match section_start {
        Some(start) => &synthesis[start..],
        None => return vec![],
    };

    // 截取到下一个一级标题或文件末尾
    let section_end = section[1..].find("\n## ").map(|i| i + 1).unwrap_or(section.len());
    let section_text = &section[..section_end];

    // 如果明确写"无需补充"，返回空
    if section_text.contains("无需补充") {
        return vec![];
    }

    let mut results = Vec::new();
    // 匹配 Markdown 列表项：- **魂名**：推荐理由...
    for line in section_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-") || trimmed.starts_with("*") {
            if let Some(name_start) = trimmed.find("**") {
                let after_first = &trimmed[name_start + 2..];
                if let Some(name_end) = after_first.find("**") {
                    let name = after_first[..name_end].trim();
                    if !name.is_empty() && name.len() < 50 {
                        // 提取推荐理由（**魂名**：后面的内容）
                        let rationale = after_first[name_end + 2..]
                            .trim_start_matches(|c| c == '：' || c == ':' || c == ' ')
                            .trim();
                        results.push(serde_json::json!({
                            "name": name,
                            "rationale": if rationale.is_empty() { "综合官推荐补充" } else { rationale },
                        }));
                    }
                }
            }
        }
    }

    results
}
