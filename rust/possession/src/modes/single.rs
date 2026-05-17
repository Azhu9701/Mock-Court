use ai_gateway::prompt::PromptBuilder;
use ai_gateway::GatewayRegistry;
use foundation::{CallConfig, LLMRequest, Result, Storage};
use registry::SoulRegistry;
use tokio::sync::mpsc;

use crate::soul::self_audit::SelfAudit;
use crate::stream;
use crate::tools::ToolRegistry;
use crate::{SoulOutput, UserPresets, WsEvent, WsEventType, WsSessionManager};

pub async fn run(
    store: &dyn Storage,
    registry: &SoulRegistry,
    gateway: &GatewayRegistry,
    ws: &WsSessionManager,
    session_id: &str,
    soul_name: &str,
    task: &str,
    presets: &UserPresets,
    system_tx: &mpsc::Sender<WsEvent>,
    tool_registry: &ToolRegistry,
) -> Result<SoulOutput> {
    let _ = system_tx.try_send(WsEvent {
        event_type: WsEventType::SoulStarted,
        payload: format!("正在召唤 {} ...", soul_name),
        reasoning_content: None,
        soul_name: Some(soul_name.to_string()),
        seq: 0,
    }).ok();

    let profile = registry.get_soul(soul_name)?;
    let info = stream::pick_provider_info(gateway);
    let prompt_builder = PromptBuilder::new();
    let prompt = prompt_builder.build_summon_prompt(
        &profile, task,
        presets.judgment.as_deref(),
        presets.worry.as_deref(),
        presets.unknown.as_deref(),
        info.tier,
        presets.search_results.as_deref(),
        presets.interrogation_context.as_deref(),
    );

    let mut config = CallConfig::default();
    let tool_names = crate::tools::parse_soul_tools(&profile.tools);
    let has_tools = !tool_names.is_empty();

    let output = if has_tools {
        let definitions = tool_registry.filter_definitions(&tool_names);
        if !definitions.is_empty() {
            config = config.with_tools(definitions);
            stream::run_tool_loop(gateway, info.provider, &prompt, &config, session_id, soul_name, ws, tool_registry).await
        } else {
            let rx = gateway.call(&LLMRequest { provider: info.provider, prompt, config })?;
            stream::stream_single_soul(rx, session_id, soul_name, ws).await
        }
    } else {
        let rx = gateway.call(&LLMRequest { provider: info.provider, prompt, config })?;
        stream::stream_single_soul(rx, session_id, soul_name, ws).await
    };

    let audit = SelfAudit::audit(&profile, task, &output.content);
    let mut audit_notes = String::new();
    if !audit.passed || audit.revision_needed {
        let alerts = audit.contradictions.iter()
            .chain(&audit.blind_spot_alerts)
            .chain(&audit.premise_shaken)
            .map(|s| format!("⚠ {}", s))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = system_tx.send(WsEvent {
            event_type: WsEventType::SystemMessage,
            payload: format!("审计 {}: {}", soul_name, alerts),
            reasoning_content: None,
            soul_name: Some(soul_name.to_string()),
            seq: u32::MAX,
        });
        audit_notes = format!("自审发现: {:?}", audit.contradictions);
    }

    crate::emit_soul_cost(system_tx, soul_name, &output.usage, Some(&info.model));

    crate::finalize_output_with_notes(
        store, session_id, &output,
        foundation::PossessionMode::Single, task, &audit_notes,
    ).await?;

    Ok(output)
}
