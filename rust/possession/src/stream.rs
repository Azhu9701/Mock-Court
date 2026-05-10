use ai_gateway::GatewayRegistry;
use foundation::{Chunk, ProviderInfo, Result, UsageStats};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{SoulOutput, WsEvent, WsEventType, WsSessionManager};

const FALLBACK_MODEL: &str = "claude-sonnet-4-6";

/// Stream LLM chunks to WebSocket, aggregating content and usage stats.
pub async fn stream_single_soul(
    mut rx: UnboundedReceiver<Result<Chunk>>,
    session_id: &str,
    soul_name: &str,
    ws: &WsSessionManager,
) -> SoulOutput {
    let mut content = String::new();
    let mut usage = UsageStats::default();
    let mut seq: u32 = 0;
    let name = soul_name.to_string();

    while let Some(result) = rx.recv().await {
        match result {
            Ok(chunk) => {
                if let Some(u) = chunk.usage {
                    usage = u;
                }
                if !chunk.content.is_empty() || chunk.reasoning_content.is_some() {
                    if let Some(rc) = &chunk.reasoning_content {
                        content.push_str(rc);
                    } else {
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

    SoulOutput { soul_name: name, content, usage, error: None }
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
    mut rx: UnboundedReceiver<Result<Chunk>>,
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
                    if let Some(rc) = &chunk.reasoning_content {
                        content.push_str(rc);
                    } else {
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
