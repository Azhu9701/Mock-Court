use std::time::Duration;

use ai_gateway::model_router::{ModelRouter, RoutingRole};
use ai_gateway::prompt::PromptBuilder;
use ai_gateway::GatewayRegistry;
use foundation::{
    CallConfig, KnowledgeCard, LLMRequest, Message, Prompt, PromptMessage, ReasoningEffort, Result, Storage,
};
use registry::SoulRegistry;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use tokio::task::JoinSet;
use tokio::sync::broadcast;
use crate::cross_detector::{CrossDetector, CollisionEvent};

use crate::stream;
use crate::{SoulOutput, UserPresets, WsEvent, WsEventType, WsSessionManager};

const MAX_PARALLEL_SOULS: usize = 10;
const SOUL_TIMEOUT_SECS: u64 = 300;

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
    system_tx: &UnboundedSender<WsEvent>,
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
        let _ = system_tx.send(WsEvent {
            event_type: WsEventType::SoulStarted,
            payload: format!("正在召唤 {} ...", soul_name),
            reasoning_content: None,
            soul_name: Some(soul_name.clone()),
            seq: 0,
        });

        // 注册魂到检测器
        cross_detector.register_soul(soul_name.clone());

        match registry.get_soul(soul_name) {
            Ok(profile) => {
                // 使用模型路由器选择合适的配置
                let soul_decision = ModelRouter::route(&providers, RoutingRole::Soul);
                let (use_cache, config) = if let Some(decision) = &soul_decision {
                    (decision.use_cache_hint, ModelRouter::create_call_config(decision))
                } else {
                    (false, CallConfig::default())
                };
                let provider = soul_decision.as_ref().map(|d| d.provider.clone()).unwrap_or_else(|| stream::pick_provider_info(gateway).provider);
                let tier = soul_decision.as_ref().map(|d| d.tier.clone()).unwrap_or_else(|| stream::pick_provider_info(gateway).tier);

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
                let _ = system_tx.send(WsEvent {
                    event_type: WsEventType::SoulError,
                    payload: e.to_string(),
                    reasoning_content: None,
                    soul_name: Some(soul_name.clone()),
                    seq: 0,
                });
            }
        }
    }

    let gateway_owned = GatewayRegistry::clone(gateway);
    let ws_c = ws.clone();
    let s_id = session_id.to_string();
    
    // 启动碰撞检测任务
    let detector = cross_detector.clone();
    let ws_clone = ws_c.clone();
    let session_id_clone = s_id.clone();
    let system_tx_clone = system_tx.clone();
    let _collision_handle = tokio::spawn(async move {
        detect_collisions_async(detector, chunk_rx, ws_clone, session_id_clone, system_tx_clone).await;
    });

    let mut set = JoinSet::new();
    for (soul_name, req) in requests {
        let s_id = session_id.to_string();
        let ws_c = ws.clone();
        let gw = gateway_owned.clone();
        let chunk_tx_clone = chunk_tx.clone();
        set.spawn(async move {
            let rx = match gw.call(&req) {
                Ok(rx) => rx,
                Err(e) => {
                    let _ = chunk_tx_clone.send(SoulStreamMessage::Error { 
                        soul_name: soul_name.clone(), 
                        error: e.to_string() 
                    });
                    return SoulOutput::error(soul_name.clone(), e.to_string());
                }
            };
            stream_single_soul_with_detection(
                rx, 
                &s_id, 
                &soul_name, 
                &ws_c, 
                chunk_tx_clone
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
            vec![]
        }
    };

    for output in &outputs {
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
        let _ = system_tx.send(WsEvent {
            event_type: WsEventType::SynthesisStarted,
            payload: "辩证综合开始...".into(),
            reasoning_content: None,
            soul_name: None,
            seq: 0,
        });

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
                // Emit cost event
                let total_tokens: u32 = outputs.iter().map(|o| o.usage.total_tokens).sum::<u32>() + synth_usage.total_tokens;
                let llm_calls = limited.len() as u32 + 1; // N souls + 1 synthesis
                let cost_estimate = estimate_cost(card_decision.as_ref().map(|d| d.provider.clone()).unwrap_or(synthesis_provider.clone()), total_tokens, true);
                let _ = system_tx.send(WsEvent {
                    event_type: WsEventType::Cost,
                    payload: serde_json::json!({
                        "llm_calls": llm_calls,
                        "tokens_used": total_tokens,
                        "estimated_cost": cost_estimate,
                        "cache_discount": true,
                    }).to_string(),
                    reasoning_content: None,
                    soul_name: None,
                    seq: 0,
                });

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
                        let card_msg = Message {
                            id: uuid::Uuid::new_v4().to_string(),
                            session_id: session_id.to_string(),
                            role: foundation::MessageRole::System,
                            soul_name: Some("知识卡片".into()),
                            content: format!("📇 知识卡片\n{}", card),
                            seq: 1,
                            created_at: chrono::Utc::now(),
                        };
                        let _ = store.append_message(&card_msg).await;

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
        let _ = system_tx.send(WsEvent {
            event_type: WsEventType::SystemMessage,
            payload: "⏳ 请设定一个24小时内可检验的具体行动，验证本次分析的结论。".into(),
            reasoning_content: None,
            soul_name: None,
            seq: 0,
        });
    }

    Ok(outputs)
}

/// 流式输出单个魂，同时发送给交叉检测器
async fn stream_single_soul_with_detection(
    mut rx: UnboundedReceiver<foundation::Result<foundation::Chunk>>,
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
    chunk_tx: broadcast::Sender<SoulStreamMessage>,
) -> SoulOutput {
    let mut content = String::new();
    let mut usage = foundation::UsageStats::default();
    let mut seq: u32 = 0;
    let name = soul_name.to_string();

    while let Some(result) = rx.recv().await {
        match result {
            Ok(chunk) => {
                if let Some(u) = chunk.usage {
                    usage = u;
                }
                if !chunk.content.is_empty() {
                    content.push_str(&chunk.content);
                    
                    // 发送给 WebSocket
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
                    
                    // 发送给检测器
                    let _ = chunk_tx.send(SoulStreamMessage::Chunk {
                        soul_name: name.clone(),
                        token: chunk.content,
                    });
                    
                    seq += 1;
                }
            }
            Err(e) => {
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
                return SoulOutput::error(name, e.to_string());
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

    let output = SoulOutput { soul_name: name.clone(), content, usage, error: None };
    let _ = chunk_tx.send(SoulStreamMessage::Done {
        soul_name: name,
        output: output.clone(),
    });

    output
}

/// 异步碰撞检测任务
async fn detect_collisions_async(
    detector: CrossDetector,
    mut chunk_rx: broadcast::Receiver<SoulStreamMessage>,
    ws: WsSessionManager,
    session_id: String,
    system_tx: UnboundedSender<WsEvent>,
) {
    loop {
        match chunk_rx.recv().await {
            Ok(msg) => {
                match msg {
                    SoulStreamMessage::Chunk { soul_name, token } => {
                        detector.add_token(&soul_name, &token);
                        
                        // 检测碰撞
                        let collisions = detector.detect_collisions();
                        for collision in collisions {
                            broadcast_collision(&ws, &session_id, &collision, &system_tx);
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
    system_tx: &UnboundedSender<WsEvent>,
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
    let _ = system_tx.send(event);
    
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
