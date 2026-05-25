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
    soul_name: &str,
    task: &str,
    presets: &UserPresets,
    system_tx: &mpsc::Sender<WsEvent>,
    _tool_registry: &ToolRegistry,
) -> Result<SoulOutput> {
    let profile = registry.get_soul(soul_name)?;

    let learn_task = format!(
        r#"{}

学习模式说明：
这是一个"论证训练"场景。用户正在输出自己的观点/论证，你的任务是作为学习伙伴，从你的意识形态立场出发，对用户的论证进行结构性反馈。

请按以下结构回应：

1. **抓准的核心**：你认为用户抓住了什么关键问题或核心矛盾？肯定用户论证中合理的部分。

2. **逻辑漏洞**：用户的论证中存在哪些逻辑链条的断裂、前提的缺失、或推理的跳步？用具体的例子指出。

3. **缺失维度**：从你的意识形态立场出发，用户忽略了哪些重要的维度、视角或矛盾？

4. **改进建议**：用户应该如何加强自己的论证？给出具体、可操作的建议。

重要原则：
- 不追求辞藻华丽，追求逻辑的清晰性
- 明确暴露你的意识形态立场——说明"从我的XX主义立场来看"
- 反馈要具体、结构化，不要泛泛而谈
- 你的目的是帮助用户训练论证能力，而不是展示你的才华"#,
        task
    );

    let prompt_builder = PromptBuilder::new();
    let info = stream::pick_provider_info(gateway);
    let provider = info.provider;
    let prompt = prompt_builder.build_summon_prompt(
        &profile, &learn_task,
        presets.judgment.as_deref(),
        presets.worry.as_deref(),
        presets.unknown.as_deref(),
        info.tier,
        presets.search_results.as_deref(),
        presets.interrogation_context.as_deref(),
    );
    let output = match gateway.call(&LLMRequest { provider, prompt, config: CallConfig::default() }) {
        Ok(rx) => {
            stream::stream_single_soul_with_provider(rx, session_id, soul_name, ws, gateway, &provider).await
        }
        Err(e) => {
            let error_msg = e.to_string();
            gateway.mark_provider_unhealthy(&provider, error_msg.clone());
            ws.broadcast_soul(
                session_id,
                soul_name,
                &WsEvent {
                    event_type: WsEventType::SoulError,
                    payload: error_msg.clone(),
                    reasoning_content: None,
                    soul_name: Some(soul_name.to_string()),
                    seq: 0,
                },
            );
            SoulOutput::error(soul_name.to_string(), error_msg)
        }
    };

    if let Some(ref err) = output.error {
        return Err(foundation::FoundationError::Validation(err.clone()));
    }

    crate::emit_soul_cost(system_tx, soul_name, &output.usage, Some(&info.model));
    crate::finalize_output(store, session_id, &output, foundation::PossessionMode::Learn, task).await?;

    Ok(output)
}
