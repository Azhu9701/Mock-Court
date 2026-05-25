use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use std::path::Path;

use crate::error::{FoundationError, Result};
use crate::models::*;

pub struct SqliteDb {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteDb {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let manager = SqliteConnectionManager::file(path).with_init(|conn| {
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
            conn.execute_batch("PRAGMA synchronous = NORMAL;")?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
            Ok(())
        });
        let pool = Pool::builder()
            .max_size(5)
            .build(manager)
            .map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        let conn = pool.get().map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        Self::migrate(&conn)?;
        Ok(SqliteDb { pool })
    }

    fn migrate(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL,
                mode        TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'active',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS messages (
                id          TEXT PRIMARY KEY,
                session_id  TEXT NOT NULL REFERENCES sessions(id),
                role        TEXT NOT NULL,
                soul_name   TEXT,
                content     TEXT NOT NULL,
                seq         INTEGER NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS call_records (
                id              TEXT PRIMARY KEY,
                session_id      TEXT NOT NULL REFERENCES sessions(id),
                soul_name       TEXT NOT NULL,
                mode            TEXT NOT NULL,
                task_summary    TEXT NOT NULL,
                effectiveness   TEXT NOT NULL,
                notes           TEXT NOT NULL,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                prompt_tokens   INTEGER NOT NULL DEFAULT 0,
                completion_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens    INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS soul_revisions (
                id              TEXT PRIMARY KEY,
                soul_name       TEXT NOT NULL,
                revision_type   TEXT NOT NULL,
                description     TEXT NOT NULL,
                old_value       TEXT,
                new_value       TEXT,
                reviewer        TEXT,
                reviewed_at     TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS blind_spots (
                id              TEXT PRIMARY KEY,
                soul_name       TEXT NOT NULL,
                dimension       TEXT NOT NULL,
                description     TEXT NOT NULL,
                detected_at     TEXT NOT NULL DEFAULT (datetime('now')),
                resolved_at     TEXT,
                resolved_by     TEXT,
                resolution      TEXT
            );

            CREATE TABLE IF NOT EXISTS knowledge_cards (
                id              TEXT PRIMARY KEY,
                title           TEXT NOT NULL,
                content         TEXT NOT NULL,
                source_soul     TEXT,
                source_session  TEXT,
                tags            TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS revision_proposals (
                id              TEXT PRIMARY KEY,
                soul_name       TEXT NOT NULL,
                proposal_type   TEXT NOT NULL,
                title           TEXT NOT NULL,
                description     TEXT NOT NULL,
                proposed_changes TEXT NOT NULL,
                status          TEXT NOT NULL DEFAULT 'pending',
                created_by      TEXT NOT NULL,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                reviewed_at     TEXT,
                reviewer        TEXT,
                review_notes    TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, seq);
            CREATE INDEX IF NOT EXISTS idx_call_records_soul ON call_records(soul_name);
            CREATE INDEX IF NOT EXISTS idx_call_records_session ON call_records(session_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_mode ON sessions(mode);
            CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at);
            CREATE INDEX IF NOT EXISTS idx_soul_revisions_soul ON soul_revisions(soul_name);
            CREATE INDEX IF NOT EXISTS idx_blind_spots_soul ON blind_spots(soul_name);
            CREATE INDEX IF NOT EXISTS idx_knowledge_cards_soul ON knowledge_cards(source_soul);
            CREATE INDEX IF NOT EXISTS idx_revision_proposals_soul ON revision_proposals(soul_name);
            CREATE INDEX IF NOT EXISTS idx_revision_proposals_status ON revision_proposals(status);

            CREATE TABLE IF NOT EXISTS llm_cache (
                hash TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                response_content TEXT NOT NULL,
                usage_json TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_llm_cache_created ON llm_cache(created_at);

            CREATE TABLE IF NOT EXISTS session_observations (
                id              TEXT PRIMARY KEY,
                session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                soul_name       TEXT,
                obs_type        TEXT NOT NULL,
                title           TEXT NOT NULL,
                content         TEXT NOT NULL,
                source_seq      INTEGER,
                read_tokens     INTEGER NOT NULL DEFAULT 0,
                work_tokens     INTEGER NOT NULL DEFAULT 0,
                confidence      REAL NOT NULL DEFAULT 0.7,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_session_obs_session ON session_observations(session_id);
            CREATE INDEX IF NOT EXISTS idx_session_obs_soul    ON session_observations(soul_name);
            CREATE INDEX IF NOT EXISTS idx_session_obs_type    ON session_observations(obs_type);
            CREATE INDEX IF NOT EXISTS idx_session_obs_created ON session_observations(created_at DESC);

            CREATE TABLE IF NOT EXISTS session_reviews (
                id              TEXT PRIMARY KEY,
                session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                most_unexpected TEXT NOT NULL DEFAULT '',
                already_known   TEXT NOT NULL DEFAULT '',
                self_negation   TEXT NOT NULL DEFAULT '',
                empty_chair     TEXT NOT NULL DEFAULT '',
                effectiveness   TEXT NOT NULL DEFAULT '',
                effectiveness_note TEXT NOT NULL DEFAULT '',
                interrogation_passed INTEGER,
                interrogation_reason TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_reviews_session ON session_reviews(session_id);
            CREATE INDEX IF NOT EXISTS idx_reviews_created  ON session_reviews(created_at DESC);

            CREATE TABLE IF NOT EXISTS annotations (
                id              TEXT PRIMARY KEY,
                session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                source_soul     TEXT NOT NULL,
                target_soul     TEXT NOT NULL,
                target_excerpt  TEXT NOT NULL,
                comment         TEXT NOT NULL,
                kind            TEXT NOT NULL DEFAULT 'nuance',
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_annotations_session ON annotations(session_id);
            CREATE INDEX IF NOT EXISTS idx_annotations_target  ON annotations(target_soul);",
        )?;

        // Migrate: add token columns to existing call_records tables (ignore error if already exists)
        for col in &["prompt_tokens", "completion_tokens", "total_tokens"] {
            let sql = format!("ALTER TABLE call_records ADD COLUMN {} INTEGER NOT NULL DEFAULT 0", col);
            let _ = conn.execute_batch(&sql);
        }

        // Migrate: add digest columns to existing sessions tables (ignore error if already exists)
        for sql in &[
            "ALTER TABLE sessions ADD COLUMN digest_summary TEXT",
            "ALTER TABLE sessions ADD COLUMN digest_at TEXT",
            "ALTER TABLE session_reviews ADD COLUMN practice_commitment TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE session_reviews ADD COLUMN practice_horizon TEXT NOT NULL DEFAULT ''",
        ] {
            let _ = conn.execute_batch(sql);
        }

        Ok(())
    }

    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let conn = self.pool.get().map_err(|e| FoundationError::InvalidState(e.to_string()))?;
        f(&conn)
    }

    // Sessions
    pub fn insert_session(&self, session: &Session) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO sessions (id, title, mode, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    session.id,
                    session.title,
                    session.mode.as_str(),
                    status_to_str(&session.status),
                    session.created_at.to_rfc3339(),
                    session.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn update_session(&self, session: &Session) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE sessions SET title=?1, mode=?2, status=?3, updated_at=?4 WHERE id=?5",
                params![
                    session.title,
                    session.mode.as_str(),
                    status_to_str(&session.status),
                    Utc::now().to_rfc3339(),
                    session.id,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_session(&self, id: &str) -> Result<Session> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, title, mode, status, created_at, updated_at, digest_summary, digest_at FROM sessions WHERE id=?1",
                params![id],
                |row| {
                    let created_at: String = row.get(4)?;
                    let updated_at: String = row.get(5)?;
                    let digest_summary: Option<String> = row.get(6).ok();
                    let digest_at_str: Option<String> = row.get(7).ok();
                    let digest_at = digest_at_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
                    });
                    Ok(Session {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        mode: PossessionMode::from_str(&row.get::<_, String>(2)?).unwrap_or(PossessionMode::Single),
                        status: str_to_status(&row.get::<_, String>(3)?),
                        created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().with_timezone(&Utc),
                        updated_at: DateTime::parse_from_rfc3339(&updated_at).unwrap_or_default().with_timezone(&Utc),
                        digest_summary,
                        digest_at,
                    })
                },
            ).map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => FoundationError::SessionNotFound(id.to_string()),
                e => FoundationError::Sqlite(e),
            })
        })
    }

    pub fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>> {
        self.with_conn(|conn| {
            let mut sql = String::from(
                "SELECT s.id, s.title, s.mode, s.status, s.created_at,
                        COUNT(DISTINCT m.id) as msg_count,
                        COUNT(DISTINCT cr.soul_name) as soul_count,
                        COALESCE(SUM(cr.total_tokens), 0) as total_tokens,
                        s.digest_summary,
                        (SELECT COUNT(*) FROM session_observations so WHERE so.session_id = s.id) as obs_count
                 FROM sessions s
                 LEFT JOIN messages m ON s.id = m.session_id
                 LEFT JOIN call_records cr ON s.id = cr.session_id
                 WHERE 1=1"
            );
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

            if let Some(ref mode) = filter.mode {
                sql.push_str(&format!(" AND s.mode = ?{}", param_values.len() + 1));
                param_values.push(Box::new(mode.as_str().to_string()));
            }
            if let Some(ref status) = filter.status {
                sql.push_str(&format!(" AND s.status = ?{}", param_values.len() + 1));
                param_values.push(Box::new(status_to_str(status).to_string()));
            }

            sql.push_str(" GROUP BY s.id ORDER BY s.created_at DESC");

            if let Some(limit) = filter.limit {
                sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
                param_values.push(Box::new(limit as i64));
            }
            if let Some(offset) = filter.offset {
                sql.push_str(&format!(" OFFSET ?{}", param_values.len() + 1));
                param_values.push(Box::new(offset as i64));
            }

            let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                let created_at: String = row.get(4)?;
                let digest_summary: Option<String> = row.get(8).ok();
                let obs_count: i64 = row.get(9).unwrap_or(0);
                Ok(SessionSummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    mode: PossessionMode::from_str(&row.get::<_, String>(2)?).unwrap_or(PossessionMode::Single),
                    status: str_to_status(&row.get::<_, String>(3)?),
                    created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().with_timezone(&Utc),
                    message_count: row.get(5)?,
                    soul_count: row.get::<_, i64>(6)? as u32,
                    total_tokens: row.get::<_, i64>(7)? as u32,
                    digest_summary,
                    observation_count: obs_count as u32,
                })
            })?;

            let mut results = vec![];
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    // Messages
    pub fn append_message(&self, msg: &Message) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO messages (id, session_id, role, soul_name, content, seq, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    msg.id,
                    msg.session_id,
                    role_to_str(&msg.role),
                    msg.soul_name.as_deref(),
                    msg.content,
                    msg.seq,
                    msg.created_at.to_rfc3339(),
                ],
            )?;
            if !msg.content.is_empty() {
                Self::ensure_fts5(conn)?;
                let created_at = msg.created_at.to_rfc3339();
                let (mode, task_summary): (String, String) = conn
                    .query_row(
                        "SELECT COALESCE(mode, 'unknown'), COALESCE(title, '') FROM sessions WHERE id = ?1",
                        params![msg.session_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .unwrap_or_else(|_| ("unknown".into(), String::new()));
                conn.execute(
                    "INSERT INTO knowledge_fts (soul_name, content, mode, task_summary, created_at, session_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        msg.soul_name.as_deref().unwrap_or(""),
                        msg.content,
                        mode,
                        task_summary,
                        created_at,
                        msg.session_id,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn rebuild_fts(&self) -> Result<usize> {
        self.with_conn(|conn| {
            Self::ensure_fts5(conn)?;
            conn.execute("DELETE FROM knowledge_fts", [])?;

            let mut stmt = conn.prepare(
                "SELECT m.soul_name, m.content, COALESCE(s.mode, 'unknown'), COALESCE(s.title, ''), m.created_at, m.session_id
                 FROM messages m LEFT JOIN sessions s ON m.session_id = s.id
                 WHERE m.content != ''"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?;

            let mut count = 0usize;
            for row in rows {
                let (soul_name, content, mode, task_summary, created_at, session_id) = row?;
                conn.execute(
                    "INSERT INTO knowledge_fts (soul_name, content, mode, task_summary, created_at, session_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![soul_name, content, mode, task_summary, created_at, session_id],
                )?;
                count += 1;
            }
            Ok(count)
        })
    }

    pub fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, role, soul_name, content, seq, created_at FROM messages WHERE session_id=?1 ORDER BY seq"
            )?;
            let rows = stmt.query_map(params![session_id], |row| {
                let created_at: String = row.get(6)?;
                Ok(Message {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: str_to_role(&row.get::<_, String>(2)?),
                    soul_name: row.get(3)?,
                    content: row.get(4)?,
                    seq: row.get(5)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().with_timezone(&Utc),
                })
            })?;

            let mut results = vec![];
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    pub fn delete_messages_from_seq(&self, session_id: &str, seq: i64) -> Result<u32> {
        self.with_conn(|conn| {
            let deleted = conn.execute(
                "DELETE FROM messages WHERE session_id = ?1 AND seq >= ?2",
                params![session_id, seq],
            )?;
            Ok(deleted as u32)
        })
    }

    // Call Records
    pub fn insert_call_record(&self, record: &CallRecord) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO call_records (id, session_id, soul_name, mode, task_summary, effectiveness, notes, created_at, prompt_tokens, completion_tokens, total_tokens) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    record.id,
                    record.session_id,
                    record.soul_name,
                    record.mode.as_str(),
                    record.task_summary,
                    effectiveness_to_str(&record.effectiveness),
                    record.notes,
                    record.created_at.to_rfc3339(),
                    record.usage.prompt_tokens,
                    record.usage.completion_tokens,
                    record.usage.total_tokens,
                ],
            )?;
            Ok(())
        })
    }

    pub fn query_call_records(&self, filter: &CallFilter) -> Result<Vec<CallRecord>> {
        self.with_conn(|conn| {
            let mut sql = String::from("SELECT id, session_id, soul_name, mode, task_summary, effectiveness, notes, created_at, prompt_tokens, completion_tokens, total_tokens FROM call_records WHERE 1=1");
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

            if let Some(ref soul) = filter.soul_name {
                sql.push_str(&format!(" AND soul_name = ?{}", param_values.len() + 1));
                param_values.push(Box::new(soul.clone()));
            }
            if let Some(ref mode) = filter.mode {
                sql.push_str(&format!(" AND mode = ?{}", param_values.len() + 1));
                param_values.push(Box::new(mode.as_str().to_string()));
            }
            if let Some(ref eff) = filter.effectiveness {
                sql.push_str(&format!(" AND effectiveness = ?{}", param_values.len() + 1));
                param_values.push(Box::new(effectiveness_to_str(eff).to_string()));
            }

            sql.push_str(" ORDER BY created_at DESC");

            if let Some(limit) = filter.limit {
                sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
                param_values.push(Box::new(limit as i64));
            }

            let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                let created_at: String = row.get(7)?;
                Ok(CallRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    soul_name: row.get(2)?,
                    mode: PossessionMode::from_str(&row.get::<_, String>(3)?).unwrap_or(PossessionMode::Single),
                    task_summary: row.get(4)?,
                    effectiveness: str_to_effectiveness(&row.get::<_, String>(5)?),
                    notes: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().with_timezone(&Utc),
                    self_negation: None,
                    empty_chair: None,
                    user_feedback: None,
                    usage: UsageStats {
                        prompt_tokens: row.get::<_, u32>(8).unwrap_or(0),
                        completion_tokens: row.get::<_, u32>(9).unwrap_or(0),
                        total_tokens: row.get::<_, u32>(10).unwrap_or(0),
                    },
                })
            })?;

            let mut results = vec![];
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    pub fn delete_session(&self, id: &str) -> Result<()> {
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            tx.execute("DELETE FROM messages WHERE session_id = ?1", params![id])?;
            tx.execute("DELETE FROM call_records WHERE session_id = ?1", params![id])?;
            tx.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
            tx.commit()?;
            Ok(())
        })
    }

    pub fn count_call_records(&self) -> Result<usize> {
        self.with_conn(|conn| {
            let count: usize = conn.query_row("SELECT COUNT(*) FROM call_records", [], |row| row.get(0))?;
            Ok(count)
        })
    }

    pub fn vacuum(&self) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute_batch("PRAGMA optimize; VACUUM;")?;
            Ok(())
        })
    }

    fn ensure_fts5(conn: &Connection) -> Result<()> {
        let needs_migration = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='knowledge_fts'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .map_or(false, |sql| !sql.contains("trigram") || !sql.contains("session_id"));

        if needs_migration {
            conn.execute_batch("DROP TABLE IF EXISTS knowledge_fts;")?;
        }
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts USING fts5(
                soul_name, content, mode, task_summary, created_at, session_id,
                tokenize='trigram'
            );"
        )?;
        Ok(())
    }

    /// Insert spaces between CJK characters so FTS5 unicode61 tokenizer can index them
    fn cjk_tokenize(text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);
        for c in text.chars() {
            if is_cjk(c) {
                result.push(' ');
                result.push(c);
                result.push(' ');
            } else {
                result.push(c);
            }
        }
        result
    }

    pub fn index_message(
        &self,
        soul_name: Option<&str>,
        content: &str,
        mode: &str,
        task_summary: &str,
        created_at: &str,
        session_id: &str,
    ) -> Result<()> {
        self.with_conn(|conn| {
            Self::ensure_fts5(conn)?;
            let content_tokenized = Self::cjk_tokenize(content);
            let summary_tokenized = Self::cjk_tokenize(task_summary);
            conn.execute(
                "INSERT INTO knowledge_fts (soul_name, content, mode, task_summary, created_at, session_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![soul_name.unwrap_or(""), content_tokenized, mode, summary_tokenized, created_at, session_id],
            )?;
            Ok(())
        })
    }

    pub fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeResult>> {
        self.with_conn(|conn| {
            if query.trim().is_empty() {
                Self::ensure_fts5(conn)?;
                let mut stmt = conn.prepare(
                    "SELECT COALESCE(m.soul_name, ''), COALESCE(m.content, ''),
                            COALESCE(s.mode, 'unknown'), COALESCE(s.title, ''),
                            m.created_at, m.session_id
                     FROM messages m LEFT JOIN sessions s ON m.session_id = s.id
                     WHERE m.content != ''
                     ORDER BY m.created_at DESC LIMIT ?1"
                )?;
                let rows = stmt.query_map(params![limit as i64], |row| {
                    let content = row.get::<_, String>(1)?;
                    let snippet = if content.chars().count() > 200 {
                        let truncated: String = content.chars().take(200).collect();
                        format!("{}...", truncated)
                    } else {
                        content
                    };
                    Ok(KnowledgeResult {
                        soul_name: row.get::<_, String>(0).ok().filter(|s| !s.is_empty()),
                        content_snippet: snippet,
                        mode: row.get(2)?,
                        task_summary: row.get(3)?,
                        created_at: row.get(4)?,
                        session_id: row.get(5)?,
                    })
                })?;

                let mut results = Vec::new();
                for row in rows {
                    results.push(row?);
                }
                return Ok(results);
            }

            Self::ensure_fts5(conn)?;
            let mut stmt = conn.prepare(
                "SELECT soul_name, snippet(knowledge_fts, 1, '<b>', '</b>', '...', 40) AS snippet,
                        mode, task_summary, created_at, session_id, rank
                 FROM knowledge_fts WHERE knowledge_fts MATCH ?1
                 ORDER BY rank LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![query, limit as i64], |row| {
                Ok(KnowledgeResult {
                    soul_name: row.get::<_, String>(0).ok().filter(|s| !s.is_empty()),
                    content_snippet: row.get(1)?,
                    mode: row.get(2)?,
                    task_summary: row.get(3)?,
                    created_at: row.get(4)?,
                    session_id: row.get(5)?,
                })
            })?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KnowledgeResult {
    pub soul_name: Option<String>,
    pub content_snippet: String,
    pub mode: String,
    pub task_summary: String,
    pub created_at: String,
    pub session_id: String,
}

fn status_to_str(s: &SessionStatus) -> &'static str {
    match s {
        SessionStatus::Active => "active",
        SessionStatus::Completed => "completed",
        SessionStatus::Archived => "archived",
        SessionStatus::Inconsistent => "inconsistent",
    }
}

fn str_to_status(s: &str) -> SessionStatus {
    match s {
        "active" => SessionStatus::Active,
        "completed" => SessionStatus::Completed,
        "archived" => SessionStatus::Archived,
        _ => SessionStatus::Inconsistent,
    }
}

fn role_to_str(r: &MessageRole) -> &'static str {
    match r {
        MessageRole::User => "user",
        MessageRole::Soul => "soul",
        MessageRole::Synthesis => "synthesis",
        MessageRole::System => "system",
    }
}

fn str_to_role(s: &str) -> MessageRole {
    match s {
        "user" => MessageRole::User,
        "soul" => MessageRole::Soul,
        "synthesis" => MessageRole::Synthesis,
        _ => MessageRole::System,
    }
}

fn effectiveness_to_str(e: &Effectiveness) -> &'static str {
    match e {
        Effectiveness::Effective => "effective",
        Effectiveness::Partial => "partial",
        Effectiveness::Invalid => "invalid",
    }
}

fn str_to_effectiveness(s: &str) -> Effectiveness {
    match s {
        "effective" => Effectiveness::Effective,
        "partial" => Effectiveness::Partial,
        _ => Effectiveness::Invalid,
    }
}

fn is_cjk(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
        || ('\u{3400}'..='\u{4DBF}').contains(&c)
        || ('\u{F900}'..='\u{FAFF}').contains(&c)
}

// ── 辅助函数 ──
fn revision_type_to_str(rt: &RevisionType) -> &'static str {
    match rt {
        RevisionType::Confirmed => "confirmed",
        RevisionType::Modified => "modified",
        RevisionType::Overturned => "overturned",
    }
}

fn str_to_revision_type(s: &str) -> RevisionType {
    match s {
        "confirmed" => RevisionType::Confirmed,
        "modified" => RevisionType::Modified,
        _ => RevisionType::Overturned,
    }
}

fn proposal_type_to_str(pt: &ProposalType) -> &'static str {
    match pt {
        ProposalType::BoundaryAdjustment => "boundary_adjustment",
        ProposalType::OntologyUpdate => "ontology_update",
        ProposalType::DomainExpansion => "domain_expansion",
        ProposalType::SelfDeclareUpdate => "self_declare_update",
        ProposalType::BlindSpotMitigation => "blind_spot_mitigation",
    }
}

fn str_to_proposal_type(s: &str) -> ProposalType {
    match s {
        "boundary_adjustment" => ProposalType::BoundaryAdjustment,
        "ontology_update" => ProposalType::OntologyUpdate,
        "domain_expansion" => ProposalType::DomainExpansion,
        "self_declare_update" => ProposalType::SelfDeclareUpdate,
        _ => ProposalType::BlindSpotMitigation,
    }
}

fn proposal_status_to_str(ps: &ProposalStatus) -> &'static str {
    match ps {
        ProposalStatus::Pending => "pending",
        ProposalStatus::Approved => "approved",
        ProposalStatus::Rejected => "rejected",
        ProposalStatus::Implemented => "implemented",
    }
}

fn str_to_proposal_status(s: &str) -> ProposalStatus {
    match s {
        "approved" => ProposalStatus::Approved,
        "rejected" => ProposalStatus::Rejected,
        "implemented" => ProposalStatus::Implemented,
        _ => ProposalStatus::Pending,
    }
}

// ── 新增表的操作方法 ──

impl SqliteDb {
    // Soul Revisions
    pub fn insert_soul_revision(&self, revision: &SoulRevision) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO soul_revisions (id, soul_name, revision_type, description, old_value, new_value, reviewer, reviewed_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    revision.id,
                    revision.soul_name,
                    revision_type_to_str(&revision.revision_type),
                    revision.description,
                    revision.old_value,
                    revision.new_value,
                    revision.reviewer,
                    revision.reviewed_at.map(|dt| dt.to_rfc3339()),
                    revision.created_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_soul_revisions(&self, filter: &SoulRevisionFilter) -> Result<Vec<SoulRevision>> {
        self.with_conn(|conn| {
            let mut sql = String::from("SELECT id, soul_name, revision_type, description, old_value, new_value, reviewer, reviewed_at, created_at FROM soul_revisions WHERE 1=1");
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

            if let Some(ref soul_name) = filter.soul_name {
                sql.push_str(&format!(" AND soul_name = ?{}", param_values.len() + 1));
                param_values.push(Box::new(soul_name.clone()));
            }
            if let Some(ref rt) = filter.revision_type {
                sql.push_str(&format!(" AND revision_type = ?{}", param_values.len() + 1));
                param_values.push(Box::new(revision_type_to_str(rt).to_string()));
            }

            sql.push_str(" ORDER BY created_at DESC");

            if let Some(limit) = filter.limit {
                sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
                param_values.push(Box::new(limit as i64));
            }
            if let Some(offset) = filter.offset {
                sql.push_str(&format!(" OFFSET ?{}", param_values.len() + 1));
                param_values.push(Box::new(offset as i64));
            }

            let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                Ok(SoulRevision {
                    id: row.get(0)?,
                    soul_name: row.get(1)?,
                    revision_type: str_to_revision_type(&row.get::<_, String>(2)?),
                    description: row.get(3)?,
                    old_value: row.get(4)?,
                    new_value: row.get(5)?,
                    reviewer: row.get(6)?,
                    reviewed_at: row.get::<_, Option<String>>(7)?.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?).unwrap_or_default().with_timezone(&Utc),
                })
            })?;

            let mut results = vec![];
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    // Blind Spots
    pub fn insert_blind_spot(&self, blind_spot: &BlindSpot) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO blind_spots (id, soul_name, dimension, description, detected_at, resolved_at, resolved_by, resolution) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    blind_spot.id,
                    blind_spot.soul_name,
                    blind_spot.dimension,
                    blind_spot.description,
                    blind_spot.detected_at.to_rfc3339(),
                    blind_spot.resolved_at.map(|dt| dt.to_rfc3339()),
                    blind_spot.resolved_by,
                    blind_spot.resolution,
                ],
            )?;
            Ok(())
        })
    }

    pub fn update_blind_spot(&self, blind_spot: &BlindSpot) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE blind_spots SET soul_name=?1, dimension=?2, description=?3, detected_at=?4, resolved_at=?5, resolved_by=?6, resolution=?7 WHERE id=?8",
                params![
                    blind_spot.soul_name,
                    blind_spot.dimension,
                    blind_spot.description,
                    blind_spot.detected_at.to_rfc3339(),
                    blind_spot.resolved_at.map(|dt| dt.to_rfc3339()),
                    blind_spot.resolved_by,
                    blind_spot.resolution,
                    blind_spot.id,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_blind_spots(&self, filter: &BlindSpotFilter) -> Result<Vec<BlindSpot>> {
        self.with_conn(|conn| {
            let mut sql = String::from("SELECT id, soul_name, dimension, description, detected_at, resolved_at, resolved_by, resolution FROM blind_spots WHERE 1=1");
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

            if let Some(ref soul_name) = filter.soul_name {
                sql.push_str(&format!(" AND soul_name = ?{}", param_values.len() + 1));
                param_values.push(Box::new(soul_name.clone()));
            }
            if let Some(resolved) = filter.resolved {
                if resolved {
                    sql.push_str(" AND resolved_at IS NOT NULL");
                } else {
                    sql.push_str(" AND resolved_at IS NULL");
                }
            }

            sql.push_str(" ORDER BY detected_at DESC");

            if let Some(limit) = filter.limit {
                sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
                param_values.push(Box::new(limit as i64));
            }
            if let Some(offset) = filter.offset {
                sql.push_str(&format!(" OFFSET ?{}", param_values.len() + 1));
                param_values.push(Box::new(offset as i64));
            }

            let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                Ok(BlindSpot {
                    id: row.get(0)?,
                    soul_name: row.get(1)?,
                    dimension: row.get(2)?,
                    description: row.get(3)?,
                    detected_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?).unwrap_or_default().with_timezone(&Utc),
                    resolved_at: row.get::<_, Option<String>>(5)?.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    resolved_by: row.get(6)?,
                    resolution: row.get(7)?,
                })
            })?;

            let mut results = vec![];
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    // Knowledge Cards
    pub fn insert_knowledge_card(&self, card: &KnowledgeCard) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO knowledge_cards (id, title, content, source_soul, source_session, tags, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    card.id,
                    card.title,
                    card.content,
                    card.source_soul,
                    card.source_session,
                    serde_json::to_string(&card.tags).ok(),
                    card.created_at.to_rfc3339(),
                    card.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn update_knowledge_card(&self, card: &KnowledgeCard) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE knowledge_cards SET title=?1, content=?2, source_soul=?3, source_session=?4, tags=?5, updated_at=?6 WHERE id=?7",
                params![
                    card.title,
                    card.content,
                    card.source_soul,
                    card.source_session,
                    serde_json::to_string(&card.tags).ok(),
                    card.updated_at.to_rfc3339(),
                    card.id,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_knowledge_cards(&self, filter: &KnowledgeCardFilter) -> Result<Vec<KnowledgeCard>> {
        self.with_conn(|conn| {
            let mut sql = String::from("SELECT c.id, c.title, c.content, c.source_soul, c.source_session, c.tags, c.created_at, c.updated_at FROM knowledge_cards c WHERE 1=1");
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

            if let Some(ref soul_name) = filter.soul_name {
                sql.push_str(&format!(" AND c.source_soul = ?{}", param_values.len() + 1));
                param_values.push(Box::new(soul_name.clone()));
            }

            sql.push_str(" ORDER BY c.created_at DESC");

            if let Some(limit) = filter.limit {
                sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
                param_values.push(Box::new(limit as i64));
            }
            if let Some(offset) = filter.offset {
                sql.push_str(&format!(" OFFSET ?{}", param_values.len() + 1));
                param_values.push(Box::new(offset as i64));
            }

            let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                let tags_str: Option<String> = row.get(5)?;
                let tags: Vec<String> = tags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
                Ok(KnowledgeCard {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    source_soul: row.get(3)?,
                    source_session: row.get(4)?,
                    tags,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?).unwrap_or_default().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?).unwrap_or_default().with_timezone(&Utc),
                })
            })?;

            let mut results = vec![];
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    pub fn list_knowledge_topics(&self, mode: Option<&str>, limit: usize, offset: usize) -> Result<Vec<KnowledgeTopic>> {
        self.with_conn(|conn| {
            let mut sql = String::from(
                "SELECT s.id, s.title, COALESCE(s.mode, 'unknown'), s.created_at
                 FROM sessions s WHERE s.status = 'completed'
                 AND EXISTS (SELECT 1 FROM messages m WHERE m.session_id = s.id AND m.role = 'synthesis')"
            );
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

            if let Some(m) = mode {
                if !m.is_empty() {
                    sql.push_str(&format!(" AND s.mode = ?{}", param_values.len() + 1));
                    param_values.push(Box::new(m.to_string()));
                }
            }

            sql.push_str(" ORDER BY s.created_at DESC");
            sql.push_str(&format!(" LIMIT ?{} OFFSET ?{}", param_values.len() + 1, param_values.len() + 2));
            param_values.push(Box::new(limit as i64));
            param_values.push(Box::new(offset as i64));

            let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                let session_id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let mode: String = row.get(2)?;
                let created_at_str: String = row.get(3)?;
                Ok((session_id, title, mode, created_at_str))
            })?;

            let mut topics = Vec::new();
            for row in rows {
                let (session_id, title, session_mode, created_at_str) = row?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .unwrap()
                    .with_timezone(&Utc);

                let soul_names: Vec<String> = {
                    let mut s = conn.prepare(
                        "SELECT DISTINCT soul_name FROM messages WHERE session_id = ?1 AND role = 'soul' ORDER BY soul_name"
                    )?;
                    let rows = s.query_map(params![session_id], |r| r.get::<_, String>(0))?;
                    let names: Vec<String> = rows.filter_map(|n| n.ok()).collect();
                    names
                };

                let card_summary: Option<String> = {
                    conn.query_row(
                        "SELECT content FROM messages WHERE session_id = ?1 AND role = 'system' AND soul_name = '知识卡片' LIMIT 1",
                        params![session_id],
                        |r| r.get::<_, String>(0),
                    ).ok()
                };

                let synthesis_preview: Option<String> = {
                    conn.query_row(
                        "SELECT SUBSTR(content, 1, 400) FROM messages WHERE session_id = ?1 AND role = 'synthesis' LIMIT 1",
                        params![session_id],
                        |r| r.get::<_, String>(0),
                    ).ok()
                };

                topics.push(KnowledgeTopic {
                    session_id,
                    title,
                    mode: session_mode,
                    created_at,
                    soul_names,
                    card_summary,
                    synthesis_preview,
                });
            }
            Ok(topics)
        })
    }

    // Revision Proposals
    pub fn insert_revision_proposal(&self, proposal: &RevisionProposal) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO revision_proposals (id, soul_name, proposal_type, title, description, proposed_changes, status, created_by, created_at, reviewed_at, reviewer, review_notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    proposal.id,
                    proposal.soul_name,
                    proposal_type_to_str(&proposal.proposal_type),
                    proposal.title,
                    proposal.description,
                    proposal.proposed_changes,
                    proposal_status_to_str(&proposal.status),
                    proposal.created_by,
                    proposal.created_at.to_rfc3339(),
                    proposal.reviewed_at.map(|dt| dt.to_rfc3339()),
                    proposal.reviewer,
                    proposal.review_notes,
                ],
            )?;
            Ok(())
        })
    }

    pub fn update_revision_proposal(&self, proposal: &RevisionProposal) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE revision_proposals SET soul_name=?1, proposal_type=?2, title=?3, description=?4, proposed_changes=?5, status=?6, created_by=?7, created_at=?8, reviewed_at=?9, reviewer=?10, review_notes=?11 WHERE id=?12",
                params![
                    proposal.soul_name,
                    proposal_type_to_str(&proposal.proposal_type),
                    proposal.title,
                    proposal.description,
                    proposal.proposed_changes,
                    proposal_status_to_str(&proposal.status),
                    proposal.created_by,
                    proposal.created_at.to_rfc3339(),
                    proposal.reviewed_at.map(|dt| dt.to_rfc3339()),
                    proposal.reviewer,
                    proposal.review_notes,
                    proposal.id,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_revision_proposals(&self, soul_name: Option<&str>, status: Option<ProposalStatus>) -> Result<Vec<RevisionProposal>> {
        self.with_conn(|conn| {
            let mut sql = String::from("SELECT id, soul_name, proposal_type, title, description, proposed_changes, status, created_by, created_at, reviewed_at, reviewer, review_notes FROM revision_proposals WHERE 1=1");
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

            if let Some(name) = soul_name {
                sql.push_str(&format!(" AND soul_name = ?{}", param_values.len() + 1));
                param_values.push(Box::new(name.to_string()));
            }
            if let Some(s) = status {
                sql.push_str(&format!(" AND status = ?{}", param_values.len() + 1));
                param_values.push(Box::new(proposal_status_to_str(&s).to_string()));
            }

            sql.push_str(" ORDER BY created_at DESC");

            let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                Ok(RevisionProposal {
                    id: row.get(0)?,
                    soul_name: row.get(1)?,
                    proposal_type: str_to_proposal_type(&row.get::<_, String>(2)?),
                    title: row.get(3)?,
                    description: row.get(4)?,
                    proposed_changes: row.get(5)?,
                    status: str_to_proposal_status(&row.get::<_, String>(6)?),
                    created_by: row.get(7)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?).unwrap_or_default().with_timezone(&Utc),
                    reviewed_at: row.get::<_, Option<String>>(9)?.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    reviewer: row.get(10)?,
                    review_notes: row.get(11)?,
                })
            })?;

            let mut results = vec![];
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    // ── Session Observations (claude-mem style) ──

    pub fn insert_session_observations(&self, observations: &[SessionObservation]) -> Result<()> {
        if observations.is_empty() { return Ok(()); }
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO session_observations \
                     (id, session_id, soul_name, obs_type, title, content, source_seq, read_tokens, work_tokens, confidence, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
                )?;
                for obs in observations {
                    stmt.execute(params![
                        obs.id,
                        obs.session_id,
                        obs.soul_name,
                        obs.obs_type.as_str(),
                        obs.title,
                        obs.content,
                        obs.source_seq,
                        obs.read_tokens as i64,
                        obs.work_tokens as i64,
                        obs.confidence,
                        obs.created_at.to_rfc3339(),
                    ])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    pub fn get_session_observations(&self, session_id: &str) -> Result<Vec<SessionObservation>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, soul_name, obs_type, title, content, source_seq, \
                        read_tokens, work_tokens, confidence, created_at \
                 FROM session_observations \
                 WHERE session_id = ?1 \
                 ORDER BY source_seq ASC NULLS LAST, created_at ASC"
            )?;
            let rows = stmt.query_map(params![session_id], |row| {
                let obs_type_str: String = row.get(3)?;
                let created_at: String = row.get(10)?;
                Ok(SessionObservation {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    soul_name: row.get(2)?,
                    obs_type: ObservationType::from_str(&obs_type_str).unwrap_or(ObservationType::Discovery),
                    title: row.get(4)?,
                    content: row.get(5)?,
                    source_seq: row.get(6)?,
                    read_tokens: row.get::<_, i64>(7)? as u32,
                    work_tokens: row.get::<_, i64>(8)? as u32,
                    confidence: row.get::<_, f64>(9)? as f32,
                    created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().with_timezone(&Utc),
                })
            })?;
            let mut results = vec![];
            for r in rows { results.push(r?); }
            Ok(results)
        })
    }

    pub fn get_observations_by_soul(&self, soul_name: &str, limit: u32) -> Result<Vec<SessionObservation>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, soul_name, obs_type, title, content, source_seq, \
                        read_tokens, work_tokens, confidence, created_at \
                 FROM session_observations \
                 WHERE soul_name = ?1 \
                 ORDER BY created_at DESC \
                 LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![soul_name, limit as i64], |row| {
                let obs_type_str: String = row.get(3)?;
                let created_at: String = row.get(10)?;
                Ok(SessionObservation {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    soul_name: row.get(2)?,
                    obs_type: ObservationType::from_str(&obs_type_str).unwrap_or(ObservationType::Discovery),
                    title: row.get(4)?,
                    content: row.get(5)?,
                    source_seq: row.get(6)?,
                    read_tokens: row.get::<_, i64>(7)? as u32,
                    work_tokens: row.get::<_, i64>(8)? as u32,
                    confidence: row.get::<_, f64>(9)? as f32,
                    created_at: DateTime::parse_from_rfc3339(&created_at).unwrap_or_default().with_timezone(&Utc),
                })
            })?;
            let mut results = vec![];
            for r in rows { results.push(r?); }
            Ok(results)
        })
    }

    pub fn update_session_digest(&self, session_id: &str, summary: &str) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE sessions SET digest_summary = ?1, digest_at = ?2 WHERE id = ?3",
                params![summary, Utc::now().to_rfc3339(), session_id],
            )?;
            Ok(())
        })
    }

    // ── Annotations (marginalia 模式) ──

    pub fn insert_annotations(&self, annotations: &[Annotation]) -> Result<()> {
        if annotations.is_empty() { return Ok(()); }
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO annotations \
                     (id, session_id, source_soul, target_soul, target_excerpt, comment, kind, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                )?;
                for ann in annotations {
                    stmt.execute(params![
                        ann.id,
                        ann.session_id,
                        ann.source_soul,
                        ann.target_soul,
                        ann.target_excerpt,
                        ann.comment,
                        ann.kind,
                        ann.created_at.to_rfc3339(),
                    ])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    pub fn get_annotations(&self, session_id: &str) -> Result<Vec<Annotation>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, source_soul, target_soul, target_excerpt, comment, kind, created_at \
                 FROM annotations \
                 WHERE session_id = ?1 \
                 ORDER BY created_at ASC"
            )?;
            let rows = stmt.query_map(params![session_id], |row| {
                let created_at: String = row.get(7)?;
                Ok(Annotation {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    source_soul: row.get(2)?,
                    target_soul: row.get(3)?,
                    target_excerpt: row.get(4)?,
                    comment: row.get(5)?,
                    kind: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    // ── Session Reviews (实践反馈闭环) ──

    pub fn insert_session_review(&self, review: &SessionReview) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO session_reviews \
                 (id, session_id, most_unexpected, already_known, self_negation, \
                  empty_chair, effectiveness, effectiveness_note, \
                  practice_commitment, practice_horizon, \
                  interrogation_passed, interrogation_reason, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    review.id,
                    review.session_id,
                    review.most_unexpected,
                    review.already_known,
                    review.self_negation,
                    review.empty_chair,
                    review.effectiveness,
                    review.effectiveness_note,
                    review.practice_commitment,
                    review.practice_horizon,
                    review.interrogation_passed.map(|b| b as i32),
                    review.interrogation_reason,
                    review.created_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_session_review(&self, session_id: &str) -> Result<Option<SessionReview>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, most_unexpected, already_known, self_negation, \
                        empty_chair, effectiveness, effectiveness_note, \
                        practice_commitment, practice_horizon, \
                        interrogation_passed, interrogation_reason, created_at \
                 FROM session_reviews WHERE session_id = ?1 LIMIT 1"
            )?;
            let mut rows = stmt.query(params![session_id])?;
            if let Some(row) = rows.next()? {
                let created_at: String = row.get(12)?;
                Ok(Some(SessionReview {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    most_unexpected: row.get(2)?,
                    already_known: row.get(3)?,
                    self_negation: row.get(4)?,
                    empty_chair: row.get(5)?,
                    effectiveness: row.get(6)?,
                    effectiveness_note: row.get(7)?,
                    practice_commitment: row.get(8)?,
                    practice_horizon: row.get(9)?,
                    interrogation_passed: row.get::<_, Option<i32>>(10)?.map(|v| v != 0),
                    interrogation_reason: row.get(11)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                }))
            } else {
                Ok(None)
            }
        })
    }

    pub fn get_recent_reviews(&self, limit: u32) -> Result<Vec<SessionReview>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, most_unexpected, already_known, self_negation, \
                        empty_chair, effectiveness, effectiveness_note, \
                        practice_commitment, practice_horizon, \
                        interrogation_passed, interrogation_reason, created_at \
                 FROM session_reviews ORDER BY created_at DESC LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![limit], |row| {
                let created_at: String = row.get(12)?;
                Ok(SessionReview {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    most_unexpected: row.get(2)?,
                    already_known: row.get(3)?,
                    self_negation: row.get(4)?,
                    empty_chair: row.get(5)?,
                    effectiveness: row.get(6)?,
                    effectiveness_note: row.get(7)?,
                    practice_commitment: row.get(8)?,
                    practice_horizon: row.get(9)?,
                    interrogation_passed: row.get::<_, Option<i32>>(10)?.map(|v| v != 0),
                    interrogation_reason: row.get(11)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }
}
