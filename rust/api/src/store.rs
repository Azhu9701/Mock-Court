use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use foundation::{
    Annotation, BlindSpot, BlindSpotFilter, CallFilter, CallRecord, HealthStatus, KnowledgeCard,
    KnowledgeCardFilter, KnowledgeResult, KnowledgeTopic, Message, Registry, Result, RevisionProposal,
    Session, SessionFilter, SessionObservation, SessionReview, SessionSummary, SoulProfile,
    SoulRevision, SoulRevisionFilter, Storage, ProposalStatus,
};

use foundation::{FileStore, SqliteDb};

pub struct AppStore {
    fs: Arc<FileStore>,
    db: Arc<SqliteDb>,
}

impl AppStore {
    pub fn new(data_dir: &str) -> Result<Self> {
        let base = PathBuf::from(data_dir);
        let mut fs = FileStore::new(
            base.join("souls"),
            base.join("archive"),
            base.join("registry.yaml"),
            base.join("call_records.yaml"),
        )?;
        // 加载内部魂目录：优先 WANMINFAN_SOULS_INTERNAL_DIR 环境变量，fallback 到 data/souls-internal/
        if let Ok(internal) = std::env::var("WANMINFAN_SOULS_INTERNAL_DIR") {
            fs.set_souls_internal_dir(PathBuf::from(internal));
        } else {
            let default = base.join("souls-internal");
            if default.exists() {
                fs.set_souls_internal_dir(default);
            }
        }
        let fs = Arc::new(fs);
        std::fs::create_dir_all(base.join("db"))?;
        let db = Arc::new(SqliteDb::open(&base.join("db/app.db"))?);
        Ok(AppStore { fs, db })
    }

    pub fn db(&self) -> Arc<SqliteDb> {
        self.db.clone()
    }
}

#[async_trait]
impl Storage for AppStore {
    async fn read_soul(&self, name: &str) -> Result<SoulProfile> {
        self.fs.read_soul(name)
    }

    async fn write_soul(&self, profile: &SoulProfile) -> Result<()> {
        self.fs.write_soul(profile)
    }

    async fn delete_soul(&self, name: &str) -> Result<()> {
        self.fs.delete_soul(name)
    }

    async fn list_soul_names(&self) -> Result<Vec<String>> {
        self.fs.list_soul_names()
    }

    async fn read_registry(&self) -> Result<Registry> {
        self.fs.read_registry_raw()
    }

    async fn write_registry(&self, registry: &Registry) -> Result<()> {
        self.fs.write_registry_raw(registry)
    }

    async fn create_session(&self, session: &Session) -> Result<()> {
        self.db.insert_session(session)
    }

    async fn update_session(&self, session: &Session) -> Result<()> {
        self.db.update_session(session)
    }

    async fn delete_session(&self, id: &str) -> Result<()> {
        self.db.delete_session(id)
    }

    async fn get_session(&self, id: &str) -> Result<Session> {
        self.db.get_session(id)
    }

    async fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>> {
        self.db.list_sessions(filter)
    }

    async fn append_message(&self, msg: &Message) -> Result<()> {
        self.db.append_message(msg)
    }

    async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        self.db.get_messages(session_id)
    }

    async fn delete_messages_from_seq(&self, session_id: &str, seq: i64) -> Result<u32> {
        self.db.delete_messages_from_seq(session_id, seq)
    }

    async fn record_call(&self, record: &CallRecord) -> Result<()> {
        self.db.insert_call_record(record)?;
        self.fs.append_call_record_yaml(record)?;
        Ok(())
    }

    async fn query_call_records(&self, filter: &CallFilter) -> Result<Vec<CallRecord>> {
        self.db.query_call_records(filter)
    }

    async fn archive_soul_output(&self, session_id: &str, soul: &str, content: &str) -> Result<String> {
        let filename = format!("{}.md", soul);
        self.fs.archive_output(session_id, &filename, content)
    }

    async fn archive_synthesis(&self, session_id: &str, content: &str) -> Result<String> {
        self.fs.archive_output(session_id, "synthesis.md", content)
    }

    async fn read_archive(&self, path: &str) -> Result<String> {
        self.fs.read_archive_path(path)
    }

    async fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeResult>> {
        self.db.search_knowledge(query, limit)
    }

    async fn rebuild_fts(&self) -> Result<usize> {
        self.db.rebuild_fts()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let sqlite_ok = self.db.with_conn(|_| Ok(())).is_ok();
        let sqlite_record_count = self.db.count_call_records().unwrap_or(0);
        Ok(HealthStatus {
            ok: sqlite_ok,
            sqlite_ok,
            fs_ok: true,
            yaml_count: self.fs.count_call_records_yaml().unwrap_or(0),
            sqlite_record_count,
            soul_files_count: self.fs.count_soul_files().unwrap_or(0),
            registry_entries_count: self.fs.registry_entry_count(),
        })
    }

    // Soul Revisions
    async fn insert_soul_revision(&self, revision: &SoulRevision) -> Result<()> {
        self.db.insert_soul_revision(revision)
    }

    async fn get_soul_revisions(&self, filter: &SoulRevisionFilter) -> Result<Vec<SoulRevision>> {
        self.db.get_soul_revisions(filter)
    }

    // Blind Spots
    async fn insert_blind_spot(&self, blind_spot: &BlindSpot) -> Result<()> {
        self.db.insert_blind_spot(blind_spot)
    }

    async fn update_blind_spot(&self, blind_spot: &BlindSpot) -> Result<()> {
        self.db.update_blind_spot(blind_spot)
    }

    async fn get_blind_spots(&self, filter: &BlindSpotFilter) -> Result<Vec<BlindSpot>> {
        self.db.get_blind_spots(filter)
    }

    // Knowledge Cards
    async fn insert_knowledge_card(&self, card: &KnowledgeCard) -> Result<()> {
        self.db.insert_knowledge_card(card)
    }

    async fn update_knowledge_card(&self, card: &KnowledgeCard) -> Result<()> {
        self.db.update_knowledge_card(card)
    }

    async fn get_knowledge_cards(&self, filter: &KnowledgeCardFilter) -> Result<Vec<KnowledgeCard>> {
        self.db.get_knowledge_cards(filter)
    }

    async fn list_knowledge_topics(&self, mode: Option<&str>, limit: usize, offset: usize) -> Result<Vec<KnowledgeTopic>> {
        self.db.list_knowledge_topics(mode, limit, offset)
    }

    // Revision Proposals
    async fn insert_revision_proposal(&self, proposal: &RevisionProposal) -> Result<()> {
        self.db.insert_revision_proposal(proposal)
    }

    async fn update_revision_proposal(&self, proposal: &RevisionProposal) -> Result<()> {
        self.db.update_revision_proposal(proposal)
    }

    async fn get_revision_proposals(&self, soul_name: Option<&str>, status: Option<ProposalStatus>) -> Result<Vec<RevisionProposal>> {
        self.db.get_revision_proposals(soul_name, status)
    }

    async fn insert_session_observations(&self, observations: &[SessionObservation]) -> Result<()> {
        self.db.insert_session_observations(observations)
    }

    async fn get_session_observations(&self, session_id: &str) -> Result<Vec<SessionObservation>> {
        self.db.get_session_observations(session_id)
    }

    async fn get_observations_by_soul(&self, soul_name: &str, limit: u32) -> Result<Vec<SessionObservation>> {
        self.db.get_observations_by_soul(soul_name, limit)
    }

    async fn update_session_digest(&self, session_id: &str, summary: &str) -> Result<()> {
        self.db.update_session_digest(session_id, summary)
    }

    async fn insert_annotations(&self, annotations: &[Annotation]) -> Result<()> {
        self.db.insert_annotations(annotations)
    }

    async fn get_annotations(&self, session_id: &str) -> Result<Vec<Annotation>> {
        self.db.get_annotations(session_id)
    }

    async fn insert_session_review(&self, review: &SessionReview) -> Result<()> {
        self.db.insert_session_review(review)
    }

    async fn get_session_review(&self, session_id: &str) -> Result<Option<SessionReview>> {
        self.db.get_session_review(session_id)
    }

    async fn get_recent_reviews(&self, limit: u32) -> Result<Vec<SessionReview>> {
        self.db.get_recent_reviews(limit)
    }
}
