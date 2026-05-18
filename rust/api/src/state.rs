use std::sync::{Arc, RwLock};

use archive::ArchiveSystem;
use dashmap::DashMap;
use foundation::Config;
use possession::PossessionEngine;
use registry::SoulRegistry;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::collector::SoulCollector;

#[derive(Debug, Clone, Serialize)]
pub struct AutoCreateEvent {
    pub task_id: String,
    pub soul_name: String,
    pub phase: String, // "collecting" | "refining" | "done" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<foundation::SoulProfile>,
}

/// 审查官入场审讯 — 在合议前拦截使用者，判断是否"以此享乐"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterrogationQuestion {
    pub text: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct InterrogationGate {
    pub task: String,
    pub questions: Vec<InterrogationQuestion>,
}

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<SoulRegistry>,
    pub engine: Arc<PossessionEngine>,
    pub archive: Arc<ArchiveSystem>,
    pub collector: Arc<SoulCollector>,
    pub config: Arc<Config>,
    pub auto_create_tasks: Arc<DashMap<String, broadcast::Sender<AutoCreateEvent>>>,
    pub interrogation_gates: Arc<DashMap<String, InterrogationGate>>,
    pub preferred_provider: Arc<RwLock<Option<foundation::Provider>>>,
}
