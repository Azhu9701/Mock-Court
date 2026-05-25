use ai_gateway::prompt::PromptBuilder;
use ai_gateway::GatewayRegistry;
use foundation::{CallConfig, LLMRequest, Result, Storage};
use registry::SoulRegistry;
use tokio::sync::mpsc;

use crate::stream;
use crate::tools::ToolRegistry;
use crate::{SoulOutput, UserPresets, WsEvent, WsEventType, WsSessionManager};

pub async fn run(
    store: &dyn Storage,
    registry: &SoulRegistry,
    gateway: &GatewayRegistry,
    ws: &WsSessionManager,
    session_id: &str,
    soul_a: &str,
    soul_b: &str,
    topic: &str,
    _presets: &UserPresets,
    system_tx: &mpsc::Sender<WsEvent>,
    _tool_registry: &ToolRegistry,
) -> Result<(SoulOutput, SoulOutput)> {
    let profile_a = registry.get_soul(soul_a)?;
    let profile_b = registry.get_soul(soul_b)?;
    let info = stream::pick_provider_info(gateway);
    let prompt_builder = PromptBuilder::new();

    let prompt_a = prompt_builder.build_debate_prompt(&profile_a, soul_b, topic, None);
    let prompt_b = prompt_builder.build_debate_prompt(&profile_b, soul_a, topic, None);

    let provider = info.provider;
    let sid = session_id.to_string();
    let sa = soul_a.to_string();
    let sb = soul_b.to_string();
    let ws_a = ws.clone();
    let ws_b = ws.clone();
    let (out_a, out_b) = match (
        gateway.call(&LLMRequest { provider, prompt: prompt_a, config: CallConfig::default() }),
        gateway.call(&LLMRequest { provider, prompt: prompt_b, config: CallConfig::default() }),
    ) {
        (Ok(rx_a), Ok(rx_b)) => {
            tokio::join!(
                stream::stream_single_soul_with_provider(rx_a, &sid, &sa, &ws_a, gateway, &provider),
                stream::stream_single_soul_with_provider(rx_b, &sid, &sb, &ws_b, gateway, &provider),
            )
        }
        (Err(e), _) => {
            let error_msg = e.to_string();
            gateway.mark_provider_unhealthy(&provider, error_msg.clone());
            ws.broadcast_soul(
                session_id,
                soul_a,
                &WsEvent {
                    event_type: WsEventType::SoulError,
                    payload: error_msg.clone(),
                    reasoning_content: None,
                    soul_name: Some(soul_a.to_string()),
                    seq: 0,
                },
            );
            (
                SoulOutput::error(soul_a.to_string(), error_msg.clone()),
                SoulOutput::error(soul_b.to_string(), error_msg),
            )
        }
        (_, Err(e)) => {
            let error_msg = e.to_string();
            gateway.mark_provider_unhealthy(&provider, error_msg.clone());
            ws.broadcast_soul(
                session_id,
                soul_b,
                &WsEvent {
                    event_type: WsEventType::SoulError,
                    payload: error_msg.clone(),
                    reasoning_content: None,
                    soul_name: Some(soul_b.to_string()),
                    seq: 0,
                },
            );
            (
                SoulOutput::error(soul_a.to_string(), error_msg.clone()),
                SoulOutput::error(soul_b.to_string(), error_msg),
            )
        }
    };

    crate::emit_soul_cost(system_tx, soul_a, &out_a.usage, Some(&info.model));
    crate::emit_soul_cost(system_tx, soul_b, &out_b.usage, Some(&info.model));
    crate::finalize_output(store, session_id, &out_a, foundation::PossessionMode::Debate, topic).await?;
    crate::finalize_output(store, session_id, &out_b, foundation::PossessionMode::Debate, topic).await?;

    Ok((out_a, out_b))
}
