use std::sync::Arc;

use archive::ArchiveSystem;
use foundation::Config;
use possession::PossessionEngine;
use registry::SoulRegistry;

use crate::collector::SoulCollector;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<SoulRegistry>,
    pub engine: Arc<PossessionEngine>,
    pub archive: Arc<ArchiveSystem>,
    pub collector: Arc<SoulCollector>,
    pub config: Arc<Config>,
}
