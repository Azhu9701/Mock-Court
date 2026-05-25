use async_trait::async_trait;

use crate::error::Result;
use crate::models::*;

#[async_trait]
pub trait Storage: Send + Sync {
    // Soul (File System)
    async fn read_soul(&self, name: &str) -> Result<SoulProfile>;
    async fn write_soul(&self, profile: &SoulProfile) -> Result<()>;
    async fn delete_soul(&self, name: &str) -> Result<()>;
    async fn list_soul_names(&self) -> Result<Vec<String>>;

    // Registry (FS + in-memory cache)
    async fn read_registry(&self) -> Result<Registry>;
    async fn write_registry(&self, registry: &Registry) -> Result<()>;

    // Session (SQLite)
    async fn create_session(&self, session: &Session) -> Result<()>;
    async fn update_session(&self, session: &Session) -> Result<()>;
    async fn delete_session(&self, id: &str) -> Result<()>;
    async fn get_session(&self, id: &str) -> Result<Session>;
    async fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>>;

    // Messages (SQLite)
    async fn append_message(&self, msg: &Message) -> Result<()>;
    async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>>;
    async fn delete_messages_from_seq(&self, session_id: &str, seq: i64) -> Result<u32>;

    // Call Records (SQLite + YAML)
    async fn record_call(&self, record: &CallRecord) -> Result<()>;
    async fn query_call_records(&self, filter: &CallFilter) -> Result<Vec<CallRecord>>;

    // Archive (File System)
    async fn archive_soul_output(&self, session_id: &str, soul: &str, content: &str) -> Result<String>;
    async fn archive_synthesis(&self, session_id: &str, content: &str) -> Result<String>;
    async fn read_archive(&self, path: &str) -> Result<String>;

    // Fulltext Search (FTS5)
    async fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<crate::sqlite::KnowledgeResult>>;
    async fn rebuild_fts(&self) -> Result<usize>;

    // Soul Revisions (SQLite)
    async fn insert_soul_revision(&self, revision: &SoulRevision) -> Result<()>;
    async fn get_soul_revisions(&self, filter: &SoulRevisionFilter) -> Result<Vec<SoulRevision>>;

    // Blind Spots (SQLite)
    async fn insert_blind_spot(&self, blind_spot: &BlindSpot) -> Result<()>;
    async fn update_blind_spot(&self, blind_spot: &BlindSpot) -> Result<()>;
    async fn get_blind_spots(&self, filter: &BlindSpotFilter) -> Result<Vec<BlindSpot>>;

    // Knowledge Cards (SQLite)
    async fn insert_knowledge_card(&self, card: &KnowledgeCard) -> Result<()>;
    async fn update_knowledge_card(&self, card: &KnowledgeCard) -> Result<()>;
    async fn get_knowledge_cards(&self, filter: &KnowledgeCardFilter) -> Result<Vec<KnowledgeCard>>;

    // Knowledge Topics
    async fn list_knowledge_topics(&self, mode: Option<&str>, limit: usize, offset: usize) -> Result<Vec<crate::KnowledgeTopic>>;

    // Revision Proposals (SQLite)
    async fn insert_revision_proposal(&self, proposal: &RevisionProposal) -> Result<()>;
    async fn update_revision_proposal(&self, proposal: &RevisionProposal) -> Result<()>;
    async fn get_revision_proposals(&self, soul_name: Option<&str>, status: Option<ProposalStatus>) -> Result<Vec<RevisionProposal>>;

    // Session Observations (claude-mem style)
    async fn insert_session_observations(&self, observations: &[SessionObservation]) -> Result<()>;
    async fn get_session_observations(&self, session_id: &str) -> Result<Vec<SessionObservation>>;
    async fn get_observations_by_soul(&self, soul_name: &str, limit: u32) -> Result<Vec<SessionObservation>>;
    async fn update_session_digest(&self, session_id: &str, summary: &str) -> Result<()>;

    // Annotations (marginalia 模式 — 事后批注)
    async fn insert_annotations(&self, annotations: &[Annotation]) -> Result<()>;
    async fn get_annotations(&self, session_id: &str) -> Result<Vec<Annotation>>;

    // Session Reviews (实践反馈闭环)
    async fn insert_session_review(&self, review: &SessionReview) -> Result<()>;
    async fn get_session_review(&self, session_id: &str) -> Result<Option<SessionReview>>;
    async fn get_recent_reviews(&self, limit: u32) -> Result<Vec<SessionReview>>;

    // Health
    async fn health_check(&self) -> Result<HealthStatus>;
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub ok: bool,
    pub sqlite_ok: bool,
    pub fs_ok: bool,
    pub yaml_count: usize,
    pub sqlite_record_count: usize,
    pub soul_files_count: usize,
    pub registry_entries_count: usize,
}
