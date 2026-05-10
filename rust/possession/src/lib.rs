mod modes;
mod recovery;
pub mod soul;
pub mod stream;
pub mod tools;
pub mod triage;
mod ws;
pub mod cross_detector;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use ai_gateway::GatewayRegistry;
use foundation::{
    FoundationError, PossessionMode, Result, Session, SessionStatus, Storage, UsageStats,
};
use registry::SoulRegistry;
use tokio::sync::mpsc;
use uuid::Uuid;

pub use recovery::RecoveryManager;
use tracing;
pub use ws::WsSessionManager;

use modes::{conference, debate, learn, practice_opening, relay, single};

#[derive(Debug, Clone)]
pub struct PossessionInput {
    pub mode: Option<PossessionMode>,
    pub task: String,
    pub souls: Vec<String>,
    pub topic: Option<String>,
    pub judgment: Option<String>,
    pub worry: Option<String>,
    pub unknown: Option<String>,
    pub search_topic: bool,
    pub search_results: Option<String>,
    #[allow(dead_code)]
    pub task_cards: std::collections::HashMap<String, String>,
}

/// 使用者预设三段式 — 独立于 task，直接注入每条魂的 prompt
pub struct UserPresets {
    pub judgment: Option<String>,
    pub worry: Option<String>,
    pub unknown: Option<String>,
    pub search_results: Option<String>,
}

impl From<&PossessionInput> for UserPresets {
    fn from(input: &PossessionInput) -> Self {
        UserPresets {
            judgment: input.judgment.clone(),
            worry: input.worry.clone(),
            unknown: input.unknown.clone(),
            search_results: input.search_results.clone(),
        }
    }
}

impl UserPresets {
    pub fn is_empty(&self) -> bool {
        self.judgment.as_ref().map_or(true, |s| s.is_empty())
            && self.worry.as_ref().map_or(true, |s| s.is_empty())
            && self.unknown.as_ref().map_or(true, |s| s.is_empty())
    }
}

/// Persist soul output: archive → message → call record, all in one call.
pub async fn finalize_output(
    store: &dyn Storage,
    session_id: &str,
    output: &SoulOutput,
    mode: PossessionMode,
    task_summary: &str,
) -> Result<()> {
    finalize_output_with_notes(store, session_id, output, mode, task_summary, "").await
}

pub async fn finalize_output_with_notes(
    store: &dyn Storage,
    session_id: &str,
    output: &SoulOutput,
    mode: PossessionMode,
    task_summary: &str,
    extra_notes: &str,
) -> Result<()> {
    let content = if output.content.is_empty() {
        output.error.clone().unwrap_or_else(|| "空回应".into())
    } else {
        output.content.clone()
    };

    if !content.is_empty() {
        let _ = store.archive_soul_output(session_id, &output.soul_name, &content).await;
    }

    let msg = foundation::Message {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session_id.to_string(),
        role: foundation::MessageRole::Soul,
        soul_name: Some(output.soul_name.clone()),
        content,
        seq: 0,
        created_at: chrono::Utc::now(),
    };
    let _ = store.append_message(&msg).await;

    let eff = if output.error.is_some() {
        foundation::Effectiveness::Invalid
    } else {
        foundation::Effectiveness::Effective
    };
    let mut notes = output.error.clone().unwrap_or_default();
    if !extra_notes.is_empty() {
        if !notes.is_empty() { notes.push_str(" | "); }
        notes.push_str(extra_notes);
    }
    let record = foundation::CallRecord {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session_id.to_string(),
        soul_name: output.soul_name.clone(),
        mode,
        task_summary: task_summary.to_string(),
        effectiveness: eff,
        notes,
        created_at: chrono::Utc::now(),
        self_negation: None,
        empty_chair: None,
        user_feedback: None,
    };
    let _ = store.record_call(&record).await;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum EntryType {
    Single,
    Conference,
    Debate,
    Relay,
    Learn,
    PracticeOpening,
}

#[derive(Debug, Clone)]
pub struct SoulOutput {
    pub soul_name: String,
    pub content: String,
    pub usage: UsageStats,
    pub error: Option<String>,
    pub tool_calls: Vec<foundation::ToolCall>,
}

impl SoulOutput {
    pub fn error(soul_name: String, err: String) -> Self {
        SoulOutput {
            soul_name,
            content: String::new(),
            usage: UsageStats::default(),
            error: Some(err),
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WsEvent {
    pub event_type: WsEventType,
    pub payload: String,
    pub reasoning_content: Option<String>,
    pub soul_name: Option<String>,
    pub seq: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCallPayload {
    pub tool_call_id: String,
    pub tool_name: String,
    pub arguments: String,
    pub soul_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResultPayload {
    pub tool_call_id: String,
    pub tool_name: String,
    pub result: String,
    pub soul_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WsEventType {
    // Soul streaming
    #[serde(rename = "soul_token")]
    SoulChunk,
    #[serde(rename = "soul_done")]
    SoulDone,
    #[serde(rename = "soul_error")]
    SoulError,
    // Synthesis
    #[serde(rename = "synthesis_chunk")]
    SynthesisChunk,
    #[serde(rename = "synthesis_done")]
    SynthesisDone,
    // System
    #[serde(rename = "system")]
    SystemMessage,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "done")]
    SessionComplete,
    // Process events
    #[serde(rename = "session_started")]
    SessionStarted,
    #[serde(rename = "entry_classified")]
    EntryClassified,
    #[serde(rename = "soul_started")]
    SoulStarted,
    #[serde(rename = "soul_calling")]
    SoulCalling,
    #[serde(rename = "synthesis_started")]
    SynthesisStarted,
    #[serde(rename = "process_step")]
    ProcessStep,
    // Cross-detection (future)
    #[serde(rename = "collision")]
    Collision,
    // Cost tracking
    #[serde(rename = "cost")]
    Cost,
    // Tool calling
    #[serde(rename = "tool_call_started")]
    ToolCallStarted,
    #[serde(rename = "tool_result")]
    ToolResult,
}

pub struct PossessionEngine {
    store: Arc<dyn Storage>,
    registry: Arc<SoulRegistry>,
    gateway: Arc<GatewayRegistry>,
    ws_manager: WsSessionManager,
    shutdown_flag: AtomicBool,
    tool_registry: tools::ToolRegistry,
}

impl PossessionEngine {
    pub fn new(
        store: Arc<dyn Storage>,
        registry: Arc<SoulRegistry>,
        gateway: Arc<GatewayRegistry>,
    ) -> Self {
        PossessionEngine {
            store,
            registry,
            gateway,
            ws_manager: WsSessionManager::new(),
            shutdown_flag: AtomicBool::new(false),
            tool_registry: tools::ToolRegistry::new(),
        }
    }

    pub fn tool_registry(&self) -> &tools::ToolRegistry {
        &self.tool_registry
    }

    pub fn tool_registry_mut(&mut self) -> &mut tools::ToolRegistry {
        &mut self.tool_registry
    }

    pub async fn start_possession(
        &self,
        input: PossessionInput,
        system_tx: mpsc::Sender<WsEvent>,
    ) -> Result<String> {
        let entry = triage::triage(&input);
        let session_id = Uuid::new_v4().to_string();
        let mode = entry_to_mode(&entry);

        let session = Session {
            id: session_id.clone(),
            title: input.task.clone(),
            mode,
            status: SessionStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.store.create_session(&session).await?;

        // Register session so WebSocket subscribe() can find it
        self.ws_manager.create_session(&session_id);

        let store = self.store.clone();
        let registry = self.registry.clone();
        let gateway = self.gateway.clone();
        let ws = self.ws_manager.clone();
        let tool_reg = self.tool_registry.clone();
        let sid = session_id.clone();
        let task = input.task.clone();
        let created_at = session.created_at;

        tokio::spawn(async move {
            let result = dispatch_mode(
                &entry,
                &*store,
                &registry,
                &gateway,
                &ws,
                &tool_reg,
                &sid,
                &input,
                &system_tx,
            )
            .await;

            let status = match &result {
                Ok(_) => SessionStatus::Completed,
                Err(_) => SessionStatus::Inconsistent,
            };

            let _ = store.update_session(&Session {
                id: sid.clone(),
                title: task,
                mode: entry_to_mode(&entry),
                status,
                created_at,
                updated_at: chrono::Utc::now(),
            })
            .await;

            let _ = system_tx.try_send(WsEvent {
        event_type: WsEventType::SessionComplete,
        payload: String::new(),
        reasoning_content: None,
        soul_name: None,
        seq: 0,
    }).ok();

            if let Err(e) = result {
                tracing::error!("Session {} failed: {}", sid, e);
            }
        });

        Ok(session_id)
    }

    pub fn ws_manager(&self) -> &WsSessionManager {
        &self.ws_manager
    }

    pub fn gateway(&self) -> &Arc<GatewayRegistry> {
        &self.gateway
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown_flag.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_shutdown(&self) {
        self.shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

async fn dispatch_mode(
    entry: &EntryType,
    store: &dyn Storage,
    registry: &SoulRegistry,
    gateway: &GatewayRegistry,
    ws: &WsSessionManager,
    tool_registry: &tools::ToolRegistry,
    session_id: &str,
    input: &PossessionInput,
    system_tx: &mpsc::Sender<WsEvent>,
) -> Result<()> {
    let presets = UserPresets::from(input);

    let _ = system_tx.try_send(WsEvent {
        event_type: WsEventType::SessionStarted,
        payload: format!("附体会话已创建，模式：{}", entry_to_mode(entry).as_str()),
        reasoning_content: None,
        soul_name: None,
        seq: 0,
    }).ok();

    let _ = system_tx.try_send(WsEvent {
        event_type: WsEventType::EntryClassified,
        payload: format!("入口分流完成 — 匹配魂：{}", input.souls.join(", ")),
        reasoning_content: None,
        soul_name: None,
        seq: 1,
    }).ok();

    if let Some(ref sr) = input.search_results {
        let preview_len = 100.min(sr.len());
        let preview = &sr[..preview_len];
        let line_count = sr.lines().count();
        let _ = system_tx.try_send(WsEvent {
            event_type: WsEventType::ProcessStep,
            payload: format!("议题背景搜索完成：已获取 {} 行背景资料\n> {}", line_count, preview),
            reasoning_content: None,
            soul_name: None,
            seq: 2,
        }).ok();
    } else if input.search_topic {
        let _ = system_tx.try_send(WsEvent {
            event_type: WsEventType::ProcessStep,
            payload: "议题背景搜索未获取到结果，魂将依赖自身知识库。".into(),
            reasoning_content: None,
            soul_name: None,
            seq: 2,
        }).ok();
    }

    match entry {
        EntryType::Single => {
            let soul = input.souls.first().ok_or_else(|| {
                FoundationError::Validation("Single mode requires a soul".into())
            })?;
            single::run(store, registry, gateway, ws, session_id, soul, &input.task, &presets, system_tx, tool_registry).await?;
        }
        EntryType::Conference => {
            conference::run(
                store, registry, gateway, ws, session_id, &input.task, &input.souls, &input.task_cards, &presets, system_tx, tool_registry,
            )
            .await?;
        }
        EntryType::Debate => {
            let (a, b) = match (input.souls.first(), input.souls.get(1)) {
                (Some(a), Some(b)) => (a.clone(), b.clone()),
                _ => {
                    return Err(FoundationError::Validation(
                        "Debate mode requires exactly 2 souls".into(),
                    ))
                }
            };
            let topic = input.topic.clone().unwrap_or_else(|| input.task.clone());
            debate::run(
                store, registry, gateway, ws, session_id, &a, &b, &topic, &presets, system_tx, tool_registry,
            )
            .await?;
        }
        EntryType::Relay => {
            if input.souls.is_empty() {
                return Err(FoundationError::Validation(
                    "Relay mode requires at least 1 soul".into(),
                ));
            }
            relay::run(
                store, registry, gateway, ws, session_id, &input.task, &input.souls, &presets, system_tx, tool_registry,
            )
            .await?;
        }
        EntryType::Learn => {
            let soul = input.souls.first().ok_or_else(|| {
                FoundationError::Validation("Learn mode requires a soul".into())
            })?;
            learn::run(store, registry, gateway, ws, session_id, soul, &input.task, &presets, system_tx, tool_registry).await?;
        }
        EntryType::PracticeOpening => {
            practice_opening::run(
                store, registry, gateway, ws, session_id, &input.task, &presets, system_tx, tool_registry,
            )
            .await?;
        }
    }
    Ok(())
}

fn entry_to_mode(entry: &EntryType) -> PossessionMode {
    match entry {
        EntryType::Single => PossessionMode::Single,
        EntryType::Conference => PossessionMode::Conference,
        EntryType::Debate => PossessionMode::Debate,
        EntryType::Relay => PossessionMode::Relay,
        EntryType::Learn => PossessionMode::Learn,
        EntryType::PracticeOpening => PossessionMode::PracticeOpening,
    }
}
