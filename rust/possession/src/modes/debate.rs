use ai_gateway::prompt::PromptBuilder;
use ai_gateway::GatewayRegistry;
use foundation::{CallConfig, LLMRequest, Result, Storage};
use registry::SoulRegistry;
use tokio::sync::mpsc;

use crate::stream;
use crate::tools::ToolRegistry;
use crate::{SoulOutput, UserPresets, WsEvent, WsSessionManager};

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
    _system_tx: &mpsc::Sender<WsEvent>,
    _tool_registry: &ToolRegistry,
) -> Result<(SoulOutput, SoulOutput)> {
    let profile_a = registry.get_soul(soul_a)?;
    let profile_b = registry.get_soul(soul_b)?;
    let info = stream::pick_provider_info(gateway);
    let prompt_builder = PromptBuilder::new();

    let prompt_a = prompt_builder.build_debate_prompt(&profile_a, soul_b, topic, None);
    let prompt_b = prompt_builder.build_debate_prompt(&profile_b, soul_a, topic, None);

    let rx_a = gateway.call(&LLMRequest { provider: info.provider.clone(), prompt: prompt_a, config: CallConfig::default() })?;
    let rx_b = gateway.call(&LLMRequest { provider: info.provider.clone(), prompt: prompt_b, config: CallConfig::default() })?;

    let sid = session_id.to_string();
    let sa = soul_a.to_string();
    let sb = soul_b.to_string();
    let ws_a = ws.clone();
    let ws_b = ws.clone();

    let (out_a, out_b) = tokio::join!(
        stream::stream_single_soul(rx_a, &sid, &sa, &ws_a),
        stream::stream_single_soul(rx_b, &sid, &sb, &ws_b),
    );

    crate::finalize_output(store, session_id, &out_a, foundation::PossessionMode::Debate, topic).await?;
    crate::finalize_output(store, session_id, &out_b, foundation::PossessionMode::Debate, topic).await?;

    Ok((out_a, out_b))
}
