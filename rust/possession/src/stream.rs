use ai_gateway::GatewayRegistry;
use foundation::{Chunk, LLMRequest, Prompt, PromptMessage, ProviderInfo, Result, UsageStats};
use tokio::sync::mpsc;

use crate::{SoulOutput, ToolCallPayload, ToolResultPayload, WsEvent, WsEventType, WsSessionManager};
use crate::tools::ToolRegistry;

const FALLBACK_MODEL: &str = "claude-sonnet-4-6";

/// Stream LLM chunks to WebSocket, aggregating content and usage stats.
pub async fn stream_single_soul(
    mut rx: mpsc::Receiver<Result<Chunk>>,
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
) -> SoulOutput {
    let mut content = String::new();
    let mut usage = UsageStats::default();
    let mut seq: u32 = 0;
    let name = soul_name.to_string();
    let mut tool_calls: Vec<foundation::ToolCall> = Vec::new();

    while let Some(result) = rx.recv().await {
        match result {
            Ok(chunk) => {
                if let Some(u) = chunk.usage {
                    usage = u;
                }
                if !chunk.tool_calls.is_empty() {
                    tool_calls.extend(chunk.tool_calls);
                }
                if !chunk.content.is_empty() || chunk.reasoning_content.is_some() {
                    if chunk.reasoning_content.is_none() {
                        content.push_str(&chunk.content);
                    }
                    ws.broadcast_soul(
                        session_id,
                        &name,
                        &WsEvent {
                            event_type: WsEventType::SoulChunk,
                            payload: chunk.content,
                            reasoning_content: chunk.reasoning_content,
                            soul_name: Some(name.clone()),
                            seq,
                        },
                    );
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

    SoulOutput { soul_name: name, content, usage, error: None, tool_calls }
}

/// Pick the first available provider info (model + tier).
/// Use `info.provider` if you only need the Provider.
pub fn pick_provider_info(gateway: &GatewayRegistry) -> ProviderInfo {
    gateway
        .list_providers()
        .into_iter()
        .find(|i| i.available)
        .unwrap_or_else(|| ProviderInfo {
            provider: foundation::Provider::Claude,
            model: FALLBACK_MODEL.into(),
            available: true,
            tier: foundation::ModelTier::Pro,
        })
}

/// Stream synthesis to WebSocket, returning aggregated content and usage.
pub async fn stream_synthesis(
    mut rx: mpsc::Receiver<Result<Chunk>>,
    session_id: &str,
    ws: &WsSessionManager,
) -> Result<(String, UsageStats)> {
    tracing::info!("Starting stream_synthesis for session: {}", session_id);
    let mut content = String::new();
    let mut usage = UsageStats::default();
    let mut seq: u32 = 0;
    let mut chunk_count = 0;

    while let Some(result) = rx.recv().await {
        match result {
            Ok(chunk) => {
                if let Some(u) = chunk.usage {
                    usage = u;
                }
                if !chunk.content.is_empty() || chunk.reasoning_content.is_some() {
                    chunk_count += 1;
                    if chunk.reasoning_content.is_none() {
                        content.push_str(&chunk.content);
                    }
                    tracing::debug!("Broadcasting synthesis chunk #{}: content={:?}, reasoning={:?}", chunk_count, chunk.content, chunk.reasoning_content);
                    ws.broadcast_system(
                        session_id,
                        &WsEvent {
                            event_type: WsEventType::SynthesisChunk,
                            payload: chunk.content,
                            reasoning_content: chunk.reasoning_content,
                            soul_name: None,
                            seq,
                        },
                    );
                    seq += 1;
                }
            }
            Err(e) => {
                tracing::error!("Error in synthesis stream: {}", e);
                return Err(e);
            }
        }
    }

    tracing::info!("Stream complete, {} chunks, total content length: {}", chunk_count, content.len());
    ws.broadcast_system(
        session_id,
        &WsEvent {
            event_type: WsEventType::SynthesisDone,
            payload: String::new(),
            reasoning_content: None,
            soul_name: None,
            seq,
        },
    );

    Ok((content, usage))
}

pub async fn run_tool_loop(
    gateway: &GatewayRegistry,
    provider: foundation::Provider,
    initial_prompt: &Prompt,
    config: &foundation::CallConfig,
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
    tool_registry: &ToolRegistry,
) -> SoulOutput {
    let mut history: Vec<PromptMessage> = initial_prompt.messages.clone();
    let max_rounds = crate::tools::max_tool_rounds();
    let name = soul_name.to_string();

    for _round in 0..max_rounds {
        let prompt = Prompt { messages: history.clone() };
        let req = LLMRequest {
            provider: provider.clone(),
            prompt,
            config: config.clone(),
        };

        let rx = match gateway.call(&req) {
            Ok(rx) => rx,
            Err(e) => return SoulOutput::error(name, e.to_string()),
        };

        let output = stream_single_soul(rx, session_id, soul_name, ws).await;

        if output.error.is_some() {
            return output;
        }

        if output.tool_calls.is_empty() {
            return output;
        }

        for tc in &output.tool_calls {
            let payload = ToolCallPayload {
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
                    let result_payload = ToolResultPayload {
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

                    history.push(PromptMessage {
                        role: "assistant".to_string(),
                        content: String::new(),
                        reasoning_content: None,
                        tool_calls: Some(output.tool_calls.clone()),
                        tool_call_id: None,
                    });
                    history.push(PromptMessage {
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

    SoulOutput {
        soul_name: name,
        content: String::new(),
        usage: UsageStats::default(),
        error: Some(format!("Tool call loop exceeded {} rounds", max_rounds)),
        tool_calls: Vec::new(),
    }
}
