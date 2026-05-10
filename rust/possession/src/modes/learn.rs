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
    soul_name: &str,
    task: &str,
    presets: &UserPresets,
    _system_tx: &UnboundedSender<WsEvent>,
) -> Result<SoulOutput> {
    let profile = registry.get_soul(soul_name)?;
    let learn_task = format!("{}。请作为学习伙伴，在回应中解释你的思考过程和分析方法。", task);
    let prompt_builder = PromptBuilder::new();
    let info = stream::pick_provider_info(gateway);
    let prompt = prompt_builder.build_summon_prompt(
        &profile, &learn_task,
        presets.judgment.as_deref(),
        presets.worry.as_deref(),
        presets.unknown.as_deref(),
        info.tier,
        presets.search_results.as_deref(),
    );
    let rx = gateway.call(&LLMRequest { provider: info.provider, prompt, config: CallConfig::default() })?;

    let output = stream::stream_single_soul(rx, session_id, soul_name, ws).await;

    if let Some(ref err) = output.error {
        return Err(foundation::FoundationError::Validation(err.clone()));
    }

    crate::finalize_output(store, session_id, &output, foundation::PossessionMode::Learn, task).await?;

    Ok(output)
}
