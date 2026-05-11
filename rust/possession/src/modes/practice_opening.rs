use std::time::Duration;

use ai_gateway::prompt::PromptBuilder;
use ai_gateway::GatewayRegistry;
use foundation::{CallConfig, LLMRequest, Result, Storage};
use registry::SoulRegistry;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::stream;
use crate::tools::ToolRegistry;
use crate::{SoulOutput, UserPresets, WsEvent, WsEventType, WsSessionManager};

const SOUL_TIMEOUT_SECS: u64 = 300;

pub async fn run(
    store: &dyn Storage,
    registry: &SoulRegistry,
    gateway: &GatewayRegistry,
    ws: &WsSessionManager,
    session_id: &str,
    task: &str,
    _presets: &UserPresets,
    system_tx: &mpsc::Sender<WsEvent>,
    _tool_registry: &ToolRegistry,
) -> Result<()> {
    let info = stream::pick_provider_info(gateway);
    let prompt_builder = PromptBuilder::new();

    ws.broadcast_system(
        session_id,
        &WsEvent {
            event_type: WsEventType::SystemMessage,
            payload: format!("P1 现场数据已收集: {}", &task[..task.len().min(100)]),
            reasoning_content: None,
            soul_name: None,
            seq: 0,
        },
    );

    // Use registry fulltext search for efficient keyword matching
    let matches = registry.search_souls(task)?;
    let names: Vec<String> = if matches.is_empty() {
        let all_souls = registry.list_souls(&foundation::IsmismFilter::default())?;
        let mut top: Vec<_> = all_souls.iter().collect();
        top.sort_by_key(|s| std::cmp::Reverse(s.summon_count));
        top.into_iter().take(3).map(|s| s.name.clone()).collect()
    } else {
        matches.into_iter().take(5).map(|m| m.entry.name).collect()
    };

    if names.is_empty() {
        return Ok(());
    }

    // Parallel spawn via JoinSet
    let task_arc = std::sync::Arc::new(task.to_string());
    let gateway_owned = GatewayRegistry::clone(gateway);
    let mut set = JoinSet::new();

    for soul_name in &names {
        let profile = match registry.get_soul(soul_name) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let prompt = prompt_builder.build_practice_opening_prompt(&profile, &task_arc);

        let s_id = session_id.to_string();
        let ws_c = ws.clone();
        let gw = gateway_owned.clone();
        let sn = soul_name.clone();
        let provider = info.provider.clone();

        set.spawn(async move {
            let rx = match gw.call(&LLMRequest { provider, prompt, config: CallConfig::default() }) {
                Ok(rx) => rx,
                Err(_) => return SoulOutput::error(sn.clone(), "call failed".into()),
            };
            stream::stream_single_soul(rx, &s_id, &sn, &ws_c).await
        });
    }

    // Collect results
    let collect = async {
        let mut acc = Vec::with_capacity(names.len());
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
            tracing::warn!("Practice opening timed out after {}s", SOUL_TIMEOUT_SECS);
            vec![]
        }
    };

    for output in &outputs {
        crate::emit_soul_cost(system_tx, &output.soul_name, &output.usage, Some(&info.model));
        let _ = crate::finalize_output(store, session_id, output, foundation::PossessionMode::PracticeOpening, task).await;
    }

    ws.broadcast_system(
        session_id,
        &WsEvent {
            event_type: WsEventType::SystemMessage,
            payload: format!("P2 魂消化完成 ({} 魂参与)，P3/P4 由后续流程处理", names.len()),
            reasoning_content: None,
            soul_name: None,
            seq: 0,
        },
    );

    Ok(())
}
