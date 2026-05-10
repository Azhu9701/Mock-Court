use ai_gateway::prompt::PromptBuilder;
use ai_gateway::GatewayRegistry;
use foundation::{CallConfig, LLMRequest, Result, Storage};
use registry::SoulRegistry;
use tokio::sync::mpsc::UnboundedSender;

use crate::stream;
use crate::{SoulOutput, UserPresets, WsEvent, WsSessionManager};

pub async fn run(
    store: &dyn Storage,
    registry: &SoulRegistry,
    gateway: &GatewayRegistry,
    ws: &WsSessionManager,
    session_id: &str,
    task: &str,
    soul_chain: &[String],
    _presets: &UserPresets,
    _system_tx: &UnboundedSender<WsEvent>,
) -> Result<Vec<SoulOutput>> {
    let info = stream::pick_provider_info(gateway);
    let prompt_builder = PromptBuilder::new();
    let mut outputs = Vec::new();
    let mut prev_content: Option<String> = None;

    for soul_name in soul_chain {
        let profile = match registry.get_soul(soul_name) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        let prompt = prompt_builder.build_relay_prompt(&profile, prev_content.as_deref(), task);
        let rx = gateway.call(&LLMRequest { provider: info.provider.clone(), prompt, config: CallConfig::default() })?;

        let output = stream::stream_single_soul(rx, session_id, soul_name, ws).await;

        crate::finalize_output(store, session_id, &output, foundation::PossessionMode::Relay, task).await?;

        if output.error.is_some() {
            outputs.push(output);
            break;
        }

        prev_content = Some(output.content.clone());
        outputs.push(output);
    }

    Ok(outputs)
}
