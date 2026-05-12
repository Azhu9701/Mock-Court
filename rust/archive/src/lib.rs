mod analytics;
mod archive;
pub mod audit;
mod call_records;
pub mod cost_tracking;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use foundation::{
    CallFilter, FailureAlert, FoundationError, PossessionMode, Result, Session, SessionFilter,
    SessionSummary, Storage,
};

pub use analytics::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionDetail {
    pub session: Session,
    pub messages: Vec<foundation::Message>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveVerification {
    pub session_id: String,
    pub ok: bool,
    pub expected_files: usize,
    pub found_files: usize,
    pub missing_files: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SummonStats {
    pub total_calls: usize,
    pub unique_souls_called: usize,
    pub total_souls_available: usize,
    pub total_tokens: u64,
    pub by_mode: HashMap<PossessionMode, usize>,
    pub by_soul: Vec<SoulCallStats>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SoulCallStats {
    pub soul_name: String,
    pub call_count: usize,
    pub effective_count: usize,
    pub partial_count: usize,
    pub invalid_count: usize,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SoulAlert {
    pub soul_name: String,
    pub alert_type: AlertType,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum AlertType {
    NeverSummoned,
    UnsummonedLongDuration,
    LowEffectiveness,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BoundaryReview {
    pub soul_name: String,
    pub effective_rate: f64,
    pub total_calls: usize,
    pub threshold: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportBundle {
    pub exported_at: DateTime<Utc>,
    pub sessions: Vec<SessionDetail>,
    pub call_records: Vec<foundation::CallRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExportStatus {
    Pending,
    Running,
    Complete(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Period {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub struct ArchiveSystem {
    store: Arc<dyn Storage>,
    export_statuses: RwLock<HashMap<String, ExportStatus>>,
    summon_stats_cache: RwLock<Option<(SummonStats, Instant)>>,
    stats_ttl: Duration,
}

impl ArchiveSystem {
    pub fn new(store: Arc<dyn Storage>) -> Self {
        ArchiveSystem {
            store,
            export_statuses: RwLock::new(HashMap::new()),
            summon_stats_cache: RwLock::new(None),
            stats_ttl: Duration::from_secs(300),
        }
    }

    // ── Archive ──

    pub async fn archive_soul_output(
        &self,
        session_id: &str,
        soul: &str,
        content: &str,
    ) -> Result<String> {
        self.store.archive_soul_output(session_id, soul, content).await
    }

    pub async fn archive_synthesis(&self, session_id: &str, content: &str) -> Result<String> {
        self.store.archive_synthesis(session_id, content).await
    }

    pub async fn archive_debate(
        &self,
        session_id: &str,
        soul_a: &str,
        soul_b: &str,
        out_a: &str,
        out_b: &str,
    ) -> Result<(String, String)> {
        let path_a = self
            .store
            .archive_soul_output(session_id, soul_a, out_a)
            .await?;
        let path_b = self
            .store
            .archive_soul_output(session_id, soul_b, out_b)
            .await?;
        Ok((path_a, path_b))
    }

    // ── Call Records ──

    pub async fn record_call(&self, record: &foundation::CallRecord) -> Result<()> {
        self.store.record_call(record).await
    }

    pub async fn query_call_records(
        &self,
        filter: &CallFilter,
    ) -> Result<Vec<foundation::CallRecord>> {
        self.store.query_call_records(filter).await
    }

    // ── Sessions ──

    pub async fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>> {
        self.store.list_sessions(filter).await
    }

    pub async fn get_session_detail(&self, id: &str) -> Result<SessionDetail> {
        let session = self.store.get_session(id).await?;
        let messages = self.store.get_messages(id).await?;
        Ok(SessionDetail { session, messages })
    }

    pub async fn update_session(&self, session: &Session) -> Result<()> {
        self.store.update_session(session).await
    }

    pub async fn delete_session(&self, id: &str) -> Result<()> {
        self.store.delete_session(id).await
    }

    pub async fn create_session(&self, session: &Session) -> Result<()> {
        self.store.create_session(session).await
    }

    pub async fn append_message(&self, msg: &foundation::Message) -> Result<()> {
        self.store.append_message(msg).await
    }

    // ── Verify ──

    pub async fn verify_archive(&self, session_id: &str) -> Result<ArchiveVerification> {
        verify_archive_impl(&*self.store, session_id).await
    }

    // ── Export ──

    pub async fn export_archive(&self) -> Result<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        {
            let mut statuses = self.export_statuses.write().map_err(|e| {
                FoundationError::InvalidState(e.to_string())
            })?;
            statuses.insert(task_id.clone(), ExportStatus::Pending);
        }

        let store = self.store.clone();
        let statuses_map = Arc::new(RwLock::new(HashMap::new()));
        {
            let mut sm = statuses_map.write().map_err(|e| {
                FoundationError::InvalidState(e.to_string())
            })?;
            sm.insert(task_id.clone(), ExportStatus::Pending);
        }
        {
            let mut statuses = self.export_statuses.write().map_err(|e| {
                FoundationError::InvalidState(e.to_string())
            })?;
            statuses.insert(task_id.clone(), ExportStatus::Running);
        }

        let tid = task_id.clone();
        tokio::spawn(async move {
            let result = build_export(&*store).await;
            let mut sm = statuses_map.write().unwrap();
            match result {
                Ok((_bundle, path)) => {
                    sm.insert(tid.clone(), ExportStatus::Complete(path));
                }
                Err(e) => {
                    sm.insert(tid.clone(), ExportStatus::Failed(e.to_string()));
                }
            }
        });

        Ok(task_id)
    }

    pub fn export_status(&self, task_id: &str) -> Option<ExportStatus> {
        self.export_statuses
            .read()
            .ok()?
            .get(task_id)
            .cloned()
    }

    // ── Analytics ──

    pub async fn get_summon_stats(&self, period: Period) -> Result<SummonStats> {
        {
            let cache = self.summon_stats_cache.read().map_err(|e| {
                FoundationError::InvalidState(e.to_string())
            })?;
            if let Some((ref stats, ts)) = *cache {
                if ts.elapsed() < self.stats_ttl {
                    return Ok(stats.clone());
                }
            }
        }

        let stats = compute_summon_stats(&*self.store, &period).await?;

        let mut cache = self.summon_stats_cache.write().map_err(|e| {
            FoundationError::InvalidState(e.to_string())
        })?;
        *cache = Some((stats.clone(), Instant::now()));

        Ok(stats)
    }

    pub async fn get_soul_effectiveness(&self, soul: &str) -> Result<EffectivenessTrend> {
        compute_soul_effectiveness(&*self.store, soul).await
    }

    pub async fn get_mode_distribution(&self) -> Result<HashMap<PossessionMode, usize>> {
        compute_mode_distribution(&*self.store).await
    }

    pub async fn detect_unsummoned_souls(
        &self,
        threshold_days: u32,
    ) -> Result<Vec<SoulAlert>> {
        detect_unsummoned_souls_impl(&*self.store, threshold_days).await
    }

    pub async fn detect_low_effectiveness(
        &self,
        threshold: f64,
    ) -> Result<Vec<BoundaryReview>> {
        detect_low_effectiveness_impl(&*self.store, threshold).await
    }

    pub fn invalidate_stats_cache(&self) {
        if let Ok(mut cache) = self.summon_stats_cache.write() {
            *cache = None;
        }
    }

    // ── Knowledge ──

    pub async fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<foundation::KnowledgeResult>> {
        self.store.search_knowledge(query, limit).await
    }

    pub async fn rebuild_fts(&self) -> Result<usize> {
        self.store.rebuild_fts().await
    }

    pub async fn list_knowledge_topics(&self, mode: Option<&str>, limit: usize, offset: usize) -> Result<Vec<foundation::KnowledgeTopic>> {
        self.store.list_knowledge_topics(mode, limit, offset).await
    }

    pub async fn get_knowledge_cards_list(&self, filter: &foundation::KnowledgeCardFilter) -> Result<Vec<foundation::KnowledgeCard>> {
        self.store.get_knowledge_cards(filter).await
    }

    // ── Audit ──

    pub async fn check_failure_conditions(&self) -> Result<Vec<FailureAlert>> {
        audit::AuditEngine::check_all(&*self.store).await
    }

    pub async fn check_soul_failure_conditions(&self, soul_name: &str) -> Result<Vec<FailureAlert>> {
        audit::AuditEngine::check_soul(&*self.store, soul_name).await
    }
}

// ── Shared helpers (used by ArchiveSystem + analytics) ──

#[derive(Debug, Clone, serde::Serialize)]
pub struct EffectivenessTrend {
    pub soul_name: String,
    pub total_calls: usize,
    pub effective: usize,
    pub partial: usize,
    pub invalid: usize,
    pub effective_rate: f64,
}

async fn verify_archive_impl(store: &dyn Storage, session_id: &str) -> Result<ArchiveVerification> {
    let session = store.get_session(session_id).await?;
    let expected = expected_files(&session);
    let dir = archive_session_dir(session_id);

    let mut found = 0;
    let mut missing = Vec::new();
    for file in &expected {
        let path = format!("{}/{}", dir, file);
        match store.read_archive(&path).await {
            Ok(_) => found += 1,
            Err(_) => missing.push(file.clone()),
        }
    }

    Ok(ArchiveVerification {
        session_id: session_id.to_string(),
        ok: missing.is_empty(),
        expected_files: expected.len(),
        found_files: found,
        missing_files: missing,
    })
}

fn expected_files(session: &Session) -> Vec<String> {
    match session.mode {
        PossessionMode::Single => {
            vec![
                format!("{}.md", session.title),
                format!("{}_record.md", session.title),
            ]
        }
        PossessionMode::Conference => {
            vec!["synthesis.md".to_string()]
        }
        PossessionMode::Debate => {
            vec![
                "debate_A_vs_B.md".to_string(),
                "verdict.md".to_string(),
            ]
        }
        PossessionMode::Relay => {
            vec!["relay_output.md".to_string()]
        }
        PossessionMode::Learn => {
            vec!["learning_output.md".to_string()]
        }
        PossessionMode::PracticeOpening => {
            vec![
                "P1_field.md".to_string(),
                "P2_digestion.md".to_string(),
                "P3_revision.md".to_string(),
                "P4_action.md".to_string(),
            ]
        }
    }
}

fn archive_session_dir(session_id: &str) -> String {
    let now = Utc::now();
    format!(
        "{}/{}/{}/{}",
        now.format("%Y"),
        now.format("%m"),
        now.format("%d"),
        session_id
    )
}

const EXPORT_PAGE_SIZE: u32 = 50;

async fn build_export(store: &dyn Storage) -> Result<(ExportBundle, String)> {
    let call_records = store
        .query_call_records(&CallFilter::default())
        .await?;

    let mut session_details = Vec::new();
    let mut offset: u32 = 0;
    loop {
        let filter = SessionFilter {
            limit: Some(EXPORT_PAGE_SIZE),
            offset: Some(offset),
            ..Default::default()
        };
        let page = store.list_sessions(&filter).await?;
        if page.is_empty() {
            break;
        }
        for s in &page {
            let session = store.get_session(&s.id).await?;
            let messages = store.get_messages(&s.id).await?;
            session_details.push(SessionDetail { session, messages });
        }
        offset += EXPORT_PAGE_SIZE;
    }

    let bundle = ExportBundle {
        exported_at: Utc::now(),
        sessions: session_details,
        call_records,
    };

    let export_dir = std::path::PathBuf::from("data/exports");
    std::fs::create_dir_all(&export_dir)?;
    let task_id = uuid::Uuid::new_v4().to_string();
    let path = export_dir.join(format!("{}.json", task_id));
    let tmp = export_dir.join(format!("{}.tmp", task_id));
    let content = serde_json::to_string_pretty(&bundle)?;
    std::fs::write(&tmp, &content)?;
    std::fs::rename(&tmp, &path)?;

    Ok((bundle, path.to_string_lossy().to_string()))
}
