use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use foundation::sqlite::SqliteDb;
use foundation::{Result, UsageStats};
use sha2::{Digest, Sha256};
use tracing;

pub struct LlMCache {
    db: Arc<SqliteDb>,
    ttl_secs: u64,
}

impl LlMCache {
    pub fn new(db: Arc<SqliteDb>, ttl_secs: u64) -> Self {
        LlMCache { db, ttl_secs }
    }

    fn hash_key(provider: &str, model: &str, system_prompt: &str, user_prompt: &str) -> String {
        let mut hasher = Sha256::new();
        // Length-prefixed encoding to prevent delimiter collisions
        for part in [provider, model, system_prompt, user_prompt] {
            let len = (part.len() as u64).to_le_bytes();
            hasher.update(len);
            hasher.update(part.as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn get(
        &self,
        provider: &str,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Option<(String, UsageStats)> {
        let hash = Self::hash_key(provider, model, system_prompt, user_prompt);
        let cutoff = Self::now_secs().saturating_sub(self.ttl_secs);

        self.db
            .with_conn(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT response_content, usage_json, created_at FROM llm_cache WHERE hash = ?1",
                )?;
                let row = stmt.query_row(
                    rusqlite::params![hash],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, Option<String>>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    },
                );

                match row {
                    Ok((content, usage_json, created_at)) => {
                        let created_secs = chrono::DateTime::parse_from_rfc3339(&created_at)
                            .map(|dt| dt.timestamp() as u64)
                            .unwrap_or(0);
                        if created_secs < cutoff {
                            let _ = conn.execute("DELETE FROM llm_cache WHERE hash = ?1", rusqlite::params![hash]);
                            Ok(None)
                        } else {
                            let usage = usage_json
                                .and_then(|j| serde_json::from_str::<UsageStats>(&j).ok())
                                .unwrap_or_default();
                            Ok(Some((content, usage)))
                        }
                    }
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(foundation::FoundationError::Sqlite(e)),
                }
            })
            .unwrap_or(None)
    }

    pub fn set(
        &self,
        provider: &str,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        content: &str,
        usage: &UsageStats,
    ) -> Result<()> {
        let hash = Self::hash_key(provider, model, system_prompt, user_prompt);
        let usage_json = serde_json::to_string(usage).unwrap_or_default();

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO llm_cache (hash, provider, model, response_content, usage_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    hash,
                    provider,
                    model,
                    content,
                    usage_json,
                    chrono::Utc::now().to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn cleanup(&self) -> Result<usize> {
        let cutoff = Self::now_secs().saturating_sub(self.ttl_secs);
        let cutoff_str = chrono::DateTime::from_timestamp(cutoff as i64, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        self.db.with_conn(|conn| {
            let count = conn.execute(
                "DELETE FROM llm_cache WHERE created_at < ?1",
                rusqlite::params![cutoff_str],
            )?;
            tracing::info!("Cleaned up {} expired cache entries", count);
            Ok(count)
        })
    }
}
